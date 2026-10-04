import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';
import { config } from '../src/config.ts';
import { checkPasswordAccount, AuthCheckError } from '../src/auth-check.ts';
import { databaseFailure } from '../src/database.ts';

// Optional local login check only: .env, real Auth, no database or registration
// writes, no admin API key, account creation, device approval or native binding.
try {
  if (process.argv.length !== 2) throw new AuthCheckError('Admin Auth check accepts no arguments');
  let local: ReturnType<typeof parseEnv>;
  try { local = parseEnv(readFileSync('.env', 'utf8')); }
  catch { throw new AuthCheckError('Cannot read local runtime .env; values withheld'); }
  const keys = ['DATABASE_URL', 'SUPABASE_URL', 'SUPABASE_PUBLISHABLE_KEY', 'AUTH_CHECK_EMAIL', 'AUTH_CHECK_PASSWORD'];
  const missing = keys.filter(key => !local[key] || local[key]!.includes('REPLACE'));
  if (missing.length) throw new AuthCheckError(`Configure ${missing.join(', ')} locally for the Admin Auth check; values withheld`);
  for (const key of keys) if (local[key] !== process.env[key]) throw new AuthCheckError(`${key} does not match local runtime .env; values withheld`);
  const runtime = config();
  await checkPasswordAccount(runtime.supabaseUrl, runtime.publishableKey, { email: local.AUTH_CHECK_EMAIL!, password: local.AUTH_CHECK_PASSWORD! });
  console.log('Administrator account password sign-in and online identity verification: PASS (real Supabase Auth; values withheld)');
  console.log('Gym/role/device registration and desktop login: not checked; no application database changes made');
} catch (error) {
  console.error(error instanceof AuthCheckError ? error.message : databaseFailure(error, 'Administrator Auth check'));
  console.error('Verified live account sign-in and online identity: no');
  process.exitCode = 2;
}
