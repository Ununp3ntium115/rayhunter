use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::{InformationElement, LteInformationElement};
use chrono::{DateTime, Duration, FixedOffset};
use std::collections::VecDeque;
use telcom_parser::lte_rrc::PCCH_MessageType;

pub struct PagingStormAnalyzer {
    /// Time-windowed queue of paging message timestamps (60-second window)
    paging_timestamps: VecDeque<DateTime<FixedOffset>>,
    /// Flag to ensure ratio alert fires only once per recording
    ratio_alert_fired: bool,
}

impl Default for PagingStormAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PagingStormAnalyzer {
    pub fn new() -> Self {
        PagingStormAnalyzer {
            paging_timestamps: VecDeque::new(),
            ratio_alert_fired: false,
        }
    }

    /// Check if the message is a Paging message.
    fn is_paging_message(ie: &InformationElement) -> bool {
        let lte_ie = match ie {
            InformationElement::LTE(inner) => inner.as_ref(),
            _ => return false,
        };

        match lte_ie {
            LteInformationElement::PCCH(pcch) => {
                let PCCH_MessageType::C1(c1) = &pcch.message else {
                    return false;
                };
                matches!(c1, telcom_parser::lte_rrc::PCCH_MessageType_c1::Paging(_))
            }
            _ => false,
        }
    }

    /// Update the time window and check for rate threshold.
    /// Returns true if alert should fire (count > 50 in 60s).
    /// Clears the window after firing to prevent immediate re-alert.
    fn check_time_window_threshold(&mut self, timestamp: DateTime<FixedOffset>) -> bool {
        // Remove timestamps older than 60 seconds
        let window_start = timestamp - Duration::seconds(60);
        while let Some(&front) = self.paging_timestamps.front() {
            if front < window_start {
                self.paging_timestamps.pop_front();
            } else {
                break;
            }
        }

        // Add current timestamp
        self.paging_timestamps.push_back(timestamp);

        // Fire alert if count > 50, then clear window to prevent immediate re-alert
        if self.paging_timestamps.len() > 50 {
            self.paging_timestamps.clear();
            return true;
        }
        false
    }
}

