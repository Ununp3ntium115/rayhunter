use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::{InformationElement, LteInformationElement};
use chrono::{DateTime, Duration, FixedOffset};
use std::collections::VecDeque;
use telcom_parser::lte_rrc::{UL_DCCH_MessageType, UL_DCCH_MessageType_c1};

pub struct MeasurementReportProfilingAnalyzer {
    /// Time-windowed queue of MeasurementReport timestamps (120-second window for MEDIUM)
    report_timestamps_120s: VecDeque<DateTime<FixedOffset>>,
    /// Time-windowed queue of MeasurementReport timestamps (60-second window for HIGH)
    report_timestamps_60s: VecDeque<DateTime<FixedOffset>>,
    /// Timestamp of last HIGH alert to prevent rapid re-firing
    last_high_alert_timestamp: Option<DateTime<FixedOffset>>,
    /// Flag to ensure MEDIUM alert fires only once per recording
    medium_alert_fired: bool,
}

impl Default for MeasurementReportProfilingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl MeasurementReportProfilingAnalyzer {
    pub fn new() -> Self {
        MeasurementReportProfilingAnalyzer {
            report_timestamps_120s: VecDeque::new(),
            report_timestamps_60s: VecDeque::new(),
            last_high_alert_timestamp: None,
            medium_alert_fired: false,
        }
    }

    /// Check if the message is a MeasurementReport message.
    fn is_measurement_report(ie: &InformationElement) -> bool {
        let lte_ie = match ie {
            InformationElement::LTE(inner) => inner.as_ref(),
            _ => return false,
        };

        match lte_ie {
            LteInformationElement::UlDcch(ul_dcch) => {
                let UL_DCCH_MessageType::C1(c1) = &ul_dcch.message else {
                    return false;
                };
                matches!(c1, UL_DCCH_MessageType_c1::MeasurementReport(_))
            }
            _ => false,
        }
    }

    /// Update the time windows and check for rate thresholds.
    /// Returns Some((event_type, count, window_seconds)) if an alert should fire,
    /// or None otherwise.
    fn check_thresholds(
        &mut self,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<(EventType, usize, usize)> {
        // Remove timestamps older than 120 seconds from 120s window
        let window_120_start = timestamp - Duration::seconds(120);
        while let Some(&front) = self.report_timestamps_120s.front() {
            if front < window_120_start {
                self.report_timestamps_120s.pop_front();
            } else {
                break;
            }
        }

        // Remove timestamps older than 60 seconds from 60s window
        let window_60_start = timestamp - Duration::seconds(60);
        while let Some(&front) = self.report_timestamps_60s.front() {
            if front < window_60_start {
                self.report_timestamps_60s.pop_front();
            } else {
                break;
            }
        }

        // Add current timestamp to both windows
        self.report_timestamps_120s.push_back(timestamp);
        self.report_timestamps_60s.push_back(timestamp);

        // Check HIGH alert condition: ≥12 reports in 60s window
        // This represents a rate of 1 report per 5 seconds
        if self.report_timestamps_60s.len() >= 12 {
            // Fire HIGH alert if enough time has passed since last HIGH alert
            if self.last_high_alert_timestamp.is_none()
                || timestamp.signed_duration_since(self.last_high_alert_timestamp.unwrap())
                    >= Duration::seconds(60)
            {
                self.last_high_alert_timestamp = Some(timestamp);
                return Some((EventType::High, self.report_timestamps_60s.len(), 60));
            }
        }

        // Check MEDIUM alert condition: >5 (i.e., ≥6) reports in 120s window
        if !self.medium_alert_fired && self.report_timestamps_120s.len() > 5 {
            self.medium_alert_fired = true;
            return Some((EventType::Medium, self.report_timestamps_120s.len(), 120));
        }

        None
    }
}

impl Analyzer for MeasurementReportProfilingAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "measurement_report_profiling_detector".into(),
            default_enabled: true,
            name: "Measurement Report Profiling".into(),
            description: "Detects aggressive measurement report patterns indicating signal strength profiling \
                attacks. A fake cell may request frequent measurement reports to build UE location profiles and \
                enable handover spoofing or location tracking. Fires HIGH severity if measurement reports exceed \
                1 per 5 seconds (≥12 in 60s window), or MEDIUM severity if >5 reports occur within a 120-second \
                window. Normal behavior involves periodic reports every 10-40 seconds during active connection."
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
        if !Self::is_measurement_report(ie) {
            return None;
        }

