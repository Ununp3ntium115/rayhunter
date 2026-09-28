use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::{InformationElement, LteInformationElement};
use chrono::{DateTime, Duration, FixedOffset};
use std::collections::VecDeque;
use telcom_parser::lte_rrc::{
    DL_DCCH_MessageType, DL_DCCH_MessageType_c1, RRCConnectionReleaseCriticalExtensions,
    RRCConnectionReleaseCriticalExtensions_c1, ReleaseCause,
};

pub struct ConnectionReleaseStormAnalyzer {
    /// Time-windowed queue of RRCConnectionRelease timestamps with cause=other (300-second window)
    release_timestamps: VecDeque<DateTime<FixedOffset>>,
    /// Last release timestamp, used to track consecutive releases
    last_release_timestamp: Option<DateTime<FixedOffset>>,
    /// Count of consecutive releases (reset after >10s gap)
    consecutive_release_count: usize,
    /// Timestamp of last HIGH alert to prevent rapid re-firing
    last_high_alert_timestamp: Option<DateTime<FixedOffset>>,
    /// Flag to ensure MEDIUM alert fires only once per recording
    medium_alert_fired: bool,
}

impl Default for ConnectionReleaseStormAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionReleaseStormAnalyzer {
    pub fn new() -> Self {
        ConnectionReleaseStormAnalyzer {
            release_timestamps: VecDeque::new(),
            last_release_timestamp: None,
            consecutive_release_count: 0,
            last_high_alert_timestamp: None,
            medium_alert_fired: false,
        }
    }

    /// Check if the message is an RRCConnectionRelease message with cause=other.
    fn is_connection_release_with_other_cause(ie: &InformationElement) -> bool {
        let lte_ie = match ie {
            InformationElement::LTE(inner) => inner.as_ref(),
            _ => return false,
        };

        match lte_ie {
            LteInformationElement::DlDcch(dl_dcch) => {
                let DL_DCCH_MessageType::C1(c1) = &dl_dcch.message else {
                    return false;
                };
                if let DL_DCCH_MessageType_c1::RrcConnectionRelease(release) = c1
                    && let RRCConnectionReleaseCriticalExtensions::C1(c1_ext) =
                        &release.critical_extensions
                    && let RRCConnectionReleaseCriticalExtensions_c1::RrcConnectionRelease_r8(
                        r8_ies,
                    ) = c1_ext
                {
                    // Filter for cause=other (value 1)
                    return r8_ies.release_cause.0 == ReleaseCause::OTHER;
                }
                false
            }
            _ => false,
        }
    }

    /// Update the time window and check for rate thresholds.
    /// Returns Some(high_alert_info) if HIGH alert should fire (>2 consecutive within 10s gap),
    /// or Some(medium_alert_info) if MEDIUM alert should fire (>3 in 300s).
    fn check_thresholds(
        &mut self,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<(EventType, usize, usize)> {
        // Remove timestamps older than 300 seconds
        let window_300_start = timestamp - Duration::seconds(300);
        while let Some(&front) = self.release_timestamps.front() {
            if front < window_300_start {
                self.release_timestamps.pop_front();
            } else {
                break;
            }
        }

        // Add current timestamp
        self.release_timestamps.push_back(timestamp);

        // Check HIGH alert condition: >2 consecutive releases within 10s gaps
        // If current release is within 10s of the last release, increment consecutive count
        if let Some(last_ts) = self.last_release_timestamp {
            let gap = timestamp.signed_duration_since(last_ts);
            if gap <= Duration::seconds(10) {
                // Still in the same burst
                self.consecutive_release_count += 1;
            } else {
                // New burst, reset count
                self.consecutive_release_count = 1;
            }
        } else {
            // First release
            self.consecutive_release_count = 1;
        }
        self.last_release_timestamp = Some(timestamp);

        // Fire HIGH alert if consecutive count > 2
        if self.consecutive_release_count > 2 {
            // Fire HIGH alert if enough time has passed since last HIGH alert
            if self.last_high_alert_timestamp.is_none()
                || timestamp.signed_duration_since(self.last_high_alert_timestamp.unwrap())
                    >= Duration::seconds(10)
            {
                self.last_high_alert_timestamp = Some(timestamp);
                return Some((EventType::High, self.consecutive_release_count, 10));
            }
        }

        // Check MEDIUM alert condition: >3 releases in 300s window
        if !self.medium_alert_fired && self.release_timestamps.len() > 3 {
            self.medium_alert_fired = true;
            return Some((EventType::Medium, self.release_timestamps.len(), 300));
        }

        None
    }
}

impl Analyzer for ConnectionReleaseStormAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "connection_release_storm".into(),
            default_enabled: false,
            name: "Connection Release Storm".into(),
            description: "Detects excessive RRC connection release messages that may indicate a forced \
                connection termination attack. A fake cell may forcibly terminate connections to trigger \
                re-attachment cycles and enable handover spoofing or cipher downgrade attacks. Fires HIGH \
                severity if >2 connection releases with cause=other occur within 10 seconds, or MEDIUM \
                severity if >3 releases occur within a 300-second window. Normal behavior involves at most \
                1 release per session end."
                .into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        if !Self::is_connection_release_with_other_cause(ie) {
            return None;
        }

