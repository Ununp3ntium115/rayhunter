//! Parse QMDL files and create a pcap file.
//! Creates a plausible IP header and [GSMtap](https://osmocom.org/projects/baseband/wiki/GSMTAP) header and then puts the rest of the data under that for wireshark to parse.
use crate::diag::diaglog::Timestamp;
use crate::gsmtap::GsmtapMessage;

use chrono::prelude::*;
use deku::prelude::*;
use pcap_file_tokio::pcapng::PcapNgWriter;
use pcap_file_tokio::pcapng::blocks::enhanced_packet::{EnhancedPacketBlock, EnhancedPacketOption};
use pcap_file_tokio::pcapng::blocks::interface_description::InterfaceDescriptionBlock;
use pcap_file_tokio::pcapng::blocks::section_header::{SectionHeaderBlock, SectionHeaderOption};
use pcap_file_tokio::{Endianness, PcapError};
use serde::Serialize;
use std::borrow::Cow;
use thiserror::Error;
use tokio::io::AsyncWrite;

#[derive(Error, Debug)]
pub enum GsmtapPcapError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Pcap error: {0}")]
    Pcap(#[from] PcapError),
    #[error("Timestamp out of range: {0}")]
    TimestampOutOfRange(#[from] chrono::OutOfRangeError),
    #[error("Deku error: {0}")]
    Deku(#[from] DekuError),
}

#[derive(Serialize)]
pub struct GpsPoint {
    pub unix_ts: i64,
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lon")]
    pub longitude: f64,
}

pub struct GsmtapPcapWriter<T>
where
    T: AsyncWrite,
{
    writer: PcapNgWriter<T>,
    ip_id: u16,
}

const IP_HEADER_LEN: u16 = 20;
#[derive(DekuWrite)]
#[deku(endian = "big")]
struct IpHeader {
    version_and_ihl: u8,
    dscp: u8,
    total_len: u16,
    identification: u16,
    flags_and_frag_offset: u8,
    idk: u8,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    src_addr: u32,
    dst_addr: u32,
}

const UDP_HEADER_LEN: u16 = 8;
const GSMTAP_PORT: u16 = 4729;
#[derive(DekuWrite)]
#[deku(endian = "big")]
struct UdpHeader {
    src_port: u16,
    dst_port: u16,
    length: u16,
    checksum: u16,
}

impl<T> GsmtapPcapWriter<T>
where
    T: AsyncWrite + Unpin + Send,
{
    pub async fn new(writer: T) -> Result<Self, GsmtapPcapError> {
        let metadata = crate::util::RuntimeMetadata::new();
        let package = format!(
            "{} {}",
            env!("CARGO_PKG_NAME").to_owned(),
            metadata.rayhunter_version
        );
        let section = SectionHeaderBlock {
            endianness: Endianness::Big,
            major_version: 1,
            minor_version: 0,
            section_length: -1,
            options: vec![
                SectionHeaderOption::Hardware(Cow::from(metadata.arch)),
                SectionHeaderOption::OS(Cow::from(metadata.system_os)),
                SectionHeaderOption::UserApplication(Cow::from(package)),
            ],
        };
        let writer = PcapNgWriter::with_section_header(writer, section).await?;
        Ok(GsmtapPcapWriter { writer, ip_id: 0 })
    }

    pub async fn write_iface_header(&mut self) -> Result<(), GsmtapPcapError> {
        let interface = InterfaceDescriptionBlock {
            linktype: pcap_file_tokio::DataLink::IPV4,
            snaplen: 0xffff,
            options: vec![],
        };
        self.writer.write_pcapng_block(interface).await?;
        Ok(())
    }

    pub async fn write_gsmtap_message(
        &mut self,
        msg: GsmtapMessage,
        timestamp: Timestamp,
        gps: Option<&GpsPoint>,
    ) -> Result<(), GsmtapPcapError> {
        let duration = timestamp
            .to_datetime()
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .to_std()?;

        // despite the timestamp above being correct, we have reduce it by
        // orders of magnitude due to a bug in pcap_file:
        // https://github.com/courvoif/pcap-file/pull/32
        let duration = std::time::Duration::from_nanos(duration.as_micros() as u64);

        let msg_bytes = msg.to_bytes()?;
        let ip_header = IpHeader {
            version_and_ihl: 0x45,
            dscp: 0,
            total_len: msg_bytes.len() as u16 + IP_HEADER_LEN + UDP_HEADER_LEN,
            identification: self.ip_id,
            flags_and_frag_offset: 0x40,
            idk: 0,
            ttl: 64,
            protocol: 0x11, // UDP
            checksum: 0xffff,
            src_addr: 0x7f000001,
            dst_addr: 0x7f000001, // TODO increment by radio_id
        };
        let udp_header = UdpHeader {
            src_port: 13337,
            dst_port: GSMTAP_PORT,
            length: msg_bytes.len() as u16 + UDP_HEADER_LEN,
            checksum: 0xffff,
        };
        let mut data: Vec<u8> = Vec::new();
        data.extend(&ip_header.to_bytes()?);
        data.extend(&udp_header.to_bytes()?);
        data.extend(&msg_bytes);

        let mut options = vec![];
        if let Some(p) = gps {
            let comment = serde_json::to_string(p).expect("GpsPoint serialization cannot fail");
            options.push(EnhancedPacketOption::Comment(Cow::Owned(comment)));
        }
        let packet = EnhancedPacketBlock {
            interface_id: 0,
            timestamp: duration,
            original_len: data.len() as u32,
            data: Cow::Owned(data),
            options,
        };
        self.writer.write_pcapng_block(packet).await?;

        self.ip_id = self.ip_id.wrapping_add(1);
        Ok(())
    }
}

/// Finds the GSMTAP message (header and payload) inside a captured packet.
///
/// Rayhunter writes raw IPv4 packets, but other tools don't: SCAT writes Ethernet frames,
/// and tcpdump on Linux often uses cooked (SLL) captures. This walks the link, IP and UDP
/// headers for the given link type and only accepts UDP traffic to or from the GSMTAP port.
pub fn find_gsmtap(data: &[u8], linktype: pcap_file_tokio::DataLink) -> Result<&[u8], String> {
    use pcap_file_tokio::DataLink;

    const ETHERTYPE_IPV4: u16 = 0x0800;
    const ETHERTYPE_IPV6: u16 = 0x86dd;
    const ETHERTYPE_VLAN: [u16; 2] = [0x8100, 0x88a8];

    fn be16(d: &[u8], at: usize) -> Result<u16, String> {
        d.get(at..at + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .ok_or_else(|| "packet truncated".to_string())
    }
    fn skip(d: &[u8], n: usize) -> Result<&[u8], String> {
        d.get(n..).ok_or_else(|| "packet truncated".to_string())
    }

    // Link layer: strip it, leaving an IP packet. `None` means "detect from the version nibble".
    let (ip, ethertype) = match linktype {
        DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => (data, None),
        // BSD loopback: 4-byte address family, in host byte order
        DataLink::NULL | DataLink::LOOP => (skip(data, 4)?, None),
        DataLink::ETHERNET => {
            let mut offset = 12;
            let mut ethertype = be16(data, offset)?;
            while ETHERTYPE_VLAN.contains(&ethertype) {
                offset += 4;
                ethertype = be16(data, offset)?;
            }
            (skip(data, offset + 2)?, Some(ethertype))
        }
        DataLink::LINUX_SLL => (skip(data, 16)?, Some(be16(data, 14)?)),
        DataLink::LINUX_SLL2 => (skip(data, 20)?, Some(be16(data, 0)?)),
        other => return Err(format!("unsupported pcap link type {other:?}")),
    };

    let version = ip.first().ok_or("empty IP packet")? >> 4;
    let udp = match (ethertype, version) {
        (None | Some(ETHERTYPE_IPV4), 4) => {
            let header_len = usize::from(ip[0] & 0x0f) * 4;
            if header_len < 20 {
                return Err(format!("invalid IPv4 header length {header_len}"));
            }
            if ip.get(9) != Some(&17) {
                return Err("not a UDP packet".to_string());
            }
            skip(ip, header_len)?
        }
        (None | Some(ETHERTYPE_IPV6), 6) => {
            if ip.get(6) != Some(&17) {
                return Err("not a UDP packet (or IPv6 extension headers)".to_string());
            }
            skip(ip, 40)?
        }
        (Some(ethertype), _) => {
            return Err(format!("not an IP packet (ethertype {ethertype:#06x})"));
        }
        (None, v) => return Err(format!("unknown IP version {v}")),
    };

    if be16(udp, 0)? != GSMTAP_PORT && be16(udp, 2)? != GSMTAP_PORT {
        return Err("not GSMTAP (UDP port is not 4729)".to_string());
    }
    let gsmtap = skip(udp, UDP_HEADER_LEN as usize)?;
    // GSMTAP header length is in 32-bit words; version 2 headers are 16 bytes
    let header_len = usize::from(*gsmtap.get(1).ok_or("GSMTAP header truncated")?) * 4;
    if header_len < 16 || gsmtap.len() < header_len {
        return Err(format!("invalid GSMTAP header length {header_len}"));
    }
    Ok(gsmtap)
}

#[cfg(test)]
mod find_gsmtap_tests {
    use super::find_gsmtap;
    use pcap_file_tokio::DataLink;

    // GSMTAP v2, 16-byte header, type 0x0d (LTE RRC), subtype 0 at byte 12, then payload
    const GSMTAP: [u8; 18] = [
        2, 4, 0x0d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xaa, 0xbb,
    ];

    fn udp(dst_port: u16) -> Vec<u8> {
        let mut v = vec![0x34, 0x19];
        v.extend(dst_port.to_be_bytes());
        v.extend([0, 26, 0, 0]);
        v.extend(GSMTAP);
        v
    }

    fn ipv4(options: usize, payload: &[u8]) -> Vec<u8> {
        let mut v = vec![
            0x45 + (options / 4) as u8,
            0,
            0,
            0,
            0,
            0,
            0x40,
            0,
            64,
            17,
            0,
            0,
        ];
        v.extend([127, 0, 0, 1, 127, 0, 0, 1]);
        v.extend(vec![0; options]);
        v.extend(payload);
        v
    }

    fn ethernet(ethertypes: &[u16], payload: &[u8]) -> Vec<u8> {
        let mut v = vec![0; 12];
        for (i, t) in ethertypes.iter().enumerate() {
            v.extend(t.to_be_bytes());
            if i + 1 < ethertypes.len() {
                v.extend([0, 1]); // VLAN tag
            }
        }
        v.extend(payload);
        v
    }

    #[test]
    fn raw_ipv4_as_written_by_rayhunter() {
        let pkt = ipv4(0, &udp(4729));
        assert_eq!(find_gsmtap(&pkt, DataLink::IPV4).unwrap(), &GSMTAP);
    }

    #[test]
    fn ethernet_as_written_by_scat() {
        let pkt = ethernet(&[0x0800], &ipv4(0, &udp(4729)));
        assert_eq!(find_gsmtap(&pkt, DataLink::ETHERNET).unwrap(), &GSMTAP);
    }

    #[test]
    fn ethernet_with_vlan_and_ip_options() {
        let pkt = ethernet(&[0x8100, 0x0800], &ipv4(8, &udp(4729)));
        assert_eq!(find_gsmtap(&pkt, DataLink::ETHERNET).unwrap(), &GSMTAP);
    }

    #[test]
    fn linux_cooked_capture() {
        let mut pkt = vec![0; 14];
        pkt.extend(0x0800u16.to_be_bytes());
        pkt.extend(ipv4(0, &udp(4729)));
        assert_eq!(find_gsmtap(&pkt, DataLink::LINUX_SLL).unwrap(), &GSMTAP);
    }

    #[test]
    fn ipv6() {
        let mut pkt = vec![0x60, 0, 0, 0, 0, 26, 17, 64];
        pkt.extend([0; 32]);
        pkt.extend(udp(4729));
        assert_eq!(find_gsmtap(&pkt, DataLink::RAW).unwrap(), &GSMTAP);
    }

    #[test]
    fn rejects_other_udp_and_truncated_packets() {
        assert!(find_gsmtap(&ipv4(0, &udp(53)), DataLink::IPV4).is_err());
        assert!(find_gsmtap(&[0x45, 0], DataLink::IPV4).is_err());
        assert!(find_gsmtap(&[], DataLink::ETHERNET).is_err());
        let arp = ethernet(&[0x0806], &[0; 28]);
        assert!(find_gsmtap(&arp, DataLink::ETHERNET).is_err());
    }
}
