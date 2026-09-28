# TunnelDesk

TunnelDesk 是一个面向 Windows/Linux/macOS 的桌面 SSH 隧道管理工具，主要服务于本地开发场景。它让本地应用继续使用真实的服务域名和端口，同时由 TunnelDesk 把选中的域名映射到本机回环地址，并通过一个或多个 SSH 跳板机转发流量。

English documentation: [README.md](README.md).

## 项目概览

开发团队经常需要通过跳板机访问数据库、缓存、内部 API 或其他私有网络服务。常见做法要么是为每个服务手动写 SSH 命令，要么是启用全局 VPN/代理，但后者经常会影响整台机器的网络环境。

TunnelDesk 采用更窄、更可控的方式：

1. 配置一个或多个 SSH 隧道。
2. 为应用正在使用的服务域名和端口创建 Profile。
3. 每个服务选择自己要走的隧道。
4. 启动 Profile 后，TunnelDesk 写入受控的 hosts 标记块，并启动所需的本地监听。
5. 应用仍然连接原来的服务域名，不需要改应用配置。

TunnelDesk 不启用 TUN 模式，也不会修改 Clash、Mihomo、系统代理、浏览器代理、WinHTTP 或 VPN 设置。

## 功能特性

- 多隧道配置：同一个 Profile 里的不同服务可以走不同 SSH 跳板机。
- Profile 化服务映射：可以按项目、环境或团队工作流新建、重命名、删除、切换、导入和导出服务组。
- 服务分组与排序：可以按分组组织服务，并在每个 Profile 内保持稳定展示和启动顺序。
- 受控 hosts 管理：只写入 `# BEGIN TUNNELDESK` 标记块内的内容。
- 本地回环监听：服务域名解析到类似 `127.77.0.10` 的地址，然后由 TunnelDesk 通过 SSH `direct-tcpip` 转发。
- 隧道级密码存储：密码存入系统凭据库，不写入 JSON 配置文件。
- 主题模式：支持 `System`、`Light`、`Dark`，默认使用 `System`。
- 启动行为：
  - `launchAtLogin`：用户登录系统后自动打开 TunnelDesk。
  - `autoStartProfile`：TunnelDesk 启动后自动启动当前 Profile。
- 后端校验隧道 id、服务 id、端口、回环地址、重复监听地址和缺失的隧道引用。
- CI 覆盖前端类型检查、前端生产构建、Rust 格式检查、Clippy 和 Rust 测试。
- CI 打包覆盖 Windows x64/ARM64、Linux x64/x86 与 macOS x64/ARM64。

## 平台支持

| 平台          | 开发支持 | CI 产物                                     | 说明                                                                   |
| ------------- | -------- | ------------------------------------------- | ---------------------------------------------------------------------- |
| Windows x64   | 支持     | 便携 `.zip` + NSIS 安装包                   | 主要 Windows 目标                                                      |
| Windows ARM64 | CI 支持  | 便携 `.zip` + NSIS 安装包                   | ARM64 目标                                                             |
| Linux x64     | 支持     | 便携 `.tar.gz` + `.deb` + `.rpm` + AppImage | 主要 Linux 目标                                                        |
| Linux x86     | 实验性   | 二进制文件 + `.deb`                         | Ubuntu 22.04 不提供 i386 AppIndicator 开发包，因此该目标不包含托盘图标 |
| macOS x64     | CI 支持  | 便携 `.tar.gz` + 未签名 `.dmg`              | Intel 目标                                                             |
| macOS ARM64   | CI 支持  | 便携 `.tar.gz` + 未签名 `.dmg`              | Apple Silicon 目标                                                     |

修改 hosts 文件需要提升权限：

- Windows：`C:\Windows\System32\drivers\etc\hosts`
- Linux：`/etc/hosts`
- macOS：`/etc/hosts`

普通配置编辑不需要管理员权限。Debian/Ubuntu 上推荐安装 `.deb` 包：主程序以普通用户运行，启动、停止或修复 hosts 映射时，通过 polkit 弹出系统授权，只让最小权限 helper 修改 `/etc/hosts`。

### Ubuntu 使用方式

推荐使用 `.deb` 安装包。它会安装：

- `/usr/lib/tunneldesk/tunneldesk-hosts-helper`
- `/usr/share/polkit-1/actions/com.tunneldesk.hosts.policy`

安装后从应用图标直接打开 TunnelDesk 即可。点击“启动”或“修复 hosts”时，系统会弹出授权窗口；授权成功后才会写入 `# BEGIN TUNNELDESK` 标记块。不要长期用 `sudo TunnelDesk` 启动 GUI，否则配置和凭据会进入 `/root/.local/share/TunnelDesk`。

