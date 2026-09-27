use crate::diag::Message;
use crate::diag::diaglog::{LogBody, Nas4GMessageDirection, Timestamp};
use crate::gsmtap::mac::mac_subpacket_to_gsmtap;
use crate::gsmtap::{
    GsmtapHeader, GsmtapMessage, GsmtapType, LteNasSubtype, LteRrcSubtype, UmSubtype,
    UmtsRrcSubtype,
};

use log::{debug, warn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GsmtapParserError {
    #[error("Invalid LteRrcOtaMessage ext header version {0}")]
    InvalidLteRrcOtaExtHeaderVersion(u8),
    #[error("Invalid LteRrcOtaMessage header/PDU number combination: {0}/{1}")]
    InvalidLteRrcOtaHeaderPduNum(u8, u8),
    #[error("Invalid LteMacRachResponse packet: {0}")]
    InvalidLteMacRachResponse(String),
}

pub fn parse(msg: Message) -> Result<Option<(Timestamp, GsmtapMessage)>, GsmtapParserError> {
    if let Message::Log {
        timestamp, body, ..
    } = msg
    {
        match log_to_gsmtap(body)? {
            Some(msg) => Ok(Some((timestamp, msg))),
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn log_to_gsmtap(value: LogBody) -> Result<Option<GsmtapMessage>, GsmtapParserError> {
    match value {
        LogBody::LteRrcOtaMessage {
            ext_header_version,
            packet,
        } => {
            let gsmtap_type = match ext_header_version {
                0x02 | 0x03 | 0x04 | 0x06 | 0x07 | 0x08 | 0x0d | 0x16 => match packet.get_pdu_num()
                {
                    1 => GsmtapType::LteRrc(LteRrcSubtype::BcchBch),
                    2 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSch),
                    3 => GsmtapType::LteRrc(LteRrcSubtype::MCCH),
                    4 => GsmtapType::LteRrc(LteRrcSubtype::PCCH),
                    5 => GsmtapType::LteRrc(LteRrcSubtype::DlCcch),
                    6 => GsmtapType::LteRrc(LteRrcSubtype::DlDcch),
                    7 => GsmtapType::LteRrc(LteRrcSubtype::UlCcch),
                    8 => GsmtapType::LteRrc(LteRrcSubtype::UlDcch),
                    pdu => {
                        return Err(GsmtapParserError::InvalidLteRrcOtaHeaderPduNum(
                            ext_header_version,
                            pdu,
                        ));
                    }
                },
                0x09 | 0x0c => match packet.get_pdu_num() {
                    8 => GsmtapType::LteRrc(LteRrcSubtype::BcchBch),
                    9 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSch),
                    10 => GsmtapType::LteRrc(LteRrcSubtype::MCCH),
                    11 => GsmtapType::LteRrc(LteRrcSubtype::PCCH),
                    12 => GsmtapType::LteRrc(LteRrcSubtype::DlCcch),
                    13 => GsmtapType::LteRrc(LteRrcSubtype::DlDcch),
                    14 => GsmtapType::LteRrc(LteRrcSubtype::UlCcch),
                    15 => GsmtapType::LteRrc(LteRrcSubtype::UlDcch),
                    pdu => {
                        return Err(GsmtapParserError::InvalidLteRrcOtaHeaderPduNum(
                            ext_header_version,
                            pdu,
                        ));
                    }
                },
                0x0e..=0x10 => match packet.get_pdu_num() {
                    1 => GsmtapType::LteRrc(LteRrcSubtype::BcchBch),
                    2 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSch),
                    4 => GsmtapType::LteRrc(LteRrcSubtype::MCCH),
                    5 => GsmtapType::LteRrc(LteRrcSubtype::PCCH),
                    6 => GsmtapType::LteRrc(LteRrcSubtype::DlCcch),
                    7 => GsmtapType::LteRrc(LteRrcSubtype::DlDcch),
                    8 => GsmtapType::LteRrc(LteRrcSubtype::UlCcch),
                    9 => GsmtapType::LteRrc(LteRrcSubtype::UlDcch),
                    pdu => {
                        return Err(GsmtapParserError::InvalidLteRrcOtaHeaderPduNum(
                            ext_header_version,
                            pdu,
                        ));
                    }
                },
                0x13 | 0x1a | 0x1b => match packet.get_pdu_num() {
                    1 => GsmtapType::LteRrc(LteRrcSubtype::BcchBch),
                    3 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSch),
                    6 => GsmtapType::LteRrc(LteRrcSubtype::MCCH),
                    7 => GsmtapType::LteRrc(LteRrcSubtype::PCCH),
                    8 => GsmtapType::LteRrc(LteRrcSubtype::DlCcch),
                    9 => GsmtapType::LteRrc(LteRrcSubtype::DlDcch),
                    10 => GsmtapType::LteRrc(LteRrcSubtype::UlCcch),
                    11 => GsmtapType::LteRrc(LteRrcSubtype::UlDcch),
                    45 => GsmtapType::LteRrc(LteRrcSubtype::BcchBchNb),
                    46 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSchNb),
                    47 => GsmtapType::LteRrc(LteRrcSubtype::PcchNb),
                    48 => GsmtapType::LteRrc(LteRrcSubtype::DlCcchNb),
                    49 => GsmtapType::LteRrc(LteRrcSubtype::DlDcchNb),
                    50 => GsmtapType::LteRrc(LteRrcSubtype::UlCcchNb),
                    52 => GsmtapType::LteRrc(LteRrcSubtype::UlDcchNb),
                    pdu => {
                        return Err(GsmtapParserError::InvalidLteRrcOtaHeaderPduNum(
                            ext_header_version,
                            pdu,
                        ));
                    }
                },
                0x14 | 0x18 | 0x19 => match packet.get_pdu_num() {
                    1 => GsmtapType::LteRrc(LteRrcSubtype::BcchBch),
                    2 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSch),
                    4 => GsmtapType::LteRrc(LteRrcSubtype::MCCH),
                    5 => GsmtapType::LteRrc(LteRrcSubtype::PCCH),
                    6 => GsmtapType::LteRrc(LteRrcSubtype::DlCcch),
                    7 => GsmtapType::LteRrc(LteRrcSubtype::DlDcch),
                    8 => GsmtapType::LteRrc(LteRrcSubtype::UlCcch),
                    9 => GsmtapType::LteRrc(LteRrcSubtype::UlDcch),
                    54 => GsmtapType::LteRrc(LteRrcSubtype::BcchBchNb),
                    55 => GsmtapType::LteRrc(LteRrcSubtype::BcchDlSchNb),
                    56 => GsmtapType::LteRrc(LteRrcSubtype::PcchNb),
                    57 => GsmtapType::LteRrc(LteRrcSubtype::DlCcchNb),
                    58 => GsmtapType::LteRrc(LteRrcSubtype::DlDcchNb),
                    59 => GsmtapType::LteRrc(LteRrcSubtype::UlCcchNb),
                    61 => GsmtapType::LteRrc(LteRrcSubtype::UlDcchNb),
                    pdu => {
                        return Err(GsmtapParserError::InvalidLteRrcOtaHeaderPduNum(
                            ext_header_version,
                            pdu,
                        ));
                    }
                },
                _ => {
                    return Err(GsmtapParserError::InvalidLteRrcOtaExtHeaderVersion(
                        ext_header_version,
                    ));
                }
            };
            let mut header = GsmtapHeader::new(gsmtap_type);
            header.arfcn = (packet.get_earfcn() as u16) & 0x3FFF;
            header.frame_number = packet.get_sfn();
            header.subslot = packet.get_subfn();
            Ok(Some(GsmtapMessage {
                header,
                payload: packet.take_payload(),
            }))
        }
        LogBody::Nas4GMessage { msg, direction, .. } => {
            // currently we only handle "plain" (i.e. non-secure) NAS messages
            let mut header = GsmtapHeader::new(GsmtapType::LteNas(LteNasSubtype::Plain));
            header.uplink = matches!(direction, Nas4GMessageDirection::Uplink);
            Ok(Some(GsmtapMessage {
                header,
                payload: msg,
            }))
        }
        LogBody::LteMacRachResponse { packet } => {
            if packet.subpackets.len() > 1 {
                warn!(
                    "expected 1 MAC subpacket for LogBody::LteMacRachResponse, but got {}! ignoring all but the first",
                    packet.subpackets.len()
                );
            }
            let Some(subpacket) = packet.subpackets.first() else {
                return Err(GsmtapParserError::InvalidLteMacRachResponse(
                    "no subpackets".to_string(),
                ));
            };
            mac_subpacket_to_gsmtap(&subpacket.body).map_err(|err| {
                GsmtapParserError::InvalidLteMacRachResponse(format!(
                    "unable to serialize GSMTAP payload: {err:?}"
                ))
            })
        }
        // Qualcomm 0x412F WCDMA RRC signalling.
        // channel_type mapping derived from SCAT diagwcdmalogparser.py.
        // channel_type >= 0x80 is a "new format" record whose payload includes
        // 4 extra header bytes (UARFCN + PSC) that deku reads into msg; we drop
        // those rather than emit misaligned PDUs.
        LogBody::WcdmaSignallingMessage {
            channel_type, msg, ..
        } => {
            if channel_type >= 0x80 {
                debug!(
                    "gsmtap_sink: dropping new-format WCDMA record (channel_type=0x{channel_type:02x})"
                );
                return Ok(None);
            }
            let (subtype, uplink) = match channel_type {
                0x00 => (UmtsRrcSubtype::UlCcch, true),
                0x01 => (UmtsRrcSubtype::UlDcch, true),
                0x02 => (UmtsRrcSubtype::DlCcch, false),
                0x03 => (UmtsRrcSubtype::DlDcch, false),
                0x04 => (UmtsRrcSubtype::BcchBch, false),
                0x05 => (UmtsRrcSubtype::BcchFach, false),
                0x06 => (UmtsRrcSubtype::Pcch, false),
                0x07 => (UmtsRrcSubtype::Mcch, false),
                0x08 => (UmtsRrcSubtype::Msch, false),
                0x0a => (UmtsRrcSubtype::SystemInformationContainer, false),
                _ => {
                    debug!(
                        "gsmtap_sink: dropping WCDMA record with unhandled channel_type=0x{channel_type:02x}"
                    );
                    return Ok(None);
                }
            };
            let mut header = GsmtapHeader::new(GsmtapType::UmtsRrc(subtype));
            header.uplink = uplink;
            Ok(Some(GsmtapMessage {
                header,
                payload: msg,
            }))
        }
        // Qualcomm 0x512F GSM RR signalling.
        // channel_type mapping derived from SCAT diaggsmlogparser.py.
        // Bit 7 of channel_type encodes direction; lower 7 bits are channel class.
        // BCCH/CCCH carry bare L3; SDCCH/SACCH need a LAPDm header prepended so
        // Wireshark can dispatch to the correct dissector.
        // SACCH maps to UmSubtype::Sdcch8 (the GSMTAP ACCH bit 0x80 is not
        // representable by the current UmSubtype enum and is left for a follow-up).
        LogBody::GsmRrSignallingMessage {
            channel_type,
            length,
            msg,
            ..
        } => {
            let uplink = (channel_type & 0x80) != 0;
            let ch = channel_type & 0x7F;
            let (subtype, payload) = match ch {
                // BCCH — bare L3, no LAPDm wrapper
                0x01 => (UmSubtype::Bcch, msg),
                // RACH — bare burst
                0x02 => (UmSubtype::Rach, msg),
                // CCCH — bare L3
                0x03 => (UmSubtype::Ccch, msg),
                // SDCCH — prepend LAPDm UI header
                0x00 => {
                    // Widen to u16 to avoid debug-mode overflow; GSM frames are ≤23 bytes.
                    let len_byte = ((u16::from(length) << 2) | 0x01) as u8;
                    let mut buf = Vec::with_capacity(msg.len() + 3);
                    buf.extend_from_slice(&[0x01, 0x03, len_byte]);
                    buf.extend_from_slice(&msg);
                    (UmSubtype::Sdcch8, buf)
                }
                // SACCH — prepend SACCH L1 header + LAPDm UI header
                0x04 => {
                    let len_byte = ((u16::from(length) << 2) | 0x01) as u8;
                    let mut buf = Vec::with_capacity(msg.len() + 5);
                    buf.extend_from_slice(&[0x00, 0x00, 0x01, 0x03, len_byte]);
                    buf.extend_from_slice(&msg);
                    (UmSubtype::Sdcch8, buf)
                }
                _ => {
                    debug!(
                        "gsmtap_sink: dropping GSM record with unhandled channel_type=0x{channel_type:02x}"
                    );
                    return Ok(None);
                }
            };
            let mut header = GsmtapHeader::new(GsmtapType::Um(subtype));
            header.uplink = uplink;
            Ok(Some(GsmtapMessage { header, payload }))
        }
        _ => {
            debug!("gsmtap_sink: ignoring unhandled log type: {value:?}");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gsmtap::GsmtapType;
    use deku::DekuContainerWrite;

    #[test]
    fn test_arfcn_exceeding_14_bits_does_not_panic() {
        let mut header = GsmtapHeader::new(GsmtapType::LteRrc(LteRrcSubtype::DlDcch));
        // EARFCN 54540 (band 46) exceeds 14-bit max of 16383
        let large_earfcn: u32 = 54540;
        header.arfcn = (large_earfcn as u16) & 0x3FFF;
        let msg = GsmtapMessage {
            header,
            payload: vec![0x00],
        };
        // This would panic before the fix with "bit size of input is larger than bit requested size"
        assert!(msg.to_bytes().is_ok());
    }

    #[test]
    fn test_wcdma_ul_dcch_channel_type() {
        // channel_type 0x01 = UL_DCCH per SCAT diagwcdmalogparser.py
        let body = LogBody::WcdmaSignallingMessage {
            channel_type: 0x01,
            radio_bearer: 0,
            length: 3,
            msg: vec![0xAA, 0xBB, 0xCC],
        };
        let result = log_to_gsmtap(body).unwrap();
        let gsmtap = result.expect("should produce a GSMTAP message");
        assert_eq!(
            gsmtap.header.gsmtap_type,
            GsmtapType::UmtsRrc(UmtsRrcSubtype::UlDcch)
        );
        assert!(gsmtap.header.uplink);
        assert_eq!(gsmtap.payload, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_wcdma_dl_dcch_channel_type() {
        // channel_type 0x03 = DL_DCCH per SCAT
        let body = LogBody::WcdmaSignallingMessage {
            channel_type: 0x03,
            radio_bearer: 0,
            length: 2,
            msg: vec![0x01, 0x02],
        };
        let result = log_to_gsmtap(body).unwrap();
        let gsmtap = result.expect("should produce a GSMTAP message");
        assert_eq!(
            gsmtap.header.gsmtap_type,
            GsmtapType::UmtsRrc(UmtsRrcSubtype::DlDcch)
        );
        assert!(!gsmtap.header.uplink);
    }

    #[test]
    fn test_wcdma_new_format_dropped() {
        // channel_type >= 0x80: new-format record with extra header bytes in payload
        let body = LogBody::WcdmaSignallingMessage {
            channel_type: 0x83,
            radio_bearer: 0,
            length: 4,
            msg: vec![0x00, 0x00, 0x01, 0x02], // first 4 bytes would be arfcn/psc
        };
        let result = log_to_gsmtap(body).unwrap();
        assert!(result.is_none(), "new-format records must be dropped");
    }

    #[test]
    fn test_gsm_bcch_no_lapdm_prepend() {
        // channel_type 0x01 = BCCH; no L2 framing, bare L3 payload
        let body = LogBody::GsmRrSignallingMessage {
            channel_type: 0x01,
            message_type: 0,
            length: 3,
            msg: vec![0x41, 0x42, 0x43],
        };
        let result = log_to_gsmtap(body).unwrap();
        let gsmtap = result.expect("should produce a GSMTAP message");
        assert_eq!(gsmtap.header.gsmtap_type, GsmtapType::Um(UmSubtype::Bcch));
        assert!(!gsmtap.header.uplink);
        assert_eq!(gsmtap.payload, vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn test_gsm_sdcch_lapdm_prepend() {
        // channel_type 0x00 = SDCCH; LAPDm UI header prepended
        let payload = vec![0x59, 0x01, 0x5A];
        let length = payload.len() as u8;
        let body = LogBody::GsmRrSignallingMessage {
            channel_type: 0x00,
            message_type: 0,
            length,
            msg: payload.clone(),
        };
        let result = log_to_gsmtap(body).unwrap();
        let gsmtap = result.expect("should produce a GSMTAP message");
        assert_eq!(gsmtap.header.gsmtap_type, GsmtapType::Um(UmSubtype::Sdcch8));
        // LAPDm UI header: addr=0x01, ctrl=0x03, len=(3<<2)|0x01=0x0D
        let expected_len_byte = ((u16::from(length) << 2) | 0x01) as u8;
        assert_eq!(gsmtap.payload[0], 0x01);
        assert_eq!(gsmtap.payload[1], 0x03);
        assert_eq!(gsmtap.payload[2], expected_len_byte);
        assert_eq!(&gsmtap.payload[3..], payload.as_slice());
    }

    #[test]
    fn test_gsm_uplink_direction_bit() {
        // bit 7 set on channel_type encodes uplink direction
        let body = LogBody::GsmRrSignallingMessage {
            channel_type: 0x80 | 0x03, // CCCH, uplink
            message_type: 0,
            length: 1,
            msg: vec![0xFF],
        };
        let result = log_to_gsmtap(body).unwrap();
        let gsmtap = result.expect("should produce a GSMTAP message");
        assert_eq!(gsmtap.header.gsmtap_type, GsmtapType::Um(UmSubtype::Ccch));
        assert!(gsmtap.header.uplink);
    }
}
