import test from 'node:test';
import assert from 'node:assert/strict';
import { verifyBusinessReadiness } from '../src/business-readiness.ts';
import { createApp } from '../src/app.ts';

const names = ['business_records', 'business_references', 'business_operations', 'business_changes'];
const permissions = () => names.map(name => ({ name, rls: true, can_read: true, can_insert: true, can_delete: false }));
const guards = () => ({ record_update: true, reference_update: true, sequence_update: true, admission_contract: true, staff_contract: true, attendance_contract:true, staff_removal_contract:true, attendance_card_guards:true, staff_nic_guard: true, history_guards: true, immutable_triggers: true });
const fixture = (tables: Record<string, unknown>[] = permissions(), state: Record<string, unknown> = guards()) => ({
  async query(sql: string) { assert.ok(sql.startsWith('SELECT')); return { rows: sql.includes('c.relname AS name') ? tables : [state] }; }
});
test('API startup requires private business tables, immutable guards and narrow runtime grants', async () => {
  await verifyBusinessReadiness(fixture());
  await assert.rejects(verifyBusinessReadiness(fixture([])), /schema or restricted runtime permissions/);
  for (const patch of [{ rls: false }, { can_read: false }, { can_insert: false }, { can_delete: true }]) {
    const rows = permissions(); rows[0] = { ...rows[0], ...patch };
    await assert.rejects(verifyBusinessReadiness(fixture(rows)), /schema or restricted runtime permissions/);
  }
  for (const key of Object.keys(guards())) await assert.rejects(verifyBusinessReadiness(fixture(permissions(), { ...guards(), [key]: false })), /history guards or runtime column permissions/);
});
test('protocol-2 deployment health identifies all-module routes without account or business access', async () => {
  const unused = async () => { throw new Error('Health must not access Auth or business data'); };
  const app = createApp({ enroll:unused,push:unused,pull:unused,pushBusiness:unused,pullBusiness:unused },unused);
  try {
    const result = await app.inject({ url:'/v2/health' });
    assert.equal(result.statusCode,200);
    assert.deepEqual(result.json(),{status:'ok',service:'armstrong-gym-api',protocolVersion:2,businessSchemaVersion:12});
  } finally { await app.close(); }
});
