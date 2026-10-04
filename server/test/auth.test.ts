import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { createVerifier } from '../src/auth.ts';
test('requires a bearer token before contacting the identity provider',async()=>{
  const verify=createVerifier('https://project.supabase.co','key',async()=>{throw new Error('should not run');});
  for(const header of [undefined,'','Bearer','Bearer a\nb','Basic abc']) await assert.rejects(verify(header),{code:'authentication_required'});
});
test('uses authenticated provider identity and never accepts a locally decoded subject',async()=>{
  const subject=randomUUID();let calls=0;
  const verify=createVerifier('https://project.supabase.co','public',async(url,options)=>{
    calls++;assert.equal(String(url),'https://project.supabase.co/auth/v1/user');assert.equal(new Headers(options!.headers).get('authorization'),'Bearer valid.token');assert.equal(options!.redirect,'error');
    return new Response(JSON.stringify({id:subject}),{status:200});
  });assert.equal(await verify('Bearer valid.token'),subject);assert.equal(calls,1);
});
test('invalid, expired and unavailable identity responses fail closed',async()=>{
  for(const status of [401,403,429,500]) {
    const verify=createVerifier('https://project.supabase.co','public',async()=>new Response('{}',{status}));
    await assert.rejects(verify('Bearer token'),{code:status===401||status===403?'invalid_session':'identity_unavailable'});
  }
  await assert.rejects(createVerifier('https://project.supabase.co','public',async()=>new Response('{"id":"forged"}'))('Bearer token'),{code:'invalid_identity_response'});
  await assert.rejects(createVerifier('https://project.supabase.co','public',async()=>{throw new Error('timeout')})('Bearer token'),{code:'identity_unavailable'});
  assert.throws(()=>createVerifier('http://project.supabase.co','public'));
});
