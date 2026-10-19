use chrono::{DateTime, FixedOffset};
use std::collections::HashSet;

use super::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use super::information_element::{InformationElement, LteInformationElement};
use telcom_parser::lte_rrc::{
    BCCH_DL_SCH_MessageType, BCCH_DL_SCH_MessageType_c1, DL_DCCH_MessageType,
    DL_DCCH_MessageType_c1, RRCConnectionReleaseCriticalExtensions,
    RRCConnectionReleaseCriticalExtensions_c1, RedirectedCarrierInfo,
    SystemInformation_r8_IEsSib_TypeAndInfo_Entry, SystemInformationCriticalExtensions,
};

// Based on HITBSecConf presentation "Forcing a targeted LTE cellphone into an
// eavesdropping network" by Lin Huang
pub struct ConnectionRedirect2GDowngradeAnalyzer {
    /// GERAN starting ARFCNs seen in SIB7 neighbor list broadcasts.
    /// A redirect to an ARFCN in this set was pre-announced and is less suspicious
    /// than a redirect to an unadvertised frequency.
    announced_geran_arfcns: HashSet<u16>,
}

impl Default for ConnectionRedirect2GDowngradeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionRedirect2GDowngradeAnalyzer {
    pub fn new() -> Self {
        Self {
            announced_geran_arfcns: HashSet::new(),
        }
    }

    fn record_sib7_neighbors(&mut self, ie: &InformationElement) {
        let lte_ie = match ie {
            InformationElement::LTE(inner) => inner,
            _ => return,
        };
        let bcch_msg = match lte_ie.as_ref() {
            LteInformationElement::BcchDlSch(m) => m,
            _ => return,
        };
        let c1 = match &bcch_msg.message {
            BCCH_DL_SCH_MessageType::C1(c1) => c1,
            _ => return,
        };
        let sib = match c1 {
            BCCH_DL_SCH_MessageType_c1::SystemInformation(si) => si,
            _ => return,
        };
        let sib_list = match &sib.critical_extensions {
            SystemInformationCriticalExtensions::SystemInformation_r8(r8) => &r8.sib_type_and_info,
            _ => return,
        };
        for entry in &sib_list.0 {
            if let SystemInformation_r8_IEsSib_TypeAndInfo_Entry::Sib7(sib7) = entry
                && let Some(freq_list) = &sib7.carrier_freqs_info_list
            {
                for info in &freq_list.0 {
                    self.announced_geran_arfcns
                        .insert(info.carrier_freqs.starting_arfcn.0);
                }
            }
        }
    }
}

impl Analyzer for ConnectionRedirect2GDowngradeAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "connection_redirect_2g_downgrade".into(),
            default_enabled: true,
            name: "Connection Release/Redirected Carrier 2G Downgrade".into(),
            description: "Tests if a cell releases our connection and redirects us to a 2G cell. \
                Cross-references the redirect target against the SIB7 neighbor list: a redirect to \
                an unadvertised GERAN frequency is HIGH severity; one to a pre-announced neighbor \
                is MEDIUM (legitimate handoffs should be pre-advertised)."
                .into(),
            version: 2,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        _timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        // Always update neighbor list from SIB7 broadcasts first
        self.record_sib7_neighbors(ie);

        if let InformationElement::LTE(lte_ie) = ie
            && let LteInformationElement::DlDcch(msg_cont) = &**lte_ie
            && let DL_DCCH_MessageType::C1(c1) = &msg_cont.message
            && let DL_DCCH_MessageType_c1::RrcConnectionRelease(release) = c1
            && let RRCConnectionReleaseCriticalExtensions::C1(c1) = &release.critical_extensions
            && let RRCConnectionReleaseCriticalExtensions_c1::RrcConnectionRelease_r8(r8_ies) = c1
            && let Some(carrier_info) = &r8_ies.redirected_carrier_info
        {
            match carrier_info {
                RedirectedCarrierInfo::Geran(carrier_freqs) => {
                    let arfcn = carrier_freqs.starting_arfcn.0;
                    if self.announced_geran_arfcns.contains(&arfcn) {
                        Some(Event {
                            event_type: EventType::Medium,
                            message: format!(
                                "2G downgrade redirect to ARFCN {arfcn} (pre-announced in SIB7 \
                                 neighbor list — may be legitimate inter-RAT handoff)"
                            ),
                            analyzer_index: 0,
                        })
                    } else {
                        Some(Event {
                            event_type: EventType::High,
                            message: format!(
                                "2G downgrade redirect to unadvertised ARFCN {arfcn} \
                                 (not in SIB7 neighbor list — strong IMSI catcher indicator)"
                            ),
                            analyzer_index: 0,
                        })
                    }
                }
                _ => Some(Event {
                    event_type: EventType::Informational,
                    message: format!("RRCConnectionRelease CarrierInfo: {carrier_info:?}"),
                    analyzer_index: 0,
                }),
            }
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

    #[test]
    fn non_lte_returns_none() {
        let mut a = ConnectionRedirect2GDowngradeAnalyzer::new();
        assert!(
            a.analyze_information_element(&InformationElement::GSM, 0, ts())
                .is_none()
        );
    }
}
