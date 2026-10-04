import { Buffer } from 'node:buffer';

export class PublicAuthConfigurationError extends Error {}

// Same canonical HTTPS/public-key contract as native AuthConfig. This validates
// configuration syntax only; JWT role inspection cannot authorize a user.
export function canonicalHttpsOrigin(value: unknown): string {
  if (typeof value !== 'string' || !value || value.includes('REPLACE')) throw new PublicAuthConfigurationError('Configure a canonical HTTPS origin; values withheld');
  let url: URL;
  try { url = new URL(value); } catch { throw new PublicAuthConfigurationError('Configure a canonical HTTPS origin; values withheld'); }
  if (url.protocol !== 'https:' || url.username || url.password || url.pathname !== '/' || url.search || url.hash || url.origin !== value) {
    throw new PublicAuthConfigurationError('Configure a canonical HTTPS origin without credentials or paths; values withheld');
  }
  return value;
}
export function publicAuthKey(value: unknown): string {
  let valid = typeof value === 'string' && value.length > 0 && value.length <= 8192 && /^[A-Za-z0-9._~-]+$/.test(value) && !value.includes('REPLACE');
  if (valid && typeof value === 'string') {
    if (value.startsWith('sb_publishable_')) valid = value.length > 'sb_publishable_'.length;
    else {
      const parts = value.split('.');
      valid = parts.length === 3 && parts.every(Boolean) && /^[A-Za-z0-9_-]+$/.test(parts[1]);
      if (valid) {
        try {
          const payload = Buffer.from(parts[1], 'base64url');
          const data: unknown = JSON.parse(payload.toString('utf8'));
          valid = payload.toString('base64url') === parts[1] && !!data && typeof data === 'object' && 'role' in data && data.role === 'anon';
        } catch { valid = false; }
      }
    }
  }
  if (!valid || typeof value !== 'string') throw new PublicAuthConfigurationError('Auth requires a publishable or legacy anon key; values withheld');
  return value;
}
