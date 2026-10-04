import { lstatSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const directory = fileURLToPath(new URL('../', import.meta.url));
const settingsPath = new URL('../desktop-auth.production.json', import.meta.url);
function checkSettings() {
  const stat = lstatSync(settingsPath);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size > 16 * 1024) throw new Error('Packaged public settings must be a bounded regular file');
  const text = readFileSync(settingsPath, 'utf8');
  const settings = JSON.parse(text);
  if (!settings || Array.isArray(settings) || Object.keys(settings).sort().join(',') !== 'apiOrigin,authOrigin,publishableKey') throw new Error('Packaged settings must contain exactly the three public fields');
  // Reject duplicate JSON fields just as native Serde does. Ignore colons in
  // quoted values so HTTPS origins cannot affect this structural check.
  let quoted = false, separators = 0;
  for (let index = 0; index < text.length; index++) {
    if (quoted && text[index] === '\\') { index++; continue; }
    if (text[index] === '"') quoted = !quoted;
    else if (!quoted && text[index] === ':') separators++;
  }
  if (separators !== 3) throw new Error('Packaged public fields must each occur once');
  for (const origin of [settings.authOrigin, settings.apiOrigin]) {
    if (typeof origin !== 'string' || origin.includes('REPLACE')) throw new Error('Configure the approved public HTTPS origins before building');
    const parsed = new URL(origin);
    if (parsed.protocol !== 'https:' || parsed.username || parsed.password || parsed.pathname !== '/' || parsed.search || parsed.hash || parsed.origin !== origin) throw new Error('Packaged settings require canonical HTTPS origins');
  }
  if (settings.authOrigin === settings.apiOrigin) throw new Error('The API and Auth origins must be separate');
  const key = settings.publishableKey;
  if (typeof key !== 'string' || key.includes('REPLACE') || key.length > 8192 || !/^[A-Za-z0-9._~-]+$/.test(key)) throw new Error('Packaged Auth key is invalid; values withheld');
  let publicKey = key.startsWith('sb_publishable_') && key.length > 'sb_publishable_'.length;
  if (!publicKey) {
    const parts = key.split('.');
    try { publicKey = parts.length === 3 && parts.every(Boolean) && JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8')).role === 'anon'; }
    catch { publicKey = false; }
  }
  if (!publicKey) throw new Error('Packaged Auth requires a publishable or legacy anon key; privileged credentials are refused');
}

try {
  if (process.argv.length > 3 || (process.argv[2] && process.argv[2] !== '--check')) throw new Error('Use this command without arguments or with --check');
  checkSettings();
  console.log('Windows public configuration: PASS (three public fields; credentials withheld)');
  if (process.argv[2] !== '--check') {
    if (process.platform !== 'win32' || process.arch !== 'x64') throw new Error('Build the Windows x64 installer on a Windows x64 machine or the Windows GitHub Actions runner');
    const probe = spawnSync('cargo', ['tauri', '--version'], { cwd: directory, encoding: 'utf8' });
    if (probe.status !== 0 || !/tauri-cli 2\.12\.1\b/.test(probe.stdout ?? '')) throw new Error('Install the pinned builder first: cargo install tauri-cli --version 2.12.1 --locked');
    const result = spawnSync('cargo', ['tauri', 'build', '--ci', '--target', 'x86_64-pc-windows-msvc', '--features', 'desktop,custom-protocol,packaged-auth', '--bundles', 'nsis', '--', '--locked'], { cwd: directory, stdio: 'inherit' });
    if (result.error || result.status !== 0) throw new Error('Windows installer build failed; inspect the build output');
    console.log('Windows installer built in src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/');
  }
} catch (error) {
  // Parse/filesystem errors can contain file bytes or paths. Only our fixed
  // validation/build messages are intended for output.
  const safe = error instanceof Error && !('code' in error) && !(error instanceof SyntaxError) && !(error instanceof TypeError);
  console.error(safe ? error.message : 'Windows build preparation failed; values withheld');
  process.exitCode = 1;
}
