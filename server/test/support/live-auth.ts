import { uuid } from '../../src/protocol.ts';
import { TestSetupError } from './supabase-config.ts';

// Real Supabase Auth HTTP; never a stub. Tokens remain in process memory only.
export async function signInTestAccount(authUrl: string, key: string, account: { email: string; password: string }) {
  const endpoint = new URL('/auth/v1/token?grant_type=password', authUrl);
  const response = await fetch(endpoint, {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(10000),
    headers: { apikey: key, 'content-type': 'application/json' },
    body: JSON.stringify(account)
  });
  if (!response.ok) throw new TestSetupError(`Live test-account sign-in refused (HTTP ${response.status}); provider payload withheld`);
  const payload: unknown = await response.json();
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) throw new TestSetupError('Invalid live test-account response; payload withheld');
  const data = payload as Record<string, unknown>;
  if (typeof data.access_token !== 'string' || !/^[A-Za-z0-9._~-]{1,8192}$/.test(data.access_token)) throw new TestSetupError('Invalid live test-account token response; payload withheld');
  const user = data.user;
  return { userId: uuid(user && typeof user === 'object' ? (user as Record<string, unknown>).id : undefined), accessToken: data.access_token };
}
