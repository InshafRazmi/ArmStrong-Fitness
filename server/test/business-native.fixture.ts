import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { parseBatch, stateFrom, applyChanges, digest } from '../src/business-protocol.ts';
test('real native SQLite transaction envelopes match server contract and receipt hashes for all business modules', async () => {
  const file = process.env.ARMSTRONG_BUSINESS_FIXTURE_PATH;
  assert.ok(file, 'Generate the fixture with the native all-module sync test first');
  const entries = JSON.parse(await readFile(file, 'utf8')) as any[];
  assert.ok(entries.length >= 10);
  const state = stateFrom([]);
  for (const entry of entries) {
    const batch = parseBatch(entry.request);
    assert.equal(digest(batch), entry.receipt.requestSha256);
    applyChanges(state, batch.changes);
  }
  assert.equal(state.get('payments')!.size, 2);
  assert.equal(state.get('payment_receipts')!.size, 2);
  assert.equal(state.get('allocation_reversals')!.size, 1);
  assert.equal(state.get('expense_voids')!.size, 1);
  assert.equal(state.get('attendance')!.size, 1);
  assert.equal(state.get('sales')!.size, 1);
  assert.equal([...state.get('stock_movements')!.values()].reduce((n, r) => n + Number(r.delta), 0), 1);
});
test('guarded native initial-profile recovery uploads audit history without overwriting the server profile', async () => {
  const file = process.env.ARMSTRONG_BUSINESS_FIXTURE_PATH;
  assert.ok(file, 'Generate the fixture with the native recovery test first');
  const fixture = JSON.parse(await readFile(`${file}.profile-recovery.json`, 'utf8'));
  const state = stateFrom([]);
  for (const entry of fixture.entries) {
    const batch = parseBatch(entry.request);
    assert.equal(digest(batch), entry.receipt.requestSha256);
    applyChanges(state, batch.changes);
  }
  const replacement = fixture.entries.find((entry: any) => entry.request.operationId === fixture.replacementBatchId);
  assert.ok(replacement, 'replacement has an actual server-compatible receipt');
  assert.ok(!fixture.entries.some((entry: any) => entry.request.operationId === fixture.originalRequest.operationId), 'refused seed is never labeled accepted');
  assert.deepEqual(replacement.request.changes, fixture.originalRequest.changes.filter((change: any) => change.table === 'audit'));
  assert.deepEqual(replacement.request.operationIds, []);
  assert.equal(state.get('gym_settings')!.get('1')!.version, 3);
  assert.equal(state.get('gym_settings')!.get('1')!.location, 'Server location 2');
});
