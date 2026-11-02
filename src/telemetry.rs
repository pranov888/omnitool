use std::net::{TcpStream, ToSocketAddrs};
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
    pub disks: Vec<DiskInfo>,
}

pub struct DiskInfo {
    pub mount_point: String,
    pub total_space_gb: f64,
    pub available_space_gb: f64,
    pub file_system: String,
}

pub struct NetworkInterfaceInfo {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

pub fn collect_system_overview() -> SystemOverview {
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
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let total_memory_mb = sys.total_memory() / 1024 / 1024;
    let used_memory_mb = sys.used_memory() / 1024 / 1024;
    let free_memory_mb = sys.free_memory() / 1024 / 1024;

    let disks_list = Disks::new_with_refreshed_list();
    let disks = disks_list
        .iter()
        .map(|d| DiskInfo {
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_space_gb: (d.total_space() as f64) / (1024.0 * 1024.0 * 1024.0),
            available_space_gb: (d.available_space() as f64) / (1024.0 * 1024.0 * 1024.0),
            file_system: d.file_system().to_string_lossy().to_string(),
        })
        .collect();

    SystemOverview {
        host_name,
        os_name,
        os_version,
        cpu_count,
        cpu_brand,
        total_memory_mb,
        used_memory_mb,
        free_memory_mb,
        disks,
    }
}

pub fn collect_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let networks = Networks::new_with_refreshed_list();
    networks
        .iter()
        .map(|(name, data)| NetworkInterfaceInfo {
            name: name.clone(),
            received_bytes: data.total_received(),
            transmitted_bytes: data.total_transmitted(),
        })
        .collect()
}

pub fn probe_tcp_latency(host: &str, port: u16) -> Result<Duration, String> {
    let target = format!("{host}:{port}");
    let socket_addrs: Vec<_> = target
        .to_socket_addrs()
        .map_err(|e| format!("Could not resolve host '{host}': {e}"))?
        .collect();

    if socket_addrs.is_empty() {
        return Err(format!("Host '{host}' did not resolve to any IP address"));
    }

    let start = Instant::now();
    match TcpStream::connect_timeout(&socket_addrs[0], Duration::from_secs(4)) {
        Ok(_) => Ok(start.elapsed()),
        Err(e) => Err(format!("Failed to connect to {target}: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_overview_collection() {
        let overview = collect_system_overview();
        assert!(overview.cpu_count > 0);
        assert!(overview.total_memory_mb > 0);
    }

    #[test]
    fn test_network_interfaces_collection() {
        let interfaces = collect_network_interfaces();
        // At least loopback or local network interface should exist
        assert!(!interfaces.is_empty());
    }
}
