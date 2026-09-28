# Architecture

TunnelDesk avoids global proxy interception. It uses scoped hosts mappings and local TCP listeners.

```text
Application -> real.service.domain:port
            -> hosts maps domain to 127.77.x.x
            -> TunnelDesk listens on 127.77.x.x:port
            -> SSH direct-tcpip through the jump host
            -> real.service.domain:port from the jump host network
```

This keeps existing proxy software untouched and lets applications keep their original service configuration.

## Components

- `config`: JSON settings and profile persistence.
- `credential`: OS credential store wrapper.
- `hosts`: platform hosts marker block management.
- `tunnel`: local listeners and SSH `direct-tcpip` forwarding.
- `health`: status and diagnostics.
- `validation`: backend-side settings and profile validation.

## Runtime Data

TunnelDesk stores all mutable user data in the per-user data directory so the repository stays generic:

- `settings.json` contains the SSH host, port, username, selected profile, and non-secret behavior flags.
- `profiles.json` contains user-managed service mappings.
- secrets are stored through the platform keyring under backend-derived tunnel keys.
- hosts backups are created in the app `backups` directory for direct writes, and under `/var/lib/tunneldesk/backups` when the Linux polkit helper performs the write.

## Credential Boundary

TunnelDesk follows the same high-level split used by desktop SSH managers such as Xshell and SecureCRT: tunnel/session metadata is persisted as normal configuration, while password values live in the operating-system credential store.

- The frontend never receives saved password values.
- The frontend cannot choose arbitrary credential keys; it calls tunnel-scoped commands such as `save_tunnel_password`.
- The backend derives password keys as `TunnelDesk:tunnel:{tunnelId}:password`.
- Loading old settings copies legacy password credentials to the stable tunnel key when possible, then normalizes the stored reference.
- Logs and exported JSON do not include password values.

## Privilege Boundary

Editing the hosts file requires elevated privileges: `C:\Windows\System32\drivers\etc\hosts` on Windows and `/etc/hosts` on Linux/macOS. On Linux `.deb` installs, the GUI stays in the normal user session and invokes `/usr/lib/tunneldesk/tunneldesk-hosts-helper` through polkit when hosts changes are needed. The helper accepts only structured TunnelDesk service mappings and only rewrites the TunnelDesk marker block.

## Validation

The Rust backend rejects invalid configuration at command boundaries:

- SSH host and username are required.
- SSH and service ports must be non-zero.
- service local IPs must be loopback addresses.
- service listener pairs must not duplicate within a profile.
- profile and service ids must be unique.

## Known MVP Limits

- Password authentication is implemented first. Private key and ssh-agent modes are visible in the UI as planned extension points.
- SSH host keys are currently accepted by policy. A production release should store fingerprints and warn on changes.
