// Regression test for EFForg/rayhunter#719: pcaps written by other tools (SCAT uses Ethernet
// framing) were read at Rayhunter's raw-IPv4 offsets, so every packet was skipped.

use deku::prelude::*;
use pcap_file_tokio::DataLink;
use pcap_file_tokio::pcapng::blocks::enhanced_packet::EnhancedPacketBlock;
use rayhunter::{
    DeviceMetadata,
    analysis::analyzer::{AnalysisRow, AnalyzerConfig, Harness},
    diag::Message,
    gsmtap::parser as gsmtap_parser,
};

// LTE RRC OTA DIAG log (v15, BCCH-DL-SCH) from test_lte_parsing.rs
const DIAG_MESSAGE: &[u8] = &[
    0x10, 0x00, 0x3b, 0x00, 0x3b, 0x00, 0xc0, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0f, 0x0d, 0x21, 0x01, 0x9e, 0x00, 0x14, 0x05, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
    0x00, 0x1c, 0x00, 0x08, 0x10, 0xa5, 0x34, 0x61, 0x41, 0xa3, 0x1c, 0x31, 0x68, 0x04, 0x40, 0x1a,
    0x00, 0x49, 0x16, 0x7c, 0x23, 0x15, 0x9f, 0x00, 0x10, 0x67, 0xc1, 0x06, 0xd9, 0xe0, 0x00,
];

fn gsmtap_bytes() -> Vec<u8> {
    let (_, msg) = Message::from_bytes((DIAG_MESSAGE, 0)).unwrap();
    let (_, gsmtap) = gsmtap_parser::parse(msg).unwrap().unwrap();
    gsmtap.to_bytes().unwrap()
}

fn ipv4_udp(gsmtap: &[u8]) -> Vec<u8> {
    let udp_len = (8 + gsmtap.len()) as u16;
    let total_len = 20 + udp_len;
    let mut pkt = vec![0x45, 0];
    pkt.extend(total_len.to_be_bytes());
    pkt.extend([0, 0, 0x40, 0, 64, 17, 0, 0, 127, 0, 0, 1, 127, 0, 0, 1]);
    pkt.extend(13337u16.to_be_bytes());
    pkt.extend(4729u16.to_be_bytes());
    pkt.extend(udp_len.to_be_bytes());
    pkt.extend([0, 0]);
    pkt.extend(gsmtap);
    pkt
}

fn analyze(data: Vec<u8>, linktype: DataLink) -> AnalysisRow {
    let mut harness =
        Harness::new_with_config(&AnalyzerConfig::default(), &DeviceMetadata::default());
    let packet = EnhancedPacketBlock {
        interface_id: 0,
        timestamp: std::time::Duration::from_secs(1),
        original_len: data.len() as u32,
        data: data.into(),
        options: vec![],
    };
    harness.analyze_pcap_packet_with_linktype(packet, linktype)
}

#[test]
fn ethernet_framed_packet_is_analyzed_like_raw_ipv4() {
    let gsmtap = gsmtap_bytes();
    let raw = analyze(ipv4_udp(&gsmtap), DataLink::IPV4);
    assert_eq!(
        raw.skipped_message_reason, None,
        "raw IPv4 packet was skipped"
    );

    // what SCAT writes: Ethernet header in front of the same IPv4/UDP/GSMTAP packet
    let mut ethernet = vec![0; 12];
    ethernet.extend(0x0800u16.to_be_bytes());
    ethernet.extend(ipv4_udp(&gsmtap));
    let framed = analyze(ethernet.clone(), DataLink::ETHERNET);
    assert_eq!(framed.skipped_message_reason, None);
    assert_eq!(framed.events, raw.events);

    // before #719 was fixed, every packet was read at the raw-IPv4 offsets like this
    let misread = analyze(ethernet, DataLink::IPV4);
    assert!(misread.skipped_message_reason.is_some());
}

#[test]
fn truncated_packet_is_skipped_not_a_panic() {
    let row = analyze(vec![0x45, 0, 0], DataLink::IPV4);
    assert!(row.skipped_message_reason.is_some());
}
