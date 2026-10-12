use std::collections::{HashMap, HashSet};
use std::future::Future;

use crate::credential;
use crate::error::{AppError, AppResult};
use crate::hosts;
use crate::listener_diagnostics::{exclude_failed_domains, prepare_listeners, BoundListeners};
use crate::model::{
    AppSettings, AuthMethod, ServiceConfig, StartMode, StartupIssue, StartupIssueKind,
    StartupOutcome, StartupReport, TunnelConfig,
};
use crate::tunnel::{self, TunnelRuntime};
use crate::validation;

trait Runtime: Send {
    fn services(&self) -> &[ServiceConfig];
    fn retain(&mut self, allowed: &HashSet<String>) -> impl Future<Output = ()> + Send;
    fn stop(self) -> impl Future<Output = ()> + Send;
}

impl Runtime for TunnelRuntime {
    fn services(&self) -> &[ServiceConfig] {
        self.services()
    }
    async fn retain(&mut self, allowed: &HashSet<String>) {
        self.retain_services(allowed).await;
    }
    async fn stop(self) {
        self.stop().await;
    }
}

trait Backend {
    type Runtime: Runtime;
    type Resources: Send;
    fn start(
        &self,
        tunnel: TunnelConfig,
        listeners: BoundListeners,
        resources: Option<Self::Resources>,
    ) -> impl Future<Output = AppResult<(Self::Runtime, Vec<StartupIssue>)>> + Send;
    fn write_hosts(&self, services: &[ServiceConfig]) -> AppResult<()>;
}

pub struct RealBackend;

impl Backend for RealBackend {
    type Runtime = TunnelRuntime;
    type Resources = (
        std::sync::Arc<tunnel::SshSessionManager>,
        tunnel::SharedServiceStatuses,
    );

    async fn start(
        &self,
        tunnel: TunnelConfig,
        listeners: BoundListeners,
        resources: Option<Self::Resources>,
    ) -> AppResult<(TunnelRuntime, Vec<StartupIssue>)> {
        if !tunnel.enabled {
            return Err(AppError::Message(format!(
                "Tunnel is disabled: {}",
                tunnel.name
            )));
        }
        validation::validate_tunnel_for_connection(&tunnel)?;
        let password = if resources.is_none() && tunnel.ssh.auth_method == AuthMethod::Password {
            Some(credential::get_secret(&credential::tunnel_password_key(
                &tunnel.id,
            ))?)
        } else {
            None
        };
        tunnel::start(tunnel, listeners, password, resources).await
    }

    fn write_hosts(&self, services: &[ServiceConfig]) -> AppResult<()> {
        hosts::write_services_block(services)
    }
}

pub struct StartExecution<R> {
    pub runtimes: HashMap<String, R>,
    pub report: StartupReport,
}

fn report(
    profile_id: &str,
    mode: StartMode,
    services: &[ServiceConfig],
    issues: Vec<StartupIssue>,
    message: Option<String>,
) -> StartupReport {
    let outcome = if services.is_empty() {
        StartupOutcome::Failed
    } else if issues.is_empty() {
        StartupOutcome::Complete
    } else {
        StartupOutcome::Partial
    };
    let description = message.unwrap_or_else(|| match outcome {
        StartupOutcome::Failed => String::from("启动失败，未启动任何服务"),
        StartupOutcome::Complete => format!("已启动 {} 个服务", services.len()),
        StartupOutcome::Partial => format!(
            "已启动 {} 个服务，跳过 {} 个服务",
            services.len(),
            issues.len()
        ),
    });
    StartupReport {
        profile_id: profile_id.into(),
        mode,
        outcome,
        started_service_ids: services.iter().map(|service| service.id.clone()).collect(),
        issues,
        message: description,
    }
}

async fn stop_all<R: Runtime>(runtimes: HashMap<String, R>) {
    for (_, runtime) in runtimes {
        runtime.stop().await;
    }
}

