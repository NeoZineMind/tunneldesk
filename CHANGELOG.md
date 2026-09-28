# Changelog

## v0.1.20 - 2026-09-10

- Moved the active Profile into a global sidebar environment selector with runtime status and guarded switching.
- Added a persistent service configuration draft bar, explicit save/discard/cancel flows for switching, reloading, starting, and exiting, and separate confirmation for unfinished drawer edits.
- Blocked service configuration changes while running or saving, retained drafts on save failures, and distinguished successful writes from status refresh failures.
- Simplified service tables with copyable connection addresses, compact actions, and per-Profile collapsible groups that automatically expand for matching filters.
- Replaced the overview cards with a compact summary and paginated service list, keeping checking and unknown states separate from errors and linking affected services directly to diagnostics.
- Reordered service forms, moved local IP settings into a collapsible advanced section, added connection previews, and improved status labels, keyboard focus, and narrow-window layouts in both themes.
- Reduced settings-page navigation stalls by moving launch-at-login registry operations off the UI thread and removing deferred sidebar navigation.
- Added regression coverage for draft protection, runtime restrictions, service interactions, diagnostics navigation, and leaving settings while startup checks are pending.

## v0.1.19 - 2026-07-15

- Fixed Profile startup hanging after SSH forwarding preflight channels opened successfully by making preflight channel cleanup non-blocking.
- Added bounded SSH connection and forwarding preflight deadlines so unresponsive routes become service errors and the Start action always returns.

## v0.1.18 - 2026-07-15

- Rejected malformed service domains before saving profiles or writing hosts entries, preventing accidental trailing quotes from bypassing local hostname mappings.
- Added per-service SSH `direct-tcpip` preflight and runtime forwarding status so policy failures such as `AdministrativelyProhibited` are reported on the affected service.
- Stopped status refreshes from probing local listeners and falsely marking services healthy before the SSH server accepts their forwarding channels.
- Treated missing and non-healthy service states as workbench errors, with structured forwarding logs and regression coverage across the Rust and Vue layers.

## v0.1.17 - 2026-07-12

- Rebuilt the desktop interface around the new TunnelDesk visual system, including the app shell, workbench, diagnostics, logs, tunnel, service, and settings pages.
- Added a consistent cross-platform application and tray icon family generated from the new brand mark.
- Added derived workbench status, quick actions, service filtering, and shared UI primitives while preserving existing configuration and backend command compatibility.
- Replaced unreliable WebView HTML drag-and-drop service sorting with pointer-driven drag handling, and added an interaction test that verifies the reordered result.

## v0.1.16 - 2026-07-09

- Added automatic SSH session reconnection for keepalive, disconnect, send, and broken-pipe failures while keeping local listeners alive.
- Serialized reconnect attempts across forwarded services with bounded retry backoff from 1 to 30 seconds.
- Kept explicit `direct-tcpip` target failures such as `AdministrativelyProhibited` and `ConnectFailed` visible instead of retrying them as session drops.

## v0.1.15 - 2026-07-04

- Reworked Services into a read-only grouped table with drawer-based view/edit flows, domain copy actions, service remarks, and drag sorting within groups.
- Added a shared service form, unsaved Profile change protection, startup configuration validation, and grouped Overview service status display.
- Added automatic configuration backups before profile saves/imports, plus UI support for listing, restoring, and deleting backups.
- Fixed title-bar double-click maximize behavior and tightened narrow-width table text rendering.

## v0.1.14 - 2026-07-03

- Fixed log rendering for existing ANSI-colored `tracing` log lines so levels parse correctly instead of showing as `UNKNOWN`.
- Disabled ANSI escape output for new TunnelDesk log files.

## v0.1.13 - 2026-07-03

- Reworked the Logs page into a compact live console inspired by Clash Verge, with level filtering, keyword search, regex/case/whole-word toggles, pause/resume, refresh, sort, and clear-view controls.
- Added a backend `read_logs` command that reads recent rolling TunnelDesk logs and returns structured timestamp, level, target, and message fields for the UI.
- Added development-mode sample logs so the logging interface can be previewed and tested in the browser without a Tauri runtime.

## v0.1.12 - 2026-07-03

- Added editable service groups and per-profile service ordering, with grouped Services and Overview displays.
- Added `group` and `sortOrder` service profile fields with backward-compatible migration for existing `profiles.json` files and imported profiles.
- Improved Windows local listener failures by reporting the affected service/listener and explaining `os error 10013` reserved-port checks.
- Added ERROR-level startup/listener failure logs and documented the runtime log filter override.

## v0.1.11 - 2026-07-03

- Added service Profile creation, rename, and deletion from the Services page, with running-state guards and duplicate-name validation.
- Consolidated Profile actions into a compact selector overflow menu so import/export and CRUD controls no longer crowd the services toolbar.
- Added frontend tests and documentation for the expanded Profile management workflow.

## v0.1.10 - 2026-07-03

- Added service Profile import/export with previewed merges, missing tunnel mapping, pre-import `profiles.json` backups, and Profile switching in the services UI.
- Added Linux x64 RPM release packaging with the hosts helper and polkit policy included.

## v0.1.9 - 2026-07-01

- Fixed macOS release packaging so the updater metadata can include signed app archive artifacts alongside dmg installers.

## v0.1.8 - 2026-07-01

- Fixed Linux in-app updates for Debian package installs by publishing deb-specific updater metadata and signatures.
- Kept AppImage updater metadata explicit while preserving the existing Linux fallback.
- Improved the updater error message when the downloaded package format does not match the current install type.

## v0.1.7 - 2026-07-01

- Added Linux polkit-based hosts authorization so the GUI can run as a normal user on Ubuntu.
- Added a minimal `tunneldesk-hosts-helper` binary and Debian policy packaging for controlled `/etc/hosts` updates.
- Expanded diagnostics to show process privilege, hosts write capability, and Linux helper availability.

## v0.1.6 - 2026-06-30

- Unified the desktop, application, and tray icon set around the new TunnelDesk visual direction.
- Added single-instance behavior so reopening TunnelDesk focuses the existing window instead of launching another copy.
- Added close behavior preferences, including minimize-to-tray, exit, a close confirmation dialog, and a remember-choice option.

## v0.1.5 - 2026-06-30

- Reduced menu switching jank by deferring route navigation work and trimming the settings page component tree.
- Blocked the default webview context menu in the desktop app.
- Refreshed the desktop and tray icon set with a reproducible icon generation script.

## v0.1.4 - 2026-06-30

- Reduced startup work by deferring Windows launch-at-login checks, saved-password checks, and silent updater checks.
- Skipped redundant hosts rewrites and DNS flushes when the generated TunnelDesk hosts block is unchanged.
- Added staged timing logs for profile startup to make future launch bottlenecks easier to diagnose.

## v0.1.3 - 2026-06-29

- Added Cargo dependency fetch retries to make release packaging resilient to transient crates.io network resets.

## v0.1.2 - 2026-06-29

- Added signed in-app update checks and install flow.
- Added a top bar update button that appears when a newer release is available.
- Added a manual update check entry in settings.
- Updated release packaging to publish updater metadata and signatures.

## v0.1.0 - 2026-06-24

- First formal TunnelDesk release.
- Adjusted Linux x86 CI packaging to avoid unavailable Ubuntu 22.04 i386 AppIndicator packages; Linux x86 remains experimental and is not part of formal releases yet.
- Fixed macOS packaging by adding the required Tauri icon assets.
- Fixed macOS log directory opening by using the platform `open` command.
