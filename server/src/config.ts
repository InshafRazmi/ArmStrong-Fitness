import { sessionDatabaseUrl } from './database.ts';
import { canonicalHttpsOrigin, publicAuthKey, PublicAuthConfigurationError } from './public-auth-config.ts';

export function config(env = process.env) {
  for (const key of ['DATABASE_URL','SUPABASE_URL','SUPABASE_PUBLISHABLE_KEY']) if (!env[key] || env[key]!.includes('REPLACE')) throw new Error(`Configure ${key}`);
  if (env.NODE_TLS_REJECT_UNAUTHORIZED === '0') throw new Error('TLS verification must not be disabled');
  const supabase = new URL(canonicalHttpsOrigin(env.SUPABASE_URL));
  const publishableKey = publicAuthKey(env.SUPABASE_PUBLISHABLE_KEY);
  const database = sessionDatabaseUrl(env.DATABASE_URL!, supabase.hostname.endsWith('.supabase.co') ? supabase.hostname.split('.')[0] : undefined);
  let apiOrigin: string | undefined;
  if (env.PUBLIC_API_ORIGIN) {
    apiOrigin = canonicalHttpsOrigin(env.PUBLIC_API_ORIGIN);
    if (apiOrigin === supabase.origin) throw new PublicAuthConfigurationError('Configure the Armstrong API origin separately from Supabase Auth');
  }
  if (env.NODE_ENV === 'production' && !apiOrigin) throw new Error('Configure PUBLIC_API_ORIGIN for production HTTPS');
  const port = Number(env.PORT || 3000);
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid PORT');
  return { databaseUrl: env.DATABASE_URL!, supabaseUrl: supabase.origin, publishableKey, apiOrigin, host: env.HOST || '127.0.0.1', port };
}

// API startup has an additional guard. Administrative registration/probes use
// config() and can retain their migration-owner connection in a controlled shell.
// A custom name alone does not certify its actual database attributes/grants.
export function runtimeConfig(env = process.env) {
  const settings = config(env);
  if (env.NODE_ENV === 'production') {
    const role = decodeURIComponent(sessionDatabaseUrl(settings.databaseUrl).username).split('.')[0];
    if (['postgres', 'anon', 'authenticated', 'authenticator', 'service_role', 'dashboard_user', 'pgbouncer'].includes(role) || /^(?:pg_|supabase_)/.test(role)) {
      throw new PublicAuthConfigurationError('Production API requires a separate restricted database runtime login; administrative and built-in roles are refused');
    }
  }
  return settings;
}
