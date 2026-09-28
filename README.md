# TunnelDesk

TunnelDesk is a Windows/Linux/macOS desktop SSH tunnel manager for development services. It lets local applications keep using real service domains and ports, while TunnelDesk maps selected domains to loopback addresses and forwards traffic through one or more SSH jump hosts.

中文文档请见 [README.zh-CN.md](README.zh-CN.md).

## Overview

Development teams often need to reach databases, caches, internal APIs, or other private services through a jump host. The usual options are either manual SSH commands per service or a global VPN/proxy that changes too much of the workstation.

TunnelDesk takes a narrower approach:

1. You define one or more SSH tunnel configurations.
2. You create service profiles for the domains and ports your apps already use.
3. Each service chooses which tunnel it should use.
4. Starting a profile writes a controlled hosts block and starts the required local listeners.
5. Applications continue connecting to their original service hostnames.

TunnelDesk does not enable TUN mode and does not change Clash, Mihomo, system proxy, browser proxy, WinHTTP, or VPN settings.

## Features

- Multi-tunnel configuration: different services in the same profile can use different SSH jump hosts.
- Profile-based service mappings: create, rename, delete, switch, import, and export service groups by project, environment, or team workflow.
- Service grouping and ordering: organize services into editable groups and keep a stable display/startup order per profile.
- Controlled hosts management: TunnelDesk writes only inside its own `# BEGIN TUNNELDESK` marker block.
- Local loopback listeners: service domains resolve to addresses such as `127.77.0.10`, then TunnelDesk forwards traffic over SSH `direct-tcpip`.
- Tunnel-scoped password storage: passwords are stored in the platform keyring and are never written to JSON settings files.
- Theme modes: `System`, `Light`, and `Dark`, with `System` as the default.
- Startup behavior:
  - `launchAtLogin` opens TunnelDesk when the user logs in.
  - `autoStartProfile` starts the current profile after TunnelDesk launches.
- Backend-side validation for tunnel ids, service ids, ports, loopback addresses, duplicated listeners, and missing tunnel references.
- CI checks for frontend type safety, production frontend build, Rust formatting, Clippy, and Rust tests.
- CI packaging for Windows x64/ARM64, Linux x64/x86, and macOS x64/ARM64.

## Platform Support

| Platform      | Development     | CI Packages                                     | Notes                                                                                         |
| ------------- | --------------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Windows x64   | Supported       | portable `.zip` + NSIS installer                | Primary Windows target                                                                        |
| Windows ARM64 | Supported in CI | portable `.zip` + NSIS installer                | ARM64 target                                                                                  |
| Linux x64     | Supported       | portable `.tar.gz` + `.deb` + `.rpm` + AppImage | Primary Linux target                                                                          |
| Linux x86     | Experimental    | binary + `.deb`                                 | Built without the tray icon because Ubuntu 22.04 does not ship i386 AppIndicator dev packages |
| macOS x64     | Supported in CI | portable `.tar.gz` + unsigned `.dmg`            | Intel target                                                                                  |
| macOS ARM64   | Supported in CI | portable `.tar.gz` + unsigned `.dmg`            | Apple Silicon target                                                                          |

Hosts file changes require elevated privileges:

- Windows: `C:\Windows\System32\drivers\etc\hosts`
- Linux: `/etc/hosts`
- macOS: `/etc/hosts`

Normal configuration editing does not require elevation. On Debian/Ubuntu, the recommended `.deb` package keeps the GUI running as the normal user and uses polkit to authorize a small helper only when TunnelDesk needs to update `/etc/hosts`.

### Ubuntu Usage

Use the `.deb` package when possible. It installs:

- `/usr/lib/tunneldesk/tunneldesk-hosts-helper`
- `/usr/share/polkit-1/actions/com.tunneldesk.hosts.policy`

After installation, open TunnelDesk from the app launcher. When you click Start or Repair hosts, the system authorization dialog appears and only the helper updates the `# BEGIN TUNNELDESK` hosts block. Avoid running the GUI with `sudo TunnelDesk` for regular use because that stores configuration and credentials under `/root/.local/share/TunnelDesk`.

The AppImage remains useful as a portable build, but it does not install the system polkit helper. Prefer a native `.deb` or `.rpm` package when hosts modification is required.

