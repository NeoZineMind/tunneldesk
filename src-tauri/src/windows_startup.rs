use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use crate::error::{AppError, AppResult};

const SCRIPT: &str = include_str!("../windows/launch-at-login.ps1");

fn command(script: &str, operation: &str, exe: &Path) -> Command {
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let mut command = Command::new(
        Path::new(&system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe"),
    );
    command
        .creation_flags(0x08000000) // CREATE_NO_WINDOW for the helper, not the app.
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        // Pass paths as data; never interpolate an executable path into PowerShell.
        .env("TUNNELDESK_STARTUP_EXE", exe)
        .env("TUNNELDESK_STARTUP_OPERATION", operation);
    command
}

fn run(operation: &str, exe: &Path) -> AppResult<String> {
    let output = command(SCRIPT, operation, exe).output()?;
    if !output.status.success() {
        return Err(AppError::Message(format!(
            "Failed to update Windows login task: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn enabled(exe: &Path) -> bool {
    run("query", exe).is_ok_and(|output| output == "true")
}

pub fn set_enabled(enabled: bool, exe: &Path) -> AppResult<()> {
    run(if enabled { "enable" } else { "disable" }, exe).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise the real script in Windows PowerShell while replacing all task
    // mutations with in-memory doubles. No installed tasks or registry are touched.
    const MOCK_TASKS: &str = r#"
function Get-ScheduledTask { param($TaskName, $TaskPath, $ErrorAction)
    if ($env:TUNNELDESK_TEST_TASK) { $env:TUNNELDESK_TEST_TASK | ConvertFrom-Json }
}
function New-ScheduledTaskAction { param($Execute) @{ Execute = $Execute } }
function New-ScheduledTaskTrigger { param([switch]$AtLogOn, $User) @{ User = $User; AtLogOn = [bool]$AtLogOn } }
function New-ScheduledTaskPrincipal { param($UserId, $LogonType, $RunLevel) @{ UserId = $UserId; LogonType = $LogonType; RunLevel = $RunLevel } }
function New-ScheduledTaskSettingsSet { param([switch]$AllowStartIfOnBatteries, [switch]$DontStopIfGoingOnBatteries, $ExecutionTimeLimit, $MultipleInstances)
    @{ Seconds = $ExecutionTimeLimit.TotalSeconds; MultipleInstances = $MultipleInstances; BatteryAllowed = [bool]$AllowStartIfOnBatteries; BatteryContinues = [bool]$DontStopIfGoingOnBatteries }
}
function Register-ScheduledTask { param($TaskName, $TaskPath, $Action, $Trigger, $Principal, $Settings, $Description, [switch]$Force)
    $script:registeredTask = @{ Name = $TaskName; Path = $TaskPath; Action = $Action; Trigger = $Trigger; Principal = $Principal; Settings = $Settings; Description = $Description }
}
function Unregister-ScheduledTask { param($TaskName, $TaskPath, $Confirm) $script:removedTask = $TaskName }
"#;

    fn mock_run(
        operation: &str,
        exe: &Path,
        setup: &str,
        task: Option<&str>,
    ) -> std::process::Output {
        let script = format!(
            "{MOCK_TASKS}\n{setup}\n{SCRIPT}\nif ($registeredTask) {{ $registeredTask | ConvertTo-Json -Compress -Depth 5 }}\nif ($removedTask) {{ $removedTask }}"
        );
        let mut process = command(&script, operation, exe);
        process.env_remove("TUNNELDESK_TEST_TASK");
        if let Some(task) = task {
            process.env("TUNNELDESK_TEST_TASK", task);
        }
        process
            .output()
            .expect("run isolated PowerShell task script")
    }

    #[test]
    fn registration_preserves_path_and_interactive_user_without_passwords() {
        let exe = Path::new(r"C:\Apps\User's $data; test\tunneldesk.exe");
        let output = mock_run("enable", exe, "", None);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let task: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(task["Action"]["Execute"], exe.to_string_lossy().as_ref());
        assert_eq!(task["Principal"]["LogonType"], "Interactive");
        assert_eq!(task["Principal"]["RunLevel"], "Highest");
        assert_eq!(task["Trigger"]["User"], task["Principal"]["UserId"]);
        assert_eq!(task["Settings"]["Seconds"], 0.0);
        assert_eq!(task["Settings"]["MultipleInstances"], "IgnoreNew");
        assert_eq!(task["Settings"]["BatteryAllowed"], true);
        assert_eq!(task["Settings"]["BatteryContinues"], true);
    }

    #[test]
    fn query_requires_enabled_elevated_task_for_this_user_and_executable() {
        let exe = Path::new(r"C:\Apps\tunneldesk.exe");
        let setup = r#"
$env:TUNNELDESK_TEST_TASK = @{
    Description = 'TunnelDesk login startup (com.tunneldesk.app)'; State = 'Ready'
    Actions = @(@{ Execute = $env:TUNNELDESK_STARTUP_EXE; Arguments = '' })
    Principal = @{ UserId = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value; LogonType = 'Interactive'; RunLevel = 'Highest' }
} | ConvertTo-Json -Compress -Depth 5
"#;
        for (change, expected) in [
            ("", "true"),
            ("State = 'Disabled'", "false"),
            ("Actions = @(@{Execute = 'C:\\other.exe'; Arguments = ''})", "false"),
            ("Principal = @{UserId = 'other'; LogonType = 'Interactive'; RunLevel = 'Highest'}", "false"),
            ("Principal = @{UserId = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value; LogonType = 'Interactive'; RunLevel = 'Limited'}", "false"),
        ] {
            let mutation = if change.is_empty() {
                String::new()
            } else {
                format!("\n$testTask = $env:TUNNELDESK_TEST_TASK | ConvertFrom-Json; $testTask.{change}; $env:TUNNELDESK_TEST_TASK = $testTask | ConvertTo-Json -Compress -Depth 5")
            };
            let output = mock_run("query", exe, &format!("{setup}{mutation}"), None);
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
        }
    }

    #[test]
    fn refuses_to_replace_or_delete_an_unrelated_task() {
        for operation in ["enable", "disable"] {
            let output = mock_run(
                operation,
                Path::new(r"C:\Apps\tunneldesk.exe"),
                "",
                Some(r#"{"Description":"Another app"}"#),
            );
            assert!(!output.status.success());
            assert!(String::from_utf8_lossy(&output.stderr).contains("different application"));
        }
    }
}
