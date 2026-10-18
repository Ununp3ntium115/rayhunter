//! Diag protocol serialization/deserialization

use crc::{Algorithm, Crc};
use deku::prelude::*;

use crate::hdlc::{self, hdlc_decapsulate};
use log::warn;
use thiserror::Error;

pub mod diaglog;

use diaglog::{LogBody, Timestamp};

pub const MESSAGE_TERMINATOR: u8 = 0x7e;
pub const MESSAGE_ESCAPE_CHAR: u8 = 0x7d;

pub const ESCAPED_MESSAGE_TERMINATOR: u8 = 0x5e;
pub const ESCAPED_MESSAGE_ESCAPE_CHAR: u8 = 0x5d;

#[derive(Debug, Clone, DekuWrite)]
pub struct RequestContainer {
    pub data_type: DataType,
    #[deku(skip)]
    pub use_mdm: bool,
    #[deku(skip, cond = "!*use_mdm")]
    pub mdm_field: i32,
    pub hdlc_encapsulated_request: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, DekuWrite)]
#[deku(id_type = "u32")]
pub enum Request {
    #[deku(id = "115")]
    LogConfig(LogConfigRequest),
}

#[derive(Debug, Clone, PartialEq, DekuWrite)]
#[deku(id_type = "u32", endian = "little")]
pub enum LogConfigRequest {
    #[deku(id = "1")]
    RetrieveIdRanges,

