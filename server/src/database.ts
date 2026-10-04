import { readFileSync } from 'node:fs';
import { X509Certificate } from 'node:crypto';
import { TLSSocket } from 'node:tls';
import { PublicAuthConfigurationError } from './public-auth-config.ts';
import { ServerConfigurationError } from './configuration-error.ts';

export class DatabaseConfigurationError extends Error {}
function fail(message: string): never { throw new DatabaseConfigurationError(message); }

// pg exposes a generic Duplex stream. Verify the actual native TLS socket before
// inspecting authorization; look-alike flags on a plaintext stream do not count.
export function authorizedDatabaseTls(stream: unknown): stream is TLSSocket {
  return stream instanceof TLSSocket && stream.encrypted === true && stream.authorized === true;
}

// Accept only the Session pooler and explicit, verified TLS. No URL overrides
// may replace the host, login, database, or SSL settings supplied here.
export function sessionDatabaseUrl(value: string, projectRef?: string, owner = false): URL {
  let url: URL;
  try { url = new URL(value); } catch { return fail('Invalid database URL; values withheld'); }
  if (!['postgres:', 'postgresql:'].includes(url.protocol) || url.hash || url.port !== '5432' || url.pathname !== '/postgres' || !url.password) fail('Database requires Session pooler port 5432, postgres database and credentials');
  let user: string;
  try { user = decodeURIComponent(url.username); decodeURIComponent(url.password); } catch { return fail('Invalid database credentials encoding; values withheld'); }
  if (!/^[a-z0-9-]+\.pooler\.supabase\.com$/.test(url.hostname) || !/^[a-zA-Z_][a-zA-Z0-9_]*\.[a-z0-9]{10,64}$/.test(user)) fail('Database requires a Supabase Session pooler host and role.project-ref username');
  if (projectRef && user.slice(user.lastIndexOf('.') + 1) !== projectRef) fail('Database username must match the configured Supabase project');
  if (owner && user !== `postgres.${projectRef}`) fail('Test database requires the isolated project migration owner');
  const keys = [...url.searchParams.keys()];
  if (keys.some(key => !['sslmode', 'sslrootcert'].includes(key)) || new Set(keys).size !== keys.length || url.searchParams.get('sslmode') !== 'verify-full' || !url.searchParams.get('sslrootcert')) fail('Database requires sslmode=verify-full and sslrootcert; no other or duplicate query parameters allowed');
  return url;
}

export function databaseOptions(value: string) {
  if (process.env.NODE_TLS_REJECT_UNAUTHORIZED === '0') fail('Database TLS verification must not be disabled');
  const url = sessionDatabaseUrl(value);
  let ca: string;
  try {
    ca = readFileSync(url.searchParams.get('sslrootcert')!, 'utf8');
    new X509Certificate(ca);
  } catch { return fail('Database sslrootcert must resolve to a readable PEM certificate; path and values withheld'); }
  // Deliberately omit connectionString: pg reparses SSL URI parameters and can
  // replace an explicit ssl object. Supply the validated fields and CA once.
  return {
    host: url.hostname, port: 5432, database: 'postgres',
    user: decodeURIComponent(url.username), password: decodeURIComponent(url.password),
    ssl: { ca, rejectUnauthorized: true },
    connectionTimeoutMillis: 10000, statement_timeout: 10000
  };
}

export function databaseFailure(error: unknown, stage: string): string {
  if (error instanceof DatabaseConfigurationError || error instanceof PublicAuthConfigurationError || error instanceof ServerConfigurationError) return error.message;
  const code = (error as { code?: unknown; cause?: { code?: unknown } })?.code ?? (error as { cause?: { code?: unknown } })?.cause?.code;
  const safe = typeof code === 'string' && /^[A-Z0-9_]{3,32}$/.test(code) ? code : 'details_withheld';
  return `${stage} failed (${safe}); connection values and driver details withheld`;
}
