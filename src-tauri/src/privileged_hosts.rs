use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{AppError, AppResult};
use crate::hosts_core::{validate_request, HostsEntry, HostsHelperRequest};

pub const LINUX_HELPER_PATH: &str = "/usr/lib/tunneldesk/tunneldesk-hosts-helper";
const PKEXEC_PROGRAM: &str = "pkexec";

pub fn helper_installed() -> bool {
    let path = Path::new(LINUX_HELPER_PATH);
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    metadata.is_file() && executable_by_anyone(&metadata)
}

pub fn authorization_available() -> bool {
    helper_installed() && command_in_path(PKEXEC_PROGRAM)
}

pub fn write_entries(entries: &[HostsEntry]) -> AppResult<()> {
    run_helper(&HostsHelperRequest::write(entries.to_vec()))
}

pub fn remove_block() -> AppResult<()> {
    run_helper(&HostsHelperRequest::remove())
}

fn run_helper(request: &HostsHelperRequest) -> AppResult<()> {
    validate_request(request).map_err(AppError::Message)?;
    if !helper_installed() {
        return Err(AppError::Message(String::from(
            "TunnelDesk Linux hosts helper is not installed",
        )));
    }
    if !command_in_path(PKEXEC_PROGRAM) {
        return Err(AppError::Message(String::from(
            "pkexec is not available; install policykit-1/polkit",
        )));
    }

    let payload = serde_json::to_vec(request)?;
    let mut child = Command::new(PKEXEC_PROGRAM)
        .arg(LINUX_HELPER_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let Some(mut stdin) = child.stdin.take() else {
        return Err(AppError::Message(String::from(
            "Failed to open hosts helper stdin",
        )));
    };
    stdin.write_all(&payload)?;
    drop(stdin);

    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.code() == Some(126)
        || stderr.to_lowercase().contains("dismissed")
        || stderr.to_lowercase().contains("cancel")
    {
        return Err(AppError::Message(String::from(
            "System authorization was cancelled",
        )));
    }

    Err(AppError::Message(if stderr.is_empty() {
        format!(
            "TunnelDesk hosts helper failed with status {}",
            output.status
        )
    } else {
        stderr
    }))
}

fn command_in_path(command: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|path| path.join(command).is_file()))
        .unwrap_or(false)
}

#[cfg(unix)]
fn executable_by_anyone(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable_by_anyone(_metadata: &std::fs::Metadata) -> bool {
    true
}
