use std::collections::HashSet;
use std::net::IpAddr;

use crate::error::{AppError, AppResult};
use crate::hosts_core;
use crate::model::{
    AppSettings, AuthMethod, ProfilesFile, ServiceProfile, SshSettings, TunnelConfig,
};

pub fn validate_settings(settings: &AppSettings) -> AppResult<()> {
    if settings.tunnels.is_empty() {
        return Err(AppError::Message(String::from(
            "At least one tunnel is required",
        )));
    }

    let mut tunnel_ids = HashSet::new();
    for tunnel in &settings.tunnels {
        validate_tunnel(tunnel)?;
        if !tunnel_ids.insert(tunnel.id.trim().to_string()) {
            return Err(AppError::Message(format!(
                "Duplicate tunnel id: {}",
                tunnel.id
            )));
        }
    }

    if !settings.current_tunnel_id.trim().is_empty()
        && !tunnel_ids.contains(settings.current_tunnel_id.trim())
    {
        return Err(AppError::Message(format!(
            "Current tunnel does not exist: {}",
            settings.current_tunnel_id
        )));
    }

    Ok(())
}

pub fn validate_settings_with_profiles(
    settings: &AppSettings,
    profiles: &ProfilesFile,
) -> AppResult<()> {
    validate_settings(settings)?;
    validate_profiles(profiles)?;
    validate_tunnel_references(settings, profiles)
}

pub fn validate_tunnel(tunnel: &TunnelConfig) -> AppResult<()> {
    if tunnel.id.trim().is_empty() {
        return Err(AppError::Message(String::from("Tunnel id is required")));
    }
    if tunnel.name.trim().is_empty() {
        return Err(AppError::Message(format!(
            "Tunnel name is required for {}",
            tunnel.id
        )));
    }
    validate_ssh_shape(&tunnel.ssh)
}

pub fn validate_tunnel_for_connection(tunnel: &TunnelConfig) -> AppResult<()> {
    validate_tunnel(tunnel)?;
    validate_ssh_connection(&tunnel.ssh)
}

pub fn validate_ssh_shape(ssh: &SshSettings) -> AppResult<()> {
    if ssh.port == 0 {
        return Err(AppError::Message(String::from(
            "SSH port must be between 1 and 65535",
        )));
    }
    if ssh.server_alive_interval < 5 {
        return Err(AppError::Message(String::from(
            "ServerAliveInterval must be at least 5 seconds",
        )));
    }
    if ssh.server_alive_count_max == 0 {
        return Err(AppError::Message(String::from(
            "ServerAliveCountMax must be at least 1",
        )));
    }
    Ok(())
}

pub fn validate_ssh_connection(ssh: &SshSettings) -> AppResult<()> {
    validate_ssh_shape(ssh)?;
    if ssh.host.trim().is_empty() {
        return Err(AppError::Message(String::from("SSH host is required")));
    }
    if ssh.username.trim().is_empty() {
        return Err(AppError::Message(String::from("SSH username is required")));
    }
    if ssh.auth_method == AuthMethod::PrivateKey && ssh.identity_file.trim().is_empty() {
        return Err(AppError::Message(String::from(
            "Identity file is required for private key authentication",
        )));
    }
    Ok(())
}

pub fn validate_profiles(file: &ProfilesFile) -> AppResult<()> {
    if file.profiles.is_empty() {
        return Err(AppError::Message(String::from(
            "At least one profile is required",
        )));
    }

    let mut profile_ids = HashSet::new();
    for profile in &file.profiles {
        validate_profile(profile)?;
        if !profile_ids.insert(profile.id.trim().to_string()) {
            return Err(AppError::Message(format!(
                "Duplicate profile id: {}",
                profile.id
            )));
        }
    }
    Ok(())
}

pub fn validate_tunnel_references(
    settings: &AppSettings,
    profiles: &ProfilesFile,
) -> AppResult<()> {
    let tunnel_ids = settings
        .tunnels
        .iter()
        .map(|tunnel| tunnel.id.trim().to_string())
        .collect::<HashSet<_>>();

    for profile in &profiles.profiles {
        for service in &profile.services {
            if !tunnel_ids.contains(service.tunnel_id.trim()) {
                return Err(AppError::Message(format!(
                    "Service {} references missing tunnel: {}",
                    service.name, service.tunnel_id
                )));
            }
        }
    }

    Ok(())
}

