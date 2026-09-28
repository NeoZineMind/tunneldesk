# Security Policy

TunnelDesk handles SSH credentials and modifies the platform hosts file.

## Secret Handling

- SSH passwords and private key passphrases are stored through the OS credential store.
- Configuration files only store credential keys.
- Logs must never include secret values.

## Reporting

For internal deployments, report suspected credential leakage, unexpected hosts modifications, or SSH host key changes to the owning team immediately.
