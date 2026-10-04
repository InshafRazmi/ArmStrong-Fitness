// Actual Fastify injection and Supabase verifier code, with MOCK registry SQL and
// identity HTTP. These tests are NOT PostgreSQL, live Auth or desktop sync proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID, createHash } from 'node:crypto';
import { createApp } from '../src/app.ts';
import { createVerifier } from '../src/auth.ts';
import { MemberService } from '../src/service.ts';

function fixture(options: { absent?: boolean; ambiguous?: boolean; revokedUnderLock?: boolean; canWrite?: boolean; role?: string; providerStatus?: number } = {}) {
  const gym = randomUUID(), user = randomUUID(), device = randomUUID();
  const secret = 'a'.repeat(64), digest = createHash('sha256').update(secret).digest('hex');
  const queries: { sql: string; values: any[] }[] = [];
  let releases = 0;
  const registry = {
    async query(sql: string, values: any[] = []) {
      queries.push({ sql, values });
      const result = (rows: any[]) => ({ rows, rowCount: rows.length });
      if (['BEGIN', 'COMMIT', 'ROLLBACK'].includes(sql)) return result([]);
      if (sql.startsWith('SELECT s.gym_id')) {
        assert.deepEqual(values, [user, device]);
        assert.ok(sql.includes('s.active') && sql.includes('d.active'));
        const rows = options.absent ? [] : [{ gym_id: gym, secret_sha256: digest }];
        if (options.ambiguous) rows.push({ gym_id: randomUUID(), secret_sha256: digest });
        return result(rows);
      }
      assert.equal(values[0], gym, 'SQL access must use the gym derived from the registry');
      if (sql.includes('FOR UPDATE')) return result([{ id: gym }]);
      if (sql.startsWith('SELECT role')) return result(options.revokedUnderLock ? [] : [{ role: options.role || 'Administrator' }]);
      if (sql.startsWith('SELECT secret_sha256')) return result([{ secret_sha256: digest, can_write: options.canWrite ?? true }]);
      if (sql.startsWith('SELECT name')) return result([{ name: 'Test gym' }]);
      if (sql.startsWith('SELECT display_name')) return result([{ display_name: 'Test staff' }]);
      if (sql.startsWith('SELECT can_write')) return result([{ can_write: options.canWrite ?? true }]);
      throw new Error('Unexpected mock SQL');
    },
    release() { releases++; }
  };
  const verify = createVerifier('https://test.supabase.co', 'test-publishable', async (_url, init) => {
    const ok = (init!.headers as Record<string, string>).authorization === 'Bearer test-token';
    return new Response(JSON.stringify({ id: user }), { status: options.providerStatus ?? (ok ? 200 : 401) });
  });
  const app = createApp(new MemberService({ async connect() { return registry; } }), verify);
  const payload = { protocolVersion: 1, deviceId: device, deviceSecret: secret };
  const headers = { authorization: 'Bearer test-token' };
  const enroll = (body: unknown = payload, h = headers) => app.inject({ method: 'POST', url: '/v1/enrollment', headers: { 'content-type': 'application/json', ...h }, payload: JSON.stringify(body) });
  return { app, enroll, payload, queries, gym, user, device, secret, releases: () => releases };
}

test('Fastify enrollment / mock registry: derives staff/gym/device and repeats without writes or secret disclosure', async () => {
  const f = fixture();
  try {
    const first = await f.enroll(), retry = await f.enroll();
    assert.equal(first.statusCode, 200);
    assert.deepEqual(first.json(), { protocolVersion: 1, gym: { id: f.gym, name: 'Test gym' }, staff: { id: f.user, name: 'Test staff', role: 'Administrator' }, device: { id: f.device, canWrite: true } });
    assert.deepEqual(retry.json(), first.json());
    assert.equal(first.headers['cache-control'], 'no-store');
    assert.ok(!first.body.includes(f.secret) && !first.body.includes('test-token'));
    assert.ok(f.queries.every(q => /^(SELECT|BEGIN|COMMIT)$/.test(q.sql.split(' ')[0])));
    assert.equal(f.queries.filter(q => q.sql === 'COMMIT').length, 2);
    assert.equal(f.releases(), 2);
  } finally { await f.app.close(); }
});

