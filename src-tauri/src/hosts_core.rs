use std::io::{Seek, SeekFrom, Write};
use std::net::IpAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const BEGIN_MARKER: &str = "# BEGIN TUNNELDESK";
pub const END_MARKER: &str = "# END TUNNELDESK";
pub const HOSTS_HELPER_REQUEST_VERSION: u32 = 1;

// Keep the hosts file's identity and permissions (it may be a bind mount), and
// restore its previous content if a write fails after changing some bytes.
pub fn write_content_with_rollback(path: &Path, original: &str, next: &str) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|error| format!("Failed to open hosts file for writing: {error}"))?;
    fn write(file: &mut std::fs::File, content: &str) -> std::io::Result<()> {
        file.seek(SeekFrom::Start(0))?;
        file.write_all(content.as_bytes())?;
        file.set_len(content.len() as u64)?;
        file.sync_all()
    }
    write_with_rollback(original, next, |content| write(&mut file, content))
}

fn write_with_rollback(
    original: &str,
    next: &str,
    mut write: impl FnMut(&str) -> std::io::Result<()>,
) -> Result<(), String> {
    if let Err(error) = write(next) {
        return Err(match write(original) {
            Ok(()) => format!("Failed to write hosts file; previous content restored: {error}"),
            Err(rollback) => format!("Failed to write hosts file: {error}; restoring previous content also failed: {rollback}"),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HostsEntry {
    pub domain: String,
    pub local_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HostsHelperOperation {
    Write,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostsHelperRequest {
    pub version: u32,
    pub operation: HostsHelperOperation,
    #[serde(default)]
    pub services: Vec<HostsEntry>,
}

impl HostsHelperRequest {
    pub fn write(services: Vec<HostsEntry>) -> Self {
        Self {
            version: HOSTS_HELPER_REQUEST_VERSION,
            operation: HostsHelperOperation::Write,
            services,
        }
    }

    pub fn remove() -> Self {
        Self {
            version: HOSTS_HELPER_REQUEST_VERSION,
            operation: HostsHelperOperation::Remove,
            services: Vec::new(),
        }
    }
}

pub fn validate_request(request: &HostsHelperRequest) -> Result<(), String> {
    if request.version != HOSTS_HELPER_REQUEST_VERSION {
        return Err(format!(
            "Unsupported hosts helper request version: {}",
            request.version
        ));
    }

    match request.operation {
        HostsHelperOperation::Write => validate_entries(&request.services),
        HostsHelperOperation::Remove => Ok(()),
    }
}

pub fn validate_entries(entries: &[HostsEntry]) -> Result<(), String> {
    for entry in entries {
        validate_loopback_ipv4(&entry.local_ip)?;
        validate_domain(&entry.domain)?;
    }
    Ok(())
}

pub fn render_block(entries: &[HostsEntry]) -> String {
    let mut lines = Vec::new();
    lines.push(String::from(BEGIN_MARKER));
    for entry in entries {
        lines.push(format!("{} {}", entry.local_ip, entry.domain));
    }
    lines.push(String::from(END_MARKER));
    lines.join("\n")
}

pub fn block_present_in_content(content: &str) -> bool {
    let mut begin = None;
    let mut end = None;

    for (index, line) in content.lines().enumerate() {
        match line.trim() {
            BEGIN_MARKER if begin.is_none() => begin = Some(index),
            END_MARKER if end.is_none() => end = Some(index),
            _ => {}
        }
    }

    matches!((begin, end), (Some(begin), Some(end)) if begin <= end)
}

pub fn remove_block_if_present_content(content: &str) -> Option<String> {
    if block_present_in_content(content) {
        Some(replace_block(content, None))
    } else {
        None
    }
}

pub fn replace_block(content: &str, block: Option<&str>) -> String {
    let mut lines = content.lines().collect::<Vec<_>>();
    let begin = lines.iter().position(|line| line.trim() == BEGIN_MARKER);
    let end = lines.iter().position(|line| line.trim() == END_MARKER);

    if let (Some(begin), Some(end)) = (begin, end) {
        if begin <= end {
            lines.drain(begin..=end);
        }
    }

    let mut next = lines.join("\n").trim_end().to_string();
    if let Some(block) = block {
        if !next.is_empty() {
            next.push_str("\n\n");
        }
        next.push_str(block);
    }
    next.push('\n');
    next
}

fn validate_loopback_ipv4(value: &str) -> Result<(), String> {
    if value.trim() != value {
        return Err(String::from(
            "hosts local IP must not contain surrounding whitespace",
        ));
    }

    match value.parse::<IpAddr>() {
        Ok(IpAddr::V4(addr)) if addr.is_loopback() => Ok(()),
        Ok(IpAddr::V4(_)) => Err(format!("hosts local IP must be loopback: {value}")),
        Ok(IpAddr::V6(_)) => Err(String::from("hosts local IP must be IPv4 loopback")),
        Err(_) => Err(format!("Invalid hosts local IP: {value}")),
    }
}

pub(crate) fn validate_domain(value: &str) -> Result<(), String> {
    if value.trim() != value {
        return Err(String::from(
            "hosts domain must not contain surrounding whitespace",
        ));
    }
    if value.is_empty() {
        return Err(String::from("hosts domain cannot be empty"));
    }
    if value.len() > 253 {
        return Err(format!("hosts domain is too long: {value}"));
    }
    if value.chars().any(|character| {
        !character.is_ascii_alphanumeric()
            && character != '.'
            && character != '-'
            && character != '_'
    }) {
        return Err(format!("hosts domain contains invalid characters: {value}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_shorter_hosts_content_without_trailing_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hosts");
        std::fs::write(&path, "before\nold long content\nafter\n").unwrap();
        write_content_with_rollback(
            &path,
            "before\nold long content\nafter\n",
            "before\nafter\n",
        )
        .unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "before\nafter\n");
    }

    #[test]
    fn missing_hosts_file_is_not_created_or_replaced_with_empty_content() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing-hosts");
        assert!(write_content_with_rollback(&path, "original", "new").is_err());
        assert!(!path.exists());
    }

    #[test]
    fn partial_write_failure_restores_previous_hosts_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hosts");
        let original = "127.0.0.1 localhost\n# existing mapping\n";
        std::fs::write(&path, original).unwrap();
        let mut first_write = true;
        let error = write_with_rollback(original, "new mapping", |content| {
            if first_write {
                first_write = false;
                std::fs::write(&path, &content[..3])?;
                Err(std::io::Error::other("injected partial write failure"))
            } else {
                std::fs::write(&path, content)
            }
        })
        .unwrap_err();
        assert!(error.contains("previous content restored"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn rollback_failure_is_reported_alongside_the_write_failure() {
        let error = write_with_rollback("old", "new", |content| {
            Err(std::io::Error::other(format!("cannot write {content}")))
        })
        .unwrap_err();
        assert!(error.contains("cannot write new"));
        assert!(error.contains("cannot write old"));
    }

    fn entry(domain: &str, local_ip: &str) -> HostsEntry {
        HostsEntry {
            domain: String::from(domain),
            local_ip: String::from(local_ip),
        }
    }

    #[test]
    fn replace_block_is_idempotent() {
        let first = replace_block(
            "a\n",
            Some("# BEGIN TUNNELDESK\n127.0.0.1 a\n# END TUNNELDESK"),
        );
        let second = replace_block(
            &first,
            Some("# BEGIN TUNNELDESK\n127.0.0.1 a\n# END TUNNELDESK"),
        );
        assert_eq!(first, second);
    }

    #[test]
    fn remove_block_keeps_other_lines() {
        let content = "before\n# BEGIN TUNNELDESK\n127.0.0.1 a\n# END TUNNELDESK\nafter\n";
        let next = replace_block(content, None);
        assert!(next.contains("before"));
        assert!(next.contains("after"));
        assert!(!next.contains("127.0.0.1 a"));
    }

    #[test]
    fn remove_block_if_present_skips_clean_content() {
        let content = "before\nafter\n";

        assert!(remove_block_if_present_content(content).is_none());
    }

    #[test]
    fn remove_block_if_present_removes_existing_block() {
        let content = "before\n# BEGIN TUNNELDESK\n127.0.0.1 a\n# END TUNNELDESK\nafter\n";
        let next = remove_block_if_present_content(content).unwrap();

        assert_eq!(next, "before\nafter\n");
        assert!(!block_present_in_content(&next));
    }

    #[test]
    fn validate_entries_accepts_loopback_mapping() {
        assert!(validate_entries(&[entry("mysql.example.internal", "127.77.0.10")]).is_ok());
    }

    #[test]
    fn validate_entries_rejects_injected_domain() {
        assert!(validate_entries(&[entry("mysql.local\n1.2.3.4 evil", "127.77.0.10")]).is_err());
        assert!(validate_entries(&[entry("mysql.local # evil", "127.77.0.10")]).is_err());
    }

    #[test]
    fn validate_entries_rejects_quoted_domain() {
        assert!(validate_entries(&[entry("mysql.example.internal\"", "127.77.0.10")]).is_err());
    }

    #[test]
    fn validate_entries_rejects_non_loopback_ip() {
        assert!(validate_entries(&[entry("mysql.example.internal", "192.168.1.10")]).is_err());
    }

    #[test]
    fn validate_request_rejects_unknown_version() {
        let request = HostsHelperRequest {
            version: HOSTS_HELPER_REQUEST_VERSION + 1,
            operation: HostsHelperOperation::Remove,
            services: Vec::new(),
        };

        assert!(validate_request(&request).is_err());
    }
}
