use std::collections::HashSet;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use sysinfo::{Disks, Networks, System};

pub struct SystemOverview {
    pub host_name: String,
    pub os_name: String,
    pub os_version: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub free_memory_mb: u64,
    pub total_swap_mb: u64,
    pub used_swap_mb: u64,
    pub disks: Vec<DiskInfo>,
}

pub struct DiskInfo {
    pub mount_point: String,
    pub total_space_gb: f64,
    pub available_space_gb: f64,
    pub file_system: String,
    pub is_virtual: bool,
}

pub struct NetworkInterfaceInfo {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

pub struct TcpProbeResult {
    pub host: String,
    pub endpoint: SocketAddr,
    pub dns_duration: Duration,
    pub tcp_latency: Duration,
}

pub struct DnsLookupResult {
    pub host: String,
    pub addresses: Vec<String>,
    pub lookup_duration: Duration,
}

fn is_virtual_mount(mount_point: &str, fs: &str, total_bytes: u64) -> bool {
    if total_bytes == 0 {
        return true;
    }
    let fs_lower = fs.to_ascii_lowercase();
    if matches!(
        fs_lower.as_str(),
        "squashfs" | "tmpfs" | "devtmpfs" | "overlay" | "proc" | "sysfs" | "cgroup" | "cgroup2"
    ) {
        return true;
    }
    mount_point.starts_with("/snap/") || mount_point == "/snap"
}

pub fn collect_system_overview_filtered(include_virtual: bool) -> SystemOverview {
    let mut sys = System::new_all();
    sys.refresh_all();

    let host_name = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());

    let cpu_count = sys.cpus().len();
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|b| !b.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let total_memory_mb = sys.total_memory() / 1024 / 1024;
    let used_memory_mb = sys.used_memory() / 1024 / 1024;
    let free_memory_mb = sys.free_memory() / 1024 / 1024;
    let total_swap_mb = sys.total_swap() / 1024 / 1024;
    let used_swap_mb = sys.used_swap() / 1024 / 1024;

    let disks_list = Disks::new_with_refreshed_list();
    let mut disks: Vec<DiskInfo> = disks_list
        .iter()
        .filter_map(|d| {
            let mount_point = d.mount_point().to_string_lossy().to_string();
            let file_system = d.file_system().to_string_lossy().to_string();
            let total_bytes = d.total_space();
            let is_virtual = is_virtual_mount(&mount_point, &file_system, total_bytes);
            if !include_virtual && is_virtual {
                return None;
            }
            Some(DiskInfo {
                mount_point,
                total_space_gb: (total_bytes as f64) / (1024.0 * 1024.0 * 1024.0),
                available_space_gb: (d.available_space() as f64) / (1024.0 * 1024.0 * 1024.0),
                file_system,
                is_virtual,
            })
        })
        .collect();

    disks.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));

    SystemOverview {
        host_name,
        os_name,
        os_version,
        cpu_count,
        cpu_brand,
        total_memory_mb,
        used_memory_mb,
        free_memory_mb,
        total_swap_mb,
        used_swap_mb,
        disks,
    }
}

pub fn collect_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let networks = Networks::new_with_refreshed_list();
    let mut list: Vec<NetworkInterfaceInfo> = networks
        .iter()
        .map(|(name, data)| NetworkInterfaceInfo {
            name: name.clone(),
            received_bytes: data.total_received(),
            transmitted_bytes: data.total_transmitted(),
        })
        .collect();

    list.sort_by(|a, b| {
        let a_total = a.received_bytes.saturating_add(a.transmitted_bytes);
        let b_total = b.received_bytes.saturating_add(b.transmitted_bytes);
        b_total.cmp(&a_total).then_with(|| a.name.cmp(&b.name))
    });

    list
}

fn clean_host(host: &str) -> Result<&str, String> {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return Err("Hostname or IP address cannot be empty".to_string());
    }
    if trimmed.contains("://") {
        return Err(
            "Pass a hostname or IP address (e.g. 'github.com'), not a full URL".to_string(),
        );
    }
    let unbracketed = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(trimmed);
    Ok(unbracketed)
}

pub fn probe_tcp_detailed(
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<TcpProbeResult, String> {
    let clean = clean_host(host)?;
    let dns_start = Instant::now();
    let socket_addrs: Vec<_> = (clean, port)
        .to_socket_addrs()
        .map_err(|e| format!("Could not resolve host '{clean}': {e}"))?
        .collect();
    let dns_duration = dns_start.elapsed();

    if socket_addrs.is_empty() {
        return Err(format!("Host '{clean}' did not resolve to any IP address"));
    }

    let mut last_error = None;
    for addr in &socket_addrs {
        let attempt_start = Instant::now();
        match TcpStream::connect_timeout(addr, timeout) {
            Ok(_) => {
                return Ok(TcpProbeResult {
                    host: clean.to_string(),
                    endpoint: *addr,
                    dns_duration,
                    tcp_latency: attempt_start.elapsed(),
                });
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    let err_msg = last_error
        .map(|e| e.to_string())
        .unwrap_or_else(|| "Unknown connection error".to_string());
    Err(format!("Failed to connect to {clean}:{port}: {err_msg}"))
}

pub fn resolve_dns(host: &str) -> Result<DnsLookupResult, String> {
    let clean = clean_host(host)?;
    let start = Instant::now();
    let mut seen = HashSet::new();
    let mut addrs = Vec::new();

    let resolved = (clean, 80)
        .to_socket_addrs()
        .map_err(|e| format!("Failed to resolve DNS for '{clean}': {e}"))?;

    for sock in resolved {
        let ip = sock.ip().to_string();
        if seen.insert(ip.clone()) {
            addrs.push(ip);
        }
    }

    if addrs.is_empty() {
        return Err(format!("No IP addresses found for '{clean}'"));
    }

    Ok(DnsLookupResult {
        host: clean.to_string(),
        addresses: addrs,
        lookup_duration: start.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_overview_collection() {
        let overview = collect_system_overview_filtered(false);
        assert!(overview.cpu_count > 0);
        assert!(overview.total_memory_mb > 0);
    }

    #[test]
    fn test_network_interfaces_collection() {
        let interfaces = collect_network_interfaces();
        // At least loopback or local network interface should exist
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn test_probe_tcp_invalid_host() {
        let res = probe_tcp_detailed(
            "non-existent-host.invalid-domain-test",
            80,
            Duration::from_secs(1),
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_clean_host_rejects_urls_and_handles_ipv6() {
        assert!(probe_tcp_detailed("https://github.com", 443, Duration::from_secs(1)).is_err());
        assert!(resolve_dns("").is_err());
        let res = resolve_dns("[127.0.0.1]").expect("bracketed IPv4 parses cleanly");
        assert_eq!(res.addresses, vec!["127.0.0.1"]);
    }

    #[test]
    fn test_resolve_dns_loopback() {
        let res = resolve_dns("localhost").expect("localhost resolves");
        assert!(!res.addresses.is_empty());
        assert!(
            res.addresses.contains(&"127.0.0.1".to_string())
                || res.addresses.contains(&"::1".to_string())
        );
    }
}
