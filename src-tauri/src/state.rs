use std::collections::HashMap;
use std::sync::Mutex;

use crate::model::{ServiceProfile, StartupReport, TunnelConfig};
use crate::tunnel::TunnelRuntime;

#[derive(Default)]
pub struct AppState {
    pub operation: tokio::sync::Mutex<()>,
    pub startup_report: Mutex<Option<StartupReport>>,
    pub startup_profile: Mutex<Option<ServiceProfile>>,
    pub startup_tunnels: Mutex<Vec<TunnelConfig>>,
    pub active_profile_id: Mutex<Option<String>>,
    pub tunnels: Mutex<HashMap<String, TunnelRuntime>>,
}
