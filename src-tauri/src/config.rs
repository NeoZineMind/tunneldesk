use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::credential;
use crate::error::{AppError, AppResult};
use crate::model::{
    AppSettings, ConfigBackupFile, ConfigBackupInfo, ConfigRestoreResult, ProfilesFile, APP_NAME,
    DEFAULT_TUNNEL_ID, PROFILES_SCHEMA_VERSION, SETTINGS_SCHEMA_VERSION,
};

pub fn app_dir() -> AppResult<PathBuf> {
    let dir = platform_app_dir()?;
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join("logs"))?;
    fs::create_dir_all(dir.join("backups"))?;
    Ok(dir)
}

#[cfg(target_os = "windows")]
fn platform_app_dir() -> AppResult<PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| crate::error::AppError::Message(String::from("APPDATA is not set")))?;
    Ok(PathBuf::from(appdata).join(APP_NAME))
}

#[cfg(target_os = "macos")]
fn platform_app_dir() -> AppResult<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| crate::error::AppError::Message(String::from("HOME is not set")))?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join(APP_NAME))
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn platform_app_dir() -> AppResult<PathBuf> {
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(data_home).join(APP_NAME));
    }
    let home = std::env::var("HOME")
        .map_err(|_| crate::error::AppError::Message(String::from("HOME is not set")))?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join(APP_NAME))
}

pub fn settings_path() -> AppResult<PathBuf> {
    Ok(app_dir()?.join("settings.json"))
}

pub fn profiles_path() -> AppResult<PathBuf> {
    Ok(app_dir()?.join("profiles.json"))
}

pub fn logs_dir() -> AppResult<PathBuf> {
    Ok(app_dir()?.join("logs"))
}

pub fn backups_dir() -> AppResult<PathBuf> {
    Ok(app_dir()?.join("backups"))
}

pub fn load_settings() -> AppResult<AppSettings> {
    read_settings_or_default(settings_path()?)
}

pub fn save_settings(settings: &AppSettings) -> AppResult<()> {
    let mut normalized = settings.clone();
    normalize_settings_credentials(&mut normalized);
    write_json_atomic(settings_path()?, &normalized)
}

pub fn load_profiles() -> AppResult<ProfilesFile> {
    read_profiles_or_default(profiles_path()?)
}

pub fn save_profiles(profiles: &ProfilesFile) -> AppResult<()> {
    write_json_atomic(profiles_path()?, profiles)
}

pub fn load_profiles_from_path(path: PathBuf) -> AppResult<ProfilesFile> {
    let content = fs::read_to_string(path)?;
    let mut value: serde_json::Value = serde_json::from_str(&content)?;
    migrate_profiles_value(&mut value);
    Ok(serde_json::from_value(value)?)
}

pub fn save_profiles_to_path(path: PathBuf, profiles: &ProfilesFile) -> AppResult<()> {
    write_json_atomic(path, profiles)
}

pub fn create_config_backup() -> AppResult<PathBuf> {
    create_config_backup_in_dir(backups_dir()?, load_settings()?, load_profiles()?)
}

pub fn create_config_backup_in_dir(
    backup_dir: PathBuf,
    settings: AppSettings,
    profiles: ProfilesFile,
) -> AppResult<PathBuf> {
    fs::create_dir_all(&backup_dir)?;
    let created_at = Local::now();
    let timestamp = created_at.format("%Y%m%d-%H%M%S");
    let mut candidate = backup_dir.join(format!("{timestamp}-config.json"));
    let mut suffix = 1;
    while candidate.exists() {
        candidate = backup_dir.join(format!("{timestamp}-{suffix}-config.json"));
        suffix += 1;
    }

    let backup = ConfigBackupFile {
        schema_version: 1,
        created_at: created_at.to_rfc3339(),
        settings,
        profiles,
    };
    write_json_atomic(candidate.clone(), &backup)?;
    prune_config_backups_in_dir(&backup_dir, 20)?;
    Ok(candidate)
}

