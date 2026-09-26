use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::{InformationElement, LteInformationElement};
use chrono::{DateTime, FixedOffset};

const MIN_SAMPLES: u32 = 20;
const SIGMA_THRESHOLD: f64 = 2.5;

pub struct TimingAdvanceAnalyzer {
    count: u32,
    mean: f64,
    m2: f64, // Welford online variance accumulator
}

impl Default for TimingAdvanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TimingAdvanceAnalyzer {
    pub fn new() -> Self {
        TimingAdvanceAnalyzer {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    fn update(&mut self, ta: f64) -> bool {
        self.count += 1;
        let delta = ta - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = ta - self.mean;
        self.m2 += delta * delta2;

        if self.count < MIN_SAMPLES {
            return false;
        }
        let variance = self.m2 / (self.count - 1) as f64;
        let std_dev = variance.sqrt();
        if std_dev < 1.0 {
            // not enough spread yet to compute meaningful outliers
            return false;
        }
        (ta - self.mean).abs() > SIGMA_THRESHOLD * std_dev
    }
}

impl Analyzer for TimingAdvanceAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "timing_advance_outlier".into(),
            default_enabled: true,
            name: "Timing Advance Outlier".into(),
            description: "Flags LTE Timing Advance values that are statistical outliers \
                (>2.5σ from the session mean). An unusually small TA can indicate a \
                nearby IMSI catcher; an unusually large TA can indicate a tower spoofed \
                at an impossible distance. Requires 20 samples to warm up."
                .into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        _timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        let lte_ie = match ie {
            InformationElement::LTE(inner) => inner,
            _ => return None,
        };
        let ta = match lte_ie.as_ref() {
            LteInformationElement::LteLl1ServingCellTiming { ta } => *ta as f64,
            _ => return None,
        };
        if self.update(ta) {
            Some(Event {
                event_type: EventType::Medium,
                message: format!(
                    "Timing Advance outlier: TA={:.0} (mean={:.1}, σ={:.1}, n={}). \
                     Outlier TA may indicate an IMSI catcher positioned at an atypical distance.",
                    ta,
                    self.mean,
                    (self.m2 / (self.count - 1).max(1) as f64).sqrt(),
                    self.count,
                ),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
    }

    fn ta_ie(ta: u16) -> InformationElement {
        InformationElement::from_ll1_timing(ta)
    }

    #[test]
    fn non_lte_returns_none() {
        let mut a = TimingAdvanceAnalyzer::new();
        assert!(
            a.analyze_information_element(&InformationElement::GSM, 0, ts())
                .is_none()
        );
    }

    #[test]
    fn warmup_suppresses_events() {
        let mut a = TimingAdvanceAnalyzer::new();
        for _ in 0..(MIN_SAMPLES - 1) {
            assert!(a.analyze_information_element(&ta_ie(10), 0, ts()).is_none());
        }
    }

    #[test]
    fn outlier_detected_after_warmup() {
        let mut a = TimingAdvanceAnalyzer::new();
        // feed stable samples to build distribution
        for _ in 0..40 {
            a.analyze_information_element(&ta_ie(10), 0, ts());
        }
        // feed a large outlier
        let event = a.analyze_information_element(&ta_ie(200), 0, ts());
        assert!(event.is_some(), "outlier should trigger event");
        let ev = event.unwrap();
        assert_eq!(ev.event_type, EventType::Medium);
        assert!(ev.message.contains("TA=200"));
    }

    #[test]
    fn normal_variation_no_event() {
        let mut a = TimingAdvanceAnalyzer::new();
        // build tight distribution around 10
        for _ in 0..60 {
            a.analyze_information_element(&ta_ie(10), 0, ts());
        }
        // slight variation — should not trigger
        assert!(a.analyze_information_element(&ta_ie(11), 0, ts()).is_none());
    }
}
