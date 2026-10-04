import { createVerifier } from './auth.ts';
import { ApiError, uuid } from './protocol.ts';
import { canonicalHttpsOrigin, publicAuthKey, PublicAuthConfigurationError } from './public-auth-config.ts';

export class AuthCheckError extends Error {}

function networkCode(error: unknown): string {
  const value = (error as { code?: unknown; cause?: { code?: unknown } })?.cause?.code ?? (error as { code?: unknown })?.code;
  return typeof value === 'string' && /^[A-Z0-9_]{3,32}$/.test(value) ? value : 'details_withheld';
}

// A password login and separate online verification, without database writes or
// role assignment. The selected account becomes Administrator only after approval
// and a server-side registration. Tokens stay in memory and are never returned.
export async function checkPasswordAccount(authUrl: string, key: string, account: { email: string; password: string }, request: typeof fetch = fetch): Promise<string> {
  let origin: URL;
  try { origin = new URL(canonicalHttpsOrigin(authUrl)); publicAuthKey(key); }
  catch (error) { throw new AuthCheckError(error instanceof PublicAuthConfigurationError ? error.message : 'Invalid Auth configuration; values withheld'); }
  if (process.env.NODE_TLS_REJECT_UNAUTHORIZED === '0') throw new AuthCheckError('Auth TLS verification must not be disabled');
  if (!account.email?.trim() || !account.password) throw new AuthCheckError('Auth check requires account credentials; values withheld');
  let response: Response;
  try {
    response = await request(new URL('/auth/v1/token?grant_type=password', origin), {
      method: 'POST', redirect: 'error', signal: AbortSignal.timeout(10000),
      headers: { apikey: key, 'content-type': 'application/json' },
      body: JSON.stringify({ email: account.email, password: account.password })
    });
  } catch (error) { throw new AuthCheckError(`Account sign-in unavailable (${networkCode(error)}); values and provider details withheld`); }
  if (!response.ok) throw new AuthCheckError(`Account sign-in refused (HTTP ${response.status}); values and provider payload withheld`);
  let token: string;
  let userId: string;
  try {
    const payload: unknown = await response.json();
    if (!payload || typeof payload !== 'object') throw new Error();
    const data = payload as Record<string, unknown>;
    if (typeof data.access_token !== 'string' || !/^[A-Za-z0-9._~-]{1,8192}$/.test(data.access_token)) throw new Error();
    token = data.access_token;
    const user = data.user;
    userId = uuid(user && typeof user === 'object' ? (user as Record<string, unknown>).id : undefined);
  } catch { throw new AuthCheckError('Invalid account sign-in response; values and payload withheld'); }
  let verifiedId: string;
  try { verifiedId = await createVerifier(origin.origin, key, request)(`Bearer ${token}`); }
  catch (error) {
    const code = error instanceof ApiError && ['invalid_session', 'identity_unavailable', 'invalid_identity_response', 'authentication_required'].includes(error.code) ? error.code : 'details_withheld';
    throw new AuthCheckError(`Online account verification failed (${code}); values and provider details withheld`);
  }
  if (verifiedId !== userId) throw new AuthCheckError('Sign-in and online-verified identity mismatch; values withheld');
  return verifiedId;
}
