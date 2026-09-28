# Contributing

## Local Checks

```powershell
pnpm typecheck
pnpm build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

## Engineering Rules

- Keep all privileged work in Rust commands.
- Do not expose generic shell or filesystem access to the frontend.
- Do not log passwords, private key passphrases, or raw private key material.
