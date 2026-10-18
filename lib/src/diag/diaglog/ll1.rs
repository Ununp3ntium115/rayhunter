use deku::prelude::*;

/// 0xb114, LTE LL1 serving cell frame timing. The layout depends on the version byte: version 1 is
/// what MDM9x07-class modems send, version 0xa1 is what SDX55 modems (Orbic R500L5) send. The
/// version byte is the enum id, so the variants do not repeat it.
#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum ServingCellFrameTiming {
    #[deku(id = "0x01")]
    V1(ServingCellTiming),
    #[deku(id = "0xa1")]
    V2(ServingCellFrameTimingV2),
}

/// Version 0xa1: a 16 byte header followed by one 48 byte record per subframe (1 ms). Layout from
/// the diaggrok notes and checked against R500L5 captures: the record sizes add up exactly and
/// the per-record timing adjustments sum to the change in frame timing between consecutive logs.
/// There is no timing advance in this version.
#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct ServingCellFrameTimingV2 {
    /// Low 5 bits are the record count; the top 3 bits are unidentified.
    #[deku(assert = "(2..=20).contains(&(*count_and_flags & 0x1f))")]
    pub count_and_flags: u8,
    pub header_adjustment: i16,
    /// Serving cell frame boundary position, in Ts (1/30.72 MHz) units.
    pub frame_timing0: u16,
    pub frame_timing1: u16,
    /// Ts clock counter, advances by 30720 per subframe.
    pub ts_counter: u32,
    pub header_reserved: u32,
    #[deku(count = "(*count_and_flags & 0x1f) as usize")]
    pub records: Vec<FrameTimingRecordV2>,
}

impl ServingCellFrameTimingV2 {
    pub fn num_records(&self) -> usize {
        (self.count_and_flags & 0x1f) as usize
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "endian: deku::ctx::Endian", endian = "endian")]
pub struct FrameTimingRecordV2 {
    /// System frame number in the low byte, plus its top 2 bits in bits 0..1 of the high byte;
    /// bits 2.. of the high byte are the subframe number.
    pub frame_and_subframe: u16,
    pub dl_frame_timing_adjustment: i16,
    pub ul_frame_timing_adjustment: i16,
    /// A third signed adjustment channel; meaning not identified.
    pub adjustment2: i16,
    pub flags: u32,
    /// 19.2 MHz crystal oscillator tick count, advances by 19200 per subframe.
    pub xo_ticks: u32,
    pub unidentified: [u32; 8],
}

impl FrameTimingRecordV2 {
    pub fn system_frame_number(&self) -> u16 {
        (self.frame_and_subframe & 0xff) | ((self.frame_and_subframe >> 8) & 0x3) << 8
    }

    pub fn subframe_number(&self) -> u8 {
        (self.frame_and_subframe >> 10) as u8
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(bit_order = "lsb")]
pub struct ServingCellTiming {
    #[deku(bits = 5, assert = "(1..=20).contains(num_records)")]
    pub num_records: u8,
    #[deku(bits = 4, assert = "*starting_sub_fn <= 9")]
    pub starting_sub_fn: u8,
    #[deku(
        bits = 10,
        pad_bits_after = "5",
        assert = "*starting_system_fn <= 1023"
    )]
    pub starting_system_fn: u16,
    #[deku(
        bits = 19,
        pad_bits_after = "13",
        assert = "*starting_dl_frame_timing_offs <= 307200"
    )]
    pub starting_dl_frame_timing_offs: u32, // in Ts units
    #[deku(bits = 19, assert = "*starting_ul_frame_timing_offs <= 307200")]
    pub starting_ul_frame_timing_offs: u32, // in Ts units
    #[deku(bits = 11, pad_bits_after = "2")]
    pub starting_ul_timing_advance: u16, // in 16 Ts units
    #[deku(count = "num_records")]
    pub timing_adjustment: Vec<TimingAdjustment>,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(bit_order = "lsb", ctx = "_: deku::ctx::Order")]