AppImage 仍可作为便携版本使用，但不会自动安装系统级 polkit helper。需要修改 hosts 时，请优先使用原生 `.deb` 或 `.rpm` 包。

### RPM 系 Linux 使用方式

在 Fedora、RHEL、openSUSE 等 RPM 系发行版上使用 `.rpm` 包。它会安装同样的 helper 和 polkit 策略：

- `/usr/lib/tunneldesk/tunneldesk-hosts-helper`
- `/usr/share/polkit-1/actions/com.tunneldesk.hosts.policy`

安装后以普通用户运行 TunnelDesk，修改 hosts 时通过系统 polkit 授权弹窗完成授权。

## 工作方式

```text
Application -> real.service.domain:port
            -> hosts maps domain to 127.77.x.x
            -> TunnelDesk listens on 127.77.x.x:port
            -> SSH direct-tcpip through the selected jump host
            -> real.service.domain:port from the jump host network
```

一个 Profile 可以包含多个启用的服务。每个服务都有自己的 `tunnelId`。启动当前 Profile 时，TunnelDesk 会按 `tunnelId` 对启用服务分组，为涉及到的每个隧道启动一个 SSH runtime，并为对应服务创建本地监听。

同一时间只建议运行一个 Profile。这样可以保持 hosts 标记块确定可控，也避免本地监听端口所有权冲突。

### Windows 套接字错误 10013

在某些 Windows 机器上，启动 Profile 可能会在 SSH 转发开始前失败，并显示 `os error 10013` / `WSAEACCES`。此时 TunnelDesk 正在绑定配置里的本地监听，例如 `127.77.0.10:3306`，但 Windows 拒绝了这个套接字。

最常见原因是该 TCP 端口落在 Windows 排除/保留端口范围内。Hyper-V、WSL、Docker、VPN、Internet Connection Sharing 或终端安全软件都可能创建这些范围。可以在管理员 PowerShell 中检查受影响端口：

```powershell
netsh interface ipv4 show excludedportrange protocol=tcp
netstat -ano | findstr :3306
```

如果端口位于排除范围内，仅以管理员身份运行 TunnelDesk 通常也不能绕过。需要把服务/应用端口换到排除范围之外，或调整保留该端口范围的 Windows 组件或安全软件。

## 配置文件

运行时数据保存在当前用户的数据目录：

- Windows：`%APPDATA%\TunnelDesk`
- Linux：`$XDG_DATA_HOME/TunnelDesk` 或 `~/.local/share/TunnelDesk`
- macOS：`~/Library/Application Support/TunnelDesk`

该目录下会创建这些文件和目录：

- `settings.json`：当前 Profile、当前隧道、隧道元数据、SSH 端点元数据、主题模式和非敏感行为设置。
- `profiles.json`：服务 Profile、服务分组和顺序、回环地址映射、服务选择的隧道 id 和启用状态。
- `logs\`：应用滚动日志。
- `backups\`：应用直接写 hosts 时创建的 hosts 备份，以及导入 Profile 前自动创建的本地 `profiles.json` 备份。Linux polkit helper 写 hosts 时，备份位于 `/var/lib/tunneldesk/backups`。

日志默认使用 `info` 过滤级别，但仍会记录 `warn` 和 `error` 事件。需要更详细排查时，可以用 `RUST_LOG=tunneldesk=debug,info` 启动 TunnelDesk。

示例服务 Profile 位于 [examples/service-profiles.example.json](examples/service-profiles.example.json)。真实团队 Profile 建议通过 UI 导入或使用团队内部渠道分发，不要把生产域名直接提交到仓库。

服务页支持新建、重命名、删除、切换、导入和导出 Profile。Profile 运行中会禁用切换和会修改 Profile 集合的操作，避免 hosts 标记块与当前监听状态和所选配置脱节。

服务 Profile 的导入/导出只包含 `profiles.json` 数据：Profile 名称、服务映射、回环地址、服务选择的 `tunnelId` 和启用状态。它不会包含 SSH 隧道元数据、应用行为设置、私钥路径或密码值。导入团队 Profile 后，每台机器仍需要提前配置好对应 `tunnelId` 的本地隧道。

## 安全模型

TunnelDesk 采用和 Xshell、SecureCRT 等桌面 SSH 管理工具相似的高层设计：连接/会话元数据作为普通配置保存，密码等敏感值交给操作系统凭据库保存。

关键边界如下：

- SSH 密码不会写入 `settings.json`、`profiles.json`、日志或导出的 Profile 元数据。
- 前端可以保存、删除或检查某个隧道是否已有密码，但不能读取已保存的密码值。
- 前端不能选择任意凭据 key。
- Rust 后端使用稳定规则派生隧道密码 key：

```text
TunnelDesk:tunnel:{tunnelId}:password
```

- 已保存的密码值由平台凭据库按当前操作系统用户边界保护。
- 删除隧道密码时，会删除该隧道 id 对应的凭据库条目。
- hosts 写入只发生在 TunnelDesk 自己的标记块内。

这种模型可以降低 JSON 文件、UI 状态、日志、截图和 Profile 导出导致的误泄露风险。但它不能替代操作系统账号安全、磁盘加密、备份保护、恶意软件防护和工作站访问控制。

当前认证限制：

- 已实现密码认证。
- 私钥认证和 ssh-agent 认证目前作为 UI/后端扩展点保留。
- 尚未实现 SSH host key 固定；当前 MVP 会接受服务端 key。面向生产或更严格环境时，应保存并校验 host key 指纹，并在变化时提示用户。

## 启动行为

TunnelDesk 有两个互相独立的启动相关设置：

- `launchAtLogin`：把应用注册到系统登录启动项，用户登录后打开 TunnelDesk。
- `autoStartProfile`：应用启动后自动启动当前选中的 Profile。

Windows 上，`launchAtLogin` 会写入当前用户的 `Run` 注册表项。Linux 上，它会写入 `~/.config/autostart/TunnelDesk.desktop`。macOS 上，它会写入用户级 LaunchAgent：`~/Library/LaunchAgents/com.tunneldesk.app.plist`。

如果希望登录系统后自动打开应用，并立即启动当前开发 Profile，需要同时开启这两个设置。

## 开发

Windows 开发前置依赖：

- Node.js 20+
- pnpm 9+
- Rust stable
- Visual Studio Build Tools，并安装 “Desktop development with C++” 工作负载
- WebView2 Runtime

安装依赖并以开发模式运行：

```powershell
pnpm install
pnpm tauri:dev
```

前端检查：

```powershell
pnpm typecheck
pnpm build
```

Rust 检查：

```powershell
cd src-tauri
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

