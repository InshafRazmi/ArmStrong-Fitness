import { DatabaseConfigurationError, sessionDatabaseUrl } from '../../src/database.ts';

// Test-only configuration. Never falls back to runtime DATABASE_URL/Auth settings.
export const testKeys = [
  'TEST_PROJECT_IS_DISPOSABLE', 'TEST_SUPABASE_PROJECT_REF', 'TEST_DATABASE_URL',
  'TEST_SUPABASE_URL', 'TEST_SUPABASE_PUBLISHABLE_KEY',
  'TEST_ADMIN_EMAIL', 'TEST_ADMIN_PASSWORD',
  'TEST_RECEPTION_EMAIL', 'TEST_RECEPTION_PASSWORD',
  'TEST_OTHER_GYM_EMAIL', 'TEST_OTHER_GYM_PASSWORD'
] as const;

export class TestSetupError extends Error {}
export function missingTestKeys(env = process.env) {
  return testKeys.filter(key => !env[key] || env[key]!.includes('REPLACE'));
}
function fail(message: string): never { throw new TestSetupError(message); }

export function supabaseTestConfig(env = process.env) {
  const missing = missingTestKeys(env);
  if (missing.length) fail(`Missing isolated Supabase test configuration: ${missing.join(', ')}`);
  if (env.TEST_PROJECT_IS_DISPOSABLE !== 'true') fail('TEST_PROJECT_IS_DISPOSABLE must be true for a dedicated test project');
  if (env.NODE_TLS_REJECT_UNAUTHORIZED === '0') fail('TLS verification must not be disabled for integration tests');
  const ref = env.TEST_SUPABASE_PROJECT_REF!;
  if (!/^[a-z0-9]{10,64}$/.test(ref)) fail('Invalid TEST_SUPABASE_PROJECT_REF');
  let database: URL, auth: URL;
  try { database = new URL(env.TEST_DATABASE_URL!); auth = new URL(env.TEST_SUPABASE_URL!); }
  catch { return fail('Invalid test connection URL; values withheld'); }
  if (auth.origin !== `https://${ref}.supabase.co` || auth.username || auth.password || auth.pathname !== '/' || auth.search || auth.hash) fail('Test Auth URL must be the canonical HTTPS origin of TEST_SUPABASE_PROJECT_REF');
  if (!['postgres:', 'postgresql:'].includes(database.protocol) || database.hash || database.pathname !== '/postgres' || database.port !== '5432' || !database.password) fail('Test database requires a password and Session pooler connection on port 5432');
  const queryKeys = [...database.searchParams.keys()];
  if (queryKeys.some(key => !['sslmode', 'sslrootcert'].includes(key)) || new Set(queryKeys).size !== queryKeys.length || database.searchParams.get('sslmode') !== 'verify-full') fail('Test database requires sslmode=verify-full; only sslmode and sslrootcert parameters are allowed');
  let user: string;
  try { user = decodeURIComponent(database.username); } catch { return fail('Invalid test database username; value withheld'); }
  const session = /^[a-z0-9-]+\.pooler\.supabase\.com$/.test(database.hostname) && user === `postgres.${ref}`;
  if (!session) fail('Test database requires the isolated project Session pooler and owner username');
  try { sessionDatabaseUrl(env.TEST_DATABASE_URL!, ref, true); }
  catch (error) { if (error instanceof DatabaseConfigurationError) fail(error.message); throw error; }
  const key = env.TEST_SUPABASE_PUBLISHABLE_KEY!;
  if (!key.startsWith('sb_publishable_')) {
    // This decoding only rejects a privileged key; it never authenticates a user.
    let role: unknown;
    try { role = JSON.parse(Buffer.from(key.split('.')[1], 'base64url').toString()).role; } catch {}
    if (role !== 'anon') fail('Use a Supabase publishable or legacy anon key, never a secret/service-role key');
  }
  const accounts = ['ADMIN', 'RECEPTION', 'OTHER_GYM'].map(name => ({
    email: env[`TEST_${name}_EMAIL`]!, password: env[`TEST_${name}_PASSWORD`]!
  }));
  if (new Set(accounts.map(account => account.email.toLowerCase())).size !== 3) fail('Provide three distinct confirmed test Auth accounts');
  if (accounts.some(account => !/^[^\s@]+@[^\s@]+$/.test(account.email))) fail('Invalid test account email; values withheld');
  return { databaseUrl: env.TEST_DATABASE_URL!, authUrl: auth.origin, publishableKey: key, accounts };
}

export function safeFailure(error: unknown, stage: string): Error {
  if (error instanceof TestSetupError || error instanceof DatabaseConfigurationError) return error;
  const code = (error as any)?.code ?? (error as any)?.cause?.code;
  const safe = typeof code === 'string' && /^[A-Z0-9_]{3,32}$/.test(code) ? code : 'details_withheld';
  return new Error(`${stage} failed (${safe}); credentials, tokens and driver/provider payloads withheld`);
}
