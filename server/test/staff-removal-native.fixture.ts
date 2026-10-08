import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { applyChanges, parseBatch, stateFrom } from '../src/business-protocol.ts';

test('native staff removal synchronizes deactivation, card revocation and assignments while retaining money and attendance', async () => {
  assert.ok(process.env.ARMSTRONG_STAFF_REMOVAL_FIXTURE_PATH, 'Generate the native staff removal fixture first');
  const batches = JSON.parse(await readFile(process.env.ARMSTRONG_STAFF_REMOVAL_FIXTURE_PATH, 'utf8'));
  const state = stateFrom([]);
  for (const request of batches) applyChanges(state, parseBatch(request).changes);
  const trainer = [...state.get('trainers')!.values()][0];
  assert.equal(trainer.active, 0);
  assert.equal(trainer.version, 3);
  assert.equal(state.get('staff_deletions')!.size, 1);
  assert.equal([...state.get('staff_deletions')!.values()][0].id, trainer.id);
  assert.equal([...state.get('member_trainers')!.values()][0].trainer_id, null);
  assert.ok([...state.get('staff_nfc_cards')!.values()].every(card => card.revoked_at !== null));
  assert.equal(state.get('training_charges')!.size, 1);
  assert.equal(state.get('staff_payouts')!.size, 1);
  assert.equal(state.get('staff_attendance')!.size, 1);
  const update = {table:'trainers',id:String(trainer.id),before:trainer,after:{...trainer,active:1,version:4}};
  assert.throws(() => applyChanges(state, [update]), /deleted_staff_immutable/);
  const marker = [...state.get('staff_deletions')!.values()][0];
  assert.throws(() => parseBatch({...batches.at(-1),changes:[{table:'staff_deletions',id:trainer.id,before:marker,after:{...marker,deleted_at:'2026-10-09T00:00:00Z'}}]}), /immutable_business_history/);
  assert.equal(state.get('payment_receipts')!.size, 1);
  assert.equal([...state.get('staff_payouts')!.values()][0].training_minor, 500025);
  const last = parseBatch(batches.at(-1));
  // A stale write without its saved operation receipt cannot reapply removal.
  assert.throws(() => applyChanges(state, last.changes), /business_revision_conflict/);
  assert.equal(state.get('staff_payouts')!.size, 1);
  assert.equal(state.get('staff_attendance')!.size, 1);
});
