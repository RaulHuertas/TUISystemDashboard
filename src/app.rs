use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

#[derive(Debug, Clone, Serialize)]
pub struct DashboardState {
    pub title: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub server_addr: String,
    pub refresh_count: u64,
}

impl DashboardState {
    pub fn new(server_addr: impl Into<String>) -> Self {
        Self {
            title: "System Monitor".to_string(),
            status: "Running".to_string(),
            uptime_seconds: 0,
            server_addr: server_addr.into(),
            refresh_count: 0,
        }
    }
}

pub type SharedState = Arc<RwLock<DashboardState>>;

pub fn create_state(server_addr: impl Into<String>) -> SharedState {
    Arc::new(RwLock::new(DashboardState::new(server_addr)))
}

pub async fn run_state_updater(state: SharedState) {
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;
        let mut state = state.write().await;
        state.uptime_seconds += 1;
        state.refresh_count += 1;
    }
}
