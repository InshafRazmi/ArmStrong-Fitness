import { Buffer } from 'node:buffer';

export const apiHealth = Object.freeze({ status: 'ok', service: 'armstrong-member-api', protocolVersion: 1 });
export class ApiCheckError extends Error {}
const bodyLimit = 4096;

function networkCode(error: unknown): string {
  const code = (error as { code?: unknown; cause?: { code?: unknown } })?.cause?.code ?? (error as { code?: unknown })?.code;
  return typeof code === 'string' && /^[A-Z0-9_]{3,32}$/.test(code) ? code : 'details_withheld';
}
// Read-only process/protocol probe. No keys, tokens, device secrets, member data
// or enrollment are sent. A PASS never certifies DB/Auth/device/sync acceptance.
export async function checkApiEndpoint(value: string | undefined, request: typeof fetch = fetch): Promise<void> {
  if (process.env.NODE_TLS_REJECT_UNAUTHORIZED === '0') throw new ApiCheckError('API TLS verification must not be disabled');
  let origin: URL;
  try { origin = new URL(value || ''); }
  catch { throw new ApiCheckError('Configure PUBLIC_API_ORIGIN as the approved canonical HTTPS origin; values withheld'); }
  if (!value || value.includes('REPLACE') || origin.protocol !== 'https:' || origin.username || origin.password || origin.pathname !== '/' || origin.search || origin.hash || origin.origin !== value) {
    throw new ApiCheckError('PUBLIC_API_ORIGIN must be a canonical HTTPS origin without credentials or paths; values withheld');
  }
  let response: Response;
  try {
    response = await request(new URL('/health', origin), {
      method: 'GET', redirect: 'error', signal: AbortSignal.timeout(10000),
      headers: { accept: 'application/json' }
    });
  } catch (error) { throw new ApiCheckError(`API endpoint unavailable (${networkCode(error)}); URL and provider details withheld`); }
  if (response.status !== 200) {
    try { await response.body?.cancel(); } catch {}
    throw new ApiCheckError(`API endpoint refused (HTTP ${response.status}); URL and provider body withheld`);
  }
  const reader = response.body?.getReader();
  try {
    if (!reader || !/^application\/json(?:\s*;|$)/i.test(response.headers.get('content-type') || '')) throw new Error();
    const chunks: Uint8Array[] = [];
    let size = 0;
    for (;;) {
      const chunk = await reader.read();
      if (chunk.done) break;
      size += chunk.value.byteLength;
      if (size > bodyLimit) throw new Error();
      chunks.push(chunk.value);
    }
    const bytes = Buffer.concat(chunks, size);
    const payload: unknown = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
    if (!payload || typeof payload !== 'object' || Array.isArray(payload)) throw new Error();
    const health = payload as Record<string, unknown>;
    if (Object.keys(health).length !== 3 || health.status !== apiHealth.status || health.service !== apiHealth.service || health.protocolVersion !== apiHealth.protocolVersion) throw new Error();
  } catch { throw new ApiCheckError('API endpoint returned an invalid or unsupported Armstrong health response; URL and body withheld'); }
  finally { try { await reader?.cancel(); } catch {} }
}
