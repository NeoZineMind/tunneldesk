# Configuration

TunnelDesk keeps repository defaults generic. Real SSH, tunnel, service, and behavior data live under the per-user data directory:

- Windows: `%APPDATA%\TunnelDesk`
- Linux: `$XDG_DATA_HOME/TunnelDesk` or `~/.local/share/TunnelDesk`
- macOS: `~/Library/Application Support/TunnelDesk`

## App Settings

Saved in `settings.json` inside that data directory:

```json
{
  "schemaVersion": 2,
  "currentProfileId": "default",
  "currentTunnelId": "default",
  "tunnels": [
    {
      "id": "default",
      "name": "Default Tunnel",
      "enabled": true,
      "ssh": {
        "host": "jump.example.com",
        "port": 22,
        "username": "developer",
        "authMethod": "password",
        "identityFile": "",
        "passwordCredentialKey": "TunnelDesk:tunnel:default:password",
        "keyPassphraseCredentialKey": "",
        "serverAliveInterval": 30,
        "serverAliveCountMax": 3
      }
    }
  ],
  "behavior": {
    "startMode": "strict",
    "startMinimized": false,
    "autoStartProfile": false,
    "launchAtLogin": false,
    "autoRepairOnStart": false,
    "cleanupOnExit": true,
    "themeMode": "system"
  }
}
```

`themeMode` accepts `system`, `light`, or `dark`. `autoStartProfile` starts the current profile after the app opens. `launchAtLogin` mirrors the platform login startup entry. The password value itself is not saved here.

`startMode` accepts `strict` (the default, including existing configurations) or `partial`. Strict startup requires every enabled listener, SSH connection, and forwarding preflight to succeed; otherwise newly acquired resources are released. Partial startup skips unavailable services and failed tunnels, and writes hosts entries only for the services that start. A failed service causes all services sharing its domain to be skipped because hosts mappings cannot distinguish ports. Services sharing a domain must use the same local IP.

Manual startup lists listener conflicts with addresses and available process names/PIDs before changing hosts. Choosing to skip applies to that startup only and does not disable services in the saved Profile. Process lookup is best effort: Windows uses the TCP connection table through PowerShell, Linux uses `ss` with `lsof` fallback, and macOS uses `lsof`; restricted permissions, missing tools, or timeouts leave the process information unavailable.

Overview and Diagnostics retain the startup report, including when all services fail. Retry skipped services reuses active SSH sessions and preserves existing listeners. If Profile or involved tunnel configuration changes, stop and start again instead. Auto-start uses the saved strategy and makes its report available when the main window is opened.

## Password Storage

SSH passwords are stored in the platform keyring through the Rust backend. This follows the same broad pattern used by desktop SSH managers such as Xshell and SecureCRT: session metadata is stored separately from secret values, and the UI never reads saved passwords back.

TunnelDesk derives password keys itself using:

```text
TunnelDesk:tunnel:{tunnelId}:password
```

The frontend only sends `tunnelId` plus a new password value to `save_tunnel_password`. It cannot choose arbitrary credential keys, cannot export saved password values, and cannot request a saved password value. Settings exports contain only tunnel metadata and the credential reference.

The platform keyring protects stored values with the current user account boundary. This is not a replacement for securing the OS account, disk, backups, and malware boundary.

## Logs

TunnelDesk writes daily rolling logs under the per-user `logs` directory. The default filter is `info`, so `info`, `warn`, and `error` events are recorded. To capture more detail while troubleshooting, start the app with a `RUST_LOG` override, for example:

```powershell
$env:RUST_LOG = "tunneldesk=debug,info"
.\TunnelDesk.exe
```

## Service Profile Sharing

The Services page supports creating, renaming, deleting, switching, importing, and exporting Profiles. Profile switching, Profile mutations, and import are disabled while a Profile is running so the active hosts block and local listeners stay aligned with the selected configuration.

Profile import/export is scoped to service Profile data only. Exported files use the same `ProfilesFile` JSON shape as `profiles.json`, including service groups and `sortOrder`; they do not include `settings.json`, SSH tunnel metadata, identity file paths, behavior settings, or password values.

When importing, TunnelDesk previews the changes, groups missing `tunnelId` references so they can be mapped to existing local tunnels once, and creates a timestamped `backups/profiles-YYYYMMDD-HHMMSS.json` copy before applying the merge. Importing never starts a Profile automatically.

## Service Profiles

Saved in `profiles.json` inside the data directory:

```json
{
  "schemaVersion": 2,
  "profiles": [
    {
      "id": "default",
      "name": "Default Profile",
      "enabled": true,
      "services": [
        {
          "id": "example-mysql",
          "name": "Example MySQL",
          "group": "Database",
          "domain": "mysql.example.internal",
          "port": 3306,
          "localIp": "127.77.0.10",
          "tunnelId": "default",
          "sortOrder": 10,
          "enabled": true
        }
      ]
    }
  ]
}
```

Each enabled service gets one hosts line:

```text
127.77.0.10 mysql.example.internal
```

TunnelDesk then listens on `127.77.0.10:3306` and forwards traffic through the selected tunnel to `mysql.example.internal:3306`.

## Windows Listener Failures

If Windows reports `os error 10013` / `WSAEACCES` while starting a profile, the failing service could not bind its local listener. For example, `127.77.0.10:3306` means Windows denied TCP port `3306` on that loopback address.

Check whether the port is already used or excluded:

```powershell
netsh interface ipv4 show excludedportrange protocol=tcp
netstat -ano | findstr :3306
```

Excluded port ranges are often created by Hyper-V, WSL, Docker, VPN software, Internet Connection Sharing, or endpoint security. Administrator privileges usually do not bypass those reservations; choose a service/application port outside the excluded range or adjust the component that reserved it.
