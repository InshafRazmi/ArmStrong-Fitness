// Existing isolated project, real Supabase Auth/PostgreSQL, all fixture rows rolled back.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash, randomBytes, randomUUID } from 'node:crypto';
import pg from 'pg';
import type { LightMyRequestResponse } from 'fastify';
import { databaseOptions, sessionDatabaseUrl } from '../src/database.ts';
import { GymService } from '../src/business-service.ts';
import { createApp } from '../src/app.ts';
import { createVerifier } from '../src/auth.ts';
import { digest } from '../src/business-protocol.ts';
import { safeFailure, TestSetupError, supabaseTestConfig } from './support/supabase-config.ts';
import { signInTestAccount } from './support/live-auth.ts';

test('LIVE Supabase Auth/PostgreSQL: native staff billing, combined collection, salary/payout guards and two-device access', async () => {
  if (!process.env.TEST_DATABASE_URL || process.env.TEST_SUPABASE_PROJECT_REF !== 'srwjyvimdktrvzpdxvpd' || !process.env.ARMSTRONG_STAFF_FIXTURE_PATH) throw new TestSetupError('Use the existing isolated project and generated native staff fixture; no production fallback');
  sessionDatabaseUrl(process.env.TEST_DATABASE_URL,process.env.TEST_SUPABASE_PROJECT_REF,true);
  const config=supabaseTestConfig();
  const identity=await signInTestAccount(config.authUrl,config.publishableKey,config.accounts[0]);
  const verify=createVerifier(config.authUrl,config.publishableKey);
  assert.equal(await verify(`Bearer ${identity.accessToken}`),identity.userId);
  const batches=JSON.parse(await readFile(process.env.ARMSTRONG_STAFF_FIXTURE_PATH,'utf8')) as any[];
  const pool=new pg.Pool(databaseOptions(process.env.TEST_DATABASE_URL));
  let db: pg.PoolClient | undefined; let app: ReturnType<typeof createApp> | undefined;
  let phase='connection and isolated fixture';
  try {
    db=await pool.connect();await db.query('BEGIN');
    assert.equal((await db.query("SELECT count(*)::int AS n FROM pg_catalog.pg_indexes WHERE schemaname='armstrong' AND indexname='trainer_nic_per_gym'")).rows[0].n,1,'Apply staff migration to isolated project first');
    const gym=randomUUID(),device=batches[0].deviceId,second=randomUUID(),secret=randomBytes(32).toString('hex');
    await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Synthetic staff acceptance')",[gym]);
    await db.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Synthetic Administrator','Administrator')",[gym,identity.userId]);
    await db.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true),($1,$4,$3,false)',[gym,device,createHash('sha256').update(secret).digest('hex'),second]);
    const fixturePool={connect:async()=>({query:(sql:string,values?:unknown[])=>db!.query(sql==='BEGIN'?'SAVEPOINT staff_api_test':sql==='COMMIT'?'RELEASE SAVEPOINT staff_api_test':sql==='ROLLBACK'?'ROLLBACK TO SAVEPOINT staff_api_test':sql,values),release:()=>{}})};
    app=createApp(new GymService(fixturePool),verify);
    const headers={authorization:`Bearer ${identity.accessToken}`,'x-gym-id':gym,'x-device-id':device,'x-device-secret':secret};
    const push=(batch:unknown,deviceId=device)=>app!.inject({method:'POST',url:'/v2/business/push',headers:{...headers,'x-device-id':deviceId},payload:batch as any});
    for(const batch of batches){
      phase='native staff transaction and exact receipt replay';
      const accepted=await push(batch);assert.equal(accepted.statusCode,200,accepted.body);assert.equal(accepted.json().requestSha256,digest(batch));assert.deepEqual((await push(batch)).json(),accepted.json());
    }
    phase='second Administrator device downloads complete shared billing history';
    let cursor=0;const downloaded:any[]=[];
    for(let i=0;i<=batches.length;i++){
      const response: LightMyRequestResponse=await app.inject({url:`/v2/business/changes?after=${cursor}`,headers:{...headers,'x-device-id':second}});
      assert.equal(response.statusCode,200,response.body);const page=response.json();downloaded.push(...page.changes);cursor=page.nextCursor;if(!page.hasMore)break;
    }
    assert.equal(downloaded.length,batches.length);
    const all=downloaded.flatMap(e=>e.request.changes);
    assert.ok(all.some(c=>c.table==='staff_payout_items'));
    const trainer=all.find(c=>c.table==='trainers').after;
    phase='second device edits staff, first device stale write rejected';
    const change={table:'trainers',id:trainer.id,before:trainer,after:{...trainer,name:'Synthetic updated trainer',version:2}};
    const edit={protocolVersion:2,operationId:randomUUID(),deviceId:second,actorSubject:identity.userId,operationIds:[],changes:[change]};
    const edited=await push(edit,second);assert.equal(edited.statusCode,200,edited.body);assert.deepEqual((await push(edit,second)).json(),edited.json());
    const stale=await push({...edit,operationId:randomUUID(),deviceId:device});assert.equal(stale.statusCode,409);assert.equal(stale.json().error,'business_revision_conflict');
    phase='duplicate salary month refused with no partial expense';
    const payout=all.find(c=>c.table==='staff_payouts').after,expense=all.find(c=>c.table==='expenses').after;
    const expenseId=randomUUID(),payoutId=randomUUID();
    const duplicate={protocolVersion:2,operationId:randomUUID(),deviceId:second,actorSubject:identity.userId,operationIds:[],changes:[
      {table:'expenses',id:expenseId,before:null,after:{...expense,id:expenseId,amount_minor:payout.salary_minor}},
      {table:'staff_payouts',id:payoutId,before:null,after:{...payout,id:payoutId,expense_id:expenseId,training_minor:0}},
    ]};
    const denied=await push(duplicate,second);assert.equal(denied.statusCode,409);assert.equal(denied.json().error,'business_unique_conflict');
    assert.equal((await db.query('SELECT count(*)::int AS n FROM armstrong.business_records WHERE gym_id=$1 AND record_id=$2',[gym,expenseId])).rows[0].n,0);
    phase='PostgreSQL immutable payout and NIC uniqueness';
    await db.query('SAVEPOINT staff_sql_guard');
    await assert.rejects(db.query("UPDATE armstrong.business_records SET data=jsonb_set(data,'{training_minor}','1') WHERE gym_id=$1 AND table_name='staff_payouts'",[gym]),/immutable/);
    await db.query('ROLLBACK TO SAVEPOINT staff_sql_guard');
    await db.query('SAVEPOINT staff_nic_guard');
    const copiedTrainerId=randomUUID();
    await assert.rejects(db.query("INSERT INTO armstrong.business_records(gym_id,table_name,record_id,data) VALUES($1,'trainers',$2,$3)",[gym,copiedTrainerId,JSON.stringify({...trainer,id:copiedTrainerId})]),/unique/);
    await db.query('ROLLBACK TO SAVEPOINT staff_nic_guard');
    const persisted=(await db.query("SELECT data FROM armstrong.business_records WHERE gym_id=$1 AND table_name='training_charges'",[gym])).rows[0].data;
    assert.equal(persisted.fee_minor,500025,'Past training rate remains unchanged by staff edits');
  } catch(error){if(error instanceof assert.AssertionError)throw error;throw safeFailure(error,`Live staff integration failed at ${phase}; values withheld`);}
  finally{if(app)await app.close();if(db){await db.query('ROLLBACK').catch(()=>{});db.release();}await pool.end();}
});