pub fn list_config_backups() -> AppResult<Vec<ConfigBackupInfo>> {
    list_config_backups_in_dir(backups_dir()?)
}

pub fn list_config_backups_in_dir(backup_dir: PathBuf) -> AppResult<Vec<ConfigBackupInfo>> {
    fs::create_dir_all(&backup_dir)?;
    let mut backups = fs::read_dir(backup_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.ends_with("-config.json") {
                return None;
            }
            let id = file_name.trim_end_matches(".json").to_string();
            let path = entry.path();
            let created_at = read_backup_created_at(&path)
                .unwrap_or_else(|| backup_created_at_from_file_name(&file_name));
            Some(ConfigBackupInfo {
                id,
                file_name,
                created_at,
            })
        })
        .collect::<Vec<_>>();
    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(backups)
}

pub fn restore_config_backup(backup_id: String) -> AppResult<ConfigRestoreResult> {
    let backup = read_config_backup(backup_id)?;
    validation_for_restore(&backup)?;
    save_settings(&backup.settings)?;
    save_profiles(&backup.profiles)?;
    Ok(ConfigRestoreResult {
        settings: load_settings()?,
        profiles: load_profiles()?,
    })
}

pub fn delete_config_backup(backup_id: String) -> AppResult<()> {
    let path = config_backup_path(backup_id)?;
    if !path.exists() {
        return Err(AppError::Message(String::from("Backup not found")));
    }
    fs::remove_file(path)?;
    Ok(())
}

