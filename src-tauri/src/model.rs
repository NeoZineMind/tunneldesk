use serde::{Deserialize, Serialize};

pub const APP_NAME: &str = "TunnelDesk";
pub const DEFAULT_PROFILE_ID: &str = "default";
pub const DEFAULT_TUNNEL_ID: &str = "default";
pub const SETTINGS_SCHEMA_VERSION: u32 = 2;
pub const PROFILES_SCHEMA_VERSION: u32 = 2;

pub fn default_tunnel_id() -> String {
    String::from(DEFAULT_TUNNEL_ID)
}

fn default_service_group() -> String {
    String::new()
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::System
}

fn default_close_action() -> CloseAction {
    CloseAction::Ask
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuthMethod {
    Password,
    PrivateKey,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CloseAction {
    Ask,
    MinimizeToTray,
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub identity_file: String,
    pub password_credential_key: String,
    pub key_passphrase_credential_key: String,
    pub server_alive_interval: u64,
    pub server_alive_count_max: u8,
}

impl Default for SshSettings {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: String::new(),
            auth_method: AuthMethod::Password,
            identity_file: String::new(),
            password_credential_key: String::new(),
            key_passphrase_credential_key: String::new(),
            server_alive_interval: 30,
            server_alive_count_max: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorSettings {
    pub start_minimized: bool,
    #[serde(default)]
    pub auto_start_profile: bool,
    #[serde(default)]
    pub launch_at_login: bool,
    pub auto_repair_on_start: bool,
    pub cleanup_on_exit: bool,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: ThemeMode,
    #[serde(default = "default_close_action")]
    pub close_action: CloseAction,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            start_minimized: false,
            auto_start_profile: false,
            launch_at_login: false,
            auto_repair_on_start: false,
            cleanup_on_exit: true,
            theme_mode: ThemeMode::System,
            close_action: CloseAction::Ask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub ssh: SshSettings,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            id: String::from(DEFAULT_TUNNEL_ID),
            name: String::from("Default Tunnel"),
            enabled: true,
            ssh: SshSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    pub current_profile_id: String,
    #[serde(default = "default_tunnel_id")]
    pub current_tunnel_id: String,
    #[serde(default)]
    pub tunnels: Vec<TunnelConfig>,
    #[serde(default)]
    pub behavior: BehaviorSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            current_profile_id: String::from(DEFAULT_PROFILE_ID),
            current_tunnel_id: String::from(DEFAULT_TUNNEL_ID),
            tunnels: vec![TunnelConfig::default()],
            behavior: BehaviorSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    pub id: String,
    pub name: String,
    #[serde(default = "default_service_group")]
    pub group: String,
    pub domain: String,
    #[serde(default)]
    pub remark: String,
    pub port: u16,
    pub local_ip: String,
    #[serde(default = "default_tunnel_id")]
    pub tunnel_id: String,
    #[serde(default)]
    pub sort_order: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProfile {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesFile {
    pub schema_version: u32,
    pub profiles: Vec<ServiceProfile>,
}

impl Default for ProfilesFile {
    fn default() -> Self {
        Self {
            schema_version: PROFILES_SCHEMA_VERSION,
            profiles: vec![ServiceProfile {
                id: String::from(DEFAULT_PROFILE_ID),
                name: String::from("Default Profile"),
                enabled: true,
                services: Vec::new(),
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigBackupFile {
    pub schema_version: u32,
    pub created_at: String,
    pub settings: AppSettings,
    pub profiles: ProfilesFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigBackupInfo {
    pub id: String,
    pub file_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRestoreResult {
    pub settings: AppSettings,
    pub profiles: ProfilesFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceState {
    Disabled,
    Stopped,
    Checking,
    Healthy,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub service_id: String,
    pub state: ServiceState,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub tunnel_id: String,
    pub name: String,
    pub running: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessPrivilege {
    Root,
    User,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HostsAccess {
    Direct,
    PolkitHelper,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegeStatus {
    pub process: ProcessPrivilege,
    pub hosts_access: HostsAccess,
    pub helper_installed: bool,
    pub can_modify_hosts: bool,
    pub message: String,
}

impl Default for PrivilegeStatus {
    fn default() -> Self {
        Self {
            process: ProcessPrivilege::Unknown,
            hosts_access: HostsAccess::Unavailable,
            helper_installed: false,
            can_modify_hosts: false,
            message: String::from("Hosts access is unavailable"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub running: bool,
    pub current_profile_id: String,
    pub running_tunnel_ids: Vec<String>,
    pub tunnels: Vec<TunnelStatus>,
    pub is_admin: bool,
    pub privilege: PrivilegeStatus,
    pub hosts_block_present: bool,
    pub message: String,
    pub services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub raw: String,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            running: false,
            current_profile_id: String::from(DEFAULT_PROFILE_ID),
            running_tunnel_ids: Vec::new(),
            tunnels: Vec::new(),
            is_admin: false,
            privilege: PrivilegeStatus::default(),
            hosts_block_present: false,
            message: String::from("Stopped"),
            services: Vec::new(),
        }
    }
}
