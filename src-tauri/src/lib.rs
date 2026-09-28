mod commands;
mod config;
mod credential;
mod error;
mod health;
mod hosts;
pub mod hosts_core;
mod model;
#[cfg(target_os = "linux")]
mod privileged_hosts;
mod profile_transfer;
mod single_instance;
mod startup;
mod state;
mod tunnel;
mod validation;

use state::AppState;
#[cfg(not(all(target_os = "linux", target_arch = "x86")))]
use tauri::menu::{Menu, MenuItem};
#[cfg(not(all(target_os = "linux", target_arch = "x86")))]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{include_image, Emitter, Listener, Manager, RunEvent};

pub fn run() {
    if single_instance::notify_existing_instance() {
        return;
    }

    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .setup(|app| {
            setup_tray(app)?;
            single_instance::start_wake_listener(app.handle().clone());
            let startup_handle = app.handle().clone();

            let settings = config::load_settings().unwrap_or_default();
            if settings.behavior.start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            if settings.behavior.auto_repair_on_start {
                std::thread::spawn(|| {
                    let started = std::time::Instant::now();
                    if !hosts::can_write_hosts() {
                        tracing::info!(
                            elapsed_ms = started.elapsed().as_millis(),
                            "Startup hosts repair skipped; direct hosts access is unavailable"
                        );
                        return;
                    }
                    match hosts::remove_block_without_elevation() {
                        Ok(true) => tracing::info!(
                            elapsed_ms = started.elapsed().as_millis(),
                            "Startup hosts repair removed TunnelDesk block"
                        ),
                        Ok(false) => tracing::info!(
                            elapsed_ms = started.elapsed().as_millis(),
                            "Startup hosts repair skipped; no TunnelDesk block"
                        ),
                        Err(error) => tracing::warn!(
                            elapsed_ms = started.elapsed().as_millis(),
                            "Failed to repair hosts on startup: {error}"
                        ),
                    }
                });
            }
            if settings.behavior.auto_start_profile {
                tauri::async_runtime::spawn(async move {
                    let state = startup_handle.state::<AppState>();
                    if let Err(error) = commands::start_profile_for_state(state.inner()).await {
                        tracing::error!("Failed to auto start profile: {error}");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_settings,
            commands::launch_at_login_enabled,
            commands::save_settings,
            commands::set_launch_at_login,
            commands::load_profiles,
            commands::save_profiles,
            commands::list_config_backups,
            commands::restore_config_backup,
            commands::delete_config_backup,
            commands::export_profiles,
            commands::preview_profiles_import,
            commands::apply_profiles_import,
            commands::save_tunnel_password,
            commands::delete_tunnel_password,
            commands::has_tunnel_password,
            commands::test_ssh,
            commands::start_profile,
            commands::stop_profile,
            commands::get_status,
            commands::test_service,
            commands::repair_hosts,
            commands::read_logs,
            commands::open_log_dir
        ])
        .build(tauri::generate_context!())
        .expect("failed to build TunnelDesk")
        .run(|_app_handle, event| {
            if let RunEvent::ExitRequested { .. } = event {
                tracing::info!("TunnelDesk exiting");
            }
        });
}

#[cfg(not(all(target_os = "linux", target_arch = "x86")))]
fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let exit_handle = app.handle().clone();
    app.listen("app-exit-confirmed", move |_| exit_app(&exit_handle));
    let show = MenuItem::with_id(app, "show", "打开 TunnelDesk", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let app_handle = app.handle().clone();
    let icon = include_image!("icons/tray-icon.png");

    TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                single_instance::show_main_window(app);
            }
            "quit" => {
                single_instance::show_main_window(app);
                if let Err(error) = app.emit("app-exit-requested", ()) {
                    tracing::warn!("Failed to request exit confirmation: {error}");
                }
            }
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                single_instance::show_main_window(&app_handle);
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(all(target_os = "linux", target_arch = "x86"))]
fn setup_tray(_app: &mut tauri::App) -> tauri::Result<()> {
    Ok(())
}

fn init_logging() {
    let logs_dir = config::logs_dir().unwrap_or_else(|_| std::env::temp_dir().join("TunnelDesk"));
    let file_appender = tracing_appender::rolling::daily(logs_dir, "tunneldesk.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    std::mem::forget(guard);

    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn exit_app(app: &tauri::AppHandle) {
    let settings = config::load_settings().unwrap_or_default();
    if settings.behavior.cleanup_on_exit {
        if hosts::can_write_hosts() {
            if let Err(error) = hosts::remove_block_without_elevation() {
                tracing::warn!("Failed to clean hosts on exit: {error}");
            }
        } else {
            tracing::info!("Skipped hosts cleanup on exit; direct hosts access is unavailable");
        }
    }
    app.exit(0);
}
