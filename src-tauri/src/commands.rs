use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use tauri::State;

use crate::config;
use crate::credential;
use crate::error::{to_command_error, AppError};
use crate::health;
use crate::hosts;
use crate::model::{
    AppSettings, AppStatus, AuthMethod, ConfigBackupInfo, ConfigRestoreResult, LogEntry,
    ProfilesFile, ServiceConfig, ServiceProfile, ServiceState, ServiceStatus, TunnelConfig,
    TunnelStatus,
};
use crate::profile_transfer::{ProfilesImportApplyResult, ProfilesImportPreview, TunnelMapping};
use crate::startup;
use crate::state::AppState;
use crate::tunnel;
use crate::validation;

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub fn load_settings() -> CommandResult<AppSettings> {
    config::load_settings().map_err(to_command_error)
}

#[tauri::command]
pub async fn launch_at_login_enabled() -> CommandResult<bool> {
    tauri::async_runtime::spawn_blocking(startup::launch_at_login_enabled)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> CommandResult<AppSettings> {
    let profiles = config::load_profiles().unwrap_or_default();
    let mut normalized = settings;
    config::normalize_settings_credentials(&mut normalized);
    validation::validate_settings_with_profiles(&normalized, &profiles)
        .map_err(to_command_error)?;
    config::save_settings(&normalized).map_err(to_command_error)?;
    Ok(normalized)
}

#[tauri::command]
pub async fn set_launch_at_login(enabled: bool) -> CommandResult<bool> {
    tauri::async_runtime::spawn_blocking(move || {
        startup::set_launch_at_login(enabled).map_err(to_command_error)?;
        Ok(startup::launch_at_login_enabled())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn load_profiles() -> CommandResult<ProfilesFile> {
    config::load_profiles().map_err(to_command_error)
}

#[tauri::command]
pub fn save_profiles(profiles: ProfilesFile) -> CommandResult<ProfilesFile> {
    validation::validate_profiles(&profiles).map_err(to_command_error)?;
    let settings = config::load_settings().unwrap_or_default();
    validation::validate_tunnel_references(&settings, &profiles).map_err(to_command_error)?;
    config::create_config_backup().map_err(to_command_error)?;
    config::save_profiles(&profiles).map_err(to_command_error)?;
    Ok(profiles)
}

#[tauri::command]
pub fn list_config_backups() -> CommandResult<Vec<ConfigBackupInfo>> {
    config::list_config_backups().map_err(to_command_error)
}

#[tauri::command]
pub fn restore_config_backup(backup_id: String) -> CommandResult<ConfigRestoreResult> {
    config::restore_config_backup(backup_id).map_err(to_command_error)
}

#[tauri::command]
pub fn delete_config_backup(backup_id: String) -> CommandResult<()> {
    config::delete_config_backup(backup_id).map_err(to_command_error)
}

#[tauri::command]
pub fn export_profiles(path: String, profile_ids: Vec<String>) -> CommandResult<()> {
    crate::profile_transfer::export_profiles(PathBuf::from(path), profile_ids)
        .map_err(to_command_error)
}

#[tauri::command]
pub fn preview_profiles_import(
    path: String,
    tunnel_mappings: Vec<TunnelMapping>,
) -> CommandResult<ProfilesImportPreview> {
    crate::profile_transfer::preview_profiles_import(PathBuf::from(path), tunnel_mappings)
        .map_err(to_command_error)
}

#[tauri::command]
pub fn apply_profiles_import(
    path: String,
    tunnel_mappings: Vec<TunnelMapping>,
) -> CommandResult<ProfilesImportApplyResult> {
    crate::profile_transfer::apply_profiles_import(PathBuf::from(path), tunnel_mappings)
        .map_err(to_command_error)
}

#[tauri::command]
pub fn save_tunnel_password(tunnel_id: String, value: String) -> CommandResult<()> {
    if value.is_empty() {
        return Err(String::from("SSH password cannot be empty"));
    }
    let settings = config::load_settings().map_err(to_command_error)?;
    let tunnel = tunnel_by_id(&settings, &tunnel_id).map_err(to_command_error)?;
    credential::save_secret(&credential::tunnel_password_key(&tunnel.id), &value)
        .map_err(to_command_error)
}

#[tauri::command]
pub fn delete_tunnel_password(tunnel_id: String) -> CommandResult<()> {
    let settings = config::load_settings().map_err(to_command_error)?;
    let tunnel = tunnel_by_id(&settings, &tunnel_id).map_err(to_command_error)?;
    credential::delete_secret(&credential::tunnel_password_key(&tunnel.id))
        .map_err(to_command_error)
}

#[tauri::command]
pub fn has_tunnel_password(tunnel_id: String) -> CommandResult<bool> {
    let settings = config::load_settings().map_err(to_command_error)?;
    let tunnel = tunnel_by_id(&settings, &tunnel_id).map_err(to_command_error)?;
    Ok(credential::has_secret(&credential::tunnel_password_key(
        &tunnel.id,
    )))
}

#[tauri::command]
pub async fn test_ssh(tunnel_id: String) -> CommandResult<()> {
    let settings = config::load_settings().map_err(to_command_error)?;
    validation::validate_settings(&settings).map_err(to_command_error)?;
    let tunnel = tunnel_by_id(&settings, &tunnel_id).map_err(to_command_error)?;
    validation::validate_tunnel_for_connection(&tunnel).map_err(to_command_error)?;
    let password = password_for_tunnel(&tunnel).map_err(to_command_error)?;
    tunnel::test_ssh(&tunnel.ssh, password)
        .await
        .map_err(to_command_error)
}

#[tauri::command]
pub async fn start_profile(state: State<'_, AppState>) -> CommandResult<AppStatus> {
    let started = std::time::Instant::now();
    match start_profile_for_state(state.inner()).await {
        Ok(status) => Ok(status),
        Err(error) => {
            tracing::error!(
                elapsed_ms = started.elapsed().as_millis(),
                error = %error,
                "Failed to start profile"
            );
            Err(error)
        }
    }
}

pub async fn start_profile_for_state(state: &AppState) -> CommandResult<AppStatus> {
    let started = std::time::Instant::now();
    let already_running = {
        let guard = state
            .tunnels
            .lock()
            .map_err(|_| String::from("Tunnel state lock is poisoned"))?;
        !guard.is_empty()
    };
    if already_running {
        return get_status_for_state(state);
    }

    let settings = config::load_settings().map_err(to_command_error)?;
    let profiles = config::load_profiles().map_err(to_command_error)?;
    validation::validate_settings_with_profiles(&settings, &profiles).map_err(to_command_error)?;
    let profile = current_profile(&settings, &profiles).map_err(to_command_error)?;
    let enabled_services = profile
        .services
        .iter()
        .filter(|service| service.enabled)
        .cloned()
        .collect::<Vec<_>>();
    tracing::info!(
        service_count = enabled_services.len(),
        elapsed_ms = started.elapsed().as_millis(),
        "Start profile config loaded"
    );

    let hosts_started = std::time::Instant::now();
    hosts::write_services_block(&enabled_services).map_err(to_command_error)?;
    tracing::info!(
        service_count = enabled_services.len(),
        elapsed_ms = hosts_started.elapsed().as_millis(),
        total_elapsed_ms = started.elapsed().as_millis(),
        "Start profile hosts updated"
    );
    let services_by_tunnel = group_services_by_tunnel(enabled_services);
    let tunnels_by_id = settings
        .tunnels
        .iter()
        .cloned()
        .map(|tunnel| (tunnel.id.clone(), tunnel))
        .collect::<HashMap<_, _>>();
    let mut start_plan = Vec::new();

    for (tunnel_id, services) in services_by_tunnel {
        let tunnel_config = tunnels_by_id
            .get(&tunnel_id)
            .cloned()
            .ok_or_else(|| format!("Tunnel not found: {tunnel_id}"))?;
        if !tunnel_config.enabled {
            let _ = hosts::remove_block();
            return Err(format!("Tunnel is disabled: {}", tunnel_config.name));
        }
        if let Err(error) = validation::validate_tunnel_for_connection(&tunnel_config) {
            let _ = hosts::remove_block();
            return Err(to_command_error(error));
        }
        let password = password_for_tunnel(&tunnel_config).map_err(|error| {
            let _ = hosts::remove_block();
            to_command_error(error)
        })?;
        start_plan.push((tunnel_config, services, password));
    }

    let mut runtimes = HashMap::new();

    for (tunnel_config, services, password) in start_plan {
        let tunnel_started = std::time::Instant::now();
        match tunnel::start(tunnel_config, services, password).await {
            Ok(runtime) => {
                let tunnel_id = runtime.tunnel_id().to_string();
                tracing::info!(
                    tunnel_id = %tunnel_id,
                    elapsed_ms = tunnel_started.elapsed().as_millis(),
                    total_elapsed_ms = started.elapsed().as_millis(),
                    "Tunnel runtime started"
                );
                runtimes.insert(tunnel_id, runtime);
            }
            Err(error) => {
                for (_, runtime) in runtimes {
                    runtime.stop().await;
                }
                let _ = hosts::remove_block();
                return Err(to_command_error(error));
            }
        }
    }

    let mut guard = state
        .tunnels
        .lock()
        .map_err(|_| String::from("Tunnel state lock is poisoned"))?;
    *guard = runtimes;
    drop(guard);

    let mut active_profile = state
        .active_profile_id
        .lock()
        .map_err(|_| String::from("Active profile state lock is poisoned"))?;
    *active_profile = Some(profile.id.clone());
    drop(active_profile);

    let status = get_status_for_state(state)?;
    tracing::info!(
        elapsed_ms = started.elapsed().as_millis(),
        "Start profile completed"
    );
    Ok(status)
}

#[tauri::command]
pub async fn stop_profile(state: State<'_, AppState>) -> CommandResult<AppStatus> {
    let runtimes = {
        let mut guard = state
            .tunnels
            .lock()
            .map_err(|_| String::from("Tunnel state lock is poisoned"))?;
        std::mem::take(&mut *guard)
    };

    for (_, runtime) in runtimes {
        runtime.stop().await;
    }
    let mut active_profile = state
        .active_profile_id
        .lock()
        .map_err(|_| String::from("Active profile state lock is poisoned"))?;
    *active_profile = None;
    drop(active_profile);

    hosts::remove_block().map_err(to_command_error)?;
    get_status_for_state(state.inner())
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> CommandResult<AppStatus> {
    get_status_for_state(state.inner())
}

pub fn get_status_for_state(state: &AppState) -> CommandResult<AppStatus> {
    let started = std::time::Instant::now();
    let settings = config::load_settings().unwrap_or_default();
    let profiles = config::load_profiles().unwrap_or_default();
    let (running_tunnel_ids, runtime_service_statuses) = state
        .tunnels
        .lock()
        .ok()
        .map(|guard| {
            let mut ids = guard.keys().cloned().collect::<Vec<_>>();
            ids.sort();
            let statuses = guard
                .values()
                .flat_map(|runtime| runtime.service_statuses())
                .map(|status| (status.service_id.clone(), status))
                .collect::<HashMap<_, _>>();
            (ids, statuses)
        })
        .unwrap_or_default();
    let running = !running_tunnel_ids.is_empty();
    let running_tunnel_set = running_tunnel_ids.iter().cloned().collect::<HashSet<_>>();
    let active_profile = state
        .active_profile_id
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    let current_profile_id = active_profile.unwrap_or_else(|| settings.current_profile_id.clone());
    let services = profiles
        .profiles
        .iter()
        .find(|profile| profile.id == current_profile_id)
        .map(|profile| {
            service_statuses_for_profile(
                &profile.services,
                &running_tunnel_set,
                &runtime_service_statuses,
            )
        })
        .unwrap_or_default();
    let tunnel_statuses = settings
        .tunnels
        .iter()
        .map(|tunnel| TunnelStatus {
            tunnel_id: tunnel.id.clone(),
            name: tunnel.name.clone(),
            running: running_tunnel_set.contains(&tunnel.id),
            message: if running_tunnel_set.contains(&tunnel.id) {
                String::from("Running")
            } else if tunnel.enabled {
                String::from("Stopped")
            } else {
                String::from("Disabled")
            },
        })
        .collect::<Vec<_>>();

    let privilege = health::privilege_status();
    let status = AppStatus {
        running,
        current_profile_id,
        running_tunnel_ids,
        tunnels: tunnel_statuses,
        is_admin: health::is_admin(),
        privilege,
        hosts_block_present: hosts::block_present(),
        message: if running {
            String::from("Running")
        } else {
            String::from("Stopped")
        },
        services,
    };

    tracing::info!(
        service_count = status.services.len(),
        elapsed_ms = started.elapsed().as_millis(),
        "Status refresh completed"
    );

    Ok(status)
}

fn service_statuses_for_profile(
    services: &[ServiceConfig],
    running_tunnel_ids: &HashSet<String>,
    runtime_statuses: &HashMap<String, ServiceStatus>,
) -> Vec<ServiceStatus> {
    services
        .iter()
        .map(|service| {
            if !service.enabled {
                return ServiceStatus {
                    service_id: service.id.clone(),
                    state: ServiceState::Disabled,
                    message: String::from("Disabled"),
                };
            }

            if !running_tunnel_ids.contains(&service.tunnel_id) {
                return ServiceStatus {
                    service_id: service.id.clone(),
                    state: ServiceState::Stopped,
                    message: String::from("Tunnel is not running"),
                };
            }

            runtime_statuses
                .get(&service.id)
                .cloned()
                .unwrap_or_else(|| ServiceStatus {
                    service_id: service.id.clone(),
                    state: ServiceState::Error,
                    message: String::from("Service runtime status is unavailable"),
                })
        })
        .collect()
}

#[tauri::command]
pub fn test_service(service_id: String) -> CommandResult<ServiceStatus> {
    let profiles = config::load_profiles().map_err(to_command_error)?;
    for profile in profiles.profiles {
        if let Some(service) = profile
            .services
            .into_iter()
            .find(|service| service.id == service_id)
        {
            return Ok(health::service_status(&service));
        }
    }
    Err(format!("Service not found: {service_id}"))
}

#[tauri::command]
pub fn repair_hosts() -> CommandResult<()> {
    hosts::remove_block().map_err(to_command_error)
}

#[tauri::command]
pub fn read_logs(max_lines: Option<usize>) -> CommandResult<Vec<LogEntry>> {
    let max_lines = max_lines.unwrap_or(600).clamp(1, 2_000);
    let logs_dir = config::logs_dir().map_err(to_command_error)?;
    let mut files = fs::read_dir(logs_dir)
        .map_err(|error| error.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with("tunneldesk.log") {
                return None;
            }

            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            Some((modified, entry.path(), file_name))
        })
        .collect::<Vec<_>>();

    files.sort_by_key(|(modified, _, file_name)| {
        (
            modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default(),
            file_name.clone(),
        )
    });

    let mut entries = VecDeque::with_capacity(max_lines);
    let recent_file_start = files.len().saturating_sub(8);

    for (_, path, file_name) in files.into_iter().skip(recent_file_start) {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };

        for (line_index, raw_line) in content.lines().enumerate() {
            let raw = strip_ansi_codes(raw_line).trim_end().to_string();
            if raw.trim().is_empty() {
                continue;
            }

            let id = format!("{file_name}:{}", line_index + 1);
            entries.push_back(parse_log_entry(id, raw));
            if entries.len() > max_lines {
                entries.pop_front();
            }
        }
    }

    Ok(entries.into_iter().collect())
}

#[tauri::command]
pub fn open_log_dir() -> CommandResult<()> {
    let path = config::logs_dir().map_err(to_command_error)?;
    #[cfg(target_os = "windows")]
    let opener = "explorer";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return Err(String::from(
        "Opening log directory is not supported on this platform",
    ));

    Command::new(opener)
        .arg(path)
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn strip_ansi_codes(value: &str) -> String {
    let mut cleaned = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            if chars.next_if_eq(&'[').is_some() {
                for sequence_character in chars.by_ref() {
                    if ('@'..='~').contains(&sequence_character) {
                        break;
                    }
                }
            }
            continue;
        }

        if character.is_control() && character != '\t' {
            continue;
        }

        cleaned.push(character);
    }

    cleaned
}

fn parse_log_entry(id: String, raw: String) -> LogEntry {
    let tokens = raw.split_whitespace().collect::<Vec<_>>();
    let level_position = tokens
        .iter()
        .enumerate()
        .find_map(|(index, token)| normalize_log_level(token).map(|level| (index, level)));

    let Some((level_index, level)) = level_position else {
        return LogEntry {
            id,
            timestamp: String::new(),
            level: String::from("UNKNOWN"),
            target: String::new(),
            message: raw.clone(),
            raw,
        };
    };

    let timestamp = tokens[..level_index].join(" ");
    let after_level = tokens
        .get(level_index + 1..)
        .map(|items| items.join(" "))
        .unwrap_or_default();
    let (target, message) = after_level
        .split_once(": ")
        .map(|(target, message)| {
            (
                target.trim_end_matches(':').to_string(),
                message.to_string(),
            )
        })
        .unwrap_or_else(|| (String::new(), after_level));

    LogEntry {
        id,
        timestamp,
        level: String::from(level),
        target,
        message,
        raw,
    }
}

fn normalize_log_level(value: &str) -> Option<&'static str> {
    match value
        .trim_matches(|character: char| !character.is_ascii_alphabetic())
        .to_ascii_uppercase()
        .as_str()
    {
        "TRACE" => Some("TRACE"),
        "DEBUG" => Some("DEBUG"),
        "INFO" => Some("INFO"),
        "WARN" | "WARNING" => Some("WARN"),
        "ERROR" => Some("ERROR"),
        _ => None,
    }
}

fn current_profile(
    settings: &AppSettings,
    profiles: &ProfilesFile,
) -> Result<ServiceProfile, AppError> {
    profiles
        .profiles
        .iter()
        .find(|profile| profile.id == settings.current_profile_id)
        .cloned()
        .or_else(|| profiles.profiles.first().cloned())
        .ok_or_else(|| AppError::Message(String::from("No service profile configured")))
}

fn tunnel_by_id(settings: &AppSettings, tunnel_id: &str) -> Result<TunnelConfig, AppError> {
    let tunnel_id = tunnel_id.trim();
    if tunnel_id.is_empty() {
        return Err(AppError::Message(String::from("Tunnel id is required")));
    }
    settings
        .tunnels
        .iter()
        .find(|tunnel| tunnel.id == tunnel_id)
        .cloned()
        .ok_or_else(|| AppError::Message(format!("Tunnel not found: {tunnel_id}")))
}

fn group_services_by_tunnel(services: Vec<ServiceConfig>) -> HashMap<String, Vec<ServiceConfig>> {
    let mut grouped = HashMap::new();
    for service in services {
        grouped
            .entry(service.tunnel_id.clone())
            .or_insert_with(Vec::new)
            .push(service);
    }
    grouped
}

fn password_for_tunnel(tunnel: &TunnelConfig) -> Result<Option<String>, AppError> {
    if tunnel.ssh.auth_method != AuthMethod::Password {
        return Ok(None);
    }
    let key = credential::tunnel_password_key(&tunnel.id);
    Ok(Some(credential::get_secret(&key)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(id: &str) -> ServiceConfig {
        ServiceConfig {
            id: String::from(id),
            name: String::from(id),
            group: String::new(),
            domain: format!("{id}.example.internal"),
            remark: String::new(),
            port: 3306,
            local_ip: String::from("127.77.0.10"),
            tunnel_id: String::from("default"),
            sort_order: 10,
            enabled: true,
        }
    }

    #[test]
    fn running_profile_uses_runtime_forwarding_status() {
        let services = vec![service("mysql")];
        let running_tunnels = HashSet::from([String::from("default")]);
        let runtime_statuses = HashMap::from([(
            String::from("mysql"),
            ServiceStatus {
                service_id: String::from("mysql"),
                state: ServiceState::Error,
                message: String::from("Failed to open channel (AdministrativelyProhibited)"),
            },
        )]);

        let statuses = service_statuses_for_profile(&services, &running_tunnels, &runtime_statuses);

        assert_eq!(statuses.len(), 1);
        assert!(matches!(statuses[0].state, ServiceState::Error));
        assert!(statuses[0].message.contains("AdministrativelyProhibited"));
    }

    #[test]
    fn non_running_and_disabled_services_do_not_probe_local_listeners() {
        let enabled = service("mysql");
        let mut disabled = service("redis");
        disabled.enabled = false;

        let statuses =
            service_statuses_for_profile(&[enabled, disabled], &HashSet::new(), &HashMap::new());

        assert!(matches!(statuses[0].state, ServiceState::Stopped));
        assert!(matches!(statuses[1].state, ServiceState::Disabled));
    }

    #[test]
    fn missing_status_for_running_service_is_an_error() {
        let services = vec![service("mysql")];
        let running_tunnels = HashSet::from([String::from("default")]);

        let statuses = service_statuses_for_profile(&services, &running_tunnels, &HashMap::new());

        assert!(matches!(statuses[0].state, ServiceState::Error));
        assert!(statuses[0].message.contains("unavailable"));
    }

    #[test]
    fn parse_log_entry_handles_ansi_colored_tracing_lines() {
        let raw = "\u{1b}[2m2026-07-03T07:04:34.410788Z\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m \u{1b}[2mtunneldesk_lib::commands\u{1b}[0m\u{1b}[2m:\u{1b}[0m Status refresh completed \u{1b}[3mservice_count\u{1b}[0m\u{1b}[2m=\u{1b}[0m10";
        let entry = parse_log_entry(String::from("test:1"), strip_ansi_codes(raw));

        assert_eq!(entry.timestamp, "2026-07-03T07:04:34.410788Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.target, "tunneldesk_lib::commands");
        assert_eq!(entry.message, "Status refresh completed service_count=10");
        assert!(!entry.raw.contains('\u{1b}'));
    }
}