fn read_config_backup(backup_id: String) -> AppResult<ConfigBackupFile> {
    let path = config_backup_path(backup_id)?;
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn config_backup_path(backup_id: String) -> AppResult<PathBuf> {
    if backup_id.contains('/') || backup_id.contains('\\') || backup_id.contains("..") {
        return Err(AppError::Message(String::from("Invalid backup id")));
    }
    Ok(backups_dir()?.join(format!("{backup_id}.json")))
}

fn read_backup_created_at(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    value
        .get("createdAt")
        .and_then(serde_json::Value::as_str)
        .map(String::from)
}

fn backup_created_at_from_file_name(file_name: &str) -> String {
    file_name
        .split("-config.json")
        .next()
        .unwrap_or(file_name)
        .to_string()
}

fn prune_config_backups_in_dir(backup_dir: &Path, keep: usize) -> AppResult<()> {
    let backups = list_config_backups_in_dir(backup_dir.to_path_buf())?;
    for backup in backups.into_iter().skip(keep) {
        let path = backup_dir.join(format!("{}.json", backup.id));
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn validation_for_restore(backup: &ConfigBackupFile) -> AppResult<()> {
    crate::validation::validate_settings_with_profiles(&backup.settings, &backup.profiles)
}

fn read_settings_or_default(path: PathBuf) -> AppResult<AppSettings> {
    if !path.exists() {
        let mut value = AppSettings::default();
        normalize_settings_credentials(&mut value);
        write_json_atomic(path, &value)?;
        return Ok(value);
    }

    let content = fs::read_to_string(path)?;
    let mut value: serde_json::Value = serde_json::from_str(&content)?;
    migrate_settings_value(&mut value);
    let mut settings: AppSettings = serde_json::from_value(value)?;
    migrate_password_credentials(&settings);
    normalize_settings_credentials(&mut settings);
    Ok(settings)
}

fn read_profiles_or_default(path: PathBuf) -> AppResult<ProfilesFile> {
    if !path.exists() {
        let value = ProfilesFile::default();
        write_json_atomic(path, &value)?;
        return Ok(value);
    }

    let content = fs::read_to_string(path)?;
    let mut value: serde_json::Value = serde_json::from_str(&content)?;
    migrate_profiles_value(&mut value);
    Ok(serde_json::from_value(value)?)
}

fn migrate_settings_value(value: &mut serde_json::Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };

    object.insert(
        String::from("schemaVersion"),
        serde_json::Value::from(SETTINGS_SCHEMA_VERSION),
    );

    let legacy_ssh = object.remove("ssh").unwrap_or_else(default_ssh_value);
    object
        .entry(String::from("currentTunnelId"))
        .or_insert_with(|| serde_json::Value::String(String::from(DEFAULT_TUNNEL_ID)));

    object.entry(String::from("tunnels")).or_insert_with(|| {
        serde_json::json!([
            {
                "id": DEFAULT_TUNNEL_ID,
                "name": "Default Tunnel",
                "enabled": true,
                "ssh": legacy_ssh
            }
        ])
    });

    let behavior = object
        .entry(String::from("behavior"))
        .or_insert_with(|| serde_json::json!({}));
    if let Some(behavior) = behavior.as_object_mut() {
        behavior
            .entry(String::from("startMinimized"))
            .or_insert(serde_json::Value::Bool(false));
        behavior
            .entry(String::from("autoStartProfile"))
            .or_insert(serde_json::Value::Bool(false));
        behavior
            .entry(String::from("launchAtLogin"))
            .or_insert(serde_json::Value::Bool(false));
        behavior
            .entry(String::from("autoRepairOnStart"))
            .or_insert(serde_json::Value::Bool(false));
        behavior
            .entry(String::from("cleanupOnExit"))
            .or_insert(serde_json::Value::Bool(true));
        behavior
            .entry(String::from("themeMode"))
            .or_insert_with(|| serde_json::Value::String(String::from("system")));
        behavior
            .entry(String::from("closeAction"))
            .or_insert_with(|| serde_json::Value::String(String::from("ask")));
    }
}

fn migrate_profiles_value(value: &mut serde_json::Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };

    object.insert(
        String::from("schemaVersion"),
        serde_json::Value::from(PROFILES_SCHEMA_VERSION),
    );

    if let Some(profiles) = object
        .get_mut("profiles")
        .and_then(serde_json::Value::as_array_mut)
    {
        for profile in profiles {
            let Some(profile) = profile.as_object_mut() else {
                continue;
            };
            if let Some(services) = profile
                .get_mut("services")
                .and_then(serde_json::Value::as_array_mut)
            {
                for (index, service) in services.iter_mut().enumerate() {
                    if let Some(service) = service.as_object_mut() {
                        service.entry(String::from("tunnelId")).or_insert_with(|| {
                            serde_json::Value::String(String::from(DEFAULT_TUNNEL_ID))
                        });
                        service
                            .entry(String::from("group"))
                            .or_insert_with(|| serde_json::Value::String(String::new()));
                        service.entry(String::from("sortOrder")).or_insert_with(|| {
                            serde_json::Value::Number(serde_json::Number::from(
                                ((index + 1) * 10) as u64,
                            ))
                        });
                    }
                }
            }
        }
    }
}

fn default_ssh_value() -> serde_json::Value {
    serde_json::json!({
        "host": "",
        "port": 22,
        "username": "",
        "authMethod": "password",
        "identityFile": "",
        "passwordCredentialKey": "",
        "keyPassphraseCredentialKey": "",
        "serverAliveInterval": 30,
        "serverAliveCountMax": 3
    })
}

pub fn normalize_settings_credentials(settings: &mut AppSettings) {
    for tunnel in &mut settings.tunnels {
        tunnel.ssh.password_credential_key = credential::tunnel_password_key(&tunnel.id);
    }
}

fn migrate_password_credentials(settings: &AppSettings) {
    for tunnel in &settings.tunnels {
        let legacy_key = tunnel.ssh.password_credential_key.trim();
        let canonical_key = credential::tunnel_password_key(&tunnel.id);
        if legacy_key.is_empty()
            || legacy_key == canonical_key
            || credential::has_secret(&canonical_key)
        {
            continue;
        }

        if let Ok(value) = credential::get_secret(legacy_key) {
            let _ = credential::save_secret(&canonical_key, &value);
        }
    }
}

fn write_json_atomic<T>(path: PathBuf, value: &T) -> AppResult<()>
where
    T: serde::Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(value)?;
    fs::write(&tmp, content)?;
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_v1_settings_to_default_tunnel() {
        let mut value = serde_json::json!({
            "schemaVersion": 1,
            "currentProfileId": "default",
            "ssh": {
                "host": "jump.example.com",
                "port": 22,
                "username": "developer",
                "authMethod": "password",
                "identityFile": "",
                "passwordCredentialKey": "legacy-key",
                "keyPassphraseCredentialKey": "",
                "serverAliveInterval": 30,
                "serverAliveCountMax": 3
            },
                "behavior": {
                    "startMinimized": false,
                    "autoRepairOnStart": false,
                    "cleanupOnExit": true
            }
        });

        migrate_settings_value(&mut value);
        let settings: AppSettings = serde_json::from_value(value).unwrap();

        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(settings.current_tunnel_id, DEFAULT_TUNNEL_ID);
        assert_eq!(settings.tunnels.len(), 1);
        assert_eq!(settings.tunnels[0].ssh.host, "jump.example.com");
        assert_eq!(
            settings.tunnels[0].ssh.password_credential_key,
            "legacy-key"
        );
        assert_eq!(
            settings.behavior.theme_mode,
            crate::model::ThemeMode::System
        );
        assert_eq!(
            settings.behavior.close_action,
            crate::model::CloseAction::Ask
        );
        assert!(!settings.behavior.auto_start_profile);
        assert!(!settings.behavior.launch_at_login);
    }

    #[test]
    fn normalizes_tunnel_password_keys() {
        let mut settings = AppSettings::default();
        settings.tunnels[0].ssh.password_credential_key = String::from("frontend-controlled-key");

        normalize_settings_credentials(&mut settings);

        assert_eq!(
            settings.tunnels[0].ssh.password_credential_key,
            "TunnelDesk:tunnel:default:password"
        );
    }

    #[test]
    fn migrates_v1_profiles_to_default_tunnel() {
        let mut value = serde_json::json!({
            "schemaVersion": 1,
            "profiles": [
                {
                    "id": "default",
                    "name": "Default Profile",
                    "enabled": true,
                    "services": [
                        {
                            "id": "mysql",
                            "name": "MySQL",
                            "domain": "mysql.example.internal",
                            "port": 3306,
                            "localIp": "127.77.0.10",
                            "enabled": true
                        }
                    ]
                }
            ]
        });

        migrate_profiles_value(&mut value);
        let profiles: ProfilesFile = serde_json::from_value(value).unwrap();

        assert_eq!(profiles.schema_version, PROFILES_SCHEMA_VERSION);
        assert_eq!(
            profiles.profiles[0].services[0].tunnel_id,
            DEFAULT_TUNNEL_ID
        );
        assert_eq!(profiles.profiles[0].services[0].group, "");
        assert_eq!(profiles.profiles[0].services[0].sort_order, 10);
    }

    #[test]
    fn creates_config_backup_with_timestamped_name() {
        let temp = tempfile::tempdir().unwrap();
        let backup_dir = temp.path().join("backups");

        let backup = create_config_backup_in_dir(
            backup_dir,
            AppSettings::default(),
            ProfilesFile::default(),
        )
        .unwrap();
        let file_name = backup.file_name().and_then(|name| name.to_str()).unwrap();

        assert!(backup.exists());
        assert!(file_name.ends_with("-config.json"));
    }

    #[test]
    fn prunes_old_config_backups() {
        let temp = tempfile::tempdir().unwrap();
        let backup_dir = temp.path().join("backups");

        for _ in 0..22 {
            create_config_backup_in_dir(
                backup_dir.clone(),
                AppSettings::default(),
                ProfilesFile::default(),
            )
            .unwrap();
        }

        let backups = list_config_backups_in_dir(backup_dir).unwrap();
        assert_eq!(backups.len(), 20);
    }
}