// The same transaction is used by manual startup, auto-start and incremental retries.
// Tests supply an isolated hosts writer and SSH backend, but bind real TCP listeners.
async fn execute<B: Backend>(
    backend: &B,
    settings: &AppSettings,
    profile_id: &str,
    candidates: &[ServiceConfig],
    existing_services: &[ServiceConfig],
    mut resources: HashMap<String, B::Resources>,
    mode: StartMode,
) -> StartExecution<B::Runtime> {
    let (listeners, mut issues) = prepare_listeners(candidates).await;
    let mut grouped = std::collections::BTreeMap::<String, BoundListeners>::new();
    if mode == StartMode::Strict && !issues.is_empty() {
        drop(listeners);
        return StartExecution {
            runtimes: HashMap::new(),
            report: report(profile_id, mode, existing_services, issues, None),
        };
    }
    for (service, listener) in listeners {
        grouped
            .entry(service.tunnel_id.clone())
            .or_default()
            .push((service, listener));
    }
    let mut runtimes = HashMap::new();
    for (id, listeners) in grouped {
        let services = listeners
            .iter()
            .map(|(service, _)| service.clone())
            .collect::<Vec<_>>();
        let result = match settings.tunnels.iter().find(|tunnel| tunnel.id == id) {
            Some(tunnel) => {
                backend
                    .start(tunnel.clone(), listeners, resources.remove(&id))
                    .await
            }
            None => Err(AppError::Message(format!("Tunnel not found: {id}"))),
        };
        match result {
            Ok((runtime, failures)) => {
                issues.extend(failures);
                runtimes.insert(id, runtime);
            }
            Err(error) => issues.extend(services.iter().map(|service| {
                StartupIssue::for_service(service, StartupIssueKind::SshError, error.to_string())
            })),
        }
    }
    exclude_failed_domains(candidates, &mut issues);
    if mode == StartMode::Strict && !issues.is_empty() {
        stop_all(runtimes).await;
        return StartExecution {
            runtimes: HashMap::new(),
            report: report(profile_id, mode, existing_services, issues, None),
        };
    }
    let failed = issues
        .iter()
        .map(|issue| issue.service_id.clone())
        .collect::<HashSet<_>>();
    let allowed = candidates
        .iter()
        .filter(|service| !failed.contains(&service.id))
        .map(|service| service.id.clone())
        .collect::<HashSet<_>>();
    let mut ready = HashMap::new();
    for (id, mut runtime) in runtimes {
        runtime.retain(&allowed).await;
        if runtime.services().is_empty() {
            runtime.stop().await;
        } else {
            ready.insert(id, runtime);
        }
    }
    let new_services = ready
        .values()
        .flat_map(|runtime| runtime.services().iter().cloned())
        .collect::<Vec<_>>();
    let mut services = existing_services.to_vec();
    services.extend(new_services.clone());
    services.sort_by_key(|service| (service.sort_order, service.id.clone()));
    if !new_services.is_empty() {
        if let Err(error) = backend.write_hosts(&services) {
            stop_all(ready).await;
            let message = format!("hosts 更新失败：{error}");
            issues.extend(new_services.iter().map(|service| {
                StartupIssue::for_service(service, StartupIssueKind::HostsError, message.clone())
            }));
            return StartExecution {
                runtimes: HashMap::new(),
                report: report(profile_id, mode, existing_services, issues, Some(message)),
            };
        }
    }
    StartExecution {
        runtimes: ready,
        report: report(profile_id, mode, &services, issues, None),
    }
}

