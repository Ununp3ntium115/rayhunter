// Config stub for the R500L5 ethernet uplink; nothing consumes it yet.
#[allow(dead_code)]
pub struct EthernetUplinkConfig {
    pub iface: String,
    pub udhcpc_bin: String,
    pub udhcpc_hook_path: String,
    pub dhcp_lease_path: String,
    pub dns_servers: Vec<String>,
}
