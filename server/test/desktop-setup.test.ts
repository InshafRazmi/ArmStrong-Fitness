// Real temporary files/SQLite and local CLI subprocesses. Synthetic settings;
// no network, OS vault, live registration/enrollment or sync acceptance evidence.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { rootCertificates } from 'node:tls';
import { desktopConfig, localSetupEnvironment, provisionDesktopConfig, setupReadiness, validateDesktopConfig } from '../src/desktop-setup.ts';

const publicSettings = () => ({ SUPABASE_URL: 'https://fixture.supabase.co', PUBLIC_API_ORIGIN: 'https://api.example.invalid', SUPABASE_PUBLISHABLE_KEY: 'sb_publishable_fixture' });
const names = () => ({ REGISTRATION_GYM_ID: randomUUID(), REGISTRATION_GYM_NAME: 'Synthetic gym', REGISTRATION_ADMIN_NAME: 'Synthetic Administrator' });
function fixture(run: (f: { dir: string; path: string; target: string; database: DatabaseSync; env: Record<string, string> }) => void) {
  const dir = mkdtempSync(join(tmpdir(), 'armstrong-desktop-setup-')), path = join(dir, 'fixture.sqlite3');
  const database = new DatabaseSync(path);
  database.exec('CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL)');
  database.prepare('INSERT INTO metadata VALUES(?,?)').run('device_id', randomUUID());
  database.prepare('INSERT INTO metadata VALUES(?,?)').run('native_device_secret_sha256', 'a'.repeat(64));
  const env = { ...publicSettings(), ...names(), REGISTRATION_SQLITE_PATH: path, REGISTRATION_DEVICE_SECRET_SHA256: 'a'.repeat(64) };
  try { run({ dir, path, target: join(dir, 'desktop-auth.json'), database, env }); }
  finally { database.close(); rmSync(dir, { recursive: true, force: true }); }
}
function redacted(run: () => unknown, hidden: string[]) {
  assert.throws(run, error => { for (const value of hidden) assert.ok(!String(error).includes(value)); return true; });
}

test('desktop setup / config: exports exactly public fields and rejects privileged/malformed keys', () => {
  const env = { ...publicSettings(), DATABASE_URL: 'private-database-password', AUTH_CHECK_PASSWORD: 'private-login-password', REGISTRATION_DEVICE_SECRET_SHA256: 'a'.repeat(64), DEVICE_SECRET: 'private-device-secret' };
  assert.deepEqual(desktopConfig(env), { authOrigin: env.SUPABASE_URL, apiOrigin: env.PUBLIC_API_ORIGIN, publishableKey: env.SUPABASE_PUBLISHABLE_KEY });
  const legacy = (role: string) => `header.${Buffer.from(JSON.stringify({ role })).toString('base64url')}.signature`;
  assert.equal(desktopConfig({ ...env, SUPABASE_PUBLISHABLE_KEY: legacy('anon') }).publishableKey, legacy('anon'));
  for (const key of ['sb_secret_private', legacy('service_role'), 'sb_publishable_', 'sb_publishable_REPLACE', 'sb_publishable_private\nheader', 'sb_publishable_' + 'x'.repeat(8192), legacy('anon') + '.extra', 'header.' + Buffer.from('{"role":"anon"}').toString('base64') + '=.signature']) {
    redacted(() => desktopConfig({ ...env, SUPABASE_PUBLISHABLE_KEY: key }), [key]);
  }
  redacted(() => validateDesktopConfig({ ...desktopConfig(env), databasePassword: env.DATABASE_URL }), [env.DATABASE_URL]);
});

test('desktop setup / config: requires distinct exact HTTPS origins compatible with native loading', () => {
  const env = publicSettings();
  for (const value of ['http://api.example.invalid', 'https://private-user:private-password@api.example.invalid', env.PUBLIC_API_ORIGIN + '/', env.PUBLIC_API_ORIGIN + '/path', env.PUBLIC_API_ORIGIN + '?private-query', env.PUBLIC_API_ORIGIN + '#private-hash', ' https://api.example.invalid', 'https://API.example.invalid', 'https://api.example.invalid:443', 'https://REPLACE.example.invalid']) {
    for (const key of ['SUPABASE_URL', 'PUBLIC_API_ORIGIN']) redacted(() => desktopConfig({ ...env, [key]: value }), [value]);
  }
  assert.throws(() => desktopConfig({ ...env, PUBLIC_API_ORIGIN: env.SUPABASE_URL }), /separately/);
});

