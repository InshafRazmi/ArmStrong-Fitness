import { Buffer } from 'node:buffer';
import { randomUUID } from 'node:crypto';
import { closeSync, constants, fsyncSync, fstatSync, linkSync, lstatSync, openSync, readSync, realpathSync, unlinkSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { parseEnv } from 'node:util';
import { config } from './config.ts';
import { databaseOptions, DatabaseConfigurationError } from './database.ts';
import { registrationConfig, registrationKeys } from './registration-config.ts';
import { RegistrationError } from './registration.ts';
import { canonicalHttpsOrigin, publicAuthKey, PublicAuthConfigurationError } from './public-auth-config.ts';

export class DesktopSetupError extends PublicAuthConfigurationError {}
class PendingSetupState extends DesktopSetupError {}
type Environment = Record<string, string | undefined>;
export interface DesktopConfig { authOrigin: string; apiOrigin: string; publishableKey: string }
export interface SetupCheck { step: string; status: 'PASS' | 'PENDING' | 'FAIL'; detail: string }
const configLimit = 16 * 1024;
const publicKeys = ['SUPABASE_URL', 'PUBLIC_API_ORIGIN', 'SUPABASE_PUBLISHABLE_KEY'] as const;
const setupKeys = ['DATABASE_URL', ...publicKeys, 'NODE_ENV', 'HOST', 'PORT', 'AUTH_CHECK_EMAIL', 'AUTH_CHECK_PASSWORD', ...registrationKeys] as const;

function errorCode(error: unknown): unknown {
  return error && typeof error === 'object' && 'code' in error ? error.code : undefined;
}
// Bound reads and refuse symlinks/non-files. Paths and bytes never enter errors.
function readRegular(path: string, limit: number, missing = false): Buffer | undefined {
  let fd: number | undefined;
  try {
    const before = lstatSync(path);
    if (!before.isFile()) throw new DesktopSetupError('Configuration must be a bounded regular file; values withheld');
    fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
    const stat = fstatSync(fd);
    if (!stat.isFile() || stat.size > limit || before.dev !== stat.dev || before.ino !== stat.ino) throw new DesktopSetupError('Configuration must be a bounded regular file; values withheld');
    const bytes = Buffer.alloc(limit + 1);
    let size = 0;
    while (size <= limit) {
      const count = readSync(fd, bytes, size, limit + 1 - size, null);
      if (count === 0) break;
      size += count;
    }
    if (size > limit) throw new DesktopSetupError('Configuration exceeds its size limit; values withheld');
    return bytes.subarray(0, size);
  } catch (error) {
    if (missing && errorCode(error) === 'ENOENT') return undefined;
    if (error instanceof DesktopSetupError) throw error;
    throw new DesktopSetupError('Cannot safely read configuration; paths and values withheld');
  } finally { if (fd !== undefined) closeSync(fd); }
}

export function localSetupEnvironment(inherited: Environment, path = '.env'): Environment {
  let local: ReturnType<typeof parseEnv>;
  try { local = parseEnv(readRegular(path, 128 * 1024)!.toString('utf8')); }
  catch { throw new DesktopSetupError('Cannot read local runtime .env; paths and values withheld'); }
  for (const key of setupKeys) {
    if (local[key] !== inherited[key]) throw new DesktopSetupError(`${key} does not match local runtime .env; values withheld`);
  }
  if (inherited.NODE_TLS_REJECT_UNAUTHORIZED === '0') throw new DesktopSetupError('TLS verification must not be disabled');
  return local;
}

export function validateDesktopConfig(value: unknown): DesktopConfig {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new DesktopSetupError('Invalid desktop configuration; values withheld');
  const data = value as Record<string, unknown>;
  const keys = Object.keys(data);
  if (keys.length !== 3 || keys.some(key => !['authOrigin', 'apiOrigin', 'publishableKey'].includes(key))) throw new DesktopSetupError('Desktop configuration contains unexpected fields; values withheld');
  const authOrigin = canonicalHttpsOrigin(data.authOrigin), apiOrigin = canonicalHttpsOrigin(data.apiOrigin);
  if (authOrigin === apiOrigin) throw new DesktopSetupError('Configure the Armstrong API origin separately from Supabase Auth');
  const result = { authOrigin, apiOrigin, publishableKey: publicAuthKey(data.publishableKey) };
  if (Buffer.byteLength(JSON.stringify(result, null, 2) + '\n') > configLimit) throw new DesktopSetupError('Desktop configuration exceeds its size limit; values withheld');
  return result;
}
export function desktopConfig(env: Environment): DesktopConfig {
  return validateDesktopConfig({ authOrigin: env.SUPABASE_URL, apiOrigin: env.PUBLIC_API_ORIGIN, publishableKey: env.SUPABASE_PUBLISHABLE_KEY });
}

function desktopTarget(env: Environment, settings: DesktopConfig): string {
  const registered = registrationConfig(env, settings.apiOrigin);
  if (!registered.device || !env.REGISTRATION_SQLITE_PATH) throw new DesktopSetupError('Prepare the actual computer and configure both device registration settings first');
  try { return join(dirname(realpathSync(env.REGISTRATION_SQLITE_PATH)), 'desktop-auth.json'); }
  catch { throw new DesktopSetupError('Cannot locate the existing desktop database; paths and values withheld'); }
}
function existingConfig(target: string, expected: DesktopConfig): 'missing' | 'existing' {
  const bytes = readRegular(target, configLimit, true);
  if (!bytes) return 'missing';
  let parsed: unknown;
  try { parsed = JSON.parse(bytes.toString('utf8')); }
  catch { throw new DesktopSetupError('Existing desktop configuration is invalid; preserve it for review'); }
  // JSON.parse accepts duplicate fields; native Serde rejects them. Count the
  // separators outside strings so a duplicate cannot appear to match here.
  let quoted = false, separators = 0;
  const text = bytes.toString('utf8');
  for (let index = 0; index < text.length; index++) {
    if (quoted && text[index] === '\\') { index++; continue; }
    if (text[index] === '"') quoted = !quoted;
    else if (!quoted && text[index] === ':') separators++;
  }
  if (separators !== 3) throw new DesktopSetupError('Existing desktop configuration must contain each public field exactly once; no file was replaced');
  const current = validateDesktopConfig(parsed);
  if (JSON.stringify(current) !== JSON.stringify(expected)) throw new DesktopSetupError('Existing desktop configuration differs; no file was replaced');
  return 'existing';
}

// Separate local write command. No SQL writes, credentials, server registration,
// device generation, deployment or production sync activation takes place here.
export function provisionDesktopConfig(env: Environment, write = false): 'missing' | 'existing' | 'created' {
  const settings = desktopConfig(env), target = desktopTarget(env, settings);
  const state = existingConfig(target, settings);
  if (state === 'existing' || !write) return state;
  const temporary = join(dirname(target), `.desktop-auth-${randomUUID()}.tmp`);
  let fd: number | undefined;
  let ownsTemporary = false;
  try {
    fd = openSync(temporary, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL, 0o600);
    ownsTemporary = true;
    writeFileSync(fd, JSON.stringify(settings, null, 2) + '\n');
    fsyncSync(fd);
    closeSync(fd); fd = undefined;
    try {
      // Publish complete bytes atomically, with no replacement even if another
      // setup process created the target after our initial inspection.
      linkSync(temporary, target);
      return 'created';
    } catch (error) {
      if (errorCode(error) === 'EEXIST' && existingConfig(target, settings) === 'existing') return 'existing';
      throw error;
    }
  } catch (error) {
    if (error instanceof DesktopSetupError) throw error;
    throw new DesktopSetupError('Cannot safely create desktop configuration; no existing file was replaced');
  } finally {
    if (fd !== undefined) closeSync(fd);
    if (ownsTemporary) unlinkSync(temporary);
  }
}

export function setupReadiness(env: Environment): SetupCheck[] {
  const checks: SetupCheck[] = [];
  function check(step: string, keys: readonly string[], run: () => string) {
    const missing = keys.filter(key => !env[key] || env[key]!.includes('REPLACE'));
    if (missing.length) { checks.push({ step, status: 'PENDING', detail: `Configure ${missing.join(', ')} locally; values withheld` }); return; }
    try { checks.push({ step, status: 'PASS', detail: run() }); }
    catch (error) {
      checks.push({ step, status: error instanceof PendingSetupState ? 'PENDING' : 'FAIL', detail: error instanceof PublicAuthConfigurationError || error instanceof DatabaseConfigurationError || error instanceof RegistrationError ? error.message : 'Invalid local configuration; values withheld' });
    }
  }
  check('Desktop HTTPS/Auth settings', publicKeys, () => { desktopConfig(env); return 'Public settings validate locally; endpoints not contacted'; });
  check('Database TLS settings', ['DATABASE_URL', 'SUPABASE_URL', 'SUPABASE_PUBLISHABLE_KEY'], () => {
    const runtime = config(env); databaseOptions(runtime.databaseUrl); return 'Session-pooler URL and readable CA validate locally; no TLS handshake or query';
  });
  check('Approved gym/Administrator settings', registrationKeys.slice(0, 3), () => {
    registrationConfig({ ...env, REGISTRATION_SQLITE_PATH: undefined, REGISTRATION_DEVICE_SECRET_SHA256: undefined }); return 'Names and stable gym UUID validate locally; registration not verified';
  });
  check('Administrator probe credentials', ['AUTH_CHECK_EMAIL', 'AUTH_CHECK_PASSWORD'], () => {
    if (!env.AUTH_CHECK_EMAIL!.trim() || !env.AUTH_CHECK_PASSWORD) throw new DesktopSetupError('Configure Administrator probe credentials locally; values withheld');
    return 'Credentials present; online identity not verified';
  });
  check('Prepared desktop metadata', [...publicKeys, ...registrationKeys], () => {
    desktopTarget(env, desktopConfig(env)); return 'Read-only SQLite identity/hash/scope check; OS credential possession and server approval not verified';
  });
  check('Desktop public configuration', [...publicKeys, ...registrationKeys], () => {
    if (provisionDesktopConfig(env) === 'missing') throw new PendingSetupState('Desktop configuration is missing; run desktop:config:check then desktop:config:write after reviewing the approved settings');
    return 'Existing desktop configuration matches; no files or database rows changed';
  });
  return checks;
}
