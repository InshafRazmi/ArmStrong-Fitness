import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { parseOperation, cursor } from '../src/protocol.ts';
import { config, runtimeConfig } from '../src/config.ts';
const databaseUrl = 'postgresql://postgres.abcdefghijklmnopqrst:fixture-password@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres?sslmode=verify-full&sslrootcert=certs/test-ca.pem';
const operation = () => ({ protocolVersion:1, operationId:randomUUID(), deviceId:randomUUID(), memberId:randomUUID(), action:'create', expectedRevision:0, member:{name:' Member ',phone:'0771234567',email:'',nfcId:' abCd ',joinedOn:'2026-10-03'} });
test('normalizes native ASCII cards and empty optional fields', () => {
  const parsed=parseOperation(operation());assert.equal(parsed.member!.nfcId,'ABCD');assert.equal(parsed.member!.name,'Member');
  const input=operation();input.member.nfcId=' ';assert.equal(parseOperation(input).member!.nfcId,null);
});
test('rejects invalid dates, UUIDs, unsafe revisions, unknown identities and nonmember actions', () => {
  for (const patch of [{operationId:'bad'}, {protocolVersion:2}, {action:'delete'}, {action:'payment'}, {expectedRevision:1}, {expectedRevision:2**53}, {actorUserId:randomUUID()}]) assert.throws(()=>parseOperation({...operation(),...patch}));
  for(const patch of [{joinedOn:'2026-02-30'},{joinedOn:'1899-12-31'},{name:''},{phone:''},{email:'bad email@example.com'},{nfcId:'ß'},{nfcId:'AB CD'},{nfcId:'ABC\u0000'}]) assert.throws(()=>parseOperation({...operation(),member:{...operation().member,...patch}}));
});
test('archive cannot submit a client actor or arbitrary member snapshot', () => {
  const archive={...operation(),action:'archive',expectedRevision:2,member:null};assert.equal(parseOperation(archive).member,null);
  assert.throws(()=>parseOperation({...archive,member:operation().member}));assert.throws(()=>parseOperation({...archive,actor:'Admin'}));
});
test('cursor rejects malformed, negative, fractional and unsafe positions',()=>{
  assert.equal(cursor('0'),0);assert.equal(cursor('12'),12);
  for(const c of [undefined,null,0,'-1','1.5','01','1e3','9007199254740992']) assert.throws(()=>cursor(c));
});
test('configuration fails closed for missing credentials or unverified database TLS',()=>{
  assert.throws(()=>config({}));
  const env={DATABASE_URL:databaseUrl.replace('verify-full','require'),SUPABASE_URL:'https://abcdefghijklmnopqrst.supabase.co',SUPABASE_PUBLISHABLE_KEY:'sb_publishable_fixture'};
  assert.throws(()=>config(env));assert.equal(config({...env,DATABASE_URL:databaseUrl}).port,3000);
});
test('production configuration requires canonical HTTPS API and identity origins',()=>{
  const env={DATABASE_URL:databaseUrl.replace('postgres.','armstrong_api.'),SUPABASE_URL:'https://abcdefghijklmnopqrst.supabase.co',SUPABASE_PUBLISHABLE_KEY:'sb_publishable_fixture',NODE_ENV:'production'};
  assert.throws(()=>config(env));
  for(const origin of ['http://host','https://u:p@host','https://host/path','https://host?token=secret','https://host/#token']) assert.throws(()=>config({...env,PUBLIC_API_ORIGIN:origin}));
  assert.equal(config({...env,PUBLIC_API_ORIGIN:'https://test-api.onrender.com'}).apiOrigin,'https://test-api.onrender.com');
  for(const origin of ['http://host','https://u:p@host','https://host/path','https://host?key=secret','https://host/#token']) assert.throws(()=>config({...env,PUBLIC_API_ORIGIN:'https://test-api.onrender.com',SUPABASE_URL:origin}));
});
test('production configuration refuses migration owners and built-in roles; no database connections',()=>{
  const env={DATABASE_URL:databaseUrl,SUPABASE_URL:'https://abcdefghijklmnopqrst.supabase.co',SUPABASE_PUBLISHABLE_KEY:'sb_publishable_fixture',PUBLIC_API_ORIGIN:'https://api.example.invalid',NODE_ENV:'production'};
  for(const role of ['postgres','anon','authenticated','authenticator','service_role','dashboard_user','pgbouncer','supabase_admin','pg_read_all_data']) {
    assert.throws(()=>runtimeConfig({...env,DATABASE_URL:databaseUrl.replace('postgres.',role+'.')}),/restricted database runtime login/);
  }
  assert.equal(runtimeConfig({...env,DATABASE_URL:databaseUrl.replace('postgres.','armstrong_api.')}).apiOrigin,env.PUBLIC_API_ORIGIN);
  assert.equal(config(env).port,3000); // Administrative checks keep owner access even in a production shell.
  assert.equal(runtimeConfig({...env,NODE_ENV:'development'}).port,3000);
});
test('Render startup uses only the verified-form platform origin and preserves explicit configuration; no network',()=>{
  const env={DATABASE_URL:databaseUrl.replace('postgres.','armstrong_api.'),SUPABASE_URL:'https://abcdefghijklmnopqrst.supabase.co',SUPABASE_PUBLISHABLE_KEY:'sb_publishable_fixture',NODE_ENV:'production',RENDER:'true',RENDER_EXTERNAL_URL:'https://assigned-api.onrender.com'};
  assert.equal(runtimeConfig(env).apiOrigin,env.RENDER_EXTERNAL_URL);
  assert.equal(runtimeConfig({...env,PUBLIC_API_ORIGIN:'https://api.example.invalid'}).apiOrigin,'https://api.example.invalid');
  assert.throws(()=>runtimeConfig({...env,RENDER:'false'}),/PUBLIC_API_ORIGIN/);
  assert.throws(()=>runtimeConfig({...env,RENDER:undefined}),/PUBLIC_API_ORIGIN/);
  assert.throws(()=>runtimeConfig({...env,RENDER_EXTERNAL_URL:undefined}),/PUBLIC_API_ORIGIN/);
  for(const origin of ['http://assigned-api.onrender.com','https://assigned-api.onrender.com/','https://user:password@assigned-api.onrender.com','https://assigned-api.onrender.com/path','https://assigned-api.onrender.com?token=private','https://assigned-api.onrender.com#private','https://assigned-api.onrender.com:8443','https://onrender.com','https://assigned-api.onrender.com.example.invalid','https://example.invalid']) {
    assert.throws(()=>runtimeConfig({...env,RENDER_EXTERNAL_URL:origin}));
  }
  assert.throws(()=>runtimeConfig({...env,PUBLIC_API_ORIGIN:'http://api.example.invalid'}));
  assert.throws(()=>runtimeConfig({...env,DATABASE_URL:databaseUrl}),/restricted database runtime login/);
});
test('runtime configuration rejects privileged Auth keys and identical API/Auth endpoints before network',()=>{
  const env={DATABASE_URL:databaseUrl,SUPABASE_URL:'https://abcdefghijklmnopqrst.supabase.co',SUPABASE_PUBLISHABLE_KEY:'sb_publishable_fixture'};
  const legacy=(role:string)=>`header.${Buffer.from(JSON.stringify({role})).toString('base64url')}.signature`;
  for(const key of ['sb_secret_private','sb_publishable_',legacy('service_role'),'private-invalid-key']) assert.throws(()=>config({...env,SUPABASE_PUBLISHABLE_KEY:key}),/publishable or legacy anon key/);
  assert.equal(config({...env,SUPABASE_PUBLISHABLE_KEY:legacy('anon')}).publishableKey,legacy('anon'));
  assert.throws(()=>config({...env,PUBLIC_API_ORIGIN:env.SUPABASE_URL}),/separately/);
});
