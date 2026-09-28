use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::error::{AppError, AppResult};
use crate::model::{
    AppSettings, ProfilesFile, ServiceConfig, ServiceProfile, PROFILES_SCHEMA_VERSION,
};
use crate::validation;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelMapping {
    pub source_tunnel_id: String,
    pub target_tunnel_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingTunnelImport {
    pub source_tunnel_id: String,
    pub service_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOverwritePreview {
    pub profile_id: String,
    pub profile_name: String,
    pub service_id: String,
    pub old_name: String,
    pub old_group: String,
    pub old_domain: String,
    pub old_port: u16,
    pub old_local_ip: String,
    pub old_tunnel_id: String,
    pub old_sort_order: u32,
    pub new_name: String,
    pub new_group: String,
    pub new_domain: String,
    pub new_port: u16,
    pub new_local_ip: String,
    pub new_tunnel_id: String,
    pub new_sort_order: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceImportConflict {
    pub profile_id: String,
    pub profile_name: String,
    pub service_id: String,
    pub service_name: String,
    pub local_ip: String,
    pub port: u16,
    pub existing_service_id: String,
    pub existing_service_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesImportPreview {
    pub profile_count: usize,
    pub service_count: usize,
    pub added_profile_count: usize,
    pub added_service_count: usize,
    pub updated_service_count: usize,
    pub skipped_service_count: usize,
    pub imported_profile_ids: Vec<String>,
    pub missing_tunnels: Vec<MissingTunnelImport>,
    pub overwrites: Vec<ServiceOverwritePreview>,
    pub conflicts: Vec<ServiceImportConflict>,
    pub can_apply: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesImportApplyResult {
    pub settings: AppSettings,
    pub profiles: ProfilesFile,
    pub preview: ProfilesImportPreview,
    pub backup_path: String,
}

struct PreparedImport {
    profiles: ProfilesFile,
    missing_tunnels: Vec<MissingTunnelImport>,
}

struct MergeOutcome {
    profiles: ProfilesFile,
    preview: ProfilesImportPreview,
}

pub fn export_profiles(path: PathBuf, profile_ids: Vec<String>) -> AppResult<()> {
    let profiles = config::load_profiles()?;
    let export = selected_profiles_export(&profiles, &profile_ids)?;
    config::save_profiles_to_path(path, &export)
}

pub fn preview_profiles_import(
    path: PathBuf,
    tunnel_mappings: Vec<TunnelMapping>,
) -> AppResult<ProfilesImportPreview> {
    let settings = config::load_settings()?;
    let current = config::load_profiles()?;
    let imported = config::load_profiles_from_path(path)?;
    preview_profiles_import_from_files(&settings, &current, imported, tunnel_mappings)
}

pub fn apply_profiles_import(
    path: PathBuf,
    tunnel_mappings: Vec<TunnelMapping>,
) -> AppResult<ProfilesImportApplyResult> {
    let mut settings = config::load_settings()?;
    let current = config::load_profiles()?;
    let imported = config::load_profiles_from_path(path)?;
    let prepared = prepare_imported_profiles(&settings, imported, &tunnel_mappings)?;
    if !prepared.missing_tunnels.is_empty() {
        return Err(AppError::Message(String::from(
            "Missing tunnel mappings must be resolved before import",
        )));
    }
    validation::validate_tunnel_references(&settings, &prepared.profiles)?;

    let first_imported_profile_id = prepared
        .profiles
        .profiles
        .first()
        .map(|profile| profile.id.clone())
        .ok_or_else(|| AppError::Message(String::from("No profiles found in import file")))?;
    let outcome = merge_profiles(current, prepared.profiles, Vec::new())?;

    let mut merged_settings = settings.clone();
    merged_settings.current_profile_id = first_imported_profile_id;
    validation::validate_settings_with_profiles(&merged_settings, &outcome.profiles)?;

    let backup_path = config::create_config_backup()?;
    config::save_profiles(&outcome.profiles)?;
    settings.current_profile_id = merged_settings.current_profile_id;
    config::save_settings(&settings)?;

    Ok(ProfilesImportApplyResult {
        settings,
        profiles: outcome.profiles,
        preview: outcome.preview,
        backup_path: backup_path.to_string_lossy().to_string(),
    })
}

fn preview_profiles_import_from_files(
    settings: &AppSettings,
    current: &ProfilesFile,
    imported: ProfilesFile,
    tunnel_mappings: Vec<TunnelMapping>,
) -> AppResult<ProfilesImportPreview> {
    let prepared = prepare_imported_profiles(settings, imported, &tunnel_mappings)?;
    let missing = prepared.missing_tunnels.clone();
    let mut outcome = merge_profiles(current.clone(), prepared.profiles, missing)?;
    outcome.preview.can_apply = outcome.preview.missing_tunnels.is_empty();
    Ok(outcome.preview)
}

fn selected_profiles_export(
    profiles: &ProfilesFile,
    profile_ids: &[String],
) -> AppResult<ProfilesFile> {
    let selected_ids = profile_ids
        .iter()
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .collect::<HashSet<_>>();
    let selected = if selected_ids.is_empty() {
        profiles.profiles.clone()
    } else {
        let profiles_by_id = profiles
            .profiles
            .iter()
            .map(|profile| (profile.id.as_str(), profile))
            .collect::<HashMap<_, _>>();
        let mut selected = Vec::new();
        for profile_id in selected_ids {
            let profile = profiles_by_id.get(profile_id).ok_or_else(|| {
                AppError::Message(format!("Profile not found for export: {profile_id}"))
            })?;
            selected.push((*profile).clone());
        }
        selected.sort_by(|a, b| a.id.cmp(&b.id));
        selected
    };

    if selected.is_empty() {
        return Err(AppError::Message(String::from(
            "At least one profile is required for export",
        )));
    }

    Ok(ProfilesFile {
        schema_version: PROFILES_SCHEMA_VERSION,
        profiles: selected,
    })
}

fn prepare_imported_profiles(
    settings: &AppSettings,
    imported: ProfilesFile,
    tunnel_mappings: &[TunnelMapping],
) -> AppResult<PreparedImport> {
    validation::validate_profiles(&imported)?;

    let tunnel_ids = settings
        .tunnels
        .iter()
        .map(|tunnel| tunnel.id.trim().to_string())
        .collect::<HashSet<_>>();
    let mappings = tunnel_mappings
        .iter()
        .map(|mapping| {
            (
                mapping.source_tunnel_id.trim().to_string(),
                mapping.target_tunnel_id.trim().to_string(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut missing_counts = HashMap::<String, usize>::new();
    let mut mapped = imported.clone();
    mapped.schema_version = PROFILES_SCHEMA_VERSION;

    for profile in &mut mapped.profiles {
        for service in &mut profile.services {
            let source_tunnel_id = service.tunnel_id.trim().to_string();
            if tunnel_ids.contains(&source_tunnel_id) {
                continue;
            }
            if let Some(target_tunnel_id) = mappings.get(&source_tunnel_id) {
                if tunnel_ids.contains(target_tunnel_id) {
                    service.tunnel_id = target_tunnel_id.clone();
                    continue;
                }
            }
            *missing_counts.entry(source_tunnel_id).or_default() += 1;
        }
    }

    let mut missing_tunnels = missing_counts
        .into_iter()
        .map(|(source_tunnel_id, service_count)| MissingTunnelImport {
            source_tunnel_id,
            service_count,
        })
        .collect::<Vec<_>>();
    missing_tunnels.sort_by(|a, b| a.source_tunnel_id.cmp(&b.source_tunnel_id));

    if missing_tunnels.is_empty() {
        validation::validate_tunnel_references(settings, &mapped)?;
    }

    Ok(PreparedImport {
        profiles: mapped,
        missing_tunnels,
    })
}

fn merge_profiles(
    mut current: ProfilesFile,
    imported: ProfilesFile,
    missing_tunnels: Vec<MissingTunnelImport>,
) -> AppResult<MergeOutcome> {
    let profile_count = imported.profiles.len();
    let service_count = imported
        .profiles
        .iter()
        .map(|profile| profile.services.len())
        .sum();
    let imported_profile_ids = imported
        .profiles
        .iter()
        .map(|profile| profile.id.clone())
        .collect::<Vec<_>>();
    let mut preview = ProfilesImportPreview {
        profile_count,
        service_count,
        added_profile_count: 0,
        added_service_count: 0,
        updated_service_count: 0,
        skipped_service_count: 0,
        imported_profile_ids,
        missing_tunnels,
        overwrites: Vec::new(),
        conflicts: Vec::new(),
        can_apply: true,
    };

    current.schema_version = PROFILES_SCHEMA_VERSION;

    for imported_profile in imported.profiles {
        if let Some(position) = current
            .profiles
            .iter()
            .position(|profile| profile.id == imported_profile.id)
        {
            merge_profile_services(
                &mut current.profiles[position],
                imported_profile,
                &mut preview,
            );
        } else {
            preview.added_profile_count += 1;
            preview.added_service_count += imported_profile.services.len();
            current.profiles.push(imported_profile);
        }
    }

    preview.skipped_service_count = preview.conflicts.len();
    validation::validate_profiles(&current)?;

    Ok(MergeOutcome {
        profiles: current,
        preview,
    })
}

fn merge_profile_services(
    target_profile: &mut ServiceProfile,
    imported_profile: ServiceProfile,
    preview: &mut ProfilesImportPreview,
) {
    let mut listeners = target_profile
        .services
        .iter()
        .map(|service| (listener_key(service), service.id.clone()))
        .collect::<HashMap<_, _>>();

    for imported_service in imported_profile.services {
        let listener = listener_key(&imported_service);
        let existing_listener_service_id = listeners.get(&listener).cloned();
        let existing_service_position = target_profile
            .services
            .iter()
            .position(|service| service.id == imported_service.id);

        if let Some(existing_service_id) = existing_listener_service_id {
            if existing_service_id != imported_service.id {
                let existing_service = target_profile
                    .services
                    .iter()
                    .find(|service| service.id == existing_service_id)
                    .cloned()
                    .unwrap_or_else(|| ServiceConfig {
                        id: existing_service_id.clone(),
                        name: existing_service_id.clone(),
                        group: String::new(),
                        domain: String::new(),
                        remark: String::new(),
                        port: imported_service.port,
                        local_ip: imported_service.local_ip.clone(),
                        tunnel_id: imported_service.tunnel_id.clone(),
                        sort_order: imported_service.sort_order,
                        enabled: false,
                    });
                preview.conflicts.push(ServiceImportConflict {
                    profile_id: target_profile.id.clone(),
                    profile_name: target_profile.name.clone(),
                    service_id: imported_service.id,
                    service_name: imported_service.name,
                    local_ip: imported_service.local_ip,
                    port: imported_service.port,
                    existing_service_id: existing_service.id,
                    existing_service_name: existing_service.name,
                    reason: String::from("Local listener is already used by another service"),
                });
                continue;
            }
        }

        if let Some(position) = existing_service_position {
            let old_service = target_profile.services[position].clone();
            preview.updated_service_count += 1;
            preview.overwrites.push(ServiceOverwritePreview {
                profile_id: target_profile.id.clone(),
                profile_name: target_profile.name.clone(),
                service_id: old_service.id.clone(),
                old_name: old_service.name.clone(),
                old_group: old_service.group.clone(),
                old_domain: old_service.domain.clone(),
                old_port: old_service.port,
                old_local_ip: old_service.local_ip.clone(),
                old_tunnel_id: old_service.tunnel_id.clone(),
                old_sort_order: old_service.sort_order,
                new_name: imported_service.name.clone(),
                new_group: imported_service.group.clone(),
                new_domain: imported_service.domain.clone(),
                new_port: imported_service.port,
                new_local_ip: imported_service.local_ip.clone(),
                new_tunnel_id: imported_service.tunnel_id.clone(),
                new_sort_order: imported_service.sort_order,
            });
            listeners.remove(&listener_key(&old_service));
            listeners.insert(listener, imported_service.id.clone());
            target_profile.services[position] = imported_service;
        } else {
            preview.added_service_count += 1;
            listeners.insert(listener, imported_service.id.clone());
            target_profile.services.push(imported_service);
        }
    }
}

fn listener_key(service: &ServiceConfig) -> String {
    format!("{}:{}", service.local_ip, service.port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BehaviorSettings, TunnelConfig, DEFAULT_PROFILE_ID, DEFAULT_TUNNEL_ID};

    fn settings(tunnel_ids: &[&str]) -> AppSettings {
        AppSettings {
            schema_version: 2,
            current_profile_id: String::from(DEFAULT_PROFILE_ID),
            current_tunnel_id: tunnel_ids
                .first()
                .copied()
                .unwrap_or(DEFAULT_TUNNEL_ID)
                .to_string(),
            tunnels: tunnel_ids
                .iter()
                .map(|id| TunnelConfig {
                    id: (*id).to_string(),
                    name: (*id).to_string(),
                    ..TunnelConfig::default()
                })
                .collect(),
            behavior: BehaviorSettings::default(),
        }
    }

    fn profiles(profiles: Vec<ServiceProfile>) -> ProfilesFile {
        ProfilesFile {
            schema_version: 2,
            profiles,
        }
    }

    fn profile(id: &str, services: Vec<ServiceConfig>) -> ServiceProfile {
        ServiceProfile {
            id: id.to_string(),
            name: format!("{id} Profile"),
            enabled: true,
            services,
        }
    }

    fn service(id: &str, tunnel_id: &str, local_ip: &str, port: u16) -> ServiceConfig {
        ServiceConfig {
            id: id.to_string(),
            name: id.to_string(),
            group: String::new(),
            domain: format!("{id}.example.internal"),
            remark: String::new(),
            port,
            local_ip: local_ip.to_string(),
            tunnel_id: tunnel_id.to_string(),
            sort_order: 10,
            enabled: true,
        }
    }

    #[test]
    fn exports_selected_profiles_without_settings_or_passwords() {
        let current = profiles(vec![
            profile(
                "default",
                vec![service("mysql", "default", "127.77.0.10", 3306)],
            ),
            profile(
                "team",
                vec![service("redis", "default", "127.77.0.11", 6379)],
            ),
        ]);

        let exported = selected_profiles_export(&current, &[String::from("team")]).unwrap();
        let content = serde_json::to_string(&exported).unwrap();

        assert_eq!(exported.profiles.len(), 1);
        assert_eq!(exported.profiles[0].id, "team");
        assert!(!content.contains("tunnels"));
        assert!(!content.contains("password"));
    }

    #[test]
    fn previews_new_profiles_and_service_overwrites() {
        let settings = settings(&["default"]);
        let current = profiles(vec![profile(
            "default",
            vec![service("mysql", "default", "127.77.0.10", 3306)],
        )]);
        let imported = profiles(vec![
            profile(
                "default",
                vec![ServiceConfig {
                    domain: String::from("mysql.changed.internal"),
                    remark: String::new(),
                    ..service("mysql", "default", "127.77.0.10", 3306)
                }],
            ),
            profile(
                "team",
                vec![service("redis", "default", "127.77.0.11", 6379)],
            ),
        ]);

        let preview =
            preview_profiles_import_from_files(&settings, &current, imported, Vec::new()).unwrap();

        assert_eq!(preview.added_profile_count, 1);
        assert_eq!(preview.added_service_count, 1);
        assert_eq!(preview.updated_service_count, 1);
        assert_eq!(preview.overwrites[0].old_domain, "mysql.example.internal");
        assert_eq!(preview.overwrites[0].new_domain, "mysql.changed.internal");
        assert!(preview.can_apply);
    }

    #[test]
    fn skips_listener_conflicts_in_existing_profiles() {
        let settings = settings(&["default"]);
        let current = profiles(vec![profile(
            "default",
            vec![service("mysql", "default", "127.77.0.10", 3306)],
        )]);
        let imported = profiles(vec![profile(
            "default",
            vec![service("redis", "default", "127.77.0.10", 3306)],
        )]);

        let preview =
            preview_profiles_import_from_files(&settings, &current, imported, Vec::new()).unwrap();

        assert_eq!(preview.skipped_service_count, 1);
        assert_eq!(preview.conflicts[0].service_id, "redis");
        assert_eq!(preview.conflicts[0].existing_service_id, "mysql");
    }

    #[test]
    fn reports_missing_tunnels_until_mapped() {
        let settings = settings(&["default"]);
        let current = profiles(vec![profile("default", Vec::new())]);
        let imported = profiles(vec![profile(
            "team",
            vec![service("mysql", "team-jump", "127.77.0.10", 3306)],
        )]);

        let missing_preview =
            preview_profiles_import_from_files(&settings, &current, imported.clone(), Vec::new())
                .unwrap();
        assert!(!missing_preview.can_apply);
        assert_eq!(
            missing_preview.missing_tunnels[0].source_tunnel_id,
            "team-jump"
        );

        let mapped_preview = preview_profiles_import_from_files(
            &settings,
            &current,
            imported,
            vec![TunnelMapping {
                source_tunnel_id: String::from("team-jump"),
                target_tunnel_id: String::from("default"),
            }],
        )
        .unwrap();
        assert!(mapped_preview.can_apply);
        assert!(mapped_preview.missing_tunnels.is_empty());
    }

    #[test]
    fn migrates_v1_profiles_before_preview() {
        let temp = tempfile::tempdir().unwrap();
        let import_path = temp.path().join("profiles.json");
        std::fs::write(
            &import_path,
            r#"{
              "schemaVersion": 1,
              "profiles": [{
                "id": "team",
                "name": "Team",
                "enabled": true,
                "services": [{
                  "id": "mysql",
                  "name": "MySQL",
                  "domain": "mysql.example.internal",
                  "port": 3306,
                  "localIp": "127.77.0.10",
                  "enabled": true
                }]
              }]
            }"#,
        )
        .unwrap();

        let imported = config::load_profiles_from_path(import_path).unwrap();

        assert_eq!(imported.schema_version, 2);
        assert_eq!(imported.profiles[0].services[0].tunnel_id, "default");
        assert_eq!(imported.profiles[0].services[0].group, "");
        assert_eq!(imported.profiles[0].services[0].sort_order, 10);
    }
}