test('Fastify enrollment / mock registry: no account, gym or role supplied by the client can grant access', async () => {
  const f = fixture();
  try {
    for (const extra of [{ gymId: randomUUID() }, { userId: f.user }, { role: 'Administrator' }, { canWrite: true }]) {
      assert.equal((await f.enroll({ ...f.payload, ...extra })).statusCode, 400);
    }
    for (const bad of [{ ...f.payload, deviceId: 'bad' }, { ...f.payload, protocolVersion: 2 }, { ...f.payload, deviceSecret: 'short' }, {}]) {
      assert.equal((await f.enroll(bad)).statusCode, 400);
    }
    assert.equal(f.queries.length, 0);
  } finally { await f.app.close(); }
});

test('Fastify enrollment / mock registry: unauthenticated, invalid and unavailable identity deny before SQL', async () => {
  const f = fixture();
  try {
    assert.equal((await f.enroll(f.payload, {} as any)).statusCode, 401);
    assert.equal((await f.enroll(f.payload, { authorization: 'Bearer forged' })).statusCode, 401);
    assert.equal(f.queries.length, 0);
  } finally { await f.app.close(); }
  const down = fixture({ providerStatus: 503 });
  try {
    assert.equal((await down.enroll()).statusCode, 503);
    assert.equal(down.queries.length, 0);
  } finally { await down.app.close(); }
});

test('Fastify enrollment / mock registry: missing membership, wrong secret and ambiguous registration roll back', async () => {
  for (const options of [{ absent: true }, { ambiguous: true }, {}]) {
    const f = fixture(options);
    try {
      const payload = Object.keys(options).length ? f.payload : { ...f.payload, deviceSecret: 'b'.repeat(64) };
      assert.equal((await f.enroll(payload)).statusCode, 403);
      assert.equal(f.queries.at(-1)!.sql, 'ROLLBACK');
      assert.equal(f.releases(), 1);
      assert.ok(!f.queries.some(q => q.sql === 'COMMIT'));
    } finally { await f.app.close(); }
  }
});

test('Fastify enrollment / mock registry: revocation during gym lock prevents enrollment', async () => {
  const f = fixture({ revokedUnderLock: true });
  try {
    assert.equal((await f.enroll()).statusCode, 403);
    assert.ok(f.queries.some(q => q.sql.includes('FOR UPDATE')));
    assert.equal(f.queries.at(-1)!.sql, 'ROLLBACK');
  } finally { await f.app.close(); }
});

test('Fastify enrollment / mock registry: Reception and read-only device do not gain writer or Administrator privileges', async () => {
  const f = fixture({ role: 'Reception', canWrite: false });
  try {
    const result = await f.enroll();
    assert.equal(result.statusCode, 200);
    assert.equal(result.json().staff.role, 'Reception');
    assert.equal(result.json().device.canWrite, false);
  } finally { await f.app.close(); }
});

test('Fastify route / service mock: invalid JSON, oversized bodies and internal errors stay bounded and sanitized', async () => {
  const service = {
    async enroll() { throw new Error('postgres-password secret-token member-PII'); },
    async push() { throw new Error('postgres-password'); },
    async pull() { throw new Error('postgres-password'); }
  };
  const app = createApp(service, async () => randomUUID());
  try {
    const response = await app.inject({ method: 'POST', url: '/v1/enrollment', payload: {} });
    assert.equal(response.statusCode, 503);
    assert.deepEqual(response.json(), { error: 'service_unavailable' });
    assert.equal(response.headers['cache-control'], 'no-store');
    assert.equal((await app.inject({ method: 'POST', url: '/v1/enrollment', headers: { 'content-type': 'application/json' }, payload: '{' })).statusCode, 400);
    assert.equal((await app.inject({ method: 'POST', url: '/v1/enrollment', payload: { text: 'a'.repeat(40000) } })).statusCode, 413);
    assert.deepEqual((await app.inject({ url: '/health' })).json(), { status: 'ok', service: 'armstrong-member-api', protocolVersion: 1 });
  } finally { await app.close(); }
});