        // Check both thresholds
        if let Some((event_type, count, window_seconds)) = self.check_thresholds(timestamp) {
            return Some(Event {
                event_type,
                message: format!(
                    "Connection release storm detected: {} releases with cause=other in {}-second window. \
                     This may indicate forced connection termination attacks enabling handover spoofing or \
                     cipher downgrade. Each release triggers device re-attachment and security re-negotiation.",
                    count, window_seconds
                ),
                analyzer_index: 0,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use telcom_parser::lte_rrc::{
        DL_DCCH_Message, DL_DCCH_MessageType, DL_DCCH_MessageType_c1, RRCConnectionRelease,
        RRCConnectionRelease_r8_IEs, RRCConnectionReleaseCriticalExtensions,
        RRCConnectionReleaseCriticalExtensions_c1, ReleaseCause,
    };

    fn ts() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
    }

    fn release_ie_with_cause(cause: u8) -> InformationElement {
        use telcom_parser::lte_rrc::RRC_TransactionIdentifier;

        let r8_ies = RRCConnectionRelease_r8_IEs {
            release_cause: ReleaseCause(cause),
            redirected_carrier_info: None,
            idle_mode_mobility_control_info: None,
            non_critical_extension: None,
        };
        let release = RRCConnectionRelease {
            rrc_transaction_identifier: RRC_TransactionIdentifier(0),
            critical_extensions: RRCConnectionReleaseCriticalExtensions::C1(
                RRCConnectionReleaseCriticalExtensions_c1::RrcConnectionRelease_r8(r8_ies),
            ),
        };
        let dl_dcch_msg = DL_DCCH_Message {
            message: DL_DCCH_MessageType::C1(DL_DCCH_MessageType_c1::RrcConnectionRelease(release)),
        };
        InformationElement::LTE(Box::new(
            crate::analysis::information_element::LteInformationElement::DlDcch(Box::new(
                dl_dcch_msg,
            )),
        ))
    }

    fn release_with_other_cause() -> InformationElement {
        release_ie_with_cause(ReleaseCause::OTHER)
    }

    fn release_with_load_balancing_cause() -> InformationElement {
        release_ie_with_cause(ReleaseCause::LOAD_BALANCING_TA_UREQUIRED)
    }

    fn release_with_cs_fallback_cause() -> InformationElement {
        release_ie_with_cause(ReleaseCause::CS_FALLBACK_HIGH_PRIORITY_V1020)
    }

    #[test]
    fn non_lte_returns_none() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        assert!(
            a.analyze_information_element(&InformationElement::GSM, 1, ts())
                .is_none()
        );
    }

    #[test]
    fn filters_non_other_causes() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        // Load balancing cause should be ignored
        let ev = a.analyze_information_element(&release_with_load_balancing_cause(), 1, ts());
        assert!(ev.is_none(), "load-balancing cause should be filtered");

        // CS fallback cause should be ignored
        let ev = a.analyze_information_element(&release_with_cs_fallback_cause(), 2, ts());
        assert!(ev.is_none(), "cs-fallback cause should be filtered");
    }

