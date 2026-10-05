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
