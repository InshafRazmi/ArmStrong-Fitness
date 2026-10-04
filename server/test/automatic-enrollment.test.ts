import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID,createHash } from 'node:crypto';
import { MemberService } from '../src/service.ts';
import { ApiError } from '../src/protocol.ts';

test('automatic enrollment / SQL mock: sends only verified subject, local device and hash to restricted function', async () => {
  const user=randomUUID(),device=randomUUID(),secret='a'.repeat(64);
  const enrollment={protocolVersion:1,gym:{id:randomUUID(),name:'Test gym'},staff:{id:user,name:'Admin',role:'Administrator'},device:{id:device,canWrite:false}};
  let releases=0,queries=0;
  const service=new MemberService({connect:async()=>({query:async(sql:string,args:unknown[])=>{
    queries++; assert.equal(sql,'SELECT armstrong.enroll_desktop($1::uuid,$2::uuid,$3::text) AS enrollment');
    assert.deepEqual(args,[user,device,createHash('sha256').update(secret).digest('hex')]);
    assert.ok(!args.includes(secret)); return {rows:[{enrollment}]};
  },release:()=>{releases++;}})},true);
  assert.deepEqual(await service.enroll(user,{protocolVersion:1,deviceId:device,deviceSecret:secret}),enrollment);
  assert.equal(queries,1); assert.equal(releases,1);
  for (const extra of [{gymId:randomUUID()},{canWrite:true},{role:'Administrator'}]) {
    await assert.rejects(service.enroll(user,{protocolVersion:1,deviceId:device,deviceSecret:secret,...extra}),ApiError);
  }
  await assert.rejects(service.enroll('invalid',{protocolVersion:1,deviceId:device,deviceSecret:secret}),ApiError);
  assert.equal(queries,1);
});
test('automatic enrollment / SQL mock: SQL access denial is redacted and connections are released',async()=>{
  let released=false;
  const service=new MemberService({connect:async()=>({query:async()=>{throw Object.assign(new Error('private driver details'),{code:'42501'});},release:()=>{released=true;}})},true);
  await assert.rejects(service.enroll(randomUUID(),{protocolVersion:1,deviceId:randomUUID(),deviceSecret:'a'.repeat(64)}),(error:unknown)=>error instanceof ApiError && error.status===403 && !error.message.includes('private'));
  assert.ok(released);
});
