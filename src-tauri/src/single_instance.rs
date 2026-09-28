use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use tauri::{AppHandle, Manager};

const WAKE_ADDR: &str = "127.0.0.1:40571";
const WAKE_MESSAGE: &[u8] = b"TunnelDesk:show:v1\n";
const WAKE_ACK: &[u8] = b"TunnelDesk:ok:v1\n";

pub fn notify_existing_instance() -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(
        &WAKE_ADDR
            .parse()
            .expect("valid single instance wake address"),
        Duration::from_millis(180),
    ) else {
        return false;
    };

    let _ = stream.set_write_timeout(Some(Duration::from_millis(180)));
    let _ = stream.set_read_timeout(Some(Duration::from_millis(180)));
    if stream.write_all(WAKE_MESSAGE).is_err() {
        return false;
    }

    let mut buffer = [0_u8; 32];
    matches!(stream.read(&mut buffer), Ok(size) if &buffer[..size] == WAKE_ACK)
}

pub fn start_wake_listener(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let listener = match TcpListener::bind(WAKE_ADDR) {
            Ok(listener) => listener,
            Err(error) => {
                tracing::warn!("Failed to bind single instance wake listener: {error}");
                return;
            }
        };

        for connection in listener.incoming() {
            match connection {
                Ok(mut stream) => {
                    let mut buffer = [0_u8; 32];
                    if let Ok(size) = stream.read(&mut buffer) {
                        if &buffer[..size] == WAKE_MESSAGE {
                            let _ = stream.write_all(WAKE_ACK);
                            show_main_window(&app_handle);
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!("Failed to receive single instance wake signal: {error}")
                }
            }
        }
    });
}

pub fn show_main_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