pub struct TimingAdjustment {
    #[deku(
        bits = 11,
        assert = "(-512..=511).contains(dl_frame_timing_adjustment)"
    )]
    pub dl_frame_timing_adjustment: i16, // in Ts units
    #[deku(bits = 5, assert = "(-16..=15).contains(ul_frame_timing_adjustment)")]
    pub ul_frame_timing_adjustment: i8, // in Ts units
    #[deku(assert = "(-128..=127).contains(timing_advance)")]
    pub timing_advance: i8, // in 16 Ts units
}

#[cfg(test)]
mod tests {
    use super::*;

    // Payload of a real 0xb114 log from an R500L5 (4 records), without the DIAG log header.
    const R500L5_FRAME_TIMING: [u8; 208] = [
        0xa1, 0x04, 0x1a, 0x00, 0xe4, 0x24, 0x14, 0x24, 0xe4, 0xa4, 0x3a, 0x09, 0x14, 0x1c, 0x3b,
        0x09, 0xef, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x3e, 0xa6,
        0x50, 0x00, 0x08, 0xc0, 0x21, 0x40, 0x31, 0x03, 0x00, 0x00, 0xae, 0xda, 0xff, 0xff, 0x58,
        0xf7, 0xff, 0x3f, 0x2a, 0x69, 0x66, 0x66, 0x80, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xef, 0x26, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
        0x00, 0x3e, 0xf1, 0x50, 0x00, 0x08, 0xc0, 0x21, 0x40, 0x31, 0x03, 0x00, 0x00, 0xae, 0xda,
        0xff, 0xff, 0x58, 0xf7, 0xff, 0x3f, 0x2a, 0x69, 0x66, 0x66, 0x80, 0x80, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0x17, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x34, 0x55, 0xaf, 0x00, 0x08, 0x00, 0x00, 0x40, 0xdc, 0x02, 0x00,
        0x00, 0x83, 0xde, 0xff, 0xff, 0x3d, 0xf8, 0xff, 0x3f, 0xe2, 0x68, 0x66, 0x66, 0x80, 0x80,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0x18, 0xc6, 0xd0, 0xc6,
        0xd0, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x77, 0x93, 0x85, 0x00, 0xf8, 0x3f, 0x0b, 0x40,
        0xa4, 0x02, 0x00, 0x00, 0x10, 0xe1, 0xff, 0xff, 0xd5, 0xf8, 0xff, 0x3f, 0xb2, 0x68, 0x66,
        0x66, 0x80, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    fn parse(bytes: &[u8]) -> Result<ServingCellFrameTiming, DekuError> {
        ServingCellFrameTiming::from_bytes((bytes, 0)).map(|(_, parsed)| parsed)
    }

    #[test]
    fn test_parses_r500l5_version_a1_log() {
        let ServingCellFrameTiming::V2(timing) = parse(&R500L5_FRAME_TIMING).unwrap() else {
            panic!("expected a version 0xa1 log");
        };
        assert_eq!(timing.num_records(), 4);
        assert_eq!(timing.records.len(), 4);
        assert_eq!(timing.frame_timing0, 0x24e4);
        assert_eq!(timing.frame_timing1, 0x2414);

        let first = &timing.records[0];
        assert_eq!(first.system_frame_number(), 751);
        assert_eq!(first.subframe_number(), 8);
        assert_eq!(first.xo_ticks, 0x50a63e);
        let second = &timing.records[1];
        assert_eq!(second.system_frame_number(), 751);
        assert_eq!(second.subframe_number(), 9);
        // 1 ms apart at 19.2 MHz
        assert_eq!(second.xo_ticks - first.xo_ticks, 19200);
    }

    #[test]
    fn test_version_a1_rejects_truncated_records() {
        // header claims 4 records but the last one is cut short
        assert!(parse(&R500L5_FRAME_TIMING[..R500L5_FRAME_TIMING.len() - 1]).is_err());
    }

    #[test]
    fn test_version_a1_rejects_bad_record_count() {
        let mut bytes = R500L5_FRAME_TIMING;
        bytes[1] = 0;
        assert!(parse(&bytes).is_err());
        bytes[1] = 21;
        assert!(parse(&bytes).is_err());
    }

    #[test]
    fn test_unknown_version_is_rejected() {
        let mut bytes = R500L5_FRAME_TIMING;
        bytes[0] = 0x7a;
        assert!(parse(&bytes).is_err());
    }
}
