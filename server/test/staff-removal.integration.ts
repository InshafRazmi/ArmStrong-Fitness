// Native SQLite requests, real isolated Auth/PostgreSQL; every fixture row rolls back.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash, randomBytes, randomUUID } from 'node:crypto';
import pg from 'pg';
import type { LightMyRequestResponse } from 'fastify';
import { databaseOptions } from '../src/database.ts';
import { GymService } from '../src/business-service.ts';
import { createApp } from '../src/app.ts';
import { createVerifier } from '../src/auth.ts';
import { digest } from '../src/business-protocol.ts';
import { safeFailure, TestSetupError, supabaseTestConfig } from './support/supabase-config.ts';
import { signInTestAccount } from './support/live-auth.ts';

test('LIVE staff deletion: exact native retries, retained payroll/attendance, second-device download and SQL immutability', async () => {
  const config = supabaseTestConfig();
  if (config.authUrl !== 'https://srwjyvimdktrvzpdxvpd.supabase.co' || !process.env.ARMSTRONG_STAFF_REMOVAL_FIXTURE_PATH) throw new TestSetupError('Use isolated project and native staff-removal fixture');
  const identity = await signInTestAccount(config.authUrl, config.publishableKey, config.accounts[0]);
  const batches = JSON.parse(await readFile(process.env.ARMSTRONG_STAFF_REMOVAL_FIXTURE_PATH, 'utf8')) as any[];
  const pool = new pg.Pool(databaseOptions(config.databaseUrl));
  let db: pg.PoolClient | undefined; let app: ReturnType<typeof createApp> | undefined; let phase = 'connection';
  try {
    db = await pool.connect(); await db.query('BEGIN');
    const gym = randomUUID(), device = batches[0].deviceId, second = randomUUID(), secret = randomBytes(32).toString('hex');
    await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Synthetic staff deletion acceptance')", [gym]);
    await db.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Synthetic Administrator','Administrator')", [gym, identity.userId]);
    await db.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true),($1,$4,$3,true)', [gym, device, createHash('sha256').update(secret).digest('hex'), second]);
    const fixturePool = {connect:async () => ({query:async (sql:string,values?:unknown[]) => {
      try {return await db!.query(sql === 'BEGIN' ? 'SAVEPOINT removal_api_test' : sql === 'COMMIT' ? 'RELEASE SAVEPOINT removal_api_test' : sql === 'ROLLBACK' ? 'ROLLBACK TO SAVEPOINT removal_api_test' : sql, values);}
      catch(error) {console.error(safeFailure(error, 'Staff deletion SQL acceptance').message);throw error;}
    },release:() => {}})};
    app = createApp(new GymService(fixturePool), createVerifier(config.authUrl, config.publishableKey));
    const headers = {authorization:`Bearer ${identity.accessToken}`,'x-gym-id':gym,'x-device-id':device,'x-device-secret':secret};
    phase = 'native deletion transactions and durable receipts';
    for (const [index,batch] of batches.entries()) {
      const response: LightMyRequestResponse = await app.inject({method:'POST',url:'/v2/business/push',headers,payload:batch});
      assert.equal(response.statusCode, 200, `Batch ${index+1}: ${response.body}`); assert.equal(response.json().requestSha256, digest(batch));
      assert.deepEqual((await app.inject({method:'POST',url:'/v2/business/push',headers,payload:batch})).json(), response.json());
    }
    const persisted = (await db.query('SELECT table_name,record_id,data FROM armstrong.business_records WHERE gym_id=$1', [gym])).rows;
    for (const name of ['staff_deletions','staff_payouts','staff_attendance','training_charges','payment_receipts']) assert.equal(persisted.filter(r => r.table_name === name).length, 1, name+' retained');
    const trainer = persisted.find(r => r.table_name === 'trainers').data;
    assert.equal(trainer.active, 0); assert.equal(trainer.version, 3);
    assert.equal(persisted.find(r => r.table_name === 'member_trainers').data.trainer_id, null);
    assert.ok(persisted.filter(r => r.table_name === 'staff_nfc_cards').every(r => r.data.revoked_at !== null));
    phase = 'second device receives permanent deletion with history';
    let cursor = 0; const downloaded:any[] = [];
    for (let i=0;i<=batches.length;i++) {
      const response: LightMyRequestResponse = await app.inject({url:`/v2/business/changes?after=${cursor}`,headers:{...headers,'x-device-id':second}});
      assert.equal(response.statusCode,200,response.body); const page = response.json(); downloaded.push(...page.changes); cursor=page.nextCursor; if(!page.hasMore)break;
    }
    assert.equal(downloaded.length,batches.length); assert.ok(downloaded.flatMap(e => e.request.changes).some(c => c.table === 'staff_deletions'));
    phase = 'API and PostgreSQL refuse editing a deleted staff profile';
    const update = {protocolVersion:2,operationId:randomUUID(),deviceId:second,actorSubject:identity.userId,operationIds:[],changes:[{table:'trainers',id:trainer.id,before:trainer,after:{...trainer,active:1,version:4}}]};
    const rejected = await app.inject({method:'POST',url:'/v2/business/push',headers:{...headers,'x-device-id':second},payload:update});
    assert.equal(rejected.statusCode,409); assert.equal(rejected.json().error,'deleted_staff_immutable');
    await db.query('SAVEPOINT deleted_staff_sql');
    await assert.rejects(db.query("UPDATE armstrong.business_records SET data=jsonb_set(jsonb_set(data,'{active}','1'),'{version}','4') WHERE gym_id=$1 AND table_name='trainers'",[gym]), /cannot be edited/);
    await db.query('ROLLBACK TO SAVEPOINT deleted_staff_sql');
    await assert.rejects(db.query("UPDATE armstrong.business_records SET data=jsonb_set(data,'{deleted_at}','\"2026-10-09T00:00:00Z\"') WHERE gym_id=$1 AND table_name='staff_deletions'",[gym]), /immutable/);
  } catch(error) {if(error instanceof assert.AssertionError)throw error; throw safeFailure(error,`Live staff deletion at ${phase}`);}
  finally {if(app)await app.close(); if(db){await db.query('ROLLBACK').catch(()=>{});db.release();}await pool.end();}
});
