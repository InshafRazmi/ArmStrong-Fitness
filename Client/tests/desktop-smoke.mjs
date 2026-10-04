import { spawnSync } from 'node:child_process'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', timeout: 1_200_000, ...options })
  if (result.error || result.status !== 0) throw new Error(`${command} failed: ${result.error ?? `exit ${result.status}, signal ${result.signal}`}`)
}
if (process.env.npm_execpath) run(process.execPath, [process.env.npm_execpath, 'run', 'build'])
else run('npm', ['run', 'build'], { shell: process.platform === 'win32' })
run('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--features', 'ui-smoke', '--locked'])
const fixture = mkdtempSync(join(tmpdir(), 'armstrong-ui-smoke-'))
const executable = resolve('src-tauri/target/debug', process.platform === 'win32' ? 'armstrong-desktop.exe' : 'armstrong-desktop')
try {
  const env = { ...process.env, ARMSTRONG_SMOKE_DIR: fixture, XDG_DATA_HOME: join(fixture, 'data'), XDG_CACHE_HOME: join(fixture, 'cache') }
  run(executable, [], { env, timeout: 45_000 })
  run(executable, [], { env, timeout: 45_000 })
  console.log('Desktop UI and process-restart checks passed.')
} finally {
  rmSync(fixture, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 })
}
