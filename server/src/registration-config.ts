import { DatabaseSync } from 'node:sqlite';
import { uuid } from './protocol.ts';
import { RegistrationError, type Registration } from './registration.ts';

export const registrationKeys = ['REGISTRATION_GYM_ID', 'REGISTRATION_GYM_NAME', 'REGISTRATION_ADMIN_NAME', 'REGISTRATION_SQLITE_PATH', 'REGISTRATION_DEVICE_SECRET_SHA256'] as const;

// Registration config stays server-local. Optional device approval reads only
// metadata from an existing desktop database; never opens/creates a new database,
// performs SQLite migrations or accepts a replacement device UUID.
export function registrationConfig(env: Record<string, string | undefined>, apiOrigin?: string): Registration {
  const missing = registrationKeys.slice(0, 3).filter(key => !env[key] || env[key]!.includes('REPLACE'));
  if (missing.length) throw new RegistrationError(`Configure ${missing.join(', ')} locally; values withheld`);
  const result: Registration = { gymId: env.REGISTRATION_GYM_ID!, gymName: env.REGISTRATION_GYM_NAME!, adminName: env.REGISTRATION_ADMIN_NAME! };
  try { uuid(result.gymId); } catch { throw new RegistrationError('Invalid registration gym UUID; value withheld'); }
  for (const value of [result.gymName, result.adminName]) if (!value.trim() || [...value.trim()].length > 120) throw new RegistrationError('Registration names must contain 1–120 characters; values withheld');
  result.gymName = result.gymName.trim(); result.adminName = result.adminName.trim();
  const path = env.REGISTRATION_SQLITE_PATH, digest = env.REGISTRATION_DEVICE_SECRET_SHA256;
  if (!path && !digest) return result;
  if (!path || !digest || path.includes('REPLACE') || !/^[a-f0-9]{64}$/.test(digest)) throw new RegistrationError('Device approval requires REGISTRATION_SQLITE_PATH and REGISTRATION_DEVICE_SECRET_SHA256; values withheld');
  let database: DatabaseSync | undefined;
  try {
    database = new DatabaseSync(path, { readOnly: true });
    database.exec('BEGIN');
    const row = database.prepare("SELECT value FROM metadata WHERE key='device_id'").get();
    const deviceId = uuid(row?.value);
    const restored = database.prepare("SELECT value FROM metadata WHERE key='restore_requires_reconciliation'").get();
    if (restored) throw new RegistrationError('Restored desktop database requires reconciliation before device approval');
    const prepared = database.prepare("SELECT value FROM metadata WHERE key='native_device_secret_sha256'").get();
    if (!prepared || prepared.value !== digest) throw new RegistrationError('Device approval hash must match the credential verified by native device preparation; values withheld');
    const binding = database.prepare("SELECT value FROM metadata WHERE key='member_sync_scope'").get();
    if (binding) {
      const scope: unknown = JSON.parse(String(binding.value));
      if (!scope || typeof scope !== 'object' || Array.isArray(scope)) throw new RegistrationError('Invalid saved desktop scope; keep database for recovery');
      const saved = scope as Record<string, unknown>;
      if (Object.keys(saved).length !== 3 || Object.keys(saved).some(key => !['serverOrigin', 'gymId', 'deviceId'].includes(key))) throw new RegistrationError('Invalid saved desktop scope; keep database for recovery');
      if (saved.gymId !== result.gymId || saved.deviceId !== deviceId || !apiOrigin || saved.serverOrigin !== apiOrigin) throw new RegistrationError('Saved desktop scope differs from approved runtime scope; reconciliation required');
    }
    database.exec('COMMIT');
    result.device = { id: deviceId, secretSha256: digest };
    return result;
  } catch (error) {
    if (error instanceof RegistrationError) throw error;
    throw new RegistrationError('Cannot validate existing desktop identity; paths, values and SQLite details withheld');
  } finally { database?.close(); }
}