        // Check both thresholds
        if let Some((event_type, count, window_seconds)) = self.check_thresholds(timestamp) {
            return Some(Event {
                event_type,
                message: format!(
                    "Measurement report profiling detected: {} reports in {}-second window. \
                     This pattern indicates aggressive signal profiling for location tracking or handover spoofing. \
                     Fake cells may exploit measurement reports to build UE location maps and enable cipher downgrades.",
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
        MeasurementReport, MeasurementReport_r8_IEs, MeasurementReportCriticalExtensions,
        MeasurementReportCriticalExtensions_c1, UL_DCCH_Message, UL_DCCH_MessageType,
        UL_DCCH_MessageType_c1,
    };

    fn ts() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
    }

    fn measurement_report_ie() -> InformationElement {
        use telcom_parser::lte_rrc::{
            MeasResults, MeasResultsMeasResultPCell, RSRP_Range, RSRQ_Range,
        };

        let meas_results = MeasResults {
            meas_id: telcom_parser::lte_rrc::MeasId(0),
            meas_result_p_cell: MeasResultsMeasResultPCell {
                rsrp_result: RSRP_Range(100),
                rsrq_result: RSRQ_Range(50),
            },
            meas_result_neigh_cells: None,
        };
        let r8_ies = MeasurementReport_r8_IEs {
            meas_results,
            non_critical_extension: None,
        };
        let report = MeasurementReport {
            critical_extensions: MeasurementReportCriticalExtensions::C1(
                MeasurementReportCriticalExtensions_c1::MeasurementReport_r8(r8_ies),
            ),
        };
        let ul_dcch_msg = UL_DCCH_Message {
            message: UL_DCCH_MessageType::C1(UL_DCCH_MessageType_c1::MeasurementReport(report)),
        };
        InformationElement::LTE(Box::new(
            crate::analysis::information_element::LteInformationElement::UlDcch(ul_dcch_msg),
        ))
    }

    #[test]
    fn non_lte_returns_none() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        assert!(
            a.analyze_information_element(&InformationElement::GSM, 1, ts())
                .is_none()
        );
    }

    #[test]
    fn non_measurement_report_returns_none() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        // Test with a different LTE IE type
        assert!(
            a.analyze_information_element(&InformationElement::FiveG, 1, ts())
                .is_none()
        );
    }

    #[test]
    fn accepts_measurement_report() {
        let ie = measurement_report_ie();
        assert!(MeasurementReportProfilingAnalyzer::is_measurement_report(
            &ie
        ));
    }

