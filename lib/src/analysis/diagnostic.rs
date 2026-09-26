use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::{InformationElement, LteInformationElement};
use chrono::{DateTime, FixedOffset};
use pycrate_rs::nas::NASMessage;
use pycrate_rs::nas::emm::EMMMessage;
use pycrate_rs::nas::generated::emm::emm_attach_reject::EMMCauseEMMCause as AttachRejectEMMCause;
use pycrate_rs::nas::generated::emm::emm_detach_request_mt::EPSDetachTypeMTType;
use pycrate_rs::nas::generated::emm::emm_service_reject::EMMCauseEMMCause as ServiceRejectEMMCause;
use pycrate_rs::nas::generated::emm::emm_tracking_area_update_reject::EMMCauseEMMCause as TAURejectEMMCause;

pub struct DiagnosticAnalyzer;

impl Default for DiagnosticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticAnalyzer {
    pub fn new() -> Self {
        DiagnosticAnalyzer
    }

    fn is_imsi_exposing_nas(&self, nas_msg: &NASMessage) -> bool {
        match nas_msg {
            NASMessage::EMMMessage(emm_msg) => match emm_msg {
                EMMMessage::EMMIdentityRequest(_) => true, // Alert on all identity requests (IMSI, IMEI, IMEISV)

                EMMMessage::EMMTrackingAreaUpdateReject(reject) => {
                    matches!(
                        reject.emm_cause.inner,
                        TAURejectEMMCause::IllegalUE
                            | TAURejectEMMCause::IllegalME
                            | TAURejectEMMCause::EPSServicesNotAllowed
                            | TAURejectEMMCause::EPSServicesAndNonEPSServicesNotAllowed
                            | TAURejectEMMCause::TrackingAreaNotAllowed
                            | TAURejectEMMCause::EPSServicesNotAllowedInThisPLMN
                            | TAURejectEMMCause::RequestedServiceOptionNotAuthorizedInThisPLMN
                    )
                }

                EMMMessage::EMMAttachReject(reject) => {
                    matches!(
                        reject.emm_cause.inner,
                        AttachRejectEMMCause::IllegalUE
                            | AttachRejectEMMCause::IllegalME
                            | AttachRejectEMMCause::EPSServicesNotAllowed
                            | AttachRejectEMMCause::EPSServicesAndNonEPSServicesNotAllowed
                            | AttachRejectEMMCause::PLMNNotAllowed
                            | AttachRejectEMMCause::TrackingAreaNotAllowed
                            | AttachRejectEMMCause::RoamingNotAllowedInThisTrackingArea
                            | AttachRejectEMMCause::EPSServicesNotAllowedInThisPLMN
                            | AttachRejectEMMCause::NoSuitableCellsInTrackingArea
                            | AttachRejectEMMCause::RequestedServiceOptionNotAuthorizedInThisPLMN
                    )
                }

                EMMMessage::EMMDetachRequestMT(req) => {
                    // Original implementation: !(nas_eps.emm.detach_type_dl == 3)
                    req.eps_detach_type.inner.typ != EPSDetachTypeMTType::IMSIDetach
                }

                EMMMessage::EMMAttachRequest(_) => {
                    // just because eps_attach_type is IMSI doesn't mean that the phoen transmitted its IMSI
                    // It often sends the GUTI instead. We could check the req.epsid structure but it appears to actually
                    // not be parsed. So for now we are just ignoreing this message
                    // req.eps_attach_type.inner == EPSAttachTypeV::CombinedEPSIMSIAttach

                    false
                }

                EMMMessage::EMMServiceReject(reject) => {
                    matches!(
                        reject.emm_cause.inner,
                        ServiceRejectEMMCause::IllegalUE
                            | ServiceRejectEMMCause::IllegalME
                            | ServiceRejectEMMCause::EPSServicesNotAllowed
                            | ServiceRejectEMMCause::UEIdentityCannotBeDerivedByTheNetwork
                            | ServiceRejectEMMCause::TrackingAreaNotAllowed
                            | ServiceRejectEMMCause::EPSServicesNotAllowedInThisPLMN
                            | ServiceRejectEMMCause::RequestedServiceOptionNotAuthorizedInThisPLMN
                    )
                }

                _ => false,
            },
            _ => false,
        }
    }
}

