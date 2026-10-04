// Synthetic credentials and HTTP mocks only; NOT live Auth or desktop login proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { Buffer } from 'node:buffer';
import { checkPasswordAccount } from '../src/auth-check.ts';

const origin = 'https://project.supabase.co';
const key = 'sb_publishable_fixture';
const account = { email: 'admin@example.invalid', password: 'fixture-account-password' };
test('single-Admin Auth check / HTTP mock: signs in then verifies identity online without returning tokens or trusting roles', async () => {
  const id = randomUUID();
  const calls: string[] = [];
  const verified = await checkPasswordAccount(origin, key, account, async (url, options) => {
    calls.push(String(url));
    assert.equal(options?.redirect, 'error');
    const headers = new Headers(options?.headers);
    assert.equal(headers.get('apikey'), key);
    if (calls.length === 1) {
      assert.equal(options?.method, 'POST'); assert.deepEqual(JSON.parse(String(options?.body)), account);
      return Response.json({ access_token: 'fixture.token', user: { id, user_metadata: { role: 'Administrator' } } });
    }
    assert.equal(headers.get('authorization'), 'Bearer fixture.token');
    return Response.json({ id });
  });
  assert.equal(verified, id);
  assert.deepEqual(calls, [origin + '/auth/v1/token?grant_type=password', origin + '/auth/v1/user']);
});
test('single-Admin Auth check / HTTP mock: rejects invalid credentials/payloads and mismatched online identities with redacted errors', async () => {
  for (const status of [400, 401, 403, 429, 500]) {
    let calls = 0;
    await assert.rejects(checkPasswordAccount(origin, key, account, async () => {
      calls++; return Response.json({ error: account.password }, { status });
    }), error => {
      assert.ok(String(error).includes(`HTTP ${status}`)); assert.ok(!String(error).includes(account.password)); return true;
    });
    assert.equal(calls, 1);
  }
  for (const payload of [null, [], { access_token: 'private token', user: { id: randomUUID() } }, { access_token: 'fixture.token', user: { id: 'invalid' } }]) {
    await assert.rejects(checkPasswordAccount(origin, key, account, async () => Response.json(payload)), /Invalid account sign-in response/);
  }
  let rejectedCalls = 0;
  await assert.rejects(checkPasswordAccount(origin, key, account, async () => {
    return rejectedCalls++ === 0 ? Response.json({ access_token: 'fixture.token', user: { id: randomUUID() } })
      : Response.json({ error: account.password }, { status: 401 });
  }), error => {
    assert.ok(String(error).includes('invalid_session')); assert.ok(!String(error).includes(account.password)); return true;
  });
  assert.equal(rejectedCalls, 2);
  let calls = 0;
  await assert.rejects(checkPasswordAccount(origin, key, account, async () => {
    return calls++ === 0 ? Response.json({ access_token: 'fixture.token', user: { id: randomUUID() } }) : Response.json({ id: randomUUID() });
  }), /identity mismatch/);
  await assert.rejects(checkPasswordAccount(origin, key, account, async () => { throw Object.assign(new Error(account.password), { cause: { code: 'EAI_AGAIN' } }); }), error => {
    assert.ok(String(error).includes('EAI_AGAIN')); assert.ok(!String(error).includes(account.password)); return true;
  });
});
test('single-Admin Auth check / HTTP mock: rejects plaintext/overridden origins and privileged keys before sending credentials', async () => {
  const secretRole = 'header.' + Buffer.from(JSON.stringify({ role: 'service_role' })).toString('base64url') + '.signature';
  let calls = 0;
  const request: typeof fetch = async () => { calls++; throw new Error('Unexpected network'); };
  for (const url of ['http://project.supabase.co', 'https://user:fixture-password@project.supabase.co', origin + '/path', origin + '?override=1', 'malformed']) {
    await assert.rejects(checkPasswordAccount(url, key, account, request));
  }
  for (const badKey of ['sb_secret_fixture', secretRole, 'invalid-key']) await assert.rejects(checkPasswordAccount(origin, badKey, account, request), /publishable or legacy anon key/);
  assert.equal(calls, 0);
});
