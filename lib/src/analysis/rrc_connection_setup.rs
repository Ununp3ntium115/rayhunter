use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::{InformationElement, LteInformationElement};
use chrono::{DateTime, Duration, FixedOffset};
use std::collections::VecDeque;
use telcom_parser::lte_rrc::DL_CCCH_MessageType;

pub struct RapidConnectionSetupAnalyzer {
    /// Timestamps within the 60-second burst window
    burst_timestamps: VecDeque<DateTime<FixedOffset>>,
    /// Timestamps within the 300-second sustained window
    sustained_timestamps: VecDeque<DateTime<FixedOffset>>,
    /// Total count of RRCConnectionSetup messages seen
    total_setups: usize,
    /// Flag to ensure ratio alert fires only once per recording
    ratio_alert_fired: bool,
}

impl Default for RapidConnectionSetupAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RapidConnectionSetupAnalyzer {
    pub fn new() -> Self {
        RapidConnectionSetupAnalyzer {
            burst_timestamps: VecDeque::new(),
            sustained_timestamps: VecDeque::new(),
            total_setups: 0,
            ratio_alert_fired: false,
        }
    }

    /// Check if the message is an RRCConnectionSetup message.
    fn is_connection_setup_message(ie: &InformationElement) -> bool {
        let lte_ie = match ie {
            InformationElement::LTE(inner) => inner.as_ref(),
            _ => return false,
        };

        match lte_ie {
            LteInformationElement::DlCcch(dl_ccch) => {
                let DL_CCCH_MessageType::C1(c1) = &dl_ccch.message else {
                    return false;
                };
                matches!(
                    c1,
                    telcom_parser::lte_rrc::DL_CCCH_MessageType_c1::RrcConnectionSetup(_)
                )
            }
            _ => false,
        }
    }

    /// Check both time windows and return the highest-severity alert, if any.
    /// Burst window (60s, ≥3): HIGH — truly rapid forced re-establishment.
    /// Sustained window (300s, ≥5): HIGH — repeated re-establishment over 5 minutes.
    /// Clears the triggering window after firing to prevent immediate re-alert.
    fn check_time_window_threshold(&mut self, timestamp: DateTime<FixedOffset>) -> Option<usize> {
        // Burst window: 3+ setups in 60 seconds
        let burst_start = timestamp - Duration::seconds(60);
        while self.burst_timestamps.front().is_some_and(|&t| t < burst_start) {
            self.burst_timestamps.pop_front();
        }
        self.burst_timestamps.push_back(timestamp);
        if self.burst_timestamps.len() >= 3 {
            let count = self.burst_timestamps.len();
            self.burst_timestamps.clear();
            return Some(count);
        }

        // Sustained window: 5+ setups in 300 seconds
        let sustained_start = timestamp - Duration::seconds(300);
        while self.sustained_timestamps.front().is_some_and(|&t| t < sustained_start) {
            self.sustained_timestamps.pop_front();
        }
        self.sustained_timestamps.push_back(timestamp);
        if self.sustained_timestamps.len() >= 5 {
            let count = self.sustained_timestamps.len();
            self.sustained_timestamps.clear();
            return Some(count);
        }

        None
    }
}