impl Analyzer for DiagnosticAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "diagnostic_analyzer".into(),
            default_enabled: true,
            name: "Diagnostic Information".into(),
            description:
                "Catches any messages that may lead to IMSI Exposure. Can be quite noisy. \
                Useful as a diagnostic for finding out why an IMSI was sent or what \
                the reason for a reject message was. Not a useful indicator on its own \
                but a helpful diagnostic for understanding why another indicator was \
                triggered. Based on the list of IMSI exposing messages identified in \
                the 'Marlin' paper."
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

        match lte_ie.as_ref() {
            LteInformationElement::NAS(nas_msg) => {
                if self.is_imsi_exposing_nas(nas_msg) {
                    let message_type = match nas_msg {
                        NASMessage::EMMMessage(emm_msg) => match emm_msg {
                            EMMMessage::EMMIdentityRequest(request) => {
                                format!("EMM Identity Request ({:?})", request.id_type.inner)
                            }
                            EMMMessage::EMMTrackingAreaUpdateReject(reject) => {
                                format!(
                                    "EMM Tracking Area Update Reject ({:?})",
                                    reject.emm_cause.inner
                                )
                            }
                            EMMMessage::EMMAttachReject(reject) => {
                                format!("EMM Attach Reject ({:?})", reject.emm_cause.inner)
                            }
                            EMMMessage::EMMDetachRequestMT(request) => {
                                format!(
                                    "EMM Detach Request ({:?}:{:?})",
                                    request.eps_detach_type.inner, request.emm_cause.inner
                                )
                            }
                            EMMMessage::EMMServiceReject(reject) => {
                                format!("EMM Service Reject ({:?})", reject.emm_cause.inner)
                            }
                            EMMMessage::EMMAttachRequest(request) => {
                                format!("EPS Attach Request ({:?})", request.epsid.inner)
                            }
                            _ => "Unknown EMM Message".to_string(),
                        },
                        _ => "Unknown NAS Message".to_string(),
                    };

                    Some(Event {
                        event_type: EventType::Informational,
                        message: format!("Diagnostic: {message_type}."),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gsmtap::{GsmtapHeader, GsmtapMessage, GsmtapType, LteNasSubtype};

    fn ts() -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
    }

    fn nas_ie(payload: &[u8]) -> InformationElement {
        InformationElement::try_from(&GsmtapMessage {
            header: GsmtapHeader::new(GsmtapType::LteNas(LteNasSubtype::Plain)),
            payload: payload.to_vec(),
        })
        .expect("failed to parse test NAS payload")
    }

    #[test]
    fn test_non_lte_returns_none() {
        let mut a = DiagnosticAnalyzer::new();
        let t = ts();
        assert!(a.analyze_information_element(&InformationElement::GSM, 0, t).is_none());
        assert!(a.analyze_information_element(&InformationElement::UMTS, 0, t).is_none());
        assert!(a.analyze_information_element(&InformationElement::FiveG, 0, t).is_none());
    }

    #[test]
    fn test_identity_request_fires() {
        // 0x07 = plain EPS MM, 0x55 = Identity Request, 0x01 = IMSI
        let mut a = DiagnosticAnalyzer::new();
        let event = a
            .analyze_information_element(&nas_ie(&[0x07, 0x55, 0x01]), 0, ts())
            .expect("identity request should trigger DiagnosticAnalyzer");
        assert_eq!(event.event_type, EventType::Informational);
        assert!(event.message.contains("Identity Request"));
    }

    #[test]
    fn test_attach_reject_illegal_ue_fires() {
        // 0x07 = plain EPS MM, 0x44 = Attach Reject, 0x03 = Illegal UE
        let mut a = DiagnosticAnalyzer::new();
        let event = a
            .analyze_information_element(&nas_ie(&[0x07, 0x44, 0x03]), 0, ts())
            .expect("attach reject with Illegal UE should trigger DiagnosticAnalyzer");
        assert_eq!(event.event_type, EventType::Informational);
        assert!(event.message.contains("Attach Reject"));
    }
}