### RPM-based Linux Usage

Use the `.rpm` package on RPM-based distributions such as Fedora, RHEL, and openSUSE. It installs the same helper and polkit policy:

- `/usr/lib/tunneldesk/tunneldesk-hosts-helper`
- `/usr/share/polkit-1/actions/com.tunneldesk.hosts.policy`

After installation, run TunnelDesk as a normal user and authorize hosts changes through the system polkit prompt.

## How It Works

```text
Application -> real.service.domain:port
            -> hosts maps domain to 127.77.x.x
            -> TunnelDesk listens on 127.77.x.x:port
            -> SSH direct-tcpip through the selected jump host
            -> real.service.domain:port from the jump host network
```

A profile can contain multiple enabled services. Each service has its own `tunnelId`. When the current profile starts, TunnelDesk groups enabled services by `tunnelId`, starts one SSH runtime per required tunnel, and creates local listeners for those services.

Only one profile is intended to run at a time. This keeps the hosts marker block deterministic and avoids conflicting local listener ownership.

### Windows Socket Error 10013

On some Windows machines, starting a profile can fail before SSH forwarding begins with `os error 10013` / `WSAEACCES`. TunnelDesk is trying to bind the configured local listener, for example `127.77.0.10:3306`, and Windows denied that socket.

The most common cause is that the TCP port is in an excluded or reserved range created by Hyper-V, WSL, Docker, VPN software, Internet Connection Sharing, or endpoint security. Check the affected port in an elevated PowerShell window:

```powershell
netsh interface ipv4 show excludedportrange protocol=tcp
netstat -ano | findstr :3306
```

If the port is in an excluded range, running TunnelDesk as Administrator usually does not bypass it. Use a service/application port outside the excluded range, or adjust the Windows component or security product that reserved the port.

## Configuration Files

Runtime data is stored in the current user's data directory:

- Windows: `%APPDATA%\TunnelDesk`
- Linux: `$XDG_DATA_HOME/TunnelDesk` or `~/.local/share/TunnelDesk`
- macOS: `~/Library/Application Support/TunnelDesk`

Files created there include:

- `settings.json`: selected profile, selected tunnel, tunnel metadata, SSH endpoint metadata, theme mode, and non-secret behavior settings.
- `profiles.json`: service profiles, service groups/order, loopback mappings, selected tunnel ids, and enabled states.
- `logs\`: rolling application logs.
- `backups\`: hosts backups created when the app can write hosts directly, plus local `profiles.json` backups created before Profile imports. On Linux, the polkit helper stores hosts backups under `/var/lib/tunneldesk/backups`.

Logs default to the `info` filter, which still records `warn` and `error` events. For more verbose troubleshooting, start TunnelDesk with `RUST_LOG=tunneldesk=debug,info`.

Example service profiles live in [examples/service-profiles.example.json](examples/service-profiles.example.json). Real team profiles should be distributed out-of-band or imported through the UI, not committed with production hostnames.

Profiles can be created, renamed, deleted, switched, imported, and exported from the Services page. Profile switching and profile mutations are disabled while a profile is running so the hosts block and active listeners cannot drift from the selected configuration.

Service Profile import/export is intentionally limited to `profiles.json` data: Profile names, service mappings, loopback addresses, selected `tunnelId` values, and enabled states. It does not include SSH tunnel metadata, app behavior settings, identity file paths, or password values. After importing a team Profile, each workstation still needs matching local tunnel configurations for the referenced `tunnelId` values.

## Security Model

TunnelDesk follows the same high-level split used by desktop SSH managers such as Xshell and SecureCRT: connection/session metadata is stored as normal configuration, while secret values live in the operating-system credential store.

The important boundaries are:

- SSH passwords are never written to `settings.json`, `profiles.json`, logs, or exported profile metadata.
- The frontend can save, delete, or check whether a tunnel password exists, but it cannot read a saved password back.
- The frontend cannot choose arbitrary credential keys.
- The Rust backend derives stable tunnel password keys using:

```text
TunnelDesk:tunnel:{tunnelId}:password
```

- Saved password values are protected by the platform keyring under the current OS user account boundary.
- Deleting a tunnel password removes the keyring entry for that tunnel id.
- Hosts writes are scoped to the TunnelDesk marker block only.

This model reduces accidental disclosure through JSON files, UI state, logs, screenshots, and profile exports. It is still not a replacement for securing the OS account, disk encryption, backups, malware boundary, and workstation access.

Current authentication limits:

- Password authentication is implemented.
- Private key and ssh-agent authentication are reserved as UI/backend extension points.
- SSH host key pinning is not implemented yet; the current MVP accepts server keys. A production release should store and verify host key fingerprints, then warn on changes.

## Startup Behavior

TunnelDesk has two separate startup-related settings:

- `launchAtLogin`: registers the app with the operating system so TunnelDesk opens after login.
- `autoStartProfile`: starts the currently selected profile after the app has launched.

On Windows, `launchAtLogin` writes a current-user `Run` registry value. On Linux, it writes a desktop autostart entry under `~/.config/autostart/TunnelDesk.desktop`. On macOS, it writes a user LaunchAgent plist under `~/Library/LaunchAgents/com.tunneldesk.app.plist`.

Use both settings together if you want the app to open after login and immediately start the active development profile.

## Development

Prerequisites on Windows:

- Node.js 20+
- pnpm 9+
- Rust stable
- Visual Studio Build Tools with the "Desktop development with C++" workload
- WebView2 Runtime

Install dependencies and run the app in development mode:

```powershell
pnpm install
pnpm tauri:dev
```

Frontend checks:

```powershell
pnpm typecheck
pnpm build
```

Rust checks:

```powershell
cd src-tauri
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Build a local Tauri package:

```powershell
pnpm tauri:build
```

Build Linux `.deb`, `.rpm`, and AppImage packages:

```bash
pnpm tauri:build:linux
```

## CI and Packaging

The GitHub Actions workflow in [.github/workflows/ci.yml](.github/workflows/ci.yml) runs:

- Frontend job on `windows-latest`:
  - `pnpm install --frozen-lockfile`
  - `pnpm typecheck`
  - `pnpm build`
- Rust job on `windows-latest`:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Package job on `main` pushes and manual workflow runs:
  - Windows x64: `x86_64-pc-windows-msvc`, portable executable `.zip` and NSIS installer are separate artifacts.
  - Windows ARM64: `aarch64-pc-windows-msvc`, portable executable `.zip` and NSIS installer are separate artifacts.
  - Linux x64: `x86_64-unknown-linux-gnu`, portable binary `.tar.gz`, `.deb`, `.rpm`, and AppImage.
  - Linux x86: `i686-unknown-linux-gnu`, release binary + `.deb`, marked experimental and built without the tray icon.
  - macOS x64: `x86_64-apple-darwin`, portable binary `.tar.gz` and unsigned `.dmg`.
  - macOS ARM64: `aarch64-apple-darwin`, portable binary `.tar.gz` and unsigned `.dmg`.
- Release workflow on `v*` tags publishes only portable archives and installer files to a formal GitHub Release. Linux x86 remains CI experimental and is not included in formal releases until it is stable.

The package workflow is based on the Tauri 2 GitHub Actions distribution approach: [Tauri GitHub pipelines](https://v2.tauri.app/distribute/pipelines/github).

## Data Model

Current settings use schema version 2:

- `AppSettings.tunnels`: all tunnel configurations.
- `AppSettings.currentTunnelId`: the tunnel selected in the tunnel management UI.
- `AppSettings.behavior.themeMode`: `system`, `light`, or `dark`.
- `ServiceConfig.tunnelId`: the tunnel used by a service.

Older settings with a single SSH configuration are migrated to a default tunnel. Older service profiles are migrated so services reference the default tunnel.

## Known Limits and Roadmap

- Private key and ssh-agent authentication are not implemented yet.
- SSH host key pinning should be added before production use in stricter environments.
- Only one active profile is supported at a time.
- Linux x86 packaging is marked experimental in CI and disables the tray icon to avoid unavailable i386 AppIndicator packages on Ubuntu 22.04.
- macOS packages are unsigned and not notarized.
- More import/export and team-sharing workflows can be added once the core local model settles.

## Documentation

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Example service profiles](examples/service-profiles.example.json)
