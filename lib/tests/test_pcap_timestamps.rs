// pcap_file_tokio wraps raw pcapng timestamp ticks in Duration::from_nanos, but Rayhunter writes
// microsecond ticks, so offline analysis of its own pcaps saw time run 1000x too slowly and
// windowed heuristics ("N messages in 60 seconds") were evaluated over ~17 hours.

use deku::prelude::*;
use pcap_file_tokio::pcapng::{Block, PcapNgReader};
use rayhunter::{
    DeviceMetadata,
    analysis::analyzer::{AnalyzerConfig, Harness},
    diag::Message,
    gsmtap::parser as gsmtap_parser,
    pcap::{GsmtapPcapWriter, if_tsresol, pcapng_timestamp},
};
use std::time::Duration;

// LTE RRC OTA DIAG log (v15, BCCH-DL-SCH) from test_lte_parsing.rs
const DIAG_MESSAGE: &[u8] = &[
    0x10, 0x00, 0x3b, 0x00, 0x3b, 0x00, 0xc0, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0f, 0x0d, 0x21, 0x01, 0x9e, 0x00, 0x14, 0x05, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
    0x00, 0x1c, 0x00, 0x08, 0x10, 0xa5, 0x34, 0x61, 0x41, 0xa3, 0x1c, 0x31, 0x68, 0x04, 0x40, 0x1a,
    0x00, 0x49, 0x16, 0x7c, 0x23, 0x15, 0x9f, 0x00, 0x10, 0x67, 0xc1, 0x06, 0xd9, 0xe0, 0x00,
];

#[tokio::test]
async fn harness_recovers_timestamps_written_by_pcap_writer() {
    let (_, msg) = Message::from_bytes((DIAG_MESSAGE, 0)).unwrap();
    let (timestamp, gsmtap) = gsmtap_parser::parse(msg).unwrap().unwrap();
    let expected = timestamp.to_datetime();

    let mut buf = Vec::new();
    let mut writer = GsmtapPcapWriter::new(&mut buf).await.unwrap();
    writer.write_iface_header().await.unwrap();
    writer
        .write_gsmtap_message(gsmtap, timestamp, None)
        .await
        .unwrap();
    drop(writer);

    let mut reader = PcapNgReader::new(buf.as_slice()).await.unwrap();
    let mut harness =
        Harness::new_with_config(&AnalyzerConfig::default(), &DeviceMetadata::default());
    let mut tsresol = None;
    let mut rows = Vec::new();
    while let Some(block) = reader.next_block().await {
        match block.unwrap() {
            Block::InterfaceDescription(iface) => tsresol = if_tsresol(&iface),
            Block::EnhancedPacket(mut packet) => {
                packet.timestamp = pcapng_timestamp(packet.timestamp, tsresol);
                rows.push(harness.analyze_pcap_packet(packet));
            }
            _ => {}
        }
    }

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].packet_timestamp, Some(expected));
}

#[test]
fn pcapng_timestamp_honors_if_tsresol() {
    // pcap_file_tokio hands back raw ticks as nanoseconds
    let ticks = Duration::from_nanos(1_500);
    // default (no option) and explicit 10^-6 are microseconds
    assert_eq!(pcapng_timestamp(ticks, None), Duration::from_micros(1_500));
    assert_eq!(
        pcapng_timestamp(ticks, Some(6)),
        Duration::from_micros(1_500)
    );
    assert_eq!(
        pcapng_timestamp(ticks, Some(9)),
        Duration::from_nanos(1_500)
    );
    assert_eq!(
        pcapng_timestamp(ticks, Some(3)),
        Duration::from_millis(1_500)
    );
    // MSB set: 2^-n, here 2^-10 s per tick
    assert_eq!(
        pcapng_timestamp(Duration::from_nanos(1024), Some(0x80 | 10)),
        Duration::from_secs(1)
    );
}
