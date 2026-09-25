use serde::Serialize;
use std::{collections::BTreeMap, fs, path::Path, sync::Arc, time::Instant};
use sysinfo::System;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardState {
    pub title: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub server_addr: String,
    pub refresh_count: u64,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub cpu_usages: Vec<u64>,
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
}

impl DashboardState {
    pub fn new(server_addr: impl Into<String>) -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        system.refresh_cpu_usage();
        let (memory_total_bytes, memory_used_bytes, cpu_usages) = collect_system_metrics(&system);

        Self {
            title: "System Monitor".to_string(),
            status: "Running".to_string(),
            uptime_seconds: 0,
            server_addr: server_addr.into(),
            refresh_count: 0,
            memory_total_bytes,
            memory_used_bytes,
            cpu_usages,
            network_interfaces: collect_network_interfaces(),
        }
    }
}

pub type SharedState = Arc<RwLock<DashboardState>>;

pub fn create_state(server_addr: impl Into<String>) -> SharedState {
    Arc::new(RwLock::new(DashboardState::new(server_addr)))
}

pub async fn run_state_updater(state: SharedState) {
    let started_at = Instant::now();
    let mut system = System::new_all();
    system.refresh_memory();
    system.refresh_cpu_usage();

    let mut ticker = interval(Duration::from_secs(4));

    loop {
        ticker.tick().await;
        system.refresh_memory();
        system.refresh_cpu_usage();

        let (memory_total_bytes, memory_used_bytes, cpu_usages) = collect_system_metrics(&system);
        let mut state = state.write().await;
        state.uptime_seconds = started_at.elapsed().as_secs();
        state.refresh_count += 1;
        state.memory_total_bytes = memory_total_bytes;
        state.memory_used_bytes = memory_used_bytes;
        state.cpu_usages = cpu_usages;
        state.network_interfaces = collect_network_interfaces();
    }
}

fn collect_system_metrics(system: &System) -> (u64, u64, Vec<u64>) {
    let memory_total_bytes = system.total_memory();
    let memory_used_bytes = system.used_memory();
    let cpu_usages = system
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage().round().clamp(0.0, 100.0) as u64)
        .collect();

    (memory_total_bytes, memory_used_bytes, cpu_usages)
}

fn collect_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let mut interfaces: BTreeMap<String, NetworkInterfaceInfo> = BTreeMap::new();

    if let Ok(addrs) = if_addrs::get_if_addrs() {
        for iface in addrs {
            let ip = iface.ip().to_string();
            let entry =
                interfaces
                    .entry(iface.name.clone())
                    .or_insert_with(|| NetworkInterfaceInfo {
                        name: iface.name,
                        ip_addresses: Vec::new(),
                        mac_address: None,
                        rx_bytes: 0,
                        tx_bytes: 0,
                        rx_packets: 0,
                        tx_packets: 0,
                    });

            if !entry.ip_addresses.contains(&ip) {
                entry.ip_addresses.push(ip);
            }
        }
    }

    let sys_class_net = Path::new("/sys/class/net");
    if let Ok(entries) = fs::read_dir(sys_class_net) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let base_path = entry.path();

            let interface =
                interfaces
                    .entry(name.clone())
                    .or_insert_with(|| NetworkInterfaceInfo {
                        name,
                        ip_addresses: Vec::new(),
                        mac_address: None,
                        rx_bytes: 0,
                        tx_bytes: 0,
                        rx_packets: 0,
                        tx_packets: 0,
                    });

            interface.mac_address = read_trimmed(base_path.join("address"));
            interface.rx_bytes = read_u64(base_path.join("statistics/rx_bytes"));
            interface.tx_bytes = read_u64(base_path.join("statistics/tx_bytes"));
            interface.rx_packets = read_u64(base_path.join("statistics/rx_packets"));
            interface.tx_packets = read_u64(base_path.join("statistics/tx_packets"));
            interface.ip_addresses.sort();
        }
    }

    interfaces.into_values().collect()
}

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_u64(path: impl AsRef<Path>) -> u64 {
    read_trimmed(path)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
}
