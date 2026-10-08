import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { applyChanges,parseBatch,stateFrom,validateState } from '../src/business-protocol.ts';

test('native gender and staff/member attendance envelopes preserve separate histories and shared card ownership',async()=>{
  assert.ok(process.env.ARMSTRONG_ATTENDANCE_FIXTURE_PATH,'Generate native attendance fixture first');
  const batches=JSON.parse(await readFile(process.env.ARMSTRONG_ATTENDANCE_FIXTURE_PATH,'utf8'));
  const state=stateFrom([]);for(const b of batches)applyChanges(state,parseBatch(b).changes);
  assert.equal(state.get('member_profiles')!.size,1);assert.equal([...state.get('member_profiles')!.values()][0].gender,'Female');
  assert.equal(state.get('staff_attendance')!.size,1);assert.equal(state.get('attendance')!.size,1);
  const staffCard=[...state.get('staff_nfc_cards')!.values()][0];
  const memberCard=[...state.get('nfc_cards')!.values()][0];
  state.get('staff_nfc_cards')!.set(String(staffCard.id),{...staffCard,uid:memberCard.uid});assert.throws(()=>validateState(state),/business_unique_conflict/);
});
