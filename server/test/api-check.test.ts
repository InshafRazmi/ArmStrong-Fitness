// HTTP mocks only. The Fastify injection contract executes installed route code;
// neither proves a real HTTPS connection, live backend/Auth/enrollment or sync.
import test from 'node:test';
import assert from 'node:assert/strict';
import { checkApiEndpoint, apiHealth } from '../src/api-check.ts';
import { createApp } from '../src/app.ts';

const origin = 'https://api.example.invalid';
test('API check / HTTP mock and real Fastify injection: sends no credentials and recognizes the member API', async () => {
  const forbidden = async (): Promise<never> => { throw new Error('Health check must not use identity, SQL or members'); };
  const app = createApp({ enroll: forbidden, push: forbidden, pull: forbidden }, forbidden);
  let calls = 0;
  try {
    await checkApiEndpoint(origin, async (url, options) => {
      calls++; assert.equal(String(url), origin + '/health');
      assert.equal(options?.method, 'GET'); assert.equal(options?.redirect, 'error');
      assert.ok(options?.signal instanceof AbortSignal);
      assert.deepEqual([...new Headers(options?.headers)], [['accept', 'application/json']]);
      assert.equal(options?.body, undefined);
      const response = await app.inject({ url: '/health' });
      assert.equal(response.headers['cache-control'], 'no-store');
      return new Response(response.body, { status: response.statusCode, headers: { 'content-type': 'application/json' } });
    });
    assert.equal(calls, 1);
  } finally { await app.close(); }
});

test('API check / HTTP mock: rejects redirects, failing status, another service and incompatible versions without body disclosure', async () => {
  for (const status of [301, 302, 401, 403, 404, 429, 500, 503]) {
    await assert.rejects(checkApiEndpoint(origin, async () => Response.json({ private: 'private-provider-body' }, { status })), error => {
      assert.ok(String(error).includes(`HTTP ${status}`)); assert.ok(!String(error).includes(origin) && !String(error).includes('private-provider-body')); return true;
    });
  }
  for (const payload of [null, [], { status: 'ok' }, { ...apiHealth, service: 'another-service' }, { ...apiHealth, protocolVersion: 2 }, { ...apiHealth, private: 'private-provider-body' }]) {
    await assert.rejects(checkApiEndpoint(origin, async () => Response.json(payload)), error => {
      assert.ok(String(error).includes('invalid or unsupported')); assert.ok(!String(error).includes('private-provider-body')); return true;
    });
  }
});

test('API check / HTTP mock: bounds successful response bytes and cancels malformed streams', async () => {
  for (const response of [new Response('private-invalid-json', { headers: { 'content-type': 'application/json' } }), new Response(JSON.stringify(apiHealth), { headers: { 'content-type': 'text/html' } }), new Response('x'.repeat(4097), { headers: { 'content-type': 'application/json' } }), new Response(Uint8Array.from([0xff, 0xfe]), { headers: { 'content-type': 'application/json' } })]) {
    await assert.rejects(checkApiEndpoint(origin, async () => response), /invalid or unsupported/);
  }
  let reads = 0, cancelled = false;
  const response = new Response(new ReadableStream({
    pull(controller) { reads++; controller.enqueue(new Uint8Array(2048)); },
    cancel() { cancelled = true; }
  }), { headers: { 'content-type': 'application/json' } });
  await assert.rejects(checkApiEndpoint(origin, async () => response), /invalid or unsupported/);
  assert.ok(reads <= 4); assert.equal(cancelled, true);
});

test('API check / HTTP mock: rejects insecure/noncanonical targets before I/O and redacts DNS/TLS/provider failures', async () => {
  let calls = 0;
  for (const value of [undefined, '', 'http://api.example.invalid', 'https://private-user:private-password@api.example.invalid', origin + '/', origin + '/path', origin + '?private-query', origin + '#private-hash', ' https://api.example.invalid', 'https://API.example.invalid', 'https://api.example.invalid:443', 'https://REPLACE.example.invalid']) {
    await assert.rejects(checkApiEndpoint(value, async () => { calls++; throw new Error(); }));
  }
  assert.equal(calls, 0);
  for (const code of ['EAI_AGAIN', 'CERT_HAS_EXPIRED', 'ERR_TLS_CERT_ALTNAME_INVALID', 'private-provider-message']) {
    await assert.rejects(checkApiEndpoint(origin, async () => { throw Object.assign(new Error('private-provider-message'), { cause: { code } }); }), error => {
      assert.ok(!String(error).includes(origin) && !String(error).includes('private-provider-message'));
      if (code !== 'private-provider-message') assert.ok(String(error).includes(code)); return true;
    });
  }
});