    #[test]
    fn accepts_other_cause() {
        let ie = release_with_other_cause();
        // With cause=other, it should be tracked
        assert!(ConnectionReleaseStormAnalyzer::is_connection_release_with_other_cause(&ie));
    }

    #[test]
    fn time_window_below_high_threshold() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        // Add 2 releases in 10s window (below >2 threshold)
        for i in 0i64..2 {
            let ts = ts() + Duration::seconds(i);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            assert!(ev.is_none(), "should not alert on 2 releases in 10s window");
        }
    }

    #[test]
    fn time_window_exceeds_high_threshold() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        // Add 3 releases in 10s window (exceeds >2 threshold)
        for i in 0i64..3 {
            let ts = ts() + Duration::milliseconds(i * 100);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            if i == 2 {
                assert!(ev.is_some(), "3rd release in 10s should trigger HIGH alert");
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::High);
                assert!(event.message.contains("storm"));
                assert!(event.message.contains("10-second"));
            } else {
                assert!(ev.is_none());
            }
        }
    }

    #[test]
    fn high_alert_prevents_immediate_refire() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();

        // Add 3 releases within 10s to trigger HIGH alert
        for i in 0i64..3 {
            let ts = base_ts + Duration::milliseconds(i * 100);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            if i == 2 {
                assert!(ev.is_some(), "3rd release should trigger HIGH alert");
                assert_eq!(ev.unwrap().event_type, EventType::High);
            }
        }

        // Add one more release within 10s; should trigger MEDIUM alert when we hit 4 total
        // This verifies that HIGH prevents immediate refire, but MEDIUM can still fire
        let ev = a.analyze_information_element(
            &release_with_other_cause(),
            4,
            base_ts + Duration::milliseconds(300),
        );
        assert!(ev.is_some(), "4th release should trigger MEDIUM alert");
        assert_eq!(ev.unwrap().event_type, EventType::Medium);

        // Add more releases; nothing should fire since HIGH is still in cooldown and MEDIUM already fired
        for i in 4i64..6 {
            let ts = base_ts + Duration::milliseconds(i * 100);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            assert!(
                ev.is_none(),
                "No alert should fire: HIGH cooldown active, MEDIUM already fired"
            );
        }
    }

    #[test]
    fn high_alert_can_refire_after_10_seconds() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();

        // Add 3 releases within first 10s to trigger HIGH alert
        for i in 0i64..3 {
            let ts = base_ts + Duration::milliseconds(i * 100);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            if i == 2 {
                assert!(ev.is_some(), "3rd release should trigger HIGH alert");
            }
        }

        // Advance far past the 10-second cooldown (>300s) to avoid MEDIUM alert complications
        // Add new releases at 301s, 301.1s, 301.2s (outside the 300s window of the initial 3)

        // Add 1st new release at 301s (outside 300s window from start)
        let ev = a.analyze_information_element(
            &release_with_other_cause(),
            4,
            base_ts + Duration::seconds(301),
        );
        assert!(ev.is_none(), "only 1 release in new burst");

        // Add 2nd new release at 301.1s
        let ev = a.analyze_information_element(
            &release_with_other_cause(),
            5,
            base_ts + Duration::seconds(301) + Duration::milliseconds(100),
        );
        assert!(ev.is_none(), "only 2 releases in new burst");

        // Add 3rd new release at 301.2s; should trigger another HIGH alert after 10s cooldown
        let ev = a.analyze_information_element(
            &release_with_other_cause(),
            6,
            base_ts + Duration::seconds(301) + Duration::milliseconds(200),
        );
        assert!(ev.is_some(), "should re-fire HIGH alert after cooldown");
        assert_eq!(ev.unwrap().event_type, EventType::High);
    }

    #[test]
    fn medium_alert_below_threshold() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();
        // Add 3 releases over 300s (below >3 threshold)
        for i in 0i64..3 {
            let ts = base_ts + Duration::seconds(i * 100);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            assert!(
                ev.is_none(),
                "should not alert on 3 releases in 300s window"
            );
        }
    }

    #[test]
    fn medium_alert_exceeds_threshold() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();
        // Add 4 releases over 300s (exceeds >3 threshold)
        for i in 0i64..4 {
            let ts = base_ts + Duration::seconds(i * 75);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            if i == 3 {
                assert!(
                    ev.is_some(),
                    "4th release in 300s should trigger MEDIUM alert"
                );
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::Medium);
                assert!(event.message.contains("storm"));
                assert!(event.message.contains("300-second"));
            } else {
                assert!(ev.is_none());
            }
        }
    }

    #[test]
    fn medium_alert_fires_once_per_recording() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();

        // Add 4 releases to trigger MEDIUM alert
        for i in 0i64..4 {
            let ts = base_ts + Duration::seconds(i * 75);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            if i == 3 {
                assert!(ev.is_some(), "4th release should trigger MEDIUM alert");
            }
        }

        // Add more releases; MEDIUM alert should not fire again
        for i in 4i64..10 {
            let ts = base_ts + Duration::seconds(i * 75);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            assert!(
                ev.is_none(),
                "MEDIUM alert should only fire once per recording"
            );
        }
    }

    #[test]
    fn high_alert_and_medium_alert_independent() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();

        // Add 3 releases in first 10s to trigger HIGH alert
        for i in 0i64..3 {
            let ts = base_ts + Duration::milliseconds(i * 100);
            let ev =
                a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
            if i == 2 {
                assert!(ev.is_some(), "should fire HIGH alert");
                assert_eq!(ev.unwrap().event_type, EventType::High);
            }
        }

        // Add 1 more release at 11s to push 300s count to 4; should trigger MEDIUM alert
        let ev = a.analyze_information_element(
            &release_with_other_cause(),
            4,
            base_ts + Duration::seconds(11),
        );
        assert!(
            ev.is_some(),
            "4 releases in 300s window should trigger MEDIUM alert"
        );
        assert_eq!(ev.unwrap().event_type, EventType::Medium);
    }

    #[test]
    fn releases_outside_300s_window_not_counted() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();

        // Add 4 releases over 350 seconds; the first should age out
        for i in 0i64..4 {
            let ts = base_ts + Duration::seconds(i * 120);
            a.analyze_information_element(&release_with_other_cause(), (i + 1) as usize, ts);
        }

        // Add one more release 500s later; only the new one + recent ones remain
        // So we should not have >3 in 300s window
        let ev = a.analyze_information_element(
            &release_with_other_cause(),
            5,
            base_ts + Duration::seconds(500),
        );
        // Window cleaning should have dropped old entries, so no MEDIUM alert expected
        assert!(
            ev.is_none(),
            "old releases should have been pruned from window"
        );
    }

    #[test]
    fn high_alert_with_real_pcap_pattern() {
        // Simulate the pattern from 1790367566.pcapng: ~22 releases in rapid bursts
        let mut a = ConnectionReleaseStormAnalyzer::new();
        let base_ts = ts();

        // First burst: 3 releases within 1s
        for i in 0..3 {
            let ts = base_ts + Duration::milliseconds(i as i64 * 100);
            let ev = a.analyze_information_element(&release_with_other_cause(), i + 1, ts);
            if i == 2 {
                assert!(ev.is_some(), "should fire HIGH alert on 3rd release");
                assert_eq!(ev.unwrap().event_type, EventType::High);
            }
        }

        // After 11s, add another 3 releases; should trigger another HIGH alert
        for i in 0..3 {
            let ts = base_ts + Duration::seconds(11) + Duration::milliseconds(i as i64 * 100);
            let ev = a.analyze_information_element(&release_with_other_cause(), 3 + i + 1, ts);
            if i == 2 {
                assert!(ev.is_some(), "should fire HIGH alert after cooldown");
                assert_eq!(ev.unwrap().event_type, EventType::High);
            }
        }
    }

    #[test]
    fn non_lte_information_elements_ignored() {
        let mut a = ConnectionReleaseStormAnalyzer::new();
        // These should all be ignored and not affect detection
        for i in 0..5 {
            a.analyze_information_element(&InformationElement::GSM, i + 1, ts());
            a.analyze_information_element(&InformationElement::FiveG, i + 1, ts());
        }
        // Add one release; should not alert since only 1
        let ev = a.analyze_information_element(&release_with_other_cause(), 6, ts());
        assert!(ev.is_none());
    }
}