impl Analyzer for RapidConnectionSetupAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "rapid_connection_setup".into(),
            default_enabled: true,
            name: "Rapid RRC Connection Setup".into(),
            description: "Detects multiple RRC connection setup messages within short time windows, which may indicate \
                forced connection re-establishment attacks or IMSI catcher activity. IMSI catchers may trigger rapid setups \
                to enable multiple security procedures and facilitate cipher algorithm downgrade. Fires HIGH severity if ≥3 setup \
                messages occur within 60 seconds (burst), or if ≥5 occur within 300 seconds (sustained). Fires MEDIUM severity \
                if setups comprise >3% of total traffic. Normal LTE behavior on carriers like Verizon involves 2–4 setups \
                per 5-minute window; only genuinely elevated rates are flagged."
                .into(),
            version: 2,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        packet_num: usize,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        if !Self::is_connection_setup_message(ie) {
            return None;
        }

        // Increment total setup count for ratio check
        self.total_setups += 1;

        // Check time-window threshold (HIGH severity)
        if let Some(count) = self.check_time_window_threshold(timestamp) {
            return Some(Event {
                event_type: EventType::High,
                message: format!(
                    "Rapid RRC connection setup detected: {} setup messages in 300-second window. \
                     This may indicate a forced connection re-establishment attack or IMSI catcher activity.",
                    count
                ),
                analyzer_index: 0,
            });
        }

        // Check ratio threshold (MEDIUM severity, once per recording)
        // Require minimum warmup to avoid false positives at recording start (100 packets)
        if !self.ratio_alert_fired && packet_num >= 100 {
            let setup_ratio = self.total_setups as f64 / packet_num as f64;
            if setup_ratio > 0.03 {
                self.ratio_alert_fired = true;
                return Some(Event {
                    event_type: EventType::Medium,
                    message: format!(
                        "RRC connection setup density anomaly: {:.1}% of traffic is setup messages. \
                         Normal behavior is <1%, >3% suggests potential forced re-establishment or network anomaly.",
                        setup_ratio * 100.0
                    ),
                    analyzer_index: 0,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use telcom_parser::lte_rrc::{
        DL_CCCH_Message, DL_CCCH_MessageType, DL_CCCH_MessageType_c1, RRC_TransactionIdentifier,
        RRCConnectionSetup, RRCConnectionSetup_r8_IEs, RRCConnectionSetupCriticalExtensions,
        RRCConnectionSetupCriticalExtensions_c1, RadioResourceConfigDedicated,
    };

    fn ts() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
    }

    fn connection_setup_ie() -> InformationElement {
        // Create a minimal RRCConnectionSetup with only required fields
        let setup = RRCConnectionSetup {
            rrc_transaction_identifier: RRC_TransactionIdentifier(0),
            critical_extensions: RRCConnectionSetupCriticalExtensions::C1(
                RRCConnectionSetupCriticalExtensions_c1::RrcConnectionSetup_r8(
                    RRCConnectionSetup_r8_IEs {
                        radio_resource_config_dedicated: RadioResourceConfigDedicated {
                            srb_to_add_mod_list: None,
                            drb_to_add_mod_list: None,
                            drb_to_release_list: None,
                            mac_main_config: None,
                            sps_config: None,
                            physical_config_dedicated: None,
                        },
                        non_critical_extension: None,
                    },
                ),
            ),
        };

        let dl_ccch = DL_CCCH_Message {
            message: DL_CCCH_MessageType::C1(DL_CCCH_MessageType_c1::RrcConnectionSetup(setup)),
        };

        InformationElement::LTE(Box::new(LteInformationElement::DlCcch(dl_ccch)))
    }

    #[test]
    fn non_lte_returns_none() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        assert!(
            a.analyze_information_element(&InformationElement::GSM, 1, ts())
                .is_none()
        );
    }

    #[test]
    fn non_dl_ccch_lte_message_returns_none() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        let ie = InformationElement::FiveG;
        assert!(a.analyze_information_element(&ie, 1, ts()).is_none());
    }

    #[test]
    fn burst_window_below_threshold() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        // 2 setups within 60s — below burst threshold of 3
        for i in 0i64..2 {
            let ts = ts() + Duration::seconds(i * 10);
            let ev = a.analyze_information_element(&connection_setup_ie(), (i + 1) as usize, ts);
            assert!(ev.is_none(), "2 setups in 60s should not alert");
        }
    }

    #[test]
    fn burst_window_exceeds_threshold() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        // 3 setups within 60s — triggers burst High
        for i in 0i64..3 {
            let ts = ts() + Duration::seconds(i * 10);
            let ev = a.analyze_information_element(&connection_setup_ie(), (i + 1) as usize, ts);
            if i == 2 {
                assert!(ev.is_some(), "3rd setup in 60s should trigger HIGH");
                assert_eq!(ev.unwrap().event_type, EventType::High);
            } else {
                assert!(ev.is_none());
            }
        }
    }

    #[test]
    fn sustained_window_exceeds_threshold() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        // 5 setups spaced 70s apart — within 300s sustained window, outside 60s burst window
        for i in 0i64..5 {
            let ts = ts() + Duration::seconds(i * 70);
            let ev = a.analyze_information_element(&connection_setup_ie(), (i + 1) as usize, ts);
            if i == 4 {
                assert!(ev.is_some(), "5th setup in 300s should trigger HIGH");
                assert_eq!(ev.unwrap().event_type, EventType::High);
            } else {
                assert!(ev.is_none());
            }
        }
    }

    #[test]
    fn sustained_window_below_threshold() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        // 4 setups spaced 70s apart — 4 in 300s is below the sustained threshold of 5
        for i in 0i64..4 {
            let ts = ts() + Duration::seconds(i * 70);
            let ev = a.analyze_information_element(&connection_setup_ie(), (i + 1) as usize, ts);
            assert!(ev.is_none(), "4 setups in 300s should not alert");
        }
    }

    #[test]
    fn messages_outside_window_not_counted() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        let base_ts = ts();

        // 2 setups far apart (>300s), followed by new ones — old ones age out
        for i in 0i64..2 {
            a.analyze_information_element(
                &connection_setup_ie(),
                (i + 1) as usize,
                base_ts + Duration::seconds(i * 150),
            );
        }
        // 301s after start: the first setup (at 0s) has aged out of both windows
        let ev = a.analyze_information_element(
            &connection_setup_ie(),
            3,
            base_ts + Duration::seconds(400),
        );
        assert!(ev.is_none(), "aged-out messages should not count");
    }

    #[test]
    fn ratio_alert_fires_once() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        let base_ts = ts();

        // First, add enough non-setup packets to reach packet_num >= 100
        for i in 0..110 {
            a.analyze_information_element(&InformationElement::GSM, i + 1, base_ts);
        }

        // Now add setup messages until ratio exceeds 3%.
        // Space them >300s apart to avoid time-window alert
        // With 110 non-setup packets and ≥4 setups: 4/114 ≈ 3.5% > 3%
        for i in 0..4 {
            let ts = base_ts + Duration::seconds(i as i64 * 400); // Space 400s apart
            let pkt_num = 110 + i + 1;
            let ev = a.analyze_information_element(&connection_setup_ie(), pkt_num, ts);
            if i == 3 {
                // 4th setup should trigger ratio alert
                assert!(
                    ev.is_some(),
                    "4th setup should trigger ratio alert at ~3.5% density"
                );
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::Medium);
                assert!(event.message.contains("density"));
            } else {
                assert!(ev.is_none());
            }
        }

        // Add more setups; ratio alert should not fire again
        for i in 4..8 {
            let ts = base_ts + Duration::seconds(i as i64 * 400);
            let pkt_num = 110 + i + 1;
            let ev = a.analyze_information_element(&connection_setup_ie(), pkt_num, ts);
            assert!(
                ev.is_none(),
                "ratio alert should only fire once per recording"
            );
        }
    }

    #[test]
    fn ratio_warmup_suppresses_early_alert() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        let base_ts = ts();

        // Feed 5 setup messages within first 50 packets (10% setup rate)
        // Space them >300s apart to avoid time-window alert
        // Should not alert because packet_num < 100
        for i in 0..5 {
            let ts = base_ts + Duration::seconds(i as i64 * 400); // Space 400s apart
            let ev = a.analyze_information_element(&connection_setup_ie(), i + 1, ts);
            assert!(ev.is_none(), "should suppress alert before warmup");
        }

        // Ratio check should still be inactive
        assert!(!a.ratio_alert_fired);
    }

    #[test]
    fn burst_alert_clears_window() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        let base_ts = ts();

        // Trigger burst alert with 3 in 60s
        for i in 0i64..3 {
            a.analyze_information_element(
                &connection_setup_ie(),
                (i + 1) as usize,
                base_ts + Duration::seconds(i * 5),
            );
        }
        // One more immediately after: window was cleared, so only 1 in window — no alert
        let ev = a.analyze_information_element(
            &connection_setup_ie(),
            4,
            base_ts + Duration::seconds(16),
        );
        assert!(ev.is_none(), "burst window cleared — single message should not re-alert");
    }

    #[test]
    fn ratio_tracks_total_across_session() {
        let mut a = RapidConnectionSetupAnalyzer::new();
        let base_ts = ts();

        // Add packets with 4 setups spaced >300s apart (so no time-window alerts)
        // With 4 setups at packet positions 20, 40, 60, 110: 4/110 ≈ 3.6% > 3%
        // The 4th setup at packet 110 will trigger the ratio alert
        for i in 0..110 {
            let ts = base_ts + Duration::seconds((i / 40) as i64 * 350); // Groups of 40 packets, 350s apart
            if i == 19 || i == 39 || i == 59 || i == 109 {
                let ev = a.analyze_information_element(&connection_setup_ie(), i + 1, ts);
                if i == 109 {
                    // Should trigger ratio alert on 4th setup at packet 110
                    assert!(
                        ev.is_some(),
                        "4th setup at packet 110 should trigger ratio alert"
                    );
                    let event = ev.unwrap();
                    assert_eq!(event.event_type, EventType::Medium);
                }
            } else {
                a.analyze_information_element(&InformationElement::GSM, i + 1, ts);
            }
        }
    }
}