pub async fn start(
    settings: &AppSettings,
    profile_id: &str,
    candidates: &[ServiceConfig],
    existing_services: &[ServiceConfig],
    resources: HashMap<
        String,
        (
            std::sync::Arc<tunnel::SshSessionManager>,
            tunnel::SharedServiceStatuses,
        ),
    >,
    mode: StartMode,
) -> StartExecution<TunnelRuntime> {
    execute(
        &RealBackend,
        settings,
        profile_id,
        candidates,
        existing_services,
        resources,
        mode,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::net::TcpListener;

    struct TestRuntime {
        services: Vec<ServiceConfig>,
        listeners: BoundListeners,
    }
    impl Runtime for TestRuntime {
        fn services(&self) -> &[ServiceConfig] {
            &self.services
        }
        async fn retain(&mut self, allowed: &HashSet<String>) {
            self.services
                .retain(|service| allowed.contains(&service.id));
            self.listeners
                .retain(|(service, _)| allowed.contains(&service.id));
        }
        async fn stop(self) {
            drop(self);
        }
    }

    #[derive(Default)]
    struct TestBackend {
        fail_tunnel: Option<String>,
        fail_service: Option<String>,
        fail_hosts: bool,
        writes: Mutex<Vec<Vec<String>>>,
        starts: Mutex<Vec<(String, bool)>>,
    }
    impl Backend for TestBackend {
        type Runtime = TestRuntime;
        type Resources = ();
        async fn start(
            &self,
            tunnel: TunnelConfig,
            mut listeners: BoundListeners,
            resources: Option<()>,
        ) -> AppResult<(TestRuntime, Vec<StartupIssue>)> {
            self.starts
                .lock()
                .unwrap()
                .push((tunnel.id.clone(), resources.is_some()));
            if self.fail_tunnel.as_deref() == Some(&tunnel.id) {
                return Err(AppError::Ssh("SSH authentication failed".into()));
            }
            let issues = listeners
                .iter()
                .filter(|(service, _)| self.fail_service.as_deref() == Some(&service.id))
                .map(|(service, _)| {
                    StartupIssue::for_service(
                        service,
                        StartupIssueKind::ForwardingError,
                        "Forwarding refused".into(),
                    )
                })
                .collect::<Vec<_>>();
            listeners
                .retain(|(service, _)| !issues.iter().any(|issue| issue.service_id == service.id));
            Ok((
                TestRuntime {
                    services: listeners
                        .iter()
                        .map(|(service, _)| service.clone())
                        .collect(),
                    listeners,
                },
                issues,
            ))
        }
        fn write_hosts(&self, services: &[ServiceConfig]) -> AppResult<()> {
            if self.fail_hosts {
                return Err(AppError::Message(
                    "System authorization was cancelled".into(),
                ));
            }
            self.writes
                .lock()
                .unwrap()
                .push(services.iter().map(|service| service.id.clone()).collect());
            Ok(())
        }
    }

    fn service(id: &str, tunnel: &str) -> ServiceConfig {
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        ServiceConfig {
            id: id.into(),
            name: id.into(),
            group: String::new(),
            domain: format!("{id}.example"),
            remark: String::new(),
            port: socket.local_addr().unwrap().port(),
            local_ip: "127.0.0.1".into(),
            tunnel_id: tunnel.into(),
            sort_order: 0,
            enabled: true,
        }
    }
    fn settings() -> AppSettings {
        AppSettings {
            tunnels: vec![
                TunnelConfig {
                    id: "a".into(),
                    ..TunnelConfig::default()
                },
                TunnelConfig {
                    id: "b".into(),
                    ..TunnelConfig::default()
                },
            ],
            ..AppSettings::default()
        }
    }
    async fn assert_released(services: &[ServiceConfig]) {
        for service in services {
            assert!(
                TcpListener::bind((service.local_ip.as_str(), service.port))
                    .await
                    .is_ok(),
                "listener {} leaked",
                service.id
            );
        }
    }

    #[tokio::test]
    async fn strict_conflict_releases_other_bound_sockets_without_connecting_or_writing_hosts() {
        let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut blocked = service("blocked", "a");
        blocked.port = occupied.local_addr().unwrap().port();
        let good = service("good", "a");
        let backend = TestBackend::default();
        let result = execute(
            &backend,
            &settings(),
            "test",
            &[blocked, good.clone()],
            &[],
            HashMap::new(),
            StartMode::Strict,
        )
        .await;
        assert_eq!(result.report.outcome, StartupOutcome::Failed);
        assert!(result.runtimes.is_empty());
        assert!(backend.starts.lock().unwrap().is_empty());
        assert!(backend.writes.lock().unwrap().is_empty());
        assert_released(&[good]).await;
    }

    #[tokio::test]
    async fn all_conflicts_report_failure_without_empty_runtimes_or_hosts_writes() {
        let first = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let second = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut services = vec![service("first", "a"), service("second", "b")];
        services[0].port = first.local_addr().unwrap().port();
        services[1].port = second.local_addr().unwrap().port();
        let backend = TestBackend::default();
        let result = execute(
            &backend,
            &settings(),
            "test",
            &services,
            &[],
            HashMap::new(),
            StartMode::Partial,
        )
        .await;
        assert_eq!(result.report.outcome, StartupOutcome::Failed);
        assert_eq!(result.report.issues.len(), 2);
        assert!(result.runtimes.is_empty());
        assert!(backend.writes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn partial_conflict_keeps_other_service_on_the_same_tunnel() {
        let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut blocked = service("blocked", "a");
        blocked.port = occupied.local_addr().unwrap().port();
        let good = service("good", "a");
        let backend = TestBackend::default();
        let result = execute(
            &backend,
            &settings(),
            "test",
            &[blocked, good.clone()],
            &[],
            HashMap::new(),
            StartMode::Partial,
        )
        .await;
        assert_eq!(result.report.outcome, StartupOutcome::Partial);
        assert_eq!(result.report.started_service_ids, vec!["good"]);
        assert_eq!(*backend.writes.lock().unwrap(), vec![vec!["good"]]);
        assert!(TcpListener::bind(("127.0.0.1", good.port)).await.is_err());
        stop_all(result.runtimes).await;
        assert_released(&[good]).await;
    }

    #[tokio::test]
    async fn ssh_failure_isolated_in_partial_mode_and_rolls_back_strict_mode() {
        for mode in [StartMode::Partial, StartMode::Strict] {
            let services = vec![service("good", "a"), service("bad", "b")];
            let backend = TestBackend {
                fail_tunnel: Some("b".into()),
                ..Default::default()
            };
            let result = execute(
                &backend,
                &settings(),
                "test",
                &services,
                &[],
                HashMap::new(),
                mode,
            )
            .await;
            assert_eq!(result.report.issues[0].kind, StartupIssueKind::SshError);
            assert_eq!(
                result.runtimes.len(),
                usize::from(mode == StartMode::Partial)
            );
            assert_eq!(
                backend.writes.lock().unwrap().len(),
                usize::from(mode == StartMode::Partial)
            );
            stop_all(result.runtimes).await;
            assert_released(&services).await;
        }
    }

    #[tokio::test]
    async fn forwarding_failure_prunes_shared_domain_across_tunnels_and_strict_rolls_back() {
        for mode in [StartMode::Partial, StartMode::Strict] {
            let mut shared = service("shared", "a");
            shared.domain = "bad.example".into();
            let services = vec![shared, service("bad", "b"), service("good", "a")];
            let backend = TestBackend {
                fail_service: Some("bad".into()),
                ..Default::default()
            };
            let result = execute(
                &backend,
                &settings(),
                "test",
                &services,
                &[],
                HashMap::new(),
                mode,
            )
            .await;
            assert_eq!(result.report.issues.len(), 2);
            assert!(result
                .report
                .issues
                .iter()
                .any(|issue| issue.kind == StartupIssueKind::SharedDomain));
            if mode == StartMode::Partial {
                assert_eq!(result.report.started_service_ids, vec!["good"]);
            } else {
                assert_eq!(result.report.outcome, StartupOutcome::Failed);
                assert!(backend.writes.lock().unwrap().is_empty());
            }
            stop_all(result.runtimes).await;
            assert_released(&services).await;
        }
    }

    #[tokio::test]
    async fn retry_uses_existing_resources_and_writes_combined_hosts() {
        let old = service("old", "a");
        let new = service("new", "a");
        let backend = TestBackend::default();
        let result = execute(
            &backend,
            &settings(),
            "test",
            &[new],
            &[old],
            HashMap::from([("a".into(), ())]),
            StartMode::Partial,
        )
        .await;
        assert_eq!(result.report.outcome, StartupOutcome::Complete);
        assert_eq!(*backend.starts.lock().unwrap(), vec![("a".into(), true)]);
        assert_eq!(backend.writes.lock().unwrap()[0], vec!["new", "old"]);
        stop_all(result.runtimes).await;
    }

    #[tokio::test]
    async fn hosts_cancellation_releases_new_listeners_and_preserves_existing_listener() {
        let old = service("old", "a");
        let new = service("new", "a");
        let existing = TcpListener::bind(("127.0.0.1", old.port)).await.unwrap();
        let backend = TestBackend {
            fail_hosts: true,
            ..Default::default()
        };
        let result = execute(
            &backend,
            &settings(),
            "test",
            std::slice::from_ref(&new),
            std::slice::from_ref(&old),
            HashMap::from([("a".into(), ())]),
            StartMode::Partial,
        )
        .await;
        assert!(result.runtimes.is_empty());
        assert_eq!(result.report.started_service_ids, vec!["old"]);
        assert_eq!(result.report.issues[0].kind, StartupIssueKind::HostsError);
        assert_released(&[new]).await;
        assert!(TcpListener::bind(("127.0.0.1", old.port)).await.is_err());
        drop(existing);
    }

    #[test]
    fn legacy_settings_default_to_strict() {
        let mut json = serde_json::to_value(AppSettings::default()).unwrap();
        json["behavior"]
            .as_object_mut()
            .unwrap()
            .remove("startMode");
        let settings: AppSettings = serde_json::from_value(json).unwrap();
        assert_eq!(settings.behavior.start_mode, StartMode::Strict);
    }
}
