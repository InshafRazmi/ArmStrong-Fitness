// Uses real PostgreSQL transactions and Fastify injection. Never substitutes an
// in-memory database. AUTH IS MOCKED: this is not live Supabase verification.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID, randomBytes, createHash } from 'node:crypto';
import { applyMigrations } from '../src/migrations.ts';
import { lockEmptyDatabase } from './support/isolated-database.ts';
import { safeFailure, TestSetupError } from './support/supabase-config.ts';
import { databaseOptions, sessionDatabaseUrl } from '../src/database.ts';
test('REAL PostgreSQL / MOCK Auth: member API replay, races, conflicts, archive and cursors', {skip: !process.env.TEST_DATABASE_URL}, async()=>{
  try {
  if (process.env.TEST_PROJECT_IS_DISPOSABLE !== 'true') throw new TestSetupError('TEST_PROJECT_IS_DISPOSABLE must be true for the PostgreSQL/Auth-mock suite');
  const ref = process.env.TEST_SUPABASE_PROJECT_REF;
  if (!ref || !/^[a-z0-9]{10,64}$/.test(ref)) throw new TestSetupError('Configure TEST_SUPABASE_PROJECT_REF for isolated PostgreSQL tests');
  sessionDatabaseUrl(process.env.TEST_DATABASE_URL!, ref, true);
  const {default:pg}=await import('pg');const {createApp}=await import('../src/app.ts');const {MemberService}=await import('../src/service.ts');
  const pool=new pg.Pool(databaseOptions(process.env.TEST_DATABASE_URL!));
  let app:any;
  let migration:any;
  try {
    migration=await pool.connect();
    await lockEmptyDatabase(migration);
    await applyMigrations(migration);
    const gym=randomUUID(),user=randomUUID(),device=randomUUID(),secret=randomBytes(32).toString('hex');
    await pool.query('INSERT INTO armstrong.gyms(id,name) VALUES($1,\'Test gym\')',[gym]);
    await pool.query('INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES($1,$2,\'Test staff\',\'Administrator\')',[gym,user]);
    await pool.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true)',[gym,device,createHash('sha256').update(secret).digest('hex')]);
    app=createApp(new MemberService(pool),async(header)=>{if(header!=='Bearer test-only-token'){const {ApiError}=await import('../src/protocol.ts');throw new ApiError(401,'authentication_required');}return user;});
    const headers={authorization:'Bearer test-only-token','x-gym-id':gym,'x-device-id':device,'x-device-secret':secret};
    const enrollment={protocolVersion:1,deviceId:device,deviceSecret:secret};
    const enroll=()=>app.inject({method:'POST',url:'/v1/enrollment',headers:{authorization:headers.authorization},payload:enrollment});
    const enrolled=await enroll();assert.equal(enrolled.statusCode,200);
    assert.deepEqual(enrolled.json(),{protocolVersion:1,gym:{id:gym,name:'Test gym'},staff:{id:user,name:'Test staff',role:'Administrator'},device:{id:device,canWrite:true}});
    assert.deepEqual((await enroll()).json(),enrolled.json());
    assert.equal((await app.inject({method:'POST',url:'/v1/enrollment',headers:{authorization:headers.authorization},payload:{...enrollment,gymId:gym}})).statusCode,400);
    const op={protocolVersion:1,operationId:randomUUID(),deviceId:device,memberId:randomUUID(),action:'create',expectedRevision:0,member:{name:'Test member',phone:'0771234567',email:'',nfcId:'abc',joinedOn:'2026-10-03'}};
    const push=(payload:any,h=headers)=>app.inject({method:'POST',url:'/v1/members/push',headers:h,payload});
    assert.equal((await app.inject({method:'POST',url:'/v1/members/push',payload:op})).statusCode,401);
    assert.equal((await push(op,{...headers,'x-device-secret':'0'.repeat(64)})).statusCode,403);
    const responses=await Promise.all([push(op),push(op)]);assert.equal(responses[0].statusCode,200);assert.deepEqual(responses[0].json(),responses[1].json());
    assert.equal((await pool.query('SELECT count(*) FROM armstrong.member_changes')).rows[0].count,'1');
    assert.equal((await push({...op,member:{...op.member,name:'Changed'}})).statusCode,409);
    assert.equal((await push({...op,operationId:randomUUID(),memberId:randomUUID()})).statusCode,409);
    const update={...op,operationId:randomUUID(),action:'update',expectedRevision:1,member:{...op.member,name:'Updated'}};
    const races=await Promise.all([push(update),push({...update,operationId:randomUUID()})]);assert.deepEqual(races.map(r=>r.statusCode).sort(),[200,409]);
    await pool.query("UPDATE armstrong.staff SET role='Reception' WHERE gym_id=$1 AND user_id=$2",[gym,user]);
    const archive={...op,operationId:randomUUID(),action:'archive',expectedRevision:2,member:null};assert.equal((await push(archive)).statusCode,403);
    await pool.query("UPDATE armstrong.staff SET role='Administrator' WHERE gym_id=$1 AND user_id=$2",[gym,user]);
    const archived=await push(archive);assert.equal(archived.statusCode,200);assert.equal(archived.json().member.archivedBy.id,user);
    assert.equal((await push({...update,operationId:randomUUID(),expectedRevision:3})).statusCode,409);
    const pulled=await app.inject({url:'/v1/members/changes?after=0',headers});assert.equal(pulled.statusCode,200);assert.deepEqual(pulled.json().changes.map((c: { sequence: number })=>c.sequence),[1,2,3]);assert.equal(pulled.json().nextCursor,3);
    assert.equal((await app.inject({url:'/v1/members/changes?after=100',headers})).statusCode,409);
    assert.equal((await app.inject({url:'/v1/members/changes?after=0',headers:{...headers,'x-gym-id':randomUUID()}})).statusCode,403);
    const unrelatedGym=randomUUID();
    await pool.query('INSERT INTO armstrong.gyms(id,name) VALUES($1,\'Unrelated gym\')',[unrelatedGym]);
    await pool.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES($1,$2,$3,true)',[unrelatedGym,device,createHash('sha256').update(secret).digest('hex')]);
    // Even a real gym and registered device secret do not substitute for verified
    // staff membership. The client header is only an assertion, never authority.
    assert.equal((await app.inject({url:'/v1/members/changes?after=0',headers:{...headers,'x-gym-id':unrelatedGym}})).statusCode,403);
    await pool.query('UPDATE armstrong.staff SET active=false WHERE gym_id=$1 AND user_id=$2',[gym,user]);assert.equal((await push(op)).statusCode,403);assert.equal((await enroll()).statusCode,403);
  } finally {if(app)await app.close();if(migration)migration.release();await pool.end();}
  // Schema intentionally retained for inspection; destroy the disposable database externally.
  } catch(error) {throw safeFailure(error,'PostgreSQL/Auth-mock integration');}
});
