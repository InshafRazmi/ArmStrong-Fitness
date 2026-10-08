// Authorization regression with mocked SQL. Real enrollment SQL is checked
// separately against the isolated PostgreSQL project.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID, createHash } from 'node:crypto';
import { MemberService, type Scope } from '../src/service.ts';
import { ApiError } from '../src/protocol.ts';

class AccessService extends MemberService {
  edit(scope: Scope) {
    return this.transaction(scope, true, async (_db, role, gymId) => ({ role, gymId }));
  }
}

function fixture() {
  const gymId = randomUUID(), userId = randomUUID(), secret = 'a'.repeat(64);
  const deviceIds: string[] = [randomUUID(), randomUUID()];
  const hash = createHash('sha256').update(secret).digest('hex');
  const state = { role: 'Administrator', staffActive: true, deviceActive: true, canWrite: false };
  const queries: string[] = [];
  const result = (rows: unknown[]) => ({ rows, rowCount: rows.length });
  const service = new AccessService({ connect: async () => ({
    query: async (sql: string, args: unknown[] = []) => {
      queries.push(sql);
      if (['BEGIN', 'COMMIT', 'ROLLBACK'].includes(sql)) return result([]);
      if (sql.startsWith('SELECT s.gym_id')) {
        assert.equal(args[0], userId);
        return result(deviceIds.includes(String(args[1])) && state.staffActive && state.deviceActive ? [{ gym_id: gymId, secret_sha256: hash }] : []);
      }
      assert.equal(args[0], gymId);
      if (sql.includes('FOR UPDATE')) return result([{ id: gymId }]);
      if (sql.startsWith('SELECT role')) return result(state.staffActive ? [{ role: state.role }] : []);
      if (sql.startsWith('SELECT secret_sha256')) return result(state.deviceActive ? [{ secret_sha256: hash, can_write: state.canWrite }] : []);
      throw new Error('Unexpected authorization query');
    }, release() {},
  }) });
  const scope = (deviceId = deviceIds[0]): Scope => ({ gymId, userId, deviceId, deviceSecret: secret });
  return { service, scope, state, queries, deviceIds, gymId };
}

test('Administrator writes work from multiple enrolled devices even with legacy read-only flags', async () => {
  const f = fixture();
  for (const id of f.deviceIds) assert.deepEqual(await f.service.edit(f.scope(id)), { role: 'Administrator', gymId: f.gymId });
  assert.equal(f.queries.filter(sql => sql === 'COMMIT').length, 2);
});

test('Administrator access still requires matching device proof, gym and active registrations', async () => {
  for (const mutate of [
    (f: ReturnType<typeof fixture>) => { f.state.staffActive = false; },
    (f: ReturnType<typeof fixture>) => { f.state.deviceActive = false; },
  ]) {
    const f = fixture(); mutate(f);
    await assert.rejects(f.service.edit(f.scope()), error => error instanceof ApiError && error.status === 403);
    assert.ok(!f.queries.includes('COMMIT'));
  }
  const f = fixture();
  for (const scope of [
    { ...f.scope(), deviceSecret: 'b'.repeat(64) },
    { ...f.scope(), gymId: randomUUID() },
    f.scope(randomUUID()),
  ]) await assert.rejects(f.service.edit(scope), error => error instanceof ApiError && error.status === 403);
});

test('demotion takes effect on the next write and does not inherit Administrator device access', async () => {
  const f = fixture();
  await f.service.edit(f.scope());
  f.state.role = 'Reception';
  await assert.rejects(f.service.edit(f.scope()), error => error instanceof ApiError && error.status === 403);
  f.state.canWrite = true;
  assert.equal((await f.service.edit(f.scope()) as { role: string }).role, 'Reception');
});