构建本地 Tauri 安装包：

```powershell
pnpm tauri:build
```

Linux 构建 `.deb`、`.rpm` 和 AppImage：

```bash
pnpm tauri:build:linux
```

## CI 与打包

[.github/workflows/ci.yml](.github/workflows/ci.yml) 中的 GitHub Actions 工作流包含：

- 前端任务，运行在 `windows-latest`：
  - `pnpm install --frozen-lockfile`
  - `pnpm typecheck`
  - `pnpm build`
- Rust 任务，运行在 `windows-latest`：
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- 打包任务，在 `main` 分支 push 和手动触发时运行：
  - Windows x64：`x86_64-pc-windows-msvc`，便携可执行文件 `.zip` 和 NSIS 安装包作为独立产物。
  - Windows ARM64：`aarch64-pc-windows-msvc`，便携可执行文件 `.zip` 和 NSIS 安装包作为独立产物。
  - Linux x64：`x86_64-unknown-linux-gnu`，便携二进制 `.tar.gz`、`.deb`、`.rpm` 和 AppImage。
  - Linux x86：`i686-unknown-linux-gnu`，release 二进制文件 + `.deb`，标记为实验性且不包含托盘图标。
  - macOS x64：`x86_64-apple-darwin`，便携二进制 `.tar.gz` 和未签名 `.dmg`。
  - macOS ARM64：`aarch64-apple-darwin`，便携二进制 `.tar.gz` 和未签名 `.dmg`。
- Release workflow 会在推送 `v*` tag 时只发布便携压缩包和安装包。Linux x86 仍保留为 CI 实验目标，稳定前不进入正式发布。

打包流程参考了 Tauri 2 的 GitHub Actions 分发方式：[Tauri GitHub pipelines](https://v2.tauri.app/distribute/pipelines/github)。

## 数据模型

当前配置使用 schema version 2：

- `AppSettings.tunnels`：全部隧道配置。
- `AppSettings.currentTunnelId`：隧道管理界面中当前选中的隧道。
- `AppSettings.behavior.themeMode`：`system`、`light` 或 `dark`。
- `ServiceConfig.tunnelId`：服务使用的隧道。

旧版单 SSH 配置会迁移为默认隧道。旧版服务 Profile 会迁移为引用默认隧道。

## 已知限制与后续方向

- 私钥认证和 ssh-agent 认证尚未实现。
- 更严格的生产环境应增加 SSH host key 固定。
- 同一时间只支持一个运行中的 Profile。
- Linux x86 打包在 CI 中标记为实验性，并禁用托盘图标以避开 Ubuntu 22.04 不可用的 i386 AppIndicator 包。
- macOS 安装包目前未签名、未公证。
- 核心本地模型稳定后，可以继续增强导入、导出和团队共享工作流。

## 更多文档

- [架构说明](docs/architecture.md)
- [配置说明](docs/configuration.md)
- [服务 Profile 示例](examples/service-profiles.example.json)