    #[test]
    fn single_report_returns_none() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let ev = a.analyze_information_element(&measurement_report_ie(), 1, ts());
        assert!(ev.is_none(), "single report should not alert");
    }

    #[test]
    fn five_reports_120s_returns_none() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        // Add 5 reports over 120s (at threshold for MEDIUM, not exceeding >5)
        for i in 0..5 {
            let ts = ts() + Duration::seconds(i as i64 * 24);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            assert!(ev.is_none(), "5 reports should not exceed >5 threshold");
        }
    }

    #[test]
    fn six_reports_120s_triggers_medium() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        // Add 6 reports over 120s (exceeds >5 threshold)
        for i in 0..6 {
            let ts = ts() + Duration::seconds(i as i64 * 20);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                assert!(ev.is_some(), "6th report should trigger MEDIUM alert");
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::Medium);
                assert!(event.message.contains("profiling"));
                assert!(event.message.contains("120-second"));
            } else {
                assert!(ev.is_none(), "reports 1-5 should not alert");
            }
        }
    }

    #[test]
    fn medium_alert_at_6_reports() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();
        // Add reports spaced 5 seconds apart
        for i in 0..6 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                // 6th report triggers MEDIUM (>5 in 120s)
                assert!(ev.is_some(), "6th report should trigger MEDIUM alert");
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::Medium);
                assert!(event.message.contains("profiling"));
                assert!(event.message.contains("120-second"));
            } else {
                assert!(ev.is_none(), "reports 1-5 should not alert");
            }
        }
    }

    #[test]
    fn high_alert_at_12_reports() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();
        // Add 12 reports spaced 5 seconds apart (55 seconds total, within 60s window)
        for i in 0..12 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                // 6th report triggers MEDIUM
                assert!(ev.is_some(), "6th report should trigger MEDIUM");
                assert_eq!(ev.unwrap().event_type, EventType::Medium);
            } else if i == 11 {
                // 12th report triggers HIGH (≥12 in 60s)
                assert!(ev.is_some(), "12th report should trigger HIGH alert");
                let event = ev.unwrap();
                assert_eq!(event.event_type, EventType::High);
                assert!(event.message.contains("profiling"));
                assert!(event.message.contains("60-second"));
            } else {
                assert!(ev.is_none(), "other reports should not alert");
            }
        }
    }

    #[test]
    fn high_alert_prevents_immediate_refire() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 12 reports within 60s to trigger MEDIUM then HIGH
        for i in 0..12 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                assert!(
                    ev.is_some() && ev.unwrap().event_type == EventType::Medium,
                    "6th should trigger MEDIUM"
                );
            } else if i == 11 {
                assert!(
                    ev.is_some() && ev.unwrap().event_type == EventType::High,
                    "12th should trigger HIGH"
                );
            } else {
                assert!(ev.is_none(), "other reports should not alert");
            }
        }

        // Add more reports within 60s of the last alert (which was at 55s)
        // Should not fire again since HIGH cooldown is 60s
        for i in 12..14 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            assert!(
                ev.is_none(),
                "No alert should fire: HIGH in cooldown, MEDIUM already fired"
            );
        }
    }

    #[test]
    fn high_alert_can_refire_after_60_seconds() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 12 reports within first 60s to trigger MEDIUM then HIGH
        for i in 0..12 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 11 {
                // HIGH fires at 55s
                assert!(ev.is_some() && ev.unwrap().event_type == EventType::High);
            }
        }

        // Advance to 200s (well past 60s cooldown and clears both windows)
        // Add a new burst of 12 reports starting at 200s, spaced 5s apart
        for i in 0..12 {
            let ts = base_ts + Duration::seconds(200 + i as i64 * 5);
            let ev =
                a.analyze_information_element(&measurement_report_ie(), (12 + i + 1) as usize, ts);
            if i == 5 {
                // MEDIUM would refire but it already fired, so no alert
                assert!(
                    ev.is_none(),
                    "MEDIUM already fired once per recording, should not alert again"
                );
            } else if i == 11 {
                // HIGH can refire after 60s cooldown (last was at 55s, this is at 255s)
                assert!(
                    ev.is_some() && ev.unwrap().event_type == EventType::High,
                    "HIGH should re-fire after 60s cooldown"
                );
            } else {
                assert!(ev.is_none(), "other reports should not alert");
            }
        }
    }

    #[test]
    fn medium_alert_fires_once_per_recording() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 6 reports to trigger MEDIUM alert
        for i in 0..6 {
            let ts = base_ts + Duration::seconds(i as i64 * 20);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                assert!(ev.is_some(), "6th report should trigger MEDIUM alert");
            }
        }

        // Add more reports; MEDIUM alert should not fire again
        for i in 6..10 {
            let ts = base_ts + Duration::seconds(i as i64 * 20);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            assert!(
                ev.is_none(),
                "MEDIUM alert should only fire once per recording"
            );
        }
    }

    #[test]
    fn medium_then_high_both_fire_in_sequence() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 12 reports spaced 5s apart: MEDIUM fires at i=5 (6th report),
        // HIGH fires at i=11 (12th report)
        let mut medium_fired = false;
        let mut high_fired = false;
        for i in 0..12 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                assert!(ev.is_some(), "6th report should trigger MEDIUM");
                assert_eq!(ev.unwrap().event_type, EventType::Medium);
                medium_fired = true;
            } else if i == 11 {
                assert!(ev.is_some(), "12th report should trigger HIGH");
                assert_eq!(ev.unwrap().event_type, EventType::High);
                high_fired = true;
            } else {
                assert!(ev.is_none(), "other reports should not alert");
            }
        }
        assert!(
            medium_fired && high_fired,
            "both MEDIUM and HIGH should have fired"
        );
    }

    #[test]
    fn reports_outside_120s_window_not_counted() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 6 reports over 120 seconds
        for i in 0..6 {
            let ts = base_ts + Duration::seconds(i as i64 * 20);
            a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
        }

        // Add one more report 121s later; only the new one remains + most recent from the tail
        // So we should not have >5 in 120s window
        let ev = a.analyze_information_element(
            &measurement_report_ie(),
            7,
            base_ts + Duration::seconds(125),
        );
        assert!(
            ev.is_none(),
            "old reports should have been pruned from 120s window"
        );
    }

    #[test]
    fn high_alert_with_real_pcap_pattern() {
        // Simulate the pattern from 1790413135.pcapng: 12 reports in 55s
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 12 reports within 55s (5s apart, typical aggressive profiling pattern)
        for i in 0..12 {
            let ts = base_ts + Duration::seconds(i as i64 * 5);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                // 6th report triggers MEDIUM (>5 in 120s window)
                assert!(
                    ev.is_some() && ev.unwrap().event_type == EventType::Medium,
                    "6th report should trigger MEDIUM"
                );
            } else if i == 11 {
                // 12th report triggers HIGH (≥12 in 60s window)
                assert!(
                    ev.is_some() && ev.unwrap().event_type == EventType::High,
                    "12th report should trigger HIGH"
                );
            } else {
                assert!(ev.is_none(), "other reports should not alert");
            }
        }
    }

    #[test]
    fn medium_alert_with_real_pcap_pattern() {
        // Simulate patterns from multiple captures: 5-8 reports in 120s
        let mut a = MeasurementReportProfilingAnalyzer::new();
        let base_ts = ts();

        // Add 8 reports over 119s (mimicking 1790459359.pcapng pattern)
        for i in 0..8 {
            let ts = base_ts + Duration::seconds(i as i64 * 15);
            let ev = a.analyze_information_element(&measurement_report_ie(), (i + 1) as usize, ts);
            if i == 5 {
                // 6th report should trigger MEDIUM alert
                assert!(ev.is_some(), "should fire MEDIUM alert on 6th report");
                assert_eq!(ev.unwrap().event_type, EventType::Medium);
            } else if i < 5 {
                assert!(ev.is_none(), "should not alert before 6th report");
            } else if i > 5 {
                assert!(ev.is_none(), "MEDIUM should only fire once");
            }
        }
    }

    #[test]
    fn non_lte_information_elements_ignored() {
        let mut a = MeasurementReportProfilingAnalyzer::new();
        // These should all be ignored and not affect detection
        for i in 0..5 {
            a.analyze_information_element(&InformationElement::GSM, i + 1, ts());
            a.analyze_information_element(&InformationElement::FiveG, i + 1, ts());
        }
        // Add one measurement report; should not alert since only 1
        let ev = a.analyze_information_element(&measurement_report_ie(), 6, ts());
        assert!(ev.is_none());
    }
}