test('desktop setup / real files and SQLite: check is read-only, write is complete and retries preserve existing bytes', () => {
  fixture(f => {
    const before = readFileSync(f.path);
    assert.equal(provisionDesktopConfig(f.env), 'missing');
    assert.equal(existsSync(f.target), false);
    assert.deepEqual(readFileSync(f.path), before);
    assert.equal(provisionDesktopConfig(f.env, true), 'created');
    assert.deepEqual(JSON.parse(readFileSync(f.target, 'utf8')), desktopConfig(f.env));
    if (process.platform !== 'win32') assert.equal(statSync(f.target).mode & 0o777, 0o600);
    // Differently formatted but identical public settings remain untouched.
    writeFileSync(f.target, JSON.stringify({ publishableKey: f.env.SUPABASE_PUBLISHABLE_KEY, apiOrigin: f.env.PUBLIC_API_ORIGIN, authOrigin: f.env.SUPABASE_URL }));
    const bytes = readFileSync(f.target), mtime = statSync(f.target).mtimeMs;
    assert.equal(provisionDesktopConfig(f.env, true), 'existing');
    assert.equal(provisionDesktopConfig(f.env), 'existing');
    assert.deepEqual(readFileSync(f.target), bytes); assert.equal(statSync(f.target).mtimeMs, mtime);
    assert.deepEqual(readFileSync(f.path), before);
    assert.equal(readdirSync(f.dir).some(name => name.endsWith('.tmp')), false);
  });
});

test('desktop setup / real files: differing, invalid, oversized and privileged configurations are never replaced', () => {
  fixture(f => {
    for (const text of [JSON.stringify({ ...desktopConfig(f.env), apiOrigin: 'https://other.example.invalid' }), 'private-invalid-json', 'x'.repeat(16385), JSON.stringify({ ...desktopConfig(f.env), publishableKey: 'sb_secret_private' }), JSON.stringify({ ...desktopConfig(f.env), role: 'Administrator' }), '{"authOrigin":null,' + JSON.stringify(desktopConfig(f.env)).slice(1)]) {
      writeFileSync(f.target, text);
      redacted(() => provisionDesktopConfig(f.env, true), [f.path, f.target, 'sb_secret_private', 'private-invalid-json']);
      assert.equal(readFileSync(f.target, 'utf8'), text);
    }
    assert.equal(readdirSync(f.dir).some(name => name.endsWith('.tmp')), false);
  });
});

test('desktop setup / real files: directories and symlink targets fail without changing their referents', () => {
  fixture(f => {
    mkdirSync(f.target);
    assert.throws(() => provisionDesktopConfig(f.env, true));
    rmSync(f.target, { recursive: true });
    if (process.platform !== 'win32') {
      const referent = join(f.dir, 'private.json');
      writeFileSync(referent, JSON.stringify(desktopConfig(f.env)));
      const before = readFileSync(referent);
      symlinkSync(referent, f.target);
      assert.throws(() => provisionDesktopConfig(f.env, true), /regular file/);
      assert.deepEqual(readFileSync(referent), before);
    }
  });
});

test('desktop setup / real SQLite: missing preparation, changed hashes and restore block file creation', () => {
  fixture(f => {
    for (const patch of [{ REGISTRATION_DEVICE_SECRET_SHA256: 'b'.repeat(64) }, { REGISTRATION_DEVICE_SECRET_SHA256: undefined }, { REGISTRATION_SQLITE_PATH: undefined }]) {
      const before = readFileSync(f.path);
      assert.throws(() => provisionDesktopConfig({ ...f.env, ...patch }, true));
      assert.deepEqual(readFileSync(f.path), before); assert.equal(existsSync(f.target), false);
    }
    f.database.prepare('INSERT INTO metadata VALUES(?,?)').run('restore_requires_reconciliation', 'true');
    const before = readFileSync(f.path);
    assert.throws(() => provisionDesktopConfig(f.env, true), /reconciliation/);
    assert.deepEqual(readFileSync(f.path), before); assert.equal(existsSync(f.target), false);
  });
});

test('desktop setup / real SQLite: bound device/gym/API mismatches cannot provision a replacement config', () => {
  fixture(f => {
    const device = String(f.database.prepare("SELECT value FROM metadata WHERE key='device_id'").get()!.value);
    const scope = { serverOrigin: f.env.PUBLIC_API_ORIGIN, gymId: f.env.REGISTRATION_GYM_ID, deviceId: device };
    const put = (value: unknown) => f.database.prepare("INSERT INTO metadata VALUES('member_sync_scope',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value").run(JSON.stringify(value));
    for (const bad of [{ ...scope, serverOrigin: 'https://other.example.invalid' }, { ...scope, gymId: randomUUID() }, { ...scope, deviceId: randomUUID() }]) {
      put(bad); const before = readFileSync(f.path);
      assert.throws(() => provisionDesktopConfig(f.env, true), /reconciliation/);
      assert.deepEqual(readFileSync(f.path), before); assert.equal(existsSync(f.target), false);
    }
    put(scope); assert.equal(provisionDesktopConfig(f.env, true), 'created');
  });
});

test('desktop setup / real files: missing SQLite never creates a database or desktop settings', () => {
  fixture(f => {
    const missing = join(f.dir, 'missing.sqlite3');
    redacted(() => provisionDesktopConfig({ ...f.env, REGISTRATION_SQLITE_PATH: missing }, true), [missing]);
    assert.equal(existsSync(missing), false); assert.equal(existsSync(f.target), false);
  });
});

