use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::time::timeout;

use crate::model::{ListenerOwner, ServiceConfig, StartupIssue, StartupIssueKind};
use crate::tunnel::{listener_bind_error, service_socket_addr};

pub type BoundListeners = Vec<(ServiceConfig, TcpListener)>;

pub(crate) async fn bind_listener(address: SocketAddr) -> std::io::Result<TcpListener> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::io::AsRawSocket;
        use windows_sys::Win32::Networking::WinSock::{
            setsockopt, WSAGetLastError, SOL_SOCKET, SO_EXCLUSIVEADDRUSE,
        };
        let socket = if address.is_ipv4() {
            tokio::net::TcpSocket::new_v4()?
        } else {
            tokio::net::TcpSocket::new_v6()?
        };
        let enabled: i32 = 1;
        // SAFETY: the socket is live and enabled points to an initialized i32 for
        // the entire synchronous setsockopt call. Set exclusivity before bind.
        let result = unsafe {
            setsockopt(
                socket.as_raw_socket() as usize,
                SOL_SOCKET,
                SO_EXCLUSIVEADDRUSE,
                (&enabled as *const i32).cast(),
                std::mem::size_of_val(&enabled) as i32,
            )
        };
        if result != 0 {
            return Err(std::io::Error::from_raw_os_error(unsafe {
                WSAGetLastError()
            }));
        }
        socket.bind(address)?;
        socket.listen(1024)
    }
    #[cfg(not(target_os = "windows"))]
    {
        TcpListener::bind(address).await
    }
}

pub fn exclude_failed_domains(services: &[ServiceConfig], issues: &mut Vec<StartupIssue>) {
    let domains = issues
        .iter()
        .map(|issue| issue.domain.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let ids = issues
        .iter()
        .map(|issue| issue.service_id.clone())
        .collect::<HashSet<_>>();
    for service in services {
        if domains.contains(&service.domain.to_ascii_lowercase()) && !ids.contains(&service.id) {
            issues.push(StartupIssue::for_service(
                service,
                StartupIssueKind::SharedDomain,
                String::from("同域名的其他服务启动失败，已跳过整个域名组以避免错误的 hosts 映射"),
            ));
        }
    }
}

pub async fn prepare_listeners(services: &[ServiceConfig]) -> (BoundListeners, Vec<StartupIssue>) {
    let mut issues = Vec::new();
    let mut listeners = Vec::new();
    for service in services {
        if services.iter().any(|other| {
            other.domain.eq_ignore_ascii_case(&service.domain) && other.local_ip != service.local_ip
        }) {
            issues.push(StartupIssue::for_service(
                service,
                StartupIssueKind::DomainConflict,
                String::from("同一域名必须使用相同的本地 IP；hosts 无法按端口选择不同 IP"),
            ));
            continue;
        }
        let address = match service_socket_addr(service) {
            Ok(address) => address,
            Err(error) => {
                issues.push(StartupIssue::for_service(
                    service,
                    StartupIssueKind::AddressUnavailable,
                    error.to_string(),
                ));
                continue;
            }
        };
        match bind_listener(address).await {
            Ok(listener) => listeners.push((service.clone(), listener)),
            Err(error) => {
                let kind = match error.kind() {
                    std::io::ErrorKind::AddrInUse => StartupIssueKind::ListenerConflict,
                    std::io::ErrorKind::PermissionDenied => StartupIssueKind::PermissionDenied,
                    std::io::ErrorKind::AddrNotAvailable => StartupIssueKind::AddressUnavailable,
                    _ if error.raw_os_error() == Some(10013) => StartupIssueKind::PermissionDenied,
                    _ => StartupIssueKind::ListenerError,
                };
                let os_error = error.raw_os_error();
                let mut issue = StartupIssue::for_service(
                    service,
                    kind,
                    listener_bind_error(service, &address.to_string(), error).to_string(),
                );
                issue.os_error = os_error;
                tracing::warn!(service_id = %service.id, listener = %address, error = %issue.message, "Listener unavailable");
                issues.push(issue);
            }
        }
    }
    // Take one bounded snapshot for the whole startup attempt, rather than spawning a
    // diagnostic command for every unavailable port.
    if issues.iter().any(|issue| issue.os_error.is_some()) {
        let owners = listener_owners().await;
        for issue in &mut issues {
            if let Ok(address) = issue.address.parse() {
                issue.owners = filter_owners(owners.clone(), address);
                if !issue.owners.is_empty() && issue.kind == StartupIssueKind::PermissionDenied {
                    issue.kind = StartupIssueKind::ListenerConflict;
                    issue.message = format!(
                        "服务 {} 的监听地址 {} 已被其他进程占用（系统错误 {}）",
                        issue.service_name,
                        issue.address,
                        issue
                            .os_error
                            .map_or_else(|| String::from("unknown"), |code| code.to_string())
                    );
                }
            }
        }
    }
    exclude_failed_domains(services, &mut issues);
    let failed = issues
        .iter()
        .map(|issue| issue.service_id.as_str())
        .collect::<HashSet<_>>();
    listeners.retain(|(service, _)| !failed.contains(service.id.as_str()));
    (listeners, issues)
}

async fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new(program);
    command.args(args).kill_on_drop(true);
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let output = timeout(Duration::from_secs(3), command.output())
        .await
        .ok()?
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn listener_owners() -> Vec<ListenerOwner> {
    #[cfg(target_os = "windows")]
    {
        let script = "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; $ErrorActionPreference='Stop'; @(Get-NetTCPConnection -State Listen | ForEach-Object { $p=Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue; [pscustomobject]@{pid=[uint32]$_.OwningProcess;processName=$p.ProcessName;address=($_.LocalAddress+':'+$_.LocalPort)} }) | ConvertTo-Json -Compress";
        command_output(
            "powershell.exe",
            &["-NoProfile", "-NonInteractive", "-Command", script],
        )
        .await
        .and_then(|output| {
            serde_json::from_str::<Vec<ListenerOwner>>(output.trim_start_matches('\u{feff}')).ok()
        })
        .unwrap_or_default()
    }
    #[cfg(not(target_os = "windows"))]
    {
        #[cfg(target_os = "linux")]
        if let Some(output) = command_output("ss", &["-H", "-ltnp"]).await {
            let owners = parse_ss(&output);
            if !owners.is_empty() {
                return owners;
            }
        }
        command_output("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN", "-Fpcn"])
            .await
            .map(|output| parse_lsof(&output))
            .unwrap_or_default()
    }
}