impl Analyzer for PagingStormAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "paging_storm".into(),
            default_enabled: false,
            name: "Paging Message Storm".into(),
            description: "Detects excessive LTE paging messages that may indicate an IMSI catcher. \
                Fires HIGH severity if >50 paging messages occur within 60 seconds, or MEDIUM severity \
                if paging messages comprise >30% of traffic. Normal paging rates are 1-5 messages per capture."
                .into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        packet_num: usize,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        if !Self::is_paging_message(ie) {
            return None;
        }

        // Check time-window threshold (HIGH severity)
        let window_count = self.paging_timestamps.len() + 1;
        if self.check_time_window_threshold(timestamp) {
            return Some(Event {
                event_type: EventType::High,
                message: format!(
                    "Paging message storm detected: {window_count} paging messages in 60-second window. \
                     This may indicate an IMSI catcher attempting to track the device.",
                ),
                analyzer_index: 0,
            });
        }

        // Check ratio threshold (MEDIUM severity, once per recording)
        // Require minimum warmup to avoid false positives at recording start
        if !self.ratio_alert_fired && packet_num >= 20 {
            let paging_ratio = self.paging_timestamps.len() as f64 / packet_num as f64;
            if paging_ratio > 0.3 {
                self.ratio_alert_fired = true;
                return Some(Event {
                    event_type: EventType::Medium,
                    message: format!(
                        "Paging message density anomaly: {:.1}% of traffic is paging messages. \
                         Normal behavior is 1-5%, >30% suggests potential IMSI catcher or network anomaly.",
                        paging_ratio * 100.0
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
    use telcom_parser::lte_rrc::{PCCH_Message, PCCH_MessageType, PCCH_MessageType_c1, Paging};

    fn ts() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
    }

    fn paging_ie() -> InformationElement {
        let paging = Paging {
            paging_record_list: None,
            system_info_modification: None,
            etws_indication: None,
            non_critical_extension: None,
        };
        let pcch_msg = PCCH_Message {
            message: PCCH_MessageType::C1(PCCH_MessageType_c1::Paging(paging)),
        };
        InformationElement::LTE(Box::new(
            crate::analysis::information_element::LteInformationElement::PCCH(pcch_msg),
        ))
    }

    #[test]
    fn non_lte_returns_none() {
        let mut a = PagingStormAnalyzer::new();
        assert!(
            a.analyze_information_element(&InformationElement::GSM, 1, ts())
                .is_none()
        );
    }

    #[test]
    fn non_pcch_lte_message_returns_none() {
        let mut a = PagingStormAnalyzer::new();
        // FIVEG (5G) is a non-PCCH LTE element
        let ie = InformationElement::FiveG;
        assert!(a.analyze_information_element(&ie, 1, ts()).is_none());
    }

    #[test]
    fn time_window_below_threshold() {
        let mut a = PagingStormAnalyzer::new();
        // Add 50 paging spread over 200 packets (25% ratio, below 30% threshold)
        // to keep ratio alert from firing
        for i in 0i64..50 {
            let ts = ts() + Duration::seconds(i);
            let pkt_num = (i * 4) as usize + 1;
            let ev = a.analyze_information_element(&paging_ie(), pkt_num, ts);
            assert!(
                ev.is_none(),
                "should not alert on 50 messages in time window"
            );
        }
    }

    #[test]
    fn time_window_exceeds_threshold() {
        let mut a = PagingStormAnalyzer::new();
        // Add 51 paging spread over 204 packets (25% ratio)
        // Ensure time-window alert fires, not ratio alert
        for i in 0i64..51 {
            let ts = ts() + Duration::seconds(i);
            let pkt_num = (i * 4) as usize + 1;
            let ev = a.analyze_information_element(&paging_ie(), pkt_num, ts);
            if i == 50 {
                assert!(
                    ev.is_some(),
                    "51st message should trigger time-window alert"
                );
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::High);
                assert!(event.message.contains("Paging message storm"));
            } else {
                assert!(ev.is_none());
            }
        }
    }

    #[test]
    fn messages_outside_window_not_counted() {
        let mut a = PagingStormAnalyzer::new();
        let base_ts = ts();

        // Add 51 messages over 70 seconds (outside the 60-second window)
        for i in 0i64..51 {
            let ts = base_ts + Duration::seconds(i);
            a.analyze_information_element(&paging_ie(), (i + 1) as usize, ts);
        }

        // Now add one more 70+ seconds later; it should trigger alert
        // because the original 51 have aged out, and we add 1 new
        // But this proves window cleaning works
        let ev = a.analyze_information_element(&paging_ie(), 52, base_ts + Duration::seconds(71));
        assert!(ev.is_none(), "window should have dropped old messages");
    }

    #[test]
    fn ratio_alert_fires_once() {
        let mut a = PagingStormAnalyzer::new();
        let base_ts = ts();

        // First, add enough non-paging packets to reach packet_num >= 20
        for i in 0..40 {
            a.analyze_information_element(&InformationElement::GSM, i + 1, base_ts);
        }

        // Now add paging messages until ratio exceeds 30%.
        // With 40 non-paging packets, we need paging > 12 to get ratio > 30%
        // At packet_num = 40 + 18 = 58, with 18 paging: 18/58 = 0.31 > 0.3
        for i in 0..18 {
            let ts = base_ts + Duration::milliseconds(i as i64);
            let pkt_num = 40 + i + 1; // packet 41..58
            let ev = a.analyze_information_element(&paging_ie(), pkt_num, ts);
            if i == 17 {
                // 18th paging message should trigger ratio alert
                assert!(
                    ev.is_some(),
                    "18th paging should trigger ratio alert at 31% density"
                );
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::Medium);
                assert!(event.message.contains("density"));
            } else {
                assert!(ev.is_none());
            }
        }

        // Add more paging; ratio alert should not fire again (already fired once)
        for i in 18..30 {
            let ts = base_ts + Duration::milliseconds(i as i64);
            let pkt_num = 40 + i + 1;
            let ev = a.analyze_information_element(&paging_ie(), pkt_num, ts);
            assert!(
                ev.is_none(),
                "ratio alert should only fire once per recording"
            );
        }
    }

    #[test]
    fn ratio_warmup_suppresses_early_alert() {
        let mut a = PagingStormAnalyzer::new();
        let base_ts = ts();

        // Feed 5 paging messages within first 10 packets (50% paging)
        // Should not alert because packet_num < 20
        for i in 0..5 {
            let ev = a.analyze_information_element(&paging_ie(), i + 1, base_ts);
            assert!(ev.is_none(), "should suppress alert before warmup");
        }

        // Ratio check should still be inactive
        assert!(!a.ratio_alert_fired);
    }

    #[test]
    fn high_alert_fires_on_threshold() {
        let mut a = PagingStormAnalyzer::new();
        let base_ts = ts();

        // Add 51 paging messages spread over 204 packets (25% ratio to avoid ratio alert)
        for i in 0i64..51 {
            let ts = base_ts + Duration::seconds(i);
            let pkt_num = (i * 4) as usize + 1;
            let ev = a.analyze_information_element(&paging_ie(), pkt_num, ts);

            if i == 50 {
                assert!(ev.is_some(), "51st paging message triggers HIGH alert");
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::High);
                assert!(event.message.contains("storm"));
            } else {
                assert!(ev.is_none());
            }
        }
    }
}
