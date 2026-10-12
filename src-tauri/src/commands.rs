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
pub async fn check_profile_start(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::model::StartupIssue>> {
    let _operation = state.operation.lock().await;
    if !state
        .tunnels
        .lock()
        .map_err(|_| "Tunnel state lock is poisoned")?
        .is_empty()
    {
        return Err(String::from("Profile is already running"));
    }
    let settings = config::load_settings().map_err(to_command_error)?;
    let profiles = config::load_profiles().map_err(to_command_error)?;
    validation::validate_settings_with_profiles(&settings, &profiles).map_err(to_command_error)?;
    let profile = current_profile(&settings, &profiles).map_err(to_command_error)?;
    let services = profile
        .services
        .into_iter()
        .filter(|service| service.enabled)
        .collect::<Vec<_>>();
    let (listeners, issues) = crate::listener_diagnostics::prepare_listeners(&services).await;
    drop(listeners);
    Ok(issues)
}

#[tauri::command]
pub async fn start_profile(
    state: State<'_, AppState>,
    mode: Option<crate::model::StartMode>,
) -> CommandResult<AppStatus> {
    start_profile_with_mode(state.inner(), mode, false).await
}

pub async fn start_profile_for_state(state: &AppState) -> CommandResult<AppStatus> {
    start_profile_with_mode(state, None, false).await
}

#[tauri::command]
pub async fn retry_skipped_services(state: State<'_, AppState>) -> CommandResult<AppStatus> {
    start_profile_with_mode(state.inner(), Some(crate::model::StartMode::Partial), true).await
}

async fn start_profile_with_mode(
    state: &AppState,
    mode: Option<crate::model::StartMode>,
    retry: bool,
) -> CommandResult<AppStatus> {
    let _operation = state.operation.lock().await;
    let settings = config::load_settings().map_err(to_command_error)?;
    let profiles = config::load_profiles().map_err(to_command_error)?;
    validation::validate_settings_with_profiles(&settings, &profiles).map_err(to_command_error)?;
    let profile = current_profile(&settings, &profiles).map_err(to_command_error)?;
    let mode = mode.unwrap_or(settings.behavior.start_mode);
    let (existing_services, resources) = {
        let guard = state
            .tunnels
            .lock()
            .map_err(|_| "Tunnel state lock is poisoned")?;
        if !guard.is_empty() && !retry {
            drop(guard);
            return get_status_for_state(state);
        }
        if retry {
            let snapshot = state
                .startup_profile
                .lock()
                .map_err(|_| "Startup profile state lock is poisoned")?;
            if snapshot.as_ref().is_none_or(|saved| {
                serde_json::to_value(saved).ok() != serde_json::to_value(&profile).ok()
            }) {
                return Err(String::from("Profile 配置已改变，请停止后重新启动"));
            }
            let saved_tunnels = state
                .startup_tunnels
                .lock()
                .map_err(|_| "Startup tunnel state lock is poisoned")?;
            for saved in saved_tunnels.iter() {
                if settings
                    .tunnels
                    .iter()
                    .find(|tunnel| tunnel.id == saved.id)
                    .is_none_or(|tunnel| {
                        serde_json::to_value(tunnel).ok() != serde_json::to_value(saved).ok()
                    })
                {
                    return Err(String::from("隧道配置已改变，请停止后重新启动"));
                }
            }
        }
        let services = guard
            .values()
            .flat_map(|runtime| runtime.services().iter().cloned())
            .collect::<Vec<_>>();
        let resources = guard
            .iter()
            .map(|(id, runtime)| (id.clone(), runtime.resources()))
            .collect::<HashMap<_, _>>();
        (services, resources)
    };
    let active_ids = existing_services
        .iter()
        .map(|service| service.id.as_str())
        .collect::<HashSet<_>>();
    let candidates = profile
        .services
        .iter()
        .filter(|service| service.enabled && !active_ids.contains(service.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if candidates.is_empty() && existing_services.is_empty() {
        return Err(String::from("No enabled services in profile"));
    }
    let execution = crate::profile_start::start(
        &settings,
        &profile.id,
        &candidates,
        &existing_services,
        resources,
        mode,
    )
    .await;
    match execution.report.outcome {
        crate::model::StartupOutcome::Failed => {
            tracing::error!(message = %execution.report.message, "Profile startup failed")
        }
        crate::model::StartupOutcome::Partial => tracing::warn!(
            started = execution.report.started_service_ids.len(),
            skipped = execution.report.issues.len(),
            "Profile partially started"
        ),
        crate::model::StartupOutcome::Complete => tracing::info!(
            started = execution.report.started_service_ids.len(),
            "Profile startup completed"
        ),
    }
    {
        let mut guard = state
            .tunnels
            .lock()
            .map_err(|_| "Tunnel state lock is poisoned")?;
        for (id, runtime) in execution.runtimes {
            if let Some(existing) = guard.get_mut(&id) {
                existing.absorb(runtime);
            } else {
                guard.insert(id, runtime);
            }
        }
        *state
            .active_profile_id
            .lock()
            .map_err(|_| "Active profile state lock is poisoned")? =
            (!guard.is_empty()).then(|| profile.id.clone());
    }
    *state
        .startup_tunnels
        .lock()
        .map_err(|_| "Startup tunnel state lock is poisoned")? = settings
        .tunnels
        .iter()
        .filter(|tunnel| {
            profile
                .services
                .iter()
                .any(|service| service.enabled && service.tunnel_id == tunnel.id)
        })
        .cloned()
        .collect();
    *state
        .startup_profile
        .lock()
        .map_err(|_| "Startup profile state lock is poisoned")? = Some(profile);
    *state
        .startup_report
        .lock()
        .map_err(|_| "Startup report state lock is poisoned")? = Some(execution.report);
    get_status_for_state(state)
}
#[tauri::command]
pub async fn stop_profile(state: State<'_, AppState>) -> CommandResult<AppStatus> {
    let _operation = state.operation.lock().await;
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
    *state
        .startup_report
        .lock()
        .map_err(|_| "Startup report state lock is poisoned")? = None;
    *state
        .startup_profile
        .lock()
        .map_err(|_| "Startup profile state lock is poisoned")? = None;

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
    let startup_report = state
        .startup_report
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .filter(|report| report.profile_id == current_profile_id);
    let mut services = profiles
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
    if let Some(report) = &startup_report {
        for issue in &report.issues {
            if let Some(status) = services
                .iter_mut()
                .find(|status| status.service_id == issue.service_id)
            {
                status.state = if running {
                    ServiceState::Skipped
                } else {
                    ServiceState::Error
                };
                status.message = issue.message.clone();
            }
        }
    }
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
        startup_report,
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
pub fn test_service(
    state: State<'_, AppState>,
    service_id: String,
) -> CommandResult<ServiceStatus> {
    let status = get_status_for_state(state.inner())?;
    let managed_status = status
        .services
        .iter()
        .find(|service| service.service_id == service_id);
    if let Some(service) = managed_status {
        if !matches!(service.state, ServiceState::Healthy) {
            return Ok(service.clone());
        }
    }
    let profiles = config::load_profiles().map_err(to_command_error)?;
    for profile in profiles
        .profiles
        .into_iter()
        .filter(|profile| profile.id == status.current_profile_id)
    {
        if let Some(service) = profile
            .services
            .into_iter()
            .find(|service| service.id == service_id)
        {
            if managed_status.is_none() {
                return Ok(ServiceStatus {
                    service_id,
                    state: if service.enabled {
                        ServiceState::Stopped
                    } else {
                        ServiceState::Disabled
                    },
                    message: String::from("Service runtime is not running"),
                });
            }
            return Ok(health::service_status(&service));
        }
    }
    Err(format!("Service not found: {service_id}"))
}

#[tauri::command]
pub async fn repair_hosts(state: State<'_, AppState>) -> CommandResult<()> {
    let _operation = state.operation.lock().await;
    if !state
        .tunnels
        .lock()
        .map_err(|_| "Tunnel state lock is poisoned")?
        .is_empty()
    {
        return Err(String::from("请先停止 Profile 再修复 hosts"));
    }
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