fn filter_owners(owners: Vec<ListenerOwner>, target: SocketAddr) -> Vec<ListenerOwner> {
    owners
        .into_iter()
        .filter(|owner| {
            let Some((ip, port)) = owner.address.rsplit_once(':') else {
                return false;
            };
            if port.parse::<u16>().ok() != Some(target.port()) {
                return false;
            }
            let ip = ip.trim_matches(['[', ']']);
            ip == "*"
                || ip
                    .parse::<IpAddr>()
                    .is_ok_and(|ip| ip == target.ip() || ip.is_unspecified())
        })
        .collect()
}

#[cfg(any(target_os = "linux", test))]
fn parse_ss(output: &str) -> Vec<ListenerOwner> {
    let mut owners = Vec::new();
    for line in output.lines() {
        let Some(address) = line.split_whitespace().nth(3) else {
            continue;
        };
        let mut remainder = line;
        while let Some((before, after)) = remainder.split_once("pid=") {
            let Some(pid) = after.split(',').next().and_then(|pid| pid.parse().ok()) else {
                break;
            };
            let name = before
                .rsplit_once("(\"")
                .and_then(|(_, value)| value.split('"').next())
                .map(str::to_string);
            owners.push(ListenerOwner {
                pid,
                process_name: name,
                address: address.to_string(),
            });
            remainder = after;
        }
    }
    owners
}

#[cfg(any(not(target_os = "windows"), test))]
fn parse_lsof(output: &str) -> Vec<ListenerOwner> {
    let mut owners = Vec::new();
    let mut pid = None;
    let mut name = None;
    for line in output.lines() {
        if let Some(value) = line.strip_prefix('p') {
            pid = value.parse().ok();
            name = None;
        } else if let Some(value) = line.strip_prefix('c') {
            name = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix('n') {
            if let Some(pid) = pid {
                owners.push(ListenerOwner {
                    pid,
                    process_name: name.clone(),
                    address: value.to_string(),
                });
            }
        }
    }
    owners
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(id: &str, domain: &str, port: u16) -> ServiceConfig {
        ServiceConfig {
            id: id.into(),
            name: id.into(),
            group: String::new(),
            domain: domain.into(),
            remark: String::new(),
            port,
            local_ip: "127.0.0.1".into(),
            tunnel_id: "default".into(),
            sort_order: 0,
            enabled: true,
        }
    }

    #[tokio::test]
    async fn real_conflict_skips_shared_domain_and_preserves_other_listener() {
        let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let available = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let free_port = available.local_addr().unwrap().port();
        drop(available);
        let services = vec![
            service(
                "occupied",
                "db.example",
                occupied.local_addr().unwrap().port(),
            ),
            service("shared", "DB.example", 0),
            service("other", "other.example", free_port),
        ];
        let (listeners, issues) = prepare_listeners(&services).await;
        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].0.id, "other");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].kind, StartupIssueKind::ListenerConflict);
        assert_eq!(issues[1].kind, StartupIssueKind::SharedDomain);
        assert!(TcpListener::bind(("127.0.0.1", free_port)).await.is_err());
        drop(listeners);
        assert!(TcpListener::bind(("127.0.0.1", free_port)).await.is_ok());
    }

    #[tokio::test]
    async fn wildcard_listener_conflicts_with_loopback_and_recheck_succeeds_after_release() {
        let occupied = bind_listener("0.0.0.0:0".parse().unwrap()).await.unwrap();
        let services = vec![service(
            "wildcard",
            "db.example",
            occupied.local_addr().unwrap().port(),
        )];
        let (listeners, issues) = prepare_listeners(&services).await;
        assert!(listeners.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].address, format!("127.0.0.1:{}", services[0].port));
        drop(occupied);
        let (listeners, issues) = prepare_listeners(&services).await;
        assert!(issues.is_empty());
        assert_eq!(listeners.len(), 1);
    }

    #[test]
    fn matches_exact_and_wildcard_addresses_without_matching_unrelated_ip() {
        let owners = parse_lsof("p10\ncdaemon\nn*:3306\np11\ncother\nn127.0.0.2:3306\np12\nclocal\nn127.0.0.1:3306\np13\ncv6\nn[::]:3306");
        let result = filter_owners(owners, "127.0.0.1:3306".parse().unwrap());
        assert_eq!(
            result.iter().map(|owner| owner.pid).collect::<Vec<_>>(),
            vec![10, 12, 13]
        );
        let owners =
            parse_ss("LISTEN 0 128 0.0.0.0:3306 0.0.0.0:* users:((\"mysqld\",pid=42,fd=5))");
        assert_eq!(owners[0].pid, 42);
        assert_eq!(owners[0].process_name.as_deref(), Some("mysqld"));
    }
}
