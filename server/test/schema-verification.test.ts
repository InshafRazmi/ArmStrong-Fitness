// Local catalog/ledger mocks only: never loads env files or connects to Supabase.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { verifyRuntimeSchema } from '../src/schema-verification.ts';

const hash = createHash('sha256').update(await readFile(new URL('../migrations/001_members.sql', import.meta.url), 'utf8')).digest('hex');
const tableRows = () => ['gyms', 'staff', 'devices', 'members', 'member_operations', 'member_changes'].map(name => ({ name, rls: true }));
const fixture = (patch: { state?: Record<string, unknown>; migration?: Record<string, unknown>[]; tables?: Record<string, unknown>[] } = {}) => {
  const queries: string[] = [];
  return {
    queries,
    async query(sql: string) {
      queries.push(sql);
      return { rows: sql.includes('AS app_schema') ? [patch.state ?? { app_schema: true, ledger: true }]
        : sql.includes('SELECT sha256') ? (patch.migration ?? [{ sha256: hash }])
        : (patch.tables ?? tableRows()) };
    }
  };
};

test('runtime schema verification / catalog mock: reads metadata only and exposes no ledger or extra values', async () => {
  const client = fixture({ migration: [{ sha256: hash, private_value: 'fixture-secret' }] });
  const result = await verifyRuntimeSchema(client);
  assert.deepEqual(result, { tableCount: 6, checksumMatches: true, rlsEnabled: true });
  assert.ok(!JSON.stringify(result).includes('fixture-secret'));
  assert.ok(!JSON.stringify(result).includes(hash));
  assert.ok(client.queries.every(sql => sql.startsWith('SELECT')));
});
test('runtime schema verification / catalog mock: stops on missing schema/ledger or untracked/drifted migration', async () => {
  for (const state of [{ app_schema: false, ledger: true }, { app_schema: true, ledger: false }, {}]) {
    const client = fixture({ state });
    await assert.rejects(verifyRuntimeSchema(client), /schema or migration ledger missing/);
    assert.equal(client.queries.length, 1);
  }
  for (const migration of [[], [{ sha256: 'fixture-secret' }], [{ sha256: hash }, { sha256: hash }]]) {
    const client = fixture({ migration });
    await assert.rejects(verifyRuntimeSchema(client), error => {
      assert.equal((error as Error).message, 'Runtime migration version 1 missing or checksum mismatch; values withheld; no changes made');
      return true;
    });
    assert.equal(client.queries.length, 2);
  }
});
test('runtime schema verification / catalog mock: refuses missing, substituted or RLS-disabled tables', async () => {
  const missing = tableRows().slice(0, -1);
  const substituted = tableRows(); substituted[5] = { name: 'unexpected', rls: true };
  const duplicate = tableRows(); duplicate[5] = duplicate[0];
  for (const tables of [missing, substituted, duplicate]) await assert.rejects(verifyRuntimeSchema(fixture({ tables })), /private tables missing/);
  const disabled = tableRows(); disabled[0].rls = false;
  await assert.rejects(verifyRuntimeSchema(fixture({ tables: disabled })), /RLS disabled/);
});
