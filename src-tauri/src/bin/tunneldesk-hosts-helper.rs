use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use tunneldesk_lib::hosts_core::{
    remove_block_if_present_content, render_block, replace_block, validate_request,
    HostsHelperOperation, HostsHelperRequest,
};

const HOSTS_PATH: &str = "/etc/hosts";
const BACKUP_DIR: &str = "/var/lib/tunneldesk/backups";

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut payload = String::new();
    io::stdin()
        .read_to_string(&mut payload)
        .map_err(|error| format!("Failed to read helper request: {error}"))?;
    let request: HostsHelperRequest = serde_json::from_str(&payload)
        .map_err(|error| format!("Invalid helper request JSON: {error}"))?;

    let content = fs::read_to_string(HOSTS_PATH).unwrap_or_default();
    let Some(next) = apply_hosts_request(&content, &request)? else {
        return Ok(());
    };

    backup_hosts_content(&content)?;
    fs::write(HOSTS_PATH, next)
        .map_err(|error| format!("Failed to write {HOSTS_PATH}: {error}"))?;
    flush_dns();
    Ok(())
}

fn apply_hosts_request(
    content: &str,
    request: &HostsHelperRequest,
) -> Result<Option<String>, String> {
    validate_request(request)?;

    match request.operation {
        HostsHelperOperation::Write => {
            let block = render_block(&request.services);
            let next = replace_block(content, Some(&block));
            if next == content {
                Ok(None)
            } else {
                Ok(Some(next))
            }
        }
        HostsHelperOperation::Remove => Ok(remove_block_if_present_content(content)),
    }
}

fn backup_hosts_content(content: &str) -> Result<(), String> {
    fs::create_dir_all(BACKUP_DIR)
        .map_err(|error| format!("Failed to create hosts backup directory: {error}"))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let backup_path = PathBuf::from(BACKUP_DIR).join(format!("hosts.{ts}.bak"));
    let mut file = fs::File::create(&backup_path).map_err(|error| {
        format!(
            "Failed to create hosts backup {}: {error}",
            backup_path.display()
        )
    })?;
    file.write_all(content.as_bytes()).map_err(|error| {
        format!(
            "Failed to write hosts backup {}: {error}",
            backup_path.display()
        )
    })
}

fn flush_dns() {
    let _ = Command::new("resolvectl").arg("flush-caches").output();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tunneldesk_lib::hosts_core::{HostsEntry, HOSTS_HELPER_REQUEST_VERSION};

    fn entry(domain: &str, local_ip: &str) -> HostsEntry {
        HostsEntry {
            domain: String::from(domain),
            local_ip: String::from(local_ip),
        }
    }

    #[test]
    fn apply_write_request_replaces_marker_block() {
        let request =
            HostsHelperRequest::write(vec![entry("mysql.example.internal", "127.77.0.10")]);
        let next = apply_hosts_request(
            "before\n# BEGIN TUNNELDESK\n127.0.0.1 old\n# END TUNNELDESK\nafter\n",
            &request,
        )
        .unwrap()
        .unwrap();

        assert!(next.contains("127.77.0.10 mysql.example.internal"));
        assert!(!next.contains("127.0.0.1 old"));
        assert!(next.contains("before"));
        assert!(next.contains("after"));
    }

    #[test]
    fn apply_remove_request_removes_marker_block() {
        let request = HostsHelperRequest::remove();
        let next = apply_hosts_request(
            "before\n# BEGIN TUNNELDESK\n127.0.0.1 old\n# END TUNNELDESK\nafter\n",
            &request,
        )
        .unwrap()
        .unwrap();

        assert_eq!(next, "before\nafter\n");
    }

    #[test]
    fn apply_remove_request_skips_clean_hosts() {
        let request = HostsHelperRequest::remove();

        assert!(apply_hosts_request("before\nafter\n", &request)
            .unwrap()
            .is_none());
    }

    #[test]
    fn apply_request_rejects_invalid_mapping() {
        let request = HostsHelperRequest::write(vec![entry("mysql.example.internal", "10.0.0.1")]);

        assert!(apply_hosts_request("before\n", &request).is_err());
    }

    #[test]
    fn apply_request_rejects_unsupported_version() {
        let request = HostsHelperRequest {
            version: HOSTS_HELPER_REQUEST_VERSION + 1,
            operation: HostsHelperOperation::Remove,
            services: Vec::new(),
        };

        assert!(apply_hosts_request("before\n", &request).is_err());
    }
}
