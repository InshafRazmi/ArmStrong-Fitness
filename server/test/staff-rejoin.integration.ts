// Real isolated Auth/Postgres; requires migration 10 and every fixture rolls back.
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

test('LIVE staff rejoining: NIC/card reuse, saved money/attendance, retries and deferred SQL uniqueness', async () => {
  const config=supabaseTestConfig();
  if(config.authUrl!=='https://srwjyvimdktrvzpdxvpd.supabase.co'||!process.env.ARMSTRONG_STAFF_REJOIN_FIXTURE_PATH) throw new TestSetupError('Use isolated project and native staff-rejoining fixture');
  const identity=await signInTestAccount(config.authUrl,config.publishableKey,config.accounts[0]);
  const batches=JSON.parse(await readFile(process.env.ARMSTRONG_STAFF_REJOIN_FIXTURE_PATH,'utf8')) as any[];
  const db=new pg.Client(databaseOptions(config.databaseUrl));
  let app:ReturnType<typeof createApp>|undefined;
  let phase='connection';
  try {
    await db.connect();await db.query('BEGIN');
    phase='staff NIC schema check';
    // Schema DDL can lock Auth's user table through shared database event
    // triggers. Apply migrations before this transaction; only fixtures roll back.
    assert.equal((await db.query("SELECT count(*)::int AS n FROM pg_catalog.pg_trigger WHERE tgrelid='armstrong.business_records'::regclass AND tgname='staff_nic_unique' AND tgdeferrable AND tginitdeferred AND tgenabled='O'")).rows[0].n,1,'Apply staff NIC migration to the isolated project first');
    const gym=randomUUID(),device=batches[0].deviceId,secret=randomBytes(32).toString('hex');
    await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Synthetic staff rejoining')",[gym]);
    await db.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Synthetic Administrator','Administrator')",[gym,identity.userId]);
    await db.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true)',[gym,device,createHash('sha256').update(secret).digest('hex')]);
    const pool={connect:async()=>({query:async(sql:string,values?:unknown[])=>{
      if(sql==='COMMIT') { await db.query('SET CONSTRAINTS ALL IMMEDIATE');await db.query('SET CONSTRAINTS ALL DEFERRED');return db.query('RELEASE SAVEPOINT rejoin_api'); }
      return db.query(sql==='BEGIN'?'SAVEPOINT rejoin_api':sql==='ROLLBACK'?'ROLLBACK TO SAVEPOINT rejoin_api':sql,values);
    },release:()=>{}})};
    app=createApp(new GymService(pool),createVerifier(config.authUrl,config.publishableKey));
    const headers={authorization:`Bearer ${identity.accessToken}`,'x-gym-id':gym,'x-device-id':device,'x-device-secret':secret};
    phase='native sync and retry';
    for(const [index,batch] of batches.entries()) {
      const reply:LightMyRequestResponse=await app!.inject({method:'POST',url:'/v2/business/push',headers,payload:batch});
      assert.equal(reply.statusCode,200,`Batch ${index+1}: ${reply.body}`);
      assert.equal(reply.json().requestSha256,digest(batch));
      assert.deepEqual((await app.inject({method:'POST',url:'/v2/business/push',headers,payload:batch})).json(),reply.json());
    }
    const records=(await db.query('SELECT table_name,record_id,data FROM armstrong.business_records WHERE gym_id=$1',[gym])).rows;
    const staff=records.filter(r=>r.table_name==='trainers');
    assert.equal(staff.length,2);assert.equal(staff[0].data.nic,staff[1].data.nic);
    const removed=records.find(r=>r.table_name==='staff_deletions').record_id;
    const fresh=staff.find(r=>r.record_id!==removed)!.data;
    assert.equal(fresh.name,'Rejoined staff');assert.equal(fresh.active,1);
    assert.equal(records.filter(r=>r.table_name==='staff_payouts').length,1);
    const events=records.filter(r=>r.table_name==='staff_attendance');
    assert.equal(events.length,2);assert.deepEqual(new Set(events.map(r=>r.data.trainer_id)),new Set(staff.map(r=>r.record_id)));
    assert.ok(records.filter(r=>r.table_name==='training_charges').every(r=>r.data.trainer_id===removed));
    phase='deferred PostgreSQL duplicate guard';
    await db.query('SAVEPOINT duplicate_nic');
    const duplicate={...fresh,id:randomUUID(),active:0};
    await db.query("INSERT INTO armstrong.business_records VALUES($1,'trainers',$2,$3)",[gym,duplicate.id,JSON.stringify(duplicate)]);
    await assert.rejects(db.query('SET CONSTRAINTS ALL IMMEDIATE'),{code:'23505'});
    await db.query('ROLLBACK TO SAVEPOINT duplicate_nic');
    assert.equal((await db.query("SELECT count(*)::int AS n FROM armstrong.business_records WHERE gym_id=$1 AND table_name='trainers'",[gym])).rows[0].n,2);
    phase='second-device history download';
    let cursor=0;const downloaded:any[]=[];
    for(let i=0;i<=batches.length;i++) {const reply:LightMyRequestResponse=await app!.inject({url:`/v2/business/changes?after=${cursor}`,headers});assert.equal(reply.statusCode,200);const page=reply.json();downloaded.push(...page.changes);cursor=page.nextCursor;if(!page.hasMore)break;}
    assert.equal(downloaded.length,batches.length);
  } catch(error) {if(error instanceof assert.AssertionError)throw error;throw safeFailure(error,`Staff rejoining at ${phase}`);}
  finally {if(app)await app.close();await db.query('ROLLBACK').catch(()=>{});await db.end();}
});
