// Registration SQL mocks only. No env loading, PostgreSQL or live Auth.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { registerAdministrator, RegistrationError, type Registration } from '../src/registration.ts';

const gymId = randomUUID(), userId = randomUUID(), deviceId = randomUUID();
const requested: Registration = { gymId, gymName: 'Synthetic gym', adminName: 'Synthetic Administrator', device: { id: deviceId, secretSha256: 'a'.repeat(64) } };
const gym = { id: gymId, name: requested.gymName };
const staff = { gym_id: gymId, user_id: userId, display_name: requested.adminName, role: 'Administrator', active: true };
const device = { gym_id: gymId, id: deviceId, secret_sha256: requested.device!.secretSha256, active: true, can_write: true };
function fixture(patch: { confirmed?: boolean; gyms?: Record<string, unknown>[]; staff?: Record<string, unknown>[]; devices?: Record<string, unknown>[]; failInsert?: string } = {}) {
  const queries: { sql: string; values: unknown[] }[] = [];
  return {
    queries,
    async query(sql: string, values: unknown[] = []) {
      queries.push({ sql, values });
      if (sql.startsWith('SELECT EXISTS')) { assert.deepEqual(values, [userId]); return { rows: [{ confirmed: patch.confirmed ?? true }] }; }
      if (sql.startsWith('SELECT id,name')) return { rows: patch.gyms ?? [] };
      if (sql.startsWith('SELECT gym_id,user_id')) return { rows: patch.staff ?? [] };
      if (sql.startsWith('SELECT gym_id,id')) return { rows: patch.devices ?? [] };
      if (patch.failInsert && sql.startsWith(patch.failInsert)) throw Object.assign(new Error('fixture-provider-details'), { code: '23505' });
      assert.ok(/^(BEGIN|SELECT pg_advisory_xact_lock|INSERT INTO armstrong\.|COMMIT|ROLLBACK)/.test(sql));
      return { rows: [] };
    }
  };
}

test('registration / SQL mock: review is read-only, uses verified subject and exposes states only', async () => {
  const client = fixture();
  const plan = await registerAdministrator(client, userId, requested);
  assert.deepEqual(plan, { gym: 'create', administrator: 'create', device: 'create' });
  assert.equal(client.queries[0].sql, 'BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY');
  assert.ok(client.queries.every(q => /^(BEGIN|SELECT|COMMIT)/.test(q.sql) && !q.sql.includes('FOR UPDATE')));
  for (const value of [gymId, userId, deviceId, requested.device!.secretSha256, requested.adminName]) assert.ok(!JSON.stringify(plan).includes(value));
});

test('registration / SQL mock: apply inserts approved records atomically with no plaintext device credential', async () => {
  const client = fixture();
  await registerAdministrator(client, userId, requested, true);
  assert.equal(client.queries[0].sql, 'BEGIN');
  assert.equal(client.queries[1].sql, 'SELECT pg_advisory_xact_lock(714339806)');
  assert.ok(client.queries.find(q => q.sql.startsWith('SELECT id,name'))!.sql.endsWith('FOR UPDATE'));
  const inserts = client.queries.filter(q => q.sql.startsWith('INSERT'));
  assert.equal(inserts.length, 3);
  assert.deepEqual(inserts[0].values, [gymId, requested.gymName]);
  assert.deepEqual(inserts[1].values, [gymId, userId, requested.adminName]);
  assert.deepEqual(inserts[2].values, [gymId, deviceId, requested.device!.secretSha256]);
  assert.equal(client.queries.at(-1)!.sql, 'COMMIT');
  assert.ok(client.queries.every(q => !/^(UPDATE|DELETE|CREATE|ALTER|DROP)/.test(q.sql)));
});

test('registration / SQL mock: exact retry retains IDs, permissions and credential hash without writes', async () => {
  const client = fixture({ gyms: [gym], staff: [staff], devices: [device] });
  assert.deepEqual(await registerAdministrator(client, userId, requested, true), { gym: 'existing', administrator: 'existing', device: 'existing' });
  assert.equal(client.queries.filter(q => q.sql.startsWith('INSERT')).length, 0);
});