fn validate_profile(profile: &ServiceProfile) -> AppResult<()> {
    if profile.id.trim().is_empty() {
        return Err(AppError::Message(String::from("Profile id is required")));
    }
    if profile.name.trim().is_empty() {
        return Err(AppError::Message(String::from("Profile name is required")));
    }

    let mut service_ids = HashSet::new();
    let mut listeners = HashSet::new();

    for service in &profile.services {
        if service.id.trim().is_empty() {
            return Err(AppError::Message(String::from("Service id is required")));
        }
        if !service_ids.insert(service.id.trim().to_string()) {
            return Err(AppError::Message(format!(
                "Duplicate service id in profile {}: {}",
                profile.name, service.id
            )));
        }
        if service.name.trim().is_empty() {
            return Err(AppError::Message(format!(
                "Service name is required for {}",
                service.id
            )));
        }
        if service.group.chars().count() > 80 {
            return Err(AppError::Message(format!(
                "Service group is too long for {}",
                service.name
            )));
        }
        hosts_core::validate_domain(&service.domain).map_err(|error| {
            AppError::Message(format!(
                "Invalid domain for service {}: {error}",
                service.name
            ))
        })?;
        if service.tunnel_id.trim().is_empty() {
            return Err(AppError::Message(format!(
                "Tunnel is required for service {}",
                service.name
            )));
        }
        if service.port == 0 {
            return Err(AppError::Message(format!(
                "Service port must be between 1 and 65535: {}",
                service.name
            )));
        }

        let local_ip: IpAddr = service.local_ip.parse().map_err(|_| {
            AppError::Message(format!(
                "Local IP must be a valid loopback address: {}",
                service.local_ip
            ))
        })?;
        if !local_ip.is_loopback() {
            return Err(AppError::Message(format!(
                "Local IP must be loopback for {}: {}",
                service.name, service.local_ip
            )));
        }

        let listener = format!("{}:{}", service.local_ip, service.port);
        if !listeners.insert(listener.clone()) {
            return Err(AppError::Message(format!(
                "Duplicate local listener in profile {}: {}",
                profile.name, listener
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BehaviorSettings, ProfilesFile, ServiceConfig, ServiceProfile, TunnelConfig,
        DEFAULT_PROFILE_ID, DEFAULT_TUNNEL_ID,
    };

    fn ssh() -> SshSettings {
        SshSettings {
            host: String::from("jump.example.com"),
            port: 22,
            username: String::from("developer"),
            password_credential_key: String::from("TunnelDesk:tunnel:default:password"),
            ..SshSettings::default()
        }
    }

    fn settings(tunnels: Vec<TunnelConfig>) -> AppSettings {
        AppSettings {
            schema_version: 2,
            current_profile_id: String::from(DEFAULT_PROFILE_ID),
            current_tunnel_id: tunnels
                .first()
                .map(|tunnel| tunnel.id.clone())
                .unwrap_or_else(|| String::from(DEFAULT_TUNNEL_ID)),
            tunnels,
            behavior: BehaviorSettings::default(),
        }
    }

    fn profile_with_services(services: Vec<ServiceConfig>) -> ProfilesFile {
        ProfilesFile {
            schema_version: 2,
            profiles: vec![ServiceProfile {
                id: String::from(DEFAULT_PROFILE_ID),
                name: String::from("Default Profile"),
                enabled: true,
                services,
            }],
        }
    }

    fn service(id: &str, tunnel_id: &str, local_ip: &str, port: u16) -> ServiceConfig {
        ServiceConfig {
            id: String::from(id),
            name: String::from(id),
            group: String::new(),
            domain: format!("{id}.example.internal"),
            remark: String::new(),
            port,
            local_ip: String::from(local_ip),
            tunnel_id: String::from(tunnel_id),
            sort_order: 10,
            enabled: true,
        }
    }

    #[test]
    fn rejects_duplicate_tunnel_ids() {
        let tunnel = TunnelConfig {
            id: String::from(DEFAULT_TUNNEL_ID),
            name: String::from("Default Tunnel"),
            enabled: true,
            ssh: ssh(),
        };
        let error = validate_settings(&settings(vec![tunnel.clone(), tunnel]))
            .expect_err("duplicate tunnel ids should fail");

        assert!(error.to_string().contains("Duplicate tunnel id"));
    }

    #[test]
    fn allows_incomplete_ssh_when_saving_settings() {
        let tunnel = TunnelConfig::default();
        validate_settings(&settings(vec![tunnel])).expect("empty ssh fields can be saved");
    }

    #[test]
    fn rejects_missing_tunnel_reference() {
        let tunnel = TunnelConfig {
            id: String::from(DEFAULT_TUNNEL_ID),
            name: String::from("Default Tunnel"),
            enabled: true,
            ssh: ssh(),
        };
        let settings = settings(vec![tunnel]);
        let profiles =
            profile_with_services(vec![service("mysql", "missing", "127.77.0.10", 3306)]);
        let error = validate_tunnel_references(&settings, &profiles)
            .expect_err("missing tunnel reference should fail");

        assert!(error.to_string().contains("references missing tunnel"));
    }

    #[test]
    fn rejects_duplicate_listener_in_profile() {
        let profiles = profile_with_services(vec![
            service("mysql", DEFAULT_TUNNEL_ID, "127.77.0.10", 3306),
            service("mysql-copy", DEFAULT_TUNNEL_ID, "127.77.0.10", 3306),
        ]);
        let error = validate_profiles(&profiles).expect_err("duplicate listener should fail");

        assert!(error.to_string().contains("Duplicate local listener"));
    }

    #[test]
    fn rejects_domain_with_trailing_quote() {
        let mut invalid = service("mysql", DEFAULT_TUNNEL_ID, "127.77.0.10", 3306);
        invalid.domain.push('\"');
        let profiles = profile_with_services(vec![invalid]);

        let error = validate_profiles(&profiles).expect_err("quoted domain should fail");

        assert!(error
            .to_string()
            .contains("Invalid domain for service mysql"));
        assert!(error.to_string().contains("invalid characters"));
    }
}
