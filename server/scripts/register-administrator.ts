import pg from 'pg';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';
import { config } from '../src/config.ts';
import { checkPasswordAccount, AuthCheckError } from '../src/auth-check.ts';
import { authorizedDatabaseTls, databaseOptions, databaseFailure, sessionDatabaseUrl, DatabaseConfigurationError } from '../src/database.ts';
import { verifyRuntimeSchema, SchemaVerificationError } from '../src/schema-verification.ts';
import { registerAdministrator, RegistrationError } from '../src/registration.ts';
import { registrationConfig, registrationKeys } from '../src/registration-config.ts';

// Runtime .env only. Default/check mode is read-only; apply is a separate explicit
// administrative command. Tokens/IDs/config/secret hashes never reach output.
let client: InstanceType<typeof pg.Client> | undefined;
try {
  const mode = process.argv[2];
  if (process.argv.length !== 3 || !['check', 'apply'].includes(mode)) throw new RegistrationError('Use npm run registration:check or npm run registration:apply');
  let local: ReturnType<typeof parseEnv>;
  try { local = parseEnv(readFileSync('.env', 'utf8')); }
  catch { throw new RegistrationError('Cannot read local runtime .env; values withheld'); }
  const authKeys = ['DATABASE_URL', 'SUPABASE_URL', 'SUPABASE_PUBLISHABLE_KEY', 'AUTH_CHECK_EMAIL', 'AUTH_CHECK_PASSWORD'];
  const missing = authKeys.filter(key => !local[key] || local[key]!.includes('REPLACE'));
  if (missing.length) throw new RegistrationError(`Configure ${missing.join(', ')} locally; values withheld`);
  for (const key of [...authKeys, 'PUBLIC_API_ORIGIN', ...registrationKeys]) {
    if (local[key] !== process.env[key]) throw new RegistrationError(`${key} does not match local runtime .env; values withheld`);
  }
  const runtime = config(local);
  const target = sessionDatabaseUrl(runtime.databaseUrl);
  if (!decodeURIComponent(target.username).startsWith('postgres.')) throw new RegistrationError('Registration requires an administrative owner connection; do not widen API runtime grants');
  const requested = registrationConfig(local, runtime.apiOrigin);
  // Auth is completed before a SQL transaction/lock; authorization never comes
  // from a caller-supplied identity, role or decoded token metadata.
  const userId = await checkPasswordAccount(runtime.supabaseUrl, runtime.publishableKey, { email: local.AUTH_CHECK_EMAIL!, password: local.AUTH_CHECK_PASSWORD! });
  client = new pg.Client(databaseOptions(runtime.databaseUrl));
  await client.connect();
  if (!authorizedDatabaseTls(client.connection.stream)) throw new DatabaseConfigurationError('Registration requires authorized database TLS');
  if ((await client.query('SELECT 1 AS connected')).rows[0]?.connected !== 1) throw new RegistrationError('Registration database probe returned an unexpected result');
  await client.query('BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY');
  try { await verifyRuntimeSchema(client); await client.query('COMMIT'); }
  catch (error) { await client.query('ROLLBACK'); throw error; }
  const plan = await registerAdministrator(client, userId, requested, mode === 'apply');
  console.log('Runtime Auth identity, PostgreSQL/TLS and migration metadata: PASS (real services; values withheld)');
  for (const [label, state] of Object.entries(plan)) console.log(`${label} registration: ${state === 'create' && mode === 'apply' ? 'created' : state}`);
  console.log(mode === 'check' ? 'Registration review: PASS (read-only; no application changes made)' : 'Registration apply: PASS (new approved registrations only; matching records retained)');
  console.log('Desktop login, API enrollment and live sync: not verified; live sync remains disabled');
} catch (error) {
  console.error(error instanceof RegistrationError || error instanceof AuthCheckError || error instanceof SchemaVerificationError ? error.message : databaseFailure(error, 'Runtime registration'));
  console.error('Registration command: FAIL; live enrollment/sync not verified');
  process.exitCode = 2;
} finally {
  if (client) { try { await client.end(); } catch { console.error('Registration connection cleanup failed; details withheld'); process.exitCode = 2; } }
}
