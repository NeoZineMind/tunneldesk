use std::fs;
use std::io::Write;
use std::path::PathBuf;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::error::{AppError, AppResult};
use crate::hosts_core::{
    block_present_in_content, remove_block_if_present_content, render_block, replace_block,
    validate_entries, HostsEntry,
};
use crate::model::ServiceConfig;
#[cfg(target_os = "linux")]
use crate::privileged_hosts;

pub fn hosts_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        PathBuf::from("/etc/hosts")
    }
}

pub fn can_write_hosts() -> bool {
    fs::OpenOptions::new()
        .append(true)
        .open(hosts_path())
        .is_ok()
}

pub fn block_present() -> bool {
    fs::read_to_string(hosts_path())
        .map(|content| block_present_in_content(&content))
        .unwrap_or(false)
}

pub fn write_services_block(services: &[ServiceConfig]) -> AppResult<()> {
    let entries = services_to_hosts_entries(services);
    validate_entries(&entries).map_err(AppError::Message)?;

    #[cfg(target_os = "linux")]
    if should_use_privileged_helper() {
        privileged_hosts::write_entries(&entries)?;
        return Ok(());
    }

    write_entries_block_direct(&entries)
}

pub fn remove_block() -> AppResult<()> {
    #[cfg(target_os = "linux")]
    if should_use_privileged_helper() {
        privileged_hosts::remove_block()?;
        return Ok(());
    }

    remove_block_direct().map(|_| ())
}

pub fn remove_block_without_elevation() -> AppResult<bool> {
    remove_block_direct()
}

fn write_entries_block_direct(entries: &[HostsEntry]) -> AppResult<()> {
    let content = fs::read_to_string(hosts_path()).unwrap_or_default();
    let block = render_block(entries);
    let next = replace_block(&content, Some(&block));
    if next == content {
        return Ok(());
    }

    backup_hosts_content(&content)?;
    fs::write(hosts_path(), next)?;
    flush_dns();
    Ok(())
}

fn remove_block_direct() -> AppResult<bool> {
    let content = fs::read_to_string(hosts_path()).unwrap_or_default();
    let Some(next) = remove_block_if_present_content(&content) else {
        return Ok(false);
    };

    backup_hosts_content(&content)?;
    fs::write(hosts_path(), next)?;
    flush_dns();
    Ok(true)
}

fn backup_hosts_content(content: &str) -> AppResult<()> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let backup_path = config::backups_dir()?.join(format!("hosts.{ts}.bak"));
    let mut file = fs::File::create(backup_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn flush_dns() {
    #[cfg(target_os = "windows")]
    let _ = Command::new("ipconfig").arg("/flushdns").output();
    #[cfg(target_os = "linux")]
    let _ = Command::new("resolvectl").arg("flush-caches").output();
}

fn services_to_hosts_entries(services: &[ServiceConfig]) -> Vec<HostsEntry> {
    services
        .iter()
        .filter(|service| service.enabled)
        .map(|service| HostsEntry {
            domain: service.domain.clone(),
            local_ip: service.local_ip.clone(),
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn should_use_privileged_helper() -> bool {
    !can_write_hosts() && privileged_hosts::helper_installed()
}
