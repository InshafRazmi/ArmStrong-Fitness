// Authorization unit tests with database-row fixtures, not live backend evidence.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { deriveGym } from '../src/gym.ts';
const secret = '1'.repeat(64);
const hash = createHash('sha256').update(secret).digest('hex');
test('gym authorization unit: derives the gym from staff/device rows and rejects forged gym assertion', () => {
  const authorizedGym = randomUUID();
  const rows = [{gym_id:authorizedGym,secret_sha256:hash}];
  assert.equal(deriveGym(rows,secret,authorizedGym),authorizedGym);
  assert.throws(()=>deriveGym(rows,secret,randomUUID()),{code:'gym_not_authorized'});
});
test('gym authorization unit: missing staff/device, bad device secret and ambiguous registry deny access',()=>{
  const gym = randomUUID();
  assert.throws(()=>deriveGym([],secret,gym),{code:'device_not_authorized'});
  assert.throws(()=>deriveGym([{gym_id:gym,secret_sha256:hash}],'2'.repeat(64),gym),{code:'device_not_authorized'});
  assert.throws(()=>deriveGym([{gym_id:gym,secret_sha256:hash},{gym_id:randomUUID(),secret_sha256:hash}],secret,gym),{code:'device_not_authorized'});
});
