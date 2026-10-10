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
    const changed = structuredClone(entries.find(e => e.request.changes.some((c: any) => c.table === 'plans')).request); changed.changes.find((c: any) => c.table === 'plans').after.name += ' changed';
    assert.equal((await push(changed)).statusCode, 409);
    phase = 'gym isolation and read-only permissions';
    const foreignGym = randomUUID(); await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Unrelated fixture')", [foreignGym]);
    assert.equal((await push(entries[0].request, { ...headers, 'x-gym-id': foreignGym })).statusCode, 403);
    assert.equal((await push(entries[0].request, { ...headers, 'x-device-secret': '0'.repeat(64) })).statusCode, 403);
    const readerHeaders = { ...headers, 'x-device-id': reader };
    const readerAttempt = structuredClone(entries.find(e => e.request.changes.some((c: any) => c.table === 'plans')).request);
    readerAttempt.operationId = randomUUID(); readerAttempt.deviceId = reader;
    readerAttempt.changes.find((c: any) => c.table === 'plans').after.name += ' stale second-device baseline';
    const permittedReader = await push(readerAttempt, readerHeaders);
    assert.equal(permittedReader.statusCode, 409, 'Administrator on the second device passes authorization but stale rows still conflict');
    assert.equal(permittedReader.json().error, 'business_revision_conflict');
    await db.query("UPDATE armstrong.staff SET role='Reception' WHERE gym_id=$1 AND user_id=$2", [gym,user]);
    assert.equal((await push(readerAttempt, readerHeaders)).statusCode, 403, 'a read-only Reception device does not gain Administrator editing');
    await db.query("UPDATE armstrong.staff SET role='Administrator' WHERE gym_id=$1 AND user_id=$2", [gym,user]);
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
    const originalPlan = expected.get('plans')!.values().next().value!;
    const readerEdit = { protocolVersion: 2, operationId: randomUUID(), deviceId: reader, actorSubject: user, operationIds: [], changes: [{ table: 'plans', id: originalPlan.id, before: originalPlan, after: { ...originalPlan, id: String(originalPlan.id), name: 'Second computer plan', version: Number(originalPlan.version) + 1 } }] };
    const secondComputer = await push(readerEdit, readerHeaders);
    assert.equal(secondComputer.statusCode, 200, 'Administrator can commit from the second enrolled device');
    assert.deepEqual((await push(readerEdit, readerHeaders)).json(), secondComputer.json(), 'second-device retry returns its exact committed receipt');
    const staleWriter = await push({ ...readerEdit, operationId: randomUUID(), deviceId: device });
    assert.equal(staleWriter.statusCode, 409, 'the first computer cannot overwrite the second computer with stale state');
    assert.equal(staleWriter.json().error, 'business_revision_conflict');
    const oldPlan = readerEdit.changes[0].after;
    const edit = { protocolVersion: 2, operationId: randomUUID(), deviceId: device, actorSubject: user, operationIds: [], changes: [{ table: 'plans', id: oldPlan.id, before: oldPlan, after: { ...oldPlan, name: 'Updated plan', version: oldPlan.version + 1 } }] };
    assert.equal((await push(edit)).statusCode, 200);
    assert.equal((await push({ ...edit, operationId: randomUUID() })).statusCode, 409);
    await db.query('SAVEPOINT immutable_master_test');
    await assert.rejects(() => db!.query("UPDATE armstrong.business_records SET data=jsonb_set(data,'{joined_on}','\"2026-10-06\"') WHERE gym_id=$1 AND table_name='members'", [gym]));
    await db.query('ROLLBACK TO SAVEPOINT immutable_master_test');
    await db.query('UPDATE armstrong.devices SET active=false WHERE gym_id=$1 AND id=$2', [gym, device]);
    phase = 'revoked exact retry';
    assert.equal((await push(entries[0].request)).statusCode, 403, 'exact retries must still recheck revocation');
    for (const suffix of ['profile-recovery', 'profile-recovery-user']) {
      phase = `retained initial-profile recovery (${suffix})`;
      const recovery = JSON.parse(await readFile(`${process.env.ARMSTRONG_BUSINESS_FIXTURE_PATH}.${suffix}.json`, 'utf8'));
      const recovered = JSON.parse(JSON.stringify(recovery).replaceAll(recovery.originalRequest.actorSubject, user));
      const recoveryGym = randomUUID();
      await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Synthetic profile recovery')", [recoveryGym]);
      await db.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Synthetic Administrator','Administrator')", [recoveryGym, user]);
      for (const recoveryDevice of new Set<string>(recovered.entries.map((entry: any) => entry.request.deviceId))) {
        await db.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true)', [recoveryGym, recoveryDevice, createHash('sha256').update(secret).digest('hex')]);
      }
      const recoveryHeaders = { ...headers, 'x-gym-id': recoveryGym, 'x-device-id': recovered.originalRequest.deviceId };
      let checkedRefusal = false;
      for (const entry of recovered.entries) {
        if (!checkedRefusal && entry.request.deviceId === recovered.originalRequest.deviceId) {
          const refused = await push(recovered.originalRequest, recoveryHeaders);
          assert.equal(refused.statusCode, 409);
          assert.equal(refused.json().error, 'business_revision_conflict');
          checkedRefusal = true;
        }
        const reply = await push(entry.request, { ...recoveryHeaders, 'x-device-id': entry.request.deviceId });
        assert.equal(reply.statusCode, 200, reply.body);
        assert.equal(reply.json().requestSha256, digest(parseBatch(entry.request)));
        if (entry.request.operationId === recovered.replacementBatchId) {
          assert.deepEqual(entry.request.changes, recovered.originalRequest.changes.filter((change: any) => ['audit', 'users'].includes(change.table)));
          assert.deepEqual((await push(entry.request, recoveryHeaders)).json(), reply.json(), 'lost replacement reply returns the same actual receipt');
        }
      }
      assert.ok(checkedRefusal);
      assert.equal((await db.query('SELECT count(*)::int AS n FROM armstrong.business_operations WHERE gym_id=$1 AND id=$2', [recoveryGym, recovered.originalRequest.operationId])).rows[0].n, 0, 'refused original never acquires a fabricated receipt');
      const recoveredProfile: Record<string, unknown> = (await db.query("SELECT data FROM armstrong.business_records WHERE gym_id=$1 AND table_name='gym_settings' AND record_id='1'", [recoveryGym])).rows[0].data;
      assert.equal(recoveredProfile.version, 3);
      assert.equal(recoveredProfile.location, 'Server location 2');
      const recoveredHistory = stateFrom([]);
      cursor = 0;
      let recoveredCount = 0;
      for (;;) {
        const page: LightMyRequestResponse = await app.inject({ url: `/v2/business/changes?after=${cursor}`, headers: recoveryHeaders });
        assert.equal(page.statusCode, 200);
        const body = page.json();
        for (const entry of body.changes) {
          assert.notEqual(entry.request.operationId, recovered.originalRequest.operationId);
          assert.equal(entry.receipt.requestSha256, digest(parseBatch(entry.request)));
          applyChanges(recoveredHistory, entry.request.changes);
          recoveredCount++;
        }
        cursor = body.nextCursor;
        if (!body.hasMore) break;
      }
      assert.equal(recoveredCount, recovered.entries.length);
      assert.deepEqual(recoveredHistory.get('gym_settings')!.get('1'), recoveredProfile);
      for (const reference of recovered.originalRequest.changes.filter((change: any) => ['audit', 'users'].includes(change.table))) {
        assert.deepEqual(recoveredHistory.get(reference.table)!.get(reference.id), reference.after, 'original sign-in reference survives recovery');
      }
    }
  } catch (error) { throw safeFailure(error, `Business PostgreSQL integration (${phase})`); }
  finally { if (app) await app.close(); if (db) { await db.query('ROLLBACK'); db.release(); } await pool.end(); }
});
