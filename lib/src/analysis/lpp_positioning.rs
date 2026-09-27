use chrono::{DateTime, FixedOffset};

use pycrate_rs::nas::NASMessage;
use pycrate_rs::nas::emm::EMMMessage;
use pycrate_rs::nas::generated::emm::emmdl_generic_nas_transport::GenericContTypeGenericContType as DlGenericContType;
use pycrate_rs::nas::generated::emm::emmul_generic_nas_transport::GenericContTypeGenericContType as UlGenericContType;

use super::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use super::information_element::{InformationElement, LteInformationElement};

pub struct LppPositioningAnalyzer {
    dl_count: usize,
    ul_count: usize,
}

impl Default for LppPositioningAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LppPositioningAnalyzer {
    pub fn new() -> Self {
        Self {
            dl_count: 0,
            ul_count: 0,
        }
    }
}

impl Analyzer for LppPositioningAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "lpp_positioning".into(),
            default_enabled: false,
            name: "LTE Positioning Protocol (LPP)".into(),
            description: "Detects LTE Positioning Protocol (LPP) messages used for UE location \
                         tracking. LPP is used by the network to determine device location. \
                         Reports both downlink (network-to-UE) and uplink (UE-to-network) \
                         messages. Note: This is informational only and does not indicate \
                         malicious activity; legitimate networks use LPP for location services."
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
        let payload = match ie {
            InformationElement::LTE(inner) => match &**inner {
                LteInformationElement::NAS(payload) => payload,
                _ => return None,
            },
            _ => return None,
        };

        match payload {
            NASMessage::EMMMessage(EMMMessage::EMMDLGenericNASTransport(dl_msg)) => {
                if matches!(
                    dl_msg.generic_cont_type.inner,
                    DlGenericContType::LTEPositioningProtocolLPPMessageContainer
                ) {
                    self.dl_count += 1;
                    return Some(Event {
                        event_type: EventType::Informational,
                        message: format!(
                            "LTE Positioning Protocol (LPP) downlink message detected \
                             (request #{}, total UL: {})",
                            self.dl_count, self.ul_count
                        ),
                        analyzer_index: 0,
                    });
                }
            }
            NASMessage::EMMMessage(EMMMessage::EMMULGenericNASTransport(ul_msg)) => {
                if matches!(
                    ul_msg.generic_cont_type.inner,
                    UlGenericContType::LTEPositioningProtocolLPPMessageContainer
                ) {
                    self.ul_count += 1;
                    return Some(Event {
                        event_type: EventType::Informational,
                        message: format!(
                            "LTE Positioning Protocol (LPP) uplink message detected \
                             (response #{}, total DL: {})",
                            self.ul_count, self.dl_count
                        ),
                        analyzer_index: 0,
                    });
                }
            }
            _ => {}
        }

        None
    }
}
