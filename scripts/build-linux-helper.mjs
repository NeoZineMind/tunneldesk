import { spawnSync } from 'node:child_process'
import process from 'node:process'

if (process.platform !== 'linux') {
  process.exit(0)
}

const result = spawnSync(
  'cargo',
  ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--bin', 'tunneldesk-hosts-helper', '--release'],
  { stdio: 'inherit' },
)

process.exit(result.status ?? 1)