test('desktop setup / local env: refuses inherited target overrides and never reads test env as fallback', () => {
  fixture(f => {
    const path = join(f.dir, '.env');
    writeFileSync(path, Object.entries(f.env).map(([key, value]) => `${key}=${JSON.stringify(value)}`).join('\n'));
    assert.deepEqual({ ...localSetupEnvironment(f.env, path) }, f.env);
    for (const key of ['PUBLIC_API_ORIGIN', 'DATABASE_URL', 'REGISTRATION_GYM_ID', 'REGISTRATION_SQLITE_PATH', 'AUTH_CHECK_PASSWORD']) {
      redacted(() => localSetupEnvironment({ ...f.env, [key]: 'private-override-value' }, path), ['private-override-value', path]);
    }
    assert.throws(() => localSetupEnvironment({ ...f.env, NODE_TLS_REJECT_UNAUTHORIZED: '0' }, path), /TLS/);
    writeFileSync(join(f.dir, '.env.test'), 'DATABASE_URL=private-test-password');
    rmSync(path);
    redacted(() => localSetupEnvironment(f.env, path), [path, 'private-test-password']);
  });
});

test('desktop setup / readiness: reports all missing steps together without network, private values or writes', () => {
  const checks = setupReadiness({ DATABASE_URL: 'private-invalid-url', SUPABASE_URL: 'private-invalid-origin', SUPABASE_PUBLISHABLE_KEY: 'sb_secret_private' });
  assert.equal(checks.length, 6);
  assert.ok(checks.some(check => check.status === 'FAIL'));
  assert.ok(checks.some(check => check.status === 'PENDING'));
  for (const hidden of ['private-invalid-url', 'private-invalid-origin', 'sb_secret_private']) assert.ok(!JSON.stringify(checks).includes(hidden));
  fixture(f => {
    const before = readFileSync(f.path);
    const result = setupReadiness(f.env);
    assert.equal(result.find(check => check.step === 'Prepared desktop metadata')!.status, 'PASS');
    assert.equal(result.find(check => check.step === 'Desktop public configuration')!.status, 'PENDING');
    assert.equal(existsSync(f.target), false); assert.deepEqual(readFileSync(f.path), before);
  });
});

test('desktop setup / readiness: complete local configuration remains distinct from real-service acceptance', () => {
  fixture(f => {
    const cert = join(f.dir, 'public-ca.pem'); writeFileSync(cert, rootCertificates[0]);
    const env = { ...f.env, SUPABASE_URL: 'https://abcdefghijklmnopqrst.supabase.co', DATABASE_URL: `postgresql://postgres.abcdefghijklmnopqrst:synthetic-password@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres?sslmode=verify-full&sslrootcert=${encodeURIComponent(cert)}`, AUTH_CHECK_EMAIL: 'admin@example.invalid', AUTH_CHECK_PASSWORD: 'synthetic-password' };
    provisionDesktopConfig(env, true);
    const before = readFileSync(f.path);
    const checks = setupReadiness(env);
    assert.ok(checks.every(check => check.status === 'PASS'), JSON.stringify(checks));
    assert.ok(checks.find(check => check.step === 'Prepared desktop metadata')!.detail.includes('not verified'));
    assert.deepEqual(readFileSync(f.path), before);
  });
});

test('desktop setup / actual local CLI: review/write use only runtime settings and redact output', t => {
  fixture(f => {
    const envPath = join(f.dir, '.env');
    const settings: Record<string, string> = { ...f.env, DATABASE_URL: 'private-synthetic-database-password', AUTH_CHECK_PASSWORD: 'private-synthetic-login-password' };
    writeFileSync(envPath, Object.entries(settings).map(([key, value]) => `${key}=${JSON.stringify(value)}`).join('\n'));
    const script = resolve('scripts/provision-desktop-config.ts');
    const run = (mode: string) => spawnSync(process.execPath, ['--env-file=.env', script, mode], { cwd: f.dir, env: {}, encoding: 'utf8', timeout: 10000 });
    for (const [mode, state] of [['check', 'missing'], ['write', 'created'], ['write', 'existing']]) {
      const result = run(mode);
      if (result.error && 'code' in result.error && result.error.code === 'EPERM') { t.skip('Sandbox blocks node child-process execution; direct CLI acceptance must run separately'); return; }
      assert.equal(result.error, undefined);
      assert.equal(result.status, 0, result.stderr);
      assert.ok(result.stdout.includes(state), JSON.stringify({ mode, state, stdout: result.stdout, stderr: result.stderr }));
      for (const hidden of [...Object.values(settings), f.target]) assert.ok(!(result.stdout + result.stderr).includes(hidden));
    }
    const invalid = run('unexpected'); assert.equal(invalid.status, 2);
    assert.equal(JSON.parse(readFileSync(f.target, 'utf8')).publishableKey, settings.SUPABASE_PUBLISHABLE_KEY);
    const setup = spawnSync(process.execPath, ['--env-file=.env', resolve('scripts/check-setup.ts')], { cwd: f.dir, env: {}, encoding: 'utf8', timeout: 10000 });
    assert.equal(setup.status, 2); assert.ok(setup.stdout.includes('INCOMPLETE'));
    for (const hidden of [...Object.values(settings), f.target]) assert.ok(!(setup.stdout + setup.stderr).includes(hidden));
  });
});
