use std::collections::HashMap;
use std::sync::Mutex;

use crate::tunnel::TunnelRuntime;

#[derive(Default)]
pub struct AppState {
    pub active_profile_id: Mutex<Option<String>>,
    pub tunnels: Mutex<HashMap<String, TunnelRuntime>>,
}
