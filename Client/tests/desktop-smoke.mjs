import { spawnSync } from 'node:child_process'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', timeout: 1_200_000, ...options })
  if (result.error || result.status !== 0) throw new Error(`${command} failed: ${result.error ?? `exit ${result.status}, signal ${result.signal}`}`)
  return result
}
function runDesktop(executable, env) {
  // Some desktop runtimes return zero even after app.exit(1). Require the
  // explicit result from the webview, including on the second process launch.
  const result = spawnSync(executable, [], { env, timeout: 60_000, stdio: 'pipe', encoding: 'utf8' })
  process.stdout.write(result.stdout ?? '')
  process.stderr.write(result.stderr ?? '')
  if (result.error || result.status !== 0) throw new Error(`Desktop process failed: ${result.error ?? `exit ${result.status}, signal ${result.signal}`}`)
  const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`
  if (output.includes('ARMSTRONG_UI_SMOKE FAILED:') || !/^ARMSTRONG_UI_SMOKE PASSED\r?$/m.test(output)) {
    throw new Error('Desktop webview did not confirm successful smoke checks')
  }
}
if (process.env.npm_execpath) run(process.execPath, [process.env.npm_execpath, 'run', 'build'])
else run('npm', ['run', 'build'], { shell: process.platform === 'win32' })
run('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--features', 'ui-smoke', '--locked'])
const fixture = mkdtempSync(join(tmpdir(), 'armstrong-ui-smoke-'))
const executable = resolve('src-tauri/target/debug', process.platform === 'win32' ? 'armstrong-desktop.exe' : 'armstrong-desktop')
try {
  const env = { ...process.env, ARMSTRONG_SMOKE_DIR: fixture, XDG_DATA_HOME: join(fixture, 'data'), XDG_CACHE_HOME: join(fixture, 'cache') }
  runDesktop(executable, env)
  runDesktop(executable, env)
  console.log('Desktop UI and process-restart checks passed.')
} finally {
  rmSync(fixture, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 })
}
