// Real isolated Supabase Auth/PostgreSQL. Every synthetic business row is rolled back.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash,randomBytes,randomUUID } from 'node:crypto';
import pg from 'pg';
import type { LightMyRequestResponse } from 'fastify';
import { databaseOptions } from '../src/database.ts';
import { GymService } from '../src/business-service.ts';
import { createApp } from '../src/app.ts';
import { createVerifier } from '../src/auth.ts';
import { digest } from '../src/business-protocol.ts';
import { safeFailure,TestSetupError,supabaseTestConfig } from './support/supabase-config.ts';
import { signInTestAccount } from './support/live-auth.ts';

test('LIVE attendance: gender, staff NFC history, two-device download and permanent removal retain member history',async()=>{
  const config=supabaseTestConfig();
  if(config.authUrl!=='https://srwjyvimdktrvzpdxvpd.supabase.co' || !process.env.ARMSTRONG_ATTENDANCE_FIXTURE_PATH)throw new TestSetupError('Use isolated project and generated native attendance fixture');
  const identity=await signInTestAccount(config.authUrl,config.publishableKey,config.accounts[0]);
  const batches=JSON.parse(await readFile(process.env.ARMSTRONG_ATTENDANCE_FIXTURE_PATH,'utf8')) as any[];
  const pool=new pg.Pool(databaseOptions(config.databaseUrl));let db:pg.PoolClient|undefined;let app:ReturnType<typeof createApp>|undefined;let phase='connection';
  try{
    db=await pool.connect();await db.query('BEGIN');
    assert.equal((await db.query("SELECT count(*)::int AS n FROM pg_catalog.pg_indexes WHERE schemaname='armstrong' AND indexname IN ('active_attendance_card_per_gym','active_staff_card_per_gym')")).rows[0].n,2,'Attendance migration applied');
    const gym=randomUUID(),device=batches[0].deviceId,second=randomUUID(),secret=randomBytes(32).toString('hex');
    await db.query("INSERT INTO armstrong.gyms(id,name) VALUES($1,'Synthetic attendance acceptance')",[gym]);
    await db.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,'Synthetic Administrator','Administrator')",[gym,identity.userId]);
    await db.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true),($1,$4,$3,true)',[gym,device,createHash('sha256').update(secret).digest('hex'),second]);
    const fixturePool={connect:async()=>({query:(sql:string,values?:unknown[])=>db!.query(sql==='BEGIN'?'SAVEPOINT attendance_api_test':sql==='COMMIT'?'RELEASE SAVEPOINT attendance_api_test':sql==='ROLLBACK'?'ROLLBACK TO SAVEPOINT attendance_api_test':sql,values),release:()=>{}})};
    app=createApp(new GymService(fixturePool),createVerifier(config.authUrl,config.publishableKey));
    const headers={authorization:`Bearer ${identity.accessToken}`,'x-gym-id':gym,'x-device-id':device,'x-device-secret':secret};
    const push=(batch:unknown)=>app!.inject({method:'POST',url:'/v2/business/push',headers,payload:batch as any});
    phase='native transaction receipts and identical retries';
    for(const b of batches){const accepted=await push(b);assert.equal(accepted.statusCode,200,accepted.body);assert.equal(accepted.json().requestSha256,digest(b));assert.deepEqual((await push(b)).json(),accepted.json());}
    const all=batches.flatMap(b=>b.changes);const member=all.find(c=>c.table==='members').after,staffCard=all.find(c=>c.table==='staff_nfc_cards').after,memberCard=all.find(c=>c.table==='nfc_cards').after;
    const transaction=(changes:any[])=>({protocolVersion:2,operationId:randomUUID(),deviceId:device,actorSubject:identity.userId,operationIds:[],changes});
    phase='cross-category active card uniqueness and invalid attendance are atomic';
    const duplicate=await push(transaction([{table:'staff_nfc_cards',id:randomUUID(),before:null,after:{...staffCard,id:randomUUID(),uid:memberCard.uid}}]));assert.equal(duplicate.statusCode,400,'Mismatched row identity must fail before writing');
    const copiedId=randomUUID();const denied=await push(transaction([{table:'staff_nfc_cards',id:copiedId,before:null,after:{...staffCard,id:copiedId,uid:memberCard.uid}}]));assert.equal(denied.statusCode,409);assert.equal(denied.json().error,'business_unique_conflict');
    const event=all.find(c=>c.table==='staff_attendance').after;const forgedId=randomUUID();const forged=await push(transaction([{table:'staff_attendance',id:forgedId,before:null,after:{...event,id:forgedId,card_uid:'WRONG'}}]));assert.equal(forged.statusCode,409);assert.equal(forged.json().error,'business_card_conflict');
    phase='permanent removal keeps historical identity, gender and attendance';
    const user={id:identity.userId,subject:identity.userId,email:'synthetic-history@example.invalid',display_name:'Synthetic actor',active:0,version:1};
    const now=new Date().toISOString();
    const removal=transaction([{table:'users',id:user.id,before:null,after:user},{table:'members',id:member.id,before:member,after:{...member,archived_at:now,archived_by_user_id:user.id,version:2}},{table:'nfc_cards',id:memberCard.id,before:memberCard,after:{...memberCard,revoked_at:now}},{table:'member_deletions',id:member.id,before:null,after:{id:member.id,deleted_at:now,actor_user_id:user.id}}]);
    const removed=await push(removal);assert.equal(removed.statusCode,200,removed.body);assert.deepEqual((await push(removal)).json(),removed.json());
    const counts=(await db.query("SELECT table_name,count(*)::int AS n FROM armstrong.business_records WHERE gym_id=$1 GROUP BY table_name",[gym])).rows;
    for(const name of ['members','member_profiles','member_deletions','attendance','staff_attendance'])assert.equal(counts.find(c=>c.table_name===name)?.n,1,name+' retained');
    phase='second device downloads the deletion and both attendance categories';let cursor=0;const downloaded:any[]=[];
    for(let i=0;i<=batches.length+1;i++){const r:LightMyRequestResponse=await app.inject({url:`/v2/business/changes?after=${cursor}`,headers:{...headers,'x-device-id':second}});assert.equal(r.statusCode,200,r.body);const p=r.json();downloaded.push(...p.changes);cursor=p.nextCursor;if(!p.hasMore)break;}
    assert.equal(downloaded.length,batches.length+1);assert.ok(downloaded.flatMap(e=>e.request.changes).some(c=>c.table==='member_deletions'));
    phase='PostgreSQL protects attendance and shared active-card uniqueness';await db.query('SAVEPOINT attendance_sql');
    await assert.rejects(db.query("UPDATE armstrong.business_records SET data=jsonb_set(data,'{staff_name}','\"Changed\"') WHERE gym_id=$1 AND table_name='staff_attendance'",[gym]),/immutable/);await db.query('ROLLBACK TO SAVEPOINT attendance_sql');
    await db.query('SAVEPOINT attendance_card_sql');const sqlCard=randomUUID();await assert.rejects(db.query("INSERT INTO armstrong.business_records(gym_id,table_name,record_id,data) VALUES($1,'nfc_cards',$2,$3)",[gym,sqlCard,JSON.stringify({...memberCard,id:sqlCard,uid:staffCard.uid})]),/unique/);await db.query('ROLLBACK TO SAVEPOINT attendance_card_sql');
  }catch(error){if(error instanceof assert.AssertionError)throw error;throw safeFailure(error,`Live attendance integration at ${phase}`);}
  finally{if(app)await app.close();if(db){await db.query('ROLLBACK').catch(()=>{});db.release();}await pool.end();}
});
