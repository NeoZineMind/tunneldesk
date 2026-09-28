use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::hosts;
use crate::model::{
    HostsAccess, PrivilegeStatus, ProcessPrivilege, ServiceConfig, ServiceState, ServiceStatus,
};
#[cfg(target_os = "linux")]
use crate::privileged_hosts;

pub fn service_status(service: &ServiceConfig) -> ServiceStatus {
    if !service.enabled {
        return ServiceStatus {
            service_id: service.id.clone(),
            state: ServiceState::Disabled,
            message: String::from("Disabled"),
        };
    }

    let addr = format!("{}:{}", service.local_ip, service.port);
    let socket_addr = match addr
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
    {
        Some(addr) => addr,
        None => {
            return ServiceStatus {
                service_id: service.id.clone(),
                state: ServiceState::Error,
                message: String::from("Invalid local address"),
            };
        }
    };

    match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)) {
        Ok(_) => ServiceStatus {
            service_id: service.id.clone(),
            state: ServiceState::Healthy,
            message: String::from("Local listener is reachable"),
        },
        Err(error) => ServiceStatus {
            service_id: service.id.clone(),
            state: ServiceState::Stopped,
            message: error.to_string(),
        },
    }
}

pub fn is_admin() -> bool {
    platform_is_admin()
}

pub fn privilege_status() -> PrivilegeStatus {
    let process = platform_process_privilege();
    let direct_hosts_access = hosts::can_write_hosts();
    let helper_installed = linux_helper_installed();
    let helper_authorization_available = linux_helper_authorization_available();
    let hosts_access = if direct_hosts_access {
        HostsAccess::Direct
    } else if helper_authorization_available {
        HostsAccess::PolkitHelper
    } else {
        HostsAccess::Unavailable
    };
    let can_modify_hosts = !matches!(hosts_access, HostsAccess::Unavailable);
    let message = privilege_message(
        &process,
        &hosts_access,
        helper_installed,
        helper_authorization_available,
    );

    PrivilegeStatus {
        process,
        hosts_access,
        helper_installed,
        can_modify_hosts,
        message,
    }
}

#[cfg(target_os = "windows")]
fn platform_is_admin() -> bool {
    hosts::can_write_hosts()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn platform_is_admin() -> bool {
    is_root_user(effective_uid())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn effective_uid() -> u32 {
    extern "C" {
        fn geteuid() -> u32;
    }

    unsafe { geteuid() }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn platform_is_admin() -> bool {
    false
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
fn platform_process_privilege() -> ProcessPrivilege {
    if platform_is_admin() {
        ProcessPrivilege::Root
    } else {
        ProcessPrivilege::User
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn platform_process_privilege() -> ProcessPrivilege {
    ProcessPrivilege::Unknown
}

#[cfg(target_os = "linux")]
fn linux_helper_installed() -> bool {
    privileged_hosts::helper_installed()
}

#[cfg(not(target_os = "linux"))]
fn linux_helper_installed() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn linux_helper_authorization_available() -> bool {
    privileged_hosts::authorization_available()
}

#[cfg(not(target_os = "linux"))]
fn linux_helper_authorization_available() -> bool {
    false
}

fn privilege_message(
    process: &ProcessPrivilege,
    hosts_access: &HostsAccess,
    helper_installed: bool,
    helper_authorization_available: bool,
) -> String {
    match hosts_access {
        HostsAccess::Direct => String::from("Hosts can be modified directly"),
        HostsAccess::PolkitHelper => {
            String::from("Hosts can be modified with system authorization")
        }
        HostsAccess::Unavailable if helper_installed && !helper_authorization_available => {
            String::from("Linux hosts helper is installed, but pkexec is not available")
        }
        HostsAccess::Unavailable if matches!(process, ProcessPrivilege::User) => {
            String::from("Hosts modification requires elevated privileges or the Linux helper")
        }
        HostsAccess::Unavailable => String::from("Hosts modification is unavailable"),
    }
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn is_root_user(uid: u32) -> bool {
    uid == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_uid_is_admin() {
        assert!(is_root_user(0));
    }

    #[test]
    fn regular_uid_is_not_admin() {
        assert!(!is_root_user(1000));
    }

    #[test]
    fn privilege_message_reports_polkit_helper() {
        let message = privilege_message(
            &ProcessPrivilege::User,
            &HostsAccess::PolkitHelper,
            true,
            true,
        );

        assert!(message.contains("system authorization"));
    }
}
