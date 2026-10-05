// Existing isolated test project only. Real PostgreSQL/Fastify; Auth is mocked.
// Every synthetic fixture is rolled back. No production fallback or schema reset.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash, randomBytes, randomUUID } from 'node:crypto';
import pg from 'pg';
import { databaseOptions, sessionDatabaseUrl } from '../src/database.ts';
import { GymService } from '../src/business-service.ts';
import { createApp } from '../src/app.ts';
import { ApiError } from '../src/protocol.ts';
import { parseBatch, applyChanges, stateFrom, digest } from '../src/business-protocol.ts';
import { safeFailure, TestSetupError } from './support/supabase-config.ts';
import type { LightMyRequestResponse } from 'fastify';
import { createVerifier } from '../src/auth.ts';
import { signInTestAccount } from './support/live-auth.ts';
import { supabaseTestConfig } from './support/supabase-config.ts';

const liveAuth = process.env.ARMSTRONG_BUSINESS_LIVE_AUTH === 'true';
test(`${liveAuth ? 'LIVE Supabase PostgreSQL + Auth' : 'REAL PostgreSQL / MOCK Auth'}: native all-module batches, exact retries, immutable history, cross-gym access, rollback and read-only download`, async () => {
  if (!process.env.TEST_DATABASE_URL || process.env.TEST_SUPABASE_PROJECT_REF !== 'srwjyvimdktrvzpdxvpd' || !process.env.ARMSTRONG_BUSINESS_FIXTURE_PATH) throw new TestSetupError('Configure the existing isolated test connection and generated native business fixture; no production fallback');
  sessionDatabaseUrl(process.env.TEST_DATABASE_URL, process.env.TEST_SUPABASE_PROJECT_REF, true);
  let entries = JSON.parse(await readFile(process.env.ARMSTRONG_BUSINESS_FIXTURE_PATH, 'utf8')) as any[];
  let bearer = 'fixture-token';
  let verify: (header: unknown) => Promise<string> = async header => { if (header !== 'Bearer fixture-token') throw new ApiError(401, 'authentication_required'); return entries.find(e => e.request.actorSubject)?.request.actorSubject; };
  if (liveAuth) {
    const c = supabaseTestConfig();
    const identity = await signInTestAccount(c.authUrl, c.publishableKey, c.accounts[0]);
    verify = createVerifier(c.authUrl, c.publishableKey);
    assert.equal(await verify(`Bearer ${identity.accessToken}`), identity.userId);
    const original = entries.find(e => e.request.actorSubject)?.request.actorSubject;
    // Synthetic native actors are mapped to this actual verified isolated account.
    entries = JSON.parse(JSON.stringify(entries).replaceAll(original, identity.userId));
    for (const entry of entries) entry.receipt.requestSha256 = digest(entry.request);
    bearer = identity.accessToken;
  }
  const pool = new pg.Pool(databaseOptions(process.env.TEST_DATABASE_URL));
  let db: pg.PoolClient | undefined;
  let app: ReturnType<typeof createApp> | undefined;
  let phase = 'connection and fixture setup';
  try {
    db = await pool.connect();
    await db.query('BEGIN');
    assert.equal((await db.query("SELECT count(*)::int AS n FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='armstrong' AND c.relkind='r' AND c.relname IN ('business_records','business_references','business_operations','business_changes') AND c.relrowsecurity")).rows[0].n, 4);
    const gym = randomUUID(), user = entries.find(e => e.request.actorSubject)?.request.actorSubject, device = entries[0].request.deviceId, reader = randomUUID(), secret = randomBytes(32).toString('hex');
    assert.ok(user);
    if (liveAuth) assert.equal((await db.query('SELECT count(*)::int AS n FROM auth.users WHERE id=$1 AND email_confirmed_at IS NOT NULL', [user])).rows[0].n, 1);
    await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Synthetic sync fixture')", [gym]);
    await db.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Synthetic Administrator','Administrator')", [gym, user]);
    await db.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true),($1,$4,$3,false)', [gym, device, createHash('sha256').update(secret).digest('hex'), reader]);
    const fixturePool = { connect: async () => ({
      query: (sql: string, values?: unknown[]) => db!.query(sql === 'BEGIN' ? 'SAVEPOINT business_api_test' : sql === 'COMMIT' ? 'RELEASE SAVEPOINT business_api_test' : sql === 'ROLLBACK' ? 'ROLLBACK TO SAVEPOINT business_api_test' : sql, values),
      release: () => {},
    }) };
    app = createApp(new GymService(fixturePool), verify);
    const headers = { authorization: `Bearer ${bearer}`, 'x-gym-id': gym, 'x-device-id': device, 'x-device-secret': secret };
    const push = (request: Record<string, unknown>, h = headers): Promise<LightMyRequestResponse> => app!.inject({ method: 'POST', url: '/v2/business/push', headers: h, payload: request });
    const expected = stateFrom([]);
    for (const entry of entries) {
      phase = `native transaction ${entry.sequence}`;
      const batch = parseBatch(entry.request);
      applyChanges(expected, batch.changes);
      const first = await push(batch); assert.equal(first.statusCode, 200, first.body);
      assert.equal(first.json().requestSha256, digest(batch));
      assert.deepEqual((await push(batch)).json(), first.json(), 'lost-response retry must reuse the original committed receipt');
    }
    phase = 'persisted rows and receipt counts';
    const persisted = (await db.query('SELECT table_name,record_id,data FROM armstrong.business_records WHERE gym_id=$1', [gym])).rows;
    assert.equal(persisted.length, [...expected.values()].reduce((n, rows) => n + rows.size, 0));
    for (const r of persisted) assert.deepEqual(r.data, expected.get(r.table_name)!.get(r.record_id));
    assert.equal((await db.query('SELECT count(*)::int AS n FROM armstrong.business_operations WHERE gym_id=$1', [gym])).rows[0].n, entries.length);
    phase = 'operation ID reuse';
    const changed = structuredClone(entries[0].request); changed.changes.find((c: any) => c.table === 'plans').after.name += ' changed';
    assert.equal((await push(changed)).statusCode, 409);
    phase = 'gym isolation and read-only permissions';
    const foreignGym = randomUUID(); await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Unrelated fixture')", [foreignGym]);
    assert.equal((await push(entries[0].request, { ...headers, 'x-gym-id': foreignGym })).statusCode, 403);
    assert.equal((await push(entries[0].request, { ...headers, 'x-device-secret': '0'.repeat(64) })).statusCode, 403);
    const readerHeaders = { ...headers, 'x-device-id': reader };
    const readerAttempt = { ...entries[0].request, operationId: randomUUID(), deviceId: reader };
    assert.equal((await push(readerAttempt, readerHeaders)).statusCode, 403);
    let cursor = 0, downloaded = 0;
    phase = 'ordered read-only download';
    for (;;) {
      const page: LightMyRequestResponse = await app.inject({ url: `/v2/business/changes?after=${cursor}`, headers: readerHeaders });
      assert.equal(page.statusCode, 200); const body = page.json();
      for (const e of body.changes) { assert.equal(e.receipt.requestSha256, digest(e.request)); downloaded++; }
      cursor = body.nextCursor; if (!body.hasMore) break;
    }
    assert.equal(downloaded, entries.length);
    assert.equal((await app.inject({ url: `/v2/business/changes?after=${cursor + 1}`, headers: readerHeaders })).statusCode, 409);
    // Reject an orphan under the same transaction lock without a receipt/change.
    phase = 'orphan rollback and sequence';
    const orphan = structuredClone(entries.find(e => e.request.changes.some((c: any) => c.table === 'membership_periods')).request);
    orphan.operationId = randomUUID(); orphan.changes = orphan.changes.filter((c: any) => c.table === 'membership_periods');
    orphan.changes[0].after.id = randomUUID(); orphan.changes[0].id = orphan.changes[0].after.id; orphan.changes[0].after.member_id = randomUUID();
    assert.equal((await push(orphan)).statusCode, 409);
    assert.equal((await db.query('SELECT business_sequence FROM armstrong.gyms WHERE id=$1', [gym])).rows[0].business_sequence, String(entries.length));
    phase = 'reversed payment cannot receive a later allocation';
    const savedAllocation = expected.get('payment_allocations')!.values().next().value!;
    const allocation = { ...savedAllocation, id: randomUUID(), amount_minor: 1 };
    const lateAllocation = { protocolVersion: 2, operationId: randomUUID(), deviceId: device, actorSubject: user, operationIds: [], changes: [{ table: 'payment_allocations', id: allocation.id, before: null, after: allocation }] };
    const lateReply = await push(lateAllocation);
    assert.equal(lateReply.statusCode, 409);
    assert.equal(lateReply.json().error, 'payment_already_reversed');
    assert.equal((await db.query('SELECT business_sequence FROM armstrong.gyms WHERE id=$1', [gym])).rows[0].business_sequence, String(entries.length));
    await db.query('SAVEPOINT immutable_test');
    phase = 'immutable financial history';
    await assert.rejects(() => db!.query("UPDATE armstrong.business_records SET data=jsonb_set(data,'{amount_minor}','1') WHERE gym_id=$1 AND table_name='payments'", [gym]));
    await db.query('ROLLBACK TO SAVEPOINT immutable_test');
    phase = 'master version and immutable fields';
    const oldPlan = expected.get('plans')!.values().next().value!;
    const edit = { protocolVersion: 2, operationId: randomUUID(), deviceId: device, actorSubject: user, operationIds: [], changes: [{ table: 'plans', id: oldPlan.id, before: oldPlan, after: { ...oldPlan, name: 'Updated plan', version: 2 } }] };
    assert.equal((await push(edit)).statusCode, 200);
    assert.equal((await push({ ...edit, operationId: randomUUID() })).statusCode, 409);
    await db.query('SAVEPOINT immutable_master_test');
    await assert.rejects(() => db!.query("UPDATE armstrong.business_records SET data=jsonb_set(data,'{joined_on}','\"2026-10-06\"') WHERE gym_id=$1 AND table_name='members'", [gym]));
    await db.query('ROLLBACK TO SAVEPOINT immutable_master_test');
    await db.query('UPDATE armstrong.devices SET active=false WHERE gym_id=$1 AND id=$2', [gym, device]);
    phase = 'revoked exact retry';
    assert.equal((await push(entries[0].request)).statusCode, 403, 'exact retries must still recheck revocation');
  } catch (error) { throw safeFailure(error, `Business PostgreSQL integration (${phase})`); }
  finally { if (app) await app.close(); if (db) { await db.query('ROLLBACK'); db.release(); } await pool.end(); }
});