    #[deku(id = "3")]
    SetMask {
        log_type: u32,
        log_mask_bitsize: u32,
        log_mask: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u32", endian = "little")]
pub enum DataType {
    #[deku(id = "32")]
    UserSpace,
    #[deku(id_pat = "_")]
    Other(u32),
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DiagParsingError {
    #[error("Failed to parse Message: {0}, data: {1:?}")]
    MessageParsingError(deku::DekuError, Vec<u8>),
    #[error("HDLC decapsulation of message failed: {0}, data: {1:?}")]
    HdlcDecapsulationError(hdlc::HdlcError, Vec<u8>),
}

impl DiagParsingError {
    /// A one-line description without the raw message bytes, small enough to store once per
    /// skipped message in an analysis report.
    pub fn summary(&self) -> String {
        match self {
            DiagParsingError::MessageParsingError(err, data) => {
                format!("Failed to parse Message ({} bytes): {err}", data.len())
            }
            DiagParsingError::HdlcDecapsulationError(err, data) => {
                format!(
                    "HDLC decapsulation of message failed ({} bytes): {err}",
                    data.len()
                )
            }
        }
    }
}

// this is sorta based on the params qcsuper uses, plus what seems to be used in
// https://github.com/fgsect/scat/blob/f1538b397721df3ab8ba12acd26716abcf21f78b/util.py#L47
pub const CRC_CCITT_ALG: Algorithm<u16> = Algorithm {
    poly: 0x1021,
    init: 0xffff,
    refin: true,
    refout: true,
    width: 16,
    xorout: 0xffff,
    check: 0x2189,
    residue: 0x0000,
};

pub const CRC_CCITT: Crc<u16> = Crc::<u16>::new(&CRC_CCITT_ALG);
#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct MessagesContainer {
    pub data_type: DataType,
    pub num_messages: u32,
    #[deku(count = "num_messages")]
    pub messages: Vec<HdlcEncapsulatedMessage>,
}

impl MessagesContainer {
    pub fn messages(&self) -> Vec<Result<Message, DiagParsingError>> {
        let mut result = Vec::new();
        for msg in &self.messages {
            for sub_msg in msg.data.split_inclusive(|&b| b == MESSAGE_TERMINATOR) {
                result.push(Message::from_hdlc(sub_msg));
            }
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct HdlcEncapsulatedMessage {
    pub len: u32,
    #[deku(count = "len")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum Message {
    #[deku(id = "16")]
    Log {
        pending_msgs: u8,
        outer_length: u16,
        inner_length: u16,
        log_type: u16,
        timestamp: Timestamp,
        // pass the log type and log length (inner_length - (sizeof(log_type) + sizeof(timestamp)))
        #[deku(ctx = "*log_type, inner_length.saturating_sub(12)")]
        body: LogBody,
    },

    // kinda unpleasant deku hackery here. deku expects an enum's variant to be
    // right before its data, but in this case, a status value comes between the
    // variants and the data. so we need to use deku's context (ctx) feature to
    // pass those opcodes down to their respective parsers.
    #[deku(id_pat = "_")]
    Response {
        opcode1: u8, // the "id" (from deku's POV) gets parsed into this field
        opcode2: u8,
        opcode3: u8,
        opcode4: u8,
        subopcode: u32,
        status: u32,
        #[deku(ctx = "u32::from_le_bytes([*opcode1, *opcode2, *opcode3, *opcode4]), *subopcode")]
        payload: ResponsePayload,
    },
}

impl Message {
    pub fn from_hdlc(data: &[u8]) -> Result<Message, DiagParsingError> {
        match hdlc_decapsulate(data, &CRC_CCITT) {
            Ok(data) => match Message::from_bytes((strip_wrapped_log_prefix(&data), 0)) {
                Ok(((leftover_bytes, _), res)) => {
                    if !leftover_bytes.is_empty() {
                        warn!(
                            "warning: {} leftover bytes when parsing Message",
                            leftover_bytes.len()
                        );
                    }
                    Ok(res)
                }
                Err(e) => Err(DiagParsingError::MessageParsingError(e, data)),
            },
            Err(err) => Err(DiagParsingError::HdlcDecapsulationError(err, data.to_vec())),
        }
    }
}

/// SDX55-based devices (R500L5) put an 8-byte wrapper (opcode 0x198, subopcode 1) in front of
/// each plain log record. The frame's CRC covers it, so it can only be removed after HDLC
/// decapsulation. Anything not followed by a log record is left alone.
const WRAPPED_LOG_PREFIX: [u8; 8] = [0x98, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
const LOG_MESSAGE_ID: u8 = 0x10;

fn strip_wrapped_log_prefix(data: &[u8]) -> &[u8] {
    match data.strip_prefix(&WRAPPED_LOG_PREFIX) {
        Some(rest) if rest.first() == Some(&LOG_MESSAGE_ID) => rest,
        _ => data,
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "opcode: u32, subopcode: u32", id = "opcode")]
pub enum ResponsePayload {
    #[deku(id = "115")]
    LogConfig(#[deku(ctx = "subopcode")] LogConfigResponse),
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "subopcode: u32", id = "subopcode")]
pub enum LogConfigResponse {
    #[deku(id = "1")]
    RetrieveIdRanges { log_mask_sizes: [u32; 16] },

    #[deku(id = "3")]
    SetMask,
}

pub fn build_log_mask_request(
    log_type: u32,
    log_mask_bitsize: u32,
    accepted_log_codes: &[u32],
) -> Request {
    let mut current_byte: u8 = 0;
    let mut num_bits_written: u8 = 0;
    let mut log_mask: Vec<u8> = vec![];
    for i in 0..log_mask_bitsize {
        let log_code: u32 = (log_type << 12) | i;
        if accepted_log_codes.contains(&log_code) {
            current_byte |= 1 << num_bits_written;
        }
        num_bits_written += 1;

        if num_bits_written == 8 || i == log_mask_bitsize - 1 {
            log_mask.push(current_byte);
            current_byte = 0;
            num_bits_written = 0;
        }
    }

    Request::LogConfig(LogConfigRequest::SetMask {
        log_type,
        log_mask_bitsize,
        log_mask,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::diaglog::LogBody;
    use crate::gsmtap::parser;

    // A paging (PCCH) LTE RRC OTA frame captured from an R500L5: the plain log record is
    // preceded by the 8-byte wrapper, and the frame CRC covers both.
    const R500L5_WRAPPED_RRC_OTA_FRAME: [u8; 55] = [
        0x98, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x10, 0x00, 0x28, 0x00, 0x28, 0x00, 0xc0,
        0xb0, 0x97, 0x3b, 0xfc, 0xb9, 0xbf, 0x04, 0x00, 0x00, 0x1b, 0x10, 0x10, 0x0f, 0x90, 0x00,
        0x15, 0x01, 0x4c, 0x04, 0x00, 0x00, 0x89, 0xce, 0x07, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00,
        0x40, 0x00, 0xac, 0x10, 0x38, 0x44, 0xc0, 0x88, 0x25, 0x7e,
    ];

    #[test]
    fn test_wrapped_log_frame_is_parsed_as_log() {
        let msg = Message::from_hdlc(&R500L5_WRAPPED_RRC_OTA_FRAME).unwrap();
        match &msg {
            Message::Log {
                log_type,
                body: LogBody::LteRrcOtaMessage { .. },
                ..
            } => assert_eq!(*log_type, 0xb0c0),
            other => panic!("expected an LTE RRC OTA log message, got {other:?}"),
        }
        assert!(parser::parse(msg).unwrap().is_some());
    }

    #[test]
    fn test_prefix_only_stripped_before_a_log_record() {
        assert_eq!(strip_wrapped_log_prefix(&[0x10, 0x00]), &[0x10, 0x00]);
        let mut non_log = WRAPPED_LOG_PREFIX.to_vec();
        non_log.push(0x73);
        assert_eq!(strip_wrapped_log_prefix(&non_log), non_log.as_slice());
        let mut log = WRAPPED_LOG_PREFIX.to_vec();
        log.push(LOG_MESSAGE_ID);
        assert_eq!(strip_wrapped_log_prefix(&log), &[LOG_MESSAGE_ID]);
    }

    #[test]
    fn test_parsing_error_summary_omits_message_bytes() {
        let err = Message::from_hdlc(&[0x01; 400]).unwrap_err();
        let summary = err.summary();
        assert!(summary.contains("400 bytes"), "{summary}");
        assert!(summary.len() < 200, "{summary}");
    }
}
