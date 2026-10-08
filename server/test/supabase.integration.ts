// LIVE Supabase PostgreSQL + Auth. No SQL, identity HTTP or JWT-verifier mocks.
// Fastify uses in-process injection: this is not Render/desktop HTTPS acceptance.
// Requires a fresh disposable project and three confirmed synthetic Auth accounts.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID, randomBytes, createHash } from 'node:crypto';
import pg from 'pg';
import { databaseOptions } from '../src/database.ts';
import { applyMigrations } from '../src/migrations.ts';
import { createApp } from '../src/app.ts';
import { createVerifier } from '../src/auth.ts';
import { MemberService } from '../src/service.ts';
import { registerAdministrator, RegistrationError } from '../src/registration.ts';
import { supabaseTestConfig, safeFailure, TestSetupError } from './support/supabase-config.ts';
import { lockEmptyDatabase } from './support/isolated-database.ts';
import { signInTestAccount } from './support/live-auth.ts';

test('LIVE Supabase PostgreSQL + Auth: isolated migrations and member API', { timeout: 300000 }, async t => {
  let owner: InstanceType<typeof pg.Client> | undefined;
  let pool: InstanceType<typeof pg.Pool> | undefined;
  let app: ReturnType<typeof createApp> | undefined;
  const step = async (name: string, run: () => Promise<void>) => {
    let failed = false;
    await t.test(name, async () => {
      try { await run(); } catch (error) { failed = true; throw safeFailure(error, name); }
    });
    // Later cases rely on earlier fixture state; never continue after a failure.
    if (failed) throw new TestSetupError(`Live suite stopped after failed step: ${name}`);
  };
  try {
    const c = supabaseTestConfig();
    owner = new pg.Client(databaseOptions(c.databaseUrl));
    await owner.connect();
    await lockEmptyDatabase(owner);
    const identities = [];
    const verify = createVerifier(c.authUrl, c.publishableKey); // default real fetch
    for (const account of c.accounts) {
      const identity = await signInTestAccount(c.authUrl, c.publishableKey, account);
      assert.equal(await verify(`Bearer ${identity.accessToken}`), identity.userId);
      identities.push(identity);
    }
    const [admin, reception, otherStaff] = identities;
    if (new Set(identities.map(identity => identity.userId)).size !== 3) throw new TestSetupError('Test Auth subjects must be distinct');
    // The Auth and SQL connections must actually refer to the same project.
    const matched = await owner.query('SELECT id,email_confirmed_at FROM auth.users WHERE id=ANY($1::uuid[])', [identities.map(identity => identity.userId)]);
    if (matched.rowCount !== 3 || matched.rows.some((row: { email_confirmed_at: unknown }) => !row.email_confirmed_at)) throw new TestSetupError('Three confirmed Auth subjects must match the test database project');
    if ((await owner.query('SELECT count(*) FROM auth.users')).rows[0].count !== '3') throw new TestSetupError('Use a fresh isolated project containing only the three configured test Auth accounts');

    await step('real migration runner, repeatability and checksum-drift rejection', async () => {
      await applyMigrations(owner!);
      const first = (await owner!.query('SELECT version,sha256,applied_at FROM public.armstrong_migrations')).rows;
      assert.equal(first.length, 5);
      await applyMigrations(owner!);
      assert.deepEqual((await owner!.query('SELECT version,sha256,applied_at FROM public.armstrong_migrations')).rows, first);
      await owner!.query("UPDATE public.armstrong_migrations SET sha256=repeat('0',64) WHERE version=1");
      try { await assert.rejects(applyMigrations(owner!), { message: 'Migration checksum mismatch' }); }
      finally { await owner!.query('UPDATE public.armstrong_migrations SET sha256=$1 WHERE version=1', [first[0].sha256]); }
    });

    const gym = randomUUID(), otherGym = randomUUID();
    const device = randomUUID(), otherDevice = randomUUID(), reader = randomUUID();
    const secret = randomBytes(32).toString('hex'), otherSecret = randomBytes(32).toString('hex'), readerSecret = randomBytes(32).toString('hex');
    const digest = (value: string) => createHash('sha256').update(value).digest('hex');
    await owner.query('INSERT INTO armstrong.gyms(id,name) VALUES($1,\'Integration test gym A\'),($2,\'Integration test gym B\')', [gym, otherGym]);
    for (const [gymId, userId, name, role] of [[gym, admin.userId, 'Test Administrator', 'Administrator'], [gym, reception.userId, 'Test Reception', 'Reception'], [otherGym, otherStaff.userId, 'Test other-gym staff', 'Administrator']]) {
      await owner.query('INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,$3,$4)', [gymId, userId, name, role]);
    }
    for (const [gymId, id, hash, writer] of [[gym, device, digest(secret), true], [otherGym, otherDevice, digest(otherSecret), true], [gym, reader, digest(readerSecret), false]]) {
      await owner.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,$4)', [gymId, id, hash, writer]);
    }
    const poolOptions = { ...databaseOptions(c.databaseUrl), max: 4 };
    pool = new pg.Pool(poolOptions);
    app = createApp(new MemberService(pool), verify);
    const headers = { authorization: `Bearer ${admin.accessToken}`, 'x-gym-id': gym, 'x-device-id': device, 'x-device-secret': secret };
    const receptionHeaders = { ...headers, authorization: `Bearer ${reception.accessToken}` };
    const otherHeaders = { authorization: `Bearer ${otherStaff.accessToken}`, 'x-gym-id': otherGym, 'x-device-id': otherDevice, 'x-device-secret': otherSecret };
    const readerHeaders = { ...headers, 'x-device-id': reader, 'x-device-secret': readerSecret };
    const payload = { protocolVersion: 1, deviceId: device, deviceSecret: secret };
    const enroll = (h = headers, body: unknown = payload) => app!.inject({ method: 'POST', url: '/v1/enrollment', headers: { 'content-type': 'application/json', ...h }, payload: JSON.stringify(body) });
    const push = (body: unknown, h = headers) => app!.inject({ method: 'POST', url: '/v1/members/push', headers: { 'content-type': 'application/json', ...h }, payload: JSON.stringify(body) });
    const pull = (after = 0, h = headers) => app!.inject({ url: `/v1/members/changes?after=${after}`, headers: h });
    const op = { protocolVersion: 1, operationId: randomUUID(), deviceId: device, memberId: randomUUID(), action: 'create', expectedRevision: 0, member: { name: 'Synthetic integration member', phone: '0000000000', email: '', nfcId: 'test-card', joinedOn: '2026-10-04' } };
    let receipt: any;

    await step('real administrative registration: read-only review, exact retry and denied permission replacement', async () => {
      const approved = { gymId: gym, gymName: 'Integration test gym A', adminName: 'Test Administrator', device: { id: device, secretSha256: digest(secret) } };
      const existing = { gym: 'existing', administrator: 'existing', device: 'existing' };
      assert.deepEqual(await registerAdministrator(owner!, admin.userId, approved), existing);
      assert.deepEqual(await registerAdministrator(owner!, admin.userId, approved, true), existing);
      // The administrative tool also refuses implicit changes to Reception,
      // revoked mappings, writer credentials or another gym. No self-promotion.
      for (const run of [
        () => registerAdministrator(owner!, reception.userId, approved, true),
        () => registerAdministrator(owner!, admin.userId, { ...approved, gymId: otherGym }, true),
        () => registerAdministrator(owner!, admin.userId, { ...approved, device: { id: device, secretSha256: digest(otherSecret) } }, true)
      ]) await assert.rejects(run(), RegistrationError);
      assert.equal((await owner!.query('SELECT count(*) FROM armstrong.gyms')).rows[0].count, '2');
      assert.equal((await owner!.query('SELECT count(*) FROM armstrong.staff')).rows[0].count, '3');
      assert.equal((await owner!.query('SELECT count(*) FROM armstrong.devices')).rows[0].count, '3');
    });

    await step('live Auth and enrolled staff/device: derived gym, replay and denied identity/secret', async () => {
      const result = await enroll();
      assert.equal(result.statusCode, 200);
      assert.deepEqual(result.json(), { protocolVersion: 1, gym: { id: gym, name: 'Integration test gym A' }, staff: { id: admin.userId, name: 'Test Administrator', role: 'Administrator' }, device: { id: device, canWrite: true } });
      assert.deepEqual((await enroll()).json(), result.json());
      assert.equal((await enroll(receptionHeaders)).json().staff.role, 'Reception');
      assert.equal((await enroll(headers, { ...payload, role: 'Administrator' })).statusCode, 400);
      assert.equal((await enroll(headers, { ...payload, gymId: otherGym })).statusCode, 400);
      assert.equal((await enroll(headers, { ...payload, deviceSecret: '0'.repeat(64) })).statusCode, 403);
      assert.equal((await enroll({ ...headers, authorization: 'Bearer invalid.signature' })).statusCode, 401);
      const noAuth = { ...headers }; delete (noAuth as any).authorization;
      assert.equal((await enroll(noAuth)).statusCode, 401);
      assert.equal((await push(op, { ...headers, authorization: 'Bearer invalid.signature' })).statusCode, 401);
      assert.equal((await pull(0, { ...headers, 'x-device-secret': '0'.repeat(64) })).statusCode, 403);
    });

    await step('real SQL constraints: multiple editing computers, roles, NFC normalization, revision, date and archive FK', async () => {
      const expectSql = async (sql: string, values: unknown[], code: string) => {
        await owner!.query('BEGIN');
        try { await assert.rejects(owner!.query(sql, values), (error: any) => error.code === code); }
        finally { await owner!.query('ROLLBACK'); }
      };
      await owner!.query('BEGIN');
      try { await owner!.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true)', [gym, randomUUID(), digest(secret)]); }
      finally { await owner!.query('ROLLBACK'); }
      await expectSql("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Test','Owner')", [gym, randomUUID()], '23514');
      const sql = "INSERT INTO armstrong.members(gym_id,id,name,phone,email,nfc_id,joined_on,revision,archived_at,archived_by_user_id) VALUES($1,$2,'Test','0000000000','',$3,$4,$5,$6,$7)";
      await expectSql(sql, [gym, randomUUID(), 'lowercase', '2026-10-04', 1, null, null], '23514');
      await expectSql(sql, [gym, randomUUID(), null, '2026-10-04', 0, null, null], '23514');
      await expectSql(sql, [gym, randomUUID(), null, '1899-12-31', 1, null, null], '23514');
      await expectSql(sql, [gym, randomUUID(), null, '2026-10-04', 1, new Date(), otherStaff.userId], '23503');
      await expectSql(sql, [gym, randomUUID(), null, '2026-10-04', 1, new Date(), null], '23514');
    });

    await step('simultaneous duplicate member pushes commit one member, receipt and change', async () => {
      const responses = await Promise.all([push(op), push(op)]);
      assert.deepEqual(responses.map(r => r.statusCode), [200, 200]);
      receipt = responses[0].json();
      assert.deepEqual(responses[1].json(), receipt);
      assert.equal(receipt.member.nfcId, 'TEST-CARD');
      assert.equal(receipt.revision, 1);
      for (const table of ['members', 'member_operations', 'member_changes']) assert.equal((await owner!.query(`SELECT count(*) FROM armstrong.${table} WHERE gym_id=$1`, [gym])).rows[0].count, '1');
    });

    await step('lost-reply retry after Fastify/pool restart returns the original receipt', async () => {
      // Discard one real API response, close the app/pool and retry frozen bytes.
      await push(op);
      await app!.close(); await pool!.end();
      pool = new pg.Pool(poolOptions);
      app = createApp(new MemberService(pool), verify);
      const result = await push(op);
      assert.equal(result.statusCode, 200);
      assert.deepEqual(result.json(), receipt);
      assert.equal((await owner!.query('SELECT change_sequence FROM armstrong.gyms WHERE id=$1', [gym])).rows[0].change_sequence, '1');
    });

    await step('operation-ID reuse, NFC collision and concurrent/stale edits return conflicts', async () => {
      const reused = await push({ ...op, member: { ...op.member, name: 'Different content' } });
      assert.equal(reused.statusCode, 409); assert.equal(reused.json().error, 'operation_id_reused');
      const card = await push({ ...op, operationId: randomUUID(), memberId: randomUUID() });
      assert.equal(card.statusCode, 409); assert.equal(card.json().error, 'card_or_member_conflict');
      const update = { ...op, operationId: randomUUID(), action: 'update', expectedRevision: 1, member: { ...op.member, name: 'Server update' } };
      const race = await Promise.all([push(update), push({ ...update, operationId: randomUUID() })]);
      assert.deepEqual(race.map(r => r.statusCode).sort(), [200, 409]);
      const stale = await push({ ...update, operationId: randomUUID() });
      assert.equal(stale.statusCode, 409); assert.equal(stale.json().error, 'revision_conflict'); assert.equal(stale.json().details.revision, 2);
      assert.equal((await push({ ...update, operationId: randomUUID(), expectedRevision: 2, member: { ...update.member, joinedOn: '2026-10-03' } })).statusCode, 409);
      assert.equal((await owner!.query('SELECT count(*) FROM armstrong.member_changes WHERE gym_id=$1', [gym])).rows[0].count, '2');
    });

    await step('two real gyms remain isolated even with reused member/card identifiers and forged gym headers', async () => {
      const otherOp = { ...op, operationId: randomUUID(), deviceId: otherDevice };
      assert.equal((await push(otherOp, otherHeaders)).statusCode, 200);
      const otherPage = await pull(0, otherHeaders);
      assert.equal(otherPage.statusCode, 200); assert.equal(otherPage.json().changes.length, 1);
      assert.equal((await pull(0, { ...headers, 'x-gym-id': otherGym })).statusCode, 403);
      assert.equal((await pull(0, { ...headers, 'x-gym-id': otherGym, 'x-device-id': otherDevice, 'x-device-secret': otherSecret })).statusCode, 403);
      assert.equal((await push(op, { ...otherHeaders, 'x-gym-id': gym })).statusCode, 403);
      assert.equal((await enroll(otherHeaders, payload)).statusCode, 403);
    });

    await step('Administrator access follows the account and Reception retains its restrictions', async () => {
      const enrolled = await enroll(readerHeaders, { protocolVersion: 1, deviceId: reader, deviceSecret: readerSecret });
      assert.equal(enrolled.statusCode, 200); assert.equal(enrolled.json().device.canWrite, true);
      assert.equal((await pull(0, readerHeaders)).statusCode, 200);
      assert.equal((await push({ ...op, deviceId: reader }, readerHeaders)).statusCode, 409, 'Administrator passes device authorization; operation IDs remain bound to the original device');
      const receptionReader = { ...readerHeaders, authorization: `Bearer ${reception.accessToken}` };
      assert.equal((await enroll(receptionReader, { protocolVersion: 1, deviceId: reader, deviceSecret: readerSecret })).json().device.canWrite, false);
      assert.equal((await push({ ...op, deviceId: reader }, receptionReader)).statusCode, 403);
      const archive = { ...op, operationId: randomUUID(), action: 'archive', expectedRevision: 2, member: null };
      assert.equal((await push(archive, receptionHeaders)).statusCode, 403);
      const archived = await push(archive);
      assert.equal(archived.statusCode, 200); assert.equal(archived.json().member.archivedBy.id, admin.userId);
      assert.equal((await push(archive, receptionHeaders)).statusCode, 403, 'current permission check precedes replay');
      assert.equal((await push({ ...op, operationId: randomUUID(), action: 'update', expectedRevision: 3 })).statusCode, 409);
    });

    await step('ordered pulls and retry cursors retain server updates and archive history', async () => {
      const all = await pull();
      assert.equal(all.statusCode, 200);
      assert.deepEqual(all.json().changes.map((change: any) => change.sequence), [1, 2, 3]);
      assert.equal(all.json().nextCursor, 3); assert.equal(all.json().hasMore, false);
      assert.equal(all.json().changes[1].member.name, 'Server update');
      assert.ok(all.json().changes[2].member.archivedAt);
      assert.deepEqual((await pull()).json(), all.json());
      assert.deepEqual((await pull(1)).json().changes.map((change: any) => change.sequence), [2, 3]);
      assert.equal((await pull(3)).json().changes.length, 0);
      assert.equal((await pull(100)).statusCode, 409);
      assert.equal((await owner!.query('SELECT count(*) FROM armstrong.member_changes WHERE gym_id=$1', [gym])).rows[0].count, '3');
    });

    await step('append-only SQL history and archived member constraints reject direct edits', async () => {
      for (const sql of ['UPDATE armstrong.member_operations SET receipt=receipt WHERE gym_id=$1', 'DELETE FROM armstrong.member_changes WHERE gym_id=$1', 'UPDATE armstrong.members SET name=name WHERE gym_id=$1']) {
        await owner!.query('BEGIN');
        try { await assert.rejects(owner!.query(sql, [gym]), (error: any) => error.code === 'P0001'); }
        finally { await owner!.query('ROLLBACK'); }
      }
      const rls = await owner!.query("SELECT bool_and(relrowsecurity) AS enabled FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='armstrong' AND c.relkind='r'");
      assert.equal(rls.rows[0].enabled, true);
      const policies = await owner!.query("SELECT count(*) FROM pg_policies WHERE schemaname='armstrong'");
      assert.equal(policies.rows[0].count, '0');
    });

    await step('change insertion failure rolls back member, operation and gym sequence together', async () => {
      await owner!.query("CREATE FUNCTION armstrong.integration_fail_change() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'Injected integration failure'; END $$");
      await owner!.query('CREATE TRIGGER integration_fail_change BEFORE INSERT ON armstrong.member_changes FOR EACH ROW EXECUTE FUNCTION armstrong.integration_fail_change()');
      const failedOp = { ...op, operationId: randomUUID(), memberId: randomUUID(), member: { ...op.member, nfcId: null } };
      try {
        const response = await push(failedOp);
        assert.equal(response.statusCode, 503); assert.deepEqual(response.json(), { error: 'service_unavailable' });
        assert.equal((await owner!.query('SELECT count(*) FROM armstrong.members WHERE gym_id=$1 AND id=$2', [gym, failedOp.memberId])).rows[0].count, '0');
        assert.equal((await owner!.query('SELECT count(*) FROM armstrong.member_operations WHERE gym_id=$1 AND id=$2', [gym, failedOp.operationId])).rows[0].count, '0');
        assert.equal((await owner!.query('SELECT change_sequence FROM armstrong.gyms WHERE id=$1', [gym])).rows[0].change_sequence, '3');
      } finally {
        await owner!.query('DROP TRIGGER integration_fail_change ON armstrong.member_changes');
        await owner!.query('DROP FUNCTION armstrong.integration_fail_change()');
      }
      // Exact operation bytes can succeed once after the transient failure clears.
      assert.equal((await push(failedOp)).statusCode, 200);
      assert.equal((await push(failedOp)).statusCode, 200);
      assert.equal((await owner!.query('SELECT change_sequence FROM armstrong.gyms WHERE id=$1', [gym])).rows[0].change_sequence, '4');
    });

    await step('live verified staff/device revocation denies enrollment, pulls and even saved push retries', async () => {
      await owner!.query('UPDATE armstrong.devices SET active=false WHERE gym_id=$1 AND id=$2', [gym, device]);
      try { assert.equal((await enroll()).statusCode, 403); assert.equal((await push(op)).statusCode, 403); assert.equal((await pull()).statusCode, 403); }
      finally { await owner!.query('UPDATE armstrong.devices SET active=true WHERE gym_id=$1 AND id=$2', [gym, device]); }
      await owner!.query('UPDATE armstrong.staff SET active=false WHERE gym_id=$1 AND user_id=$2', [gym, admin.userId]);
      try { assert.equal((await enroll()).statusCode, 403); assert.equal((await push(op)).statusCode, 403); assert.equal((await pull()).statusCode, 403); }
      finally { await owner!.query('UPDATE armstrong.staff SET active=true WHERE gym_id=$1 AND user_id=$2', [gym, admin.userId]); }
    });
    t.diagnostic('Real SQL/Auth cases completed; synthetic fixtures retained. No desktop/Render HTTPS sync was tested.');
  } catch (error) { throw safeFailure(error, 'Live Supabase integration setup/run'); }
  finally {
    // Closing the owner releases its session lock. No data/schema/account deletion.
    try { if (app) await app.close(); if (pool) await pool.end(); if (owner) await owner.end(); }
    catch (error) { throw safeFailure(error, 'Live integration connection cleanup'); }
  }
});
