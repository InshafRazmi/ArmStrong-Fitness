import { sessionDatabaseUrl } from './database.ts';
import { canonicalHttpsOrigin, publicAuthKey, PublicAuthConfigurationError } from './public-auth-config.ts';
import { ServerConfigurationError } from './configuration-error.ts';

export function config(env = process.env) {
  for (const key of ['DATABASE_URL','SUPABASE_URL','SUPABASE_PUBLISHABLE_KEY']) if (!env[key] || env[key]!.includes('REPLACE')) throw new ServerConfigurationError(`Configure ${key}; environment value is missing or still a placeholder`);
  if (env.NODE_TLS_REJECT_UNAUTHORIZED === '0') throw new ServerConfigurationError('TLS verification must not be disabled');
  const supabase = new URL(canonicalHttpsOrigin(env.SUPABASE_URL));
  const publishableKey = publicAuthKey(env.SUPABASE_PUBLISHABLE_KEY);
  const database = sessionDatabaseUrl(env.DATABASE_URL!, supabase.hostname.endsWith('.supabase.co') ? supabase.hostname.split('.')[0] : undefined);
  let apiOrigin: string | undefined;
  // Render provides the actual public URL before the first process starts.
  // Explicit configuration takes precedence, including a custom domain.
  const configuredOrigin = env.PUBLIC_API_ORIGIN || (env.RENDER === 'true' ? env.RENDER_EXTERNAL_URL : undefined);
  if (configuredOrigin) {
    apiOrigin = canonicalHttpsOrigin(configuredOrigin);
    if (!env.PUBLIC_API_ORIGIN && (!new URL(apiOrigin).hostname.endsWith('.onrender.com') || new URL(apiOrigin).port)) {
      throw new PublicAuthConfigurationError('Render API origin must be a canonical HTTPS onrender.com origin; configure PUBLIC_API_ORIGIN for a custom domain');
    }
    if (apiOrigin === supabase.origin) throw new PublicAuthConfigurationError('Configure the Armstrong API origin separately from Supabase Auth');
  }
  if (env.NODE_ENV === 'production' && !apiOrigin) throw new ServerConfigurationError('Configure PUBLIC_API_ORIGIN for production HTTPS');
  const port = Number(env.PORT || 3000);
  if (env.AUTOMATIC_DEVICE_ENROLLMENT !== undefined && !['true','false'].includes(env.AUTOMATIC_DEVICE_ENROLLMENT)) throw new ServerConfigurationError('AUTOMATIC_DEVICE_ENROLLMENT must be true or false');
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new ServerConfigurationError('Invalid PORT');
  return { databaseUrl: env.DATABASE_URL!, supabaseUrl: supabase.origin, publishableKey, apiOrigin, automaticEnrollment: env.AUTOMATIC_DEVICE_ENROLLMENT === 'true', host: env.HOST || '127.0.0.1', port };
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