test('registration / SQL mock: different gyms, roles, revocations, names and device proofs fail before any insert', async () => {
  for (const patch of [
    { gyms: [{ ...gym, name: 'Different name' }] },
    { staff: [{ ...staff, gym_id: randomUUID() }] },
    { staff: [{ ...staff, user_id: randomUUID() }] },
    { staff: [{ ...staff, role: 'Reception' }] },
    { staff: [{ ...staff, active: false }] },
    { staff: [{ ...staff, display_name: 'Different name' }] },
    { staff: [staff, { ...staff, user_id: randomUUID() }] },
    { devices: [{ ...device, gym_id: randomUUID() }] },
    { devices: [{ ...device, id: randomUUID() }] },
    { devices: [{ ...device, secret_sha256: 'b'.repeat(64) }] },
    { devices: [{ ...device, active: false }] },
    { devices: [device, { ...device, id: randomUUID() }] }
  ]) {
    const client = fixture(patch);
    await assert.rejects(registerAdministrator(client, userId, requested, true), RegistrationError);
    assert.ok(!client.queries.some(q => q.sql.startsWith('INSERT')));
    assert.equal(client.queries.at(-1)!.sql, 'ROLLBACK');
  }
});

test('registration / SQL mock: another editing device does not block Administrator registration', async () => {
  const client = fixture({ gyms: [gym], staff: [staff] });
  assert.deepEqual(await registerAdministrator(client, userId, requested, true), { gym: 'existing', administrator: 'existing', device: 'create' });
  const lookup = client.queries.find(q => q.sql.startsWith('SELECT gym_id,id'))!;
  assert.equal(lookup.sql, 'SELECT gym_id,id,secret_sha256,active,can_write FROM armstrong.devices WHERE id=$1');
  assert.deepEqual(lookup.values, [deviceId]);
  assert.equal(client.queries.filter(q => q.sql.startsWith('INSERT')).length, 1);
  const existing = fixture({ gyms: [gym], staff: [staff], devices: [{ ...device, can_write: false }] });
  assert.deepEqual(await registerAdministrator(existing, userId, requested, true), { gym: 'existing', administrator: 'existing', device: 'existing' });
  assert.ok(!existing.queries.some(q => /^(INSERT|UPDATE|DELETE)/.test(q.sql)));
});

test('registration / SQL mock: unconfirmed or different-project identity fails before registry reads/writes', async () => {
  const client = fixture({ confirmed: false });
  await assert.rejects(registerAdministrator(client, userId, requested, true), /confirmed in the runtime database project/);
  assert.equal(client.queries.length, 4);
  assert.ok(!client.queries.some(q => q.sql.includes('FROM armstrong.') || q.sql.startsWith('INSERT')));
});

test('registration / SQL mock: insertion failure rolls back the transaction and preserves safe retry', async () => {
  const client = fixture({ failInsert: 'INSERT INTO armstrong.staff' });
  await assert.rejects(registerAdministrator(client, userId, requested, true), { code: '23505' });
  assert.equal(client.queries.at(-1)!.sql, 'ROLLBACK');
  assert.ok(!client.queries.some(q => q.sql === 'COMMIT' || q.sql.startsWith('INSERT INTO armstrong.devices')));
});

test('registration / SQL mock: gym/Admin-only setup does not fabricate or approve a device', async () => {
  const client = fixture();
  const { device: omitted, ...adminOnly } = requested;
  assert.deepEqual(await registerAdministrator(client, userId, adminOnly, true), { gym: 'create', administrator: 'create', device: 'not included' });
  assert.ok(!client.queries.some(q => q.sql.includes('armstrong.devices')));
});

test('registration / SQL mock: invalid UUIDs, names and hashes reject before transaction', async () => {
  for (const bad of [
    { ...requested, gymId: 'invalid' }, { ...requested, gymName: ' ' }, { ...requested, adminName: 'x'.repeat(121) },
    { ...requested, device: { id: 'invalid', secretSha256: 'a'.repeat(64) } },
    { ...requested, device: { id: deviceId, secretSha256: 'not-a-hash' } }
  ]) {
    const client = fixture();
    await assert.rejects(registerAdministrator(client, userId, bad, true), RegistrationError);
    assert.equal(client.queries.length, 0);
  }
});
