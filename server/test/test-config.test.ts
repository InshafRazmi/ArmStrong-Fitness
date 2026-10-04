// Test setup/guard unit tests with fake configuration and SQL rows. NOT live proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import { supabaseTestConfig, missingTestKeys, safeFailure, testKeys } from './support/supabase-config.ts';
import { lockEmptyDatabase, readApplicationSchemaObjects } from './support/isolated-database.ts';
const ref = 'abcdefghijklmnopqrst';
const config = () => ({
  TEST_PROJECT_IS_DISPOSABLE: 'true', TEST_SUPABASE_PROJECT_REF: ref,
  TEST_DATABASE_URL: `postgresql://postgres.${ref}:fixture-db-password@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres?sslmode=verify-full&sslrootcert=certs/test-ca.pem`,
  TEST_SUPABASE_URL: `https://${ref}.supabase.co`, TEST_SUPABASE_PUBLISHABLE_KEY: 'sb_publishable_fixture',
  TEST_ADMIN_EMAIL: 'admin@example.invalid', TEST_ADMIN_PASSWORD: 'fixture-admin-password',
  TEST_RECEPTION_EMAIL: 'reception@example.invalid', TEST_RECEPTION_PASSWORD: 'fixture-reception-password',
  TEST_OTHER_GYM_EMAIL: 'other@example.invalid', TEST_OTHER_GYM_PASSWORD: 'fixture-other-password'
});
test('test configuration unit: reports only missing TEST names and never falls back to runtime credentials', () => {
  assert.deepEqual(missingTestKeys({}), [...testKeys]);
  assert.throws(() => supabaseTestConfig({ DATABASE_URL: 'runtime-private-password', SUPABASE_URL: 'runtime-url' }), error => {
    assert.ok(String(error).includes('TEST_DATABASE_URL')); assert.ok(!String(error).includes('runtime-private-password')); return true;
  });
});
test('test configuration unit: accepts matching Session pooler owner with verified TLS; local validation only', () => {
  assert.equal(supabaseTestConfig(config()).accounts.length, 3);
  const env = config();
  env.TEST_DATABASE_URL = `postgresql://postgres.${ref}:fixture-db-password@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres?sslmode=verify-full&sslrootcert=certs/test-ca.pem`;
  assert.equal(supabaseTestConfig(env).authUrl, env.TEST_SUPABASE_URL);
});
test('test configuration unit: rejects mismatched projects, weak TLS, transaction pooling and URI overrides', () => {
  for (const patch of [
    { TEST_PROJECT_IS_DISPOSABLE: 'false' }, { NODE_TLS_REJECT_UNAUTHORIZED: '0' },
    { TEST_SUPABASE_URL: 'https://differentprojectrefxx.supabase.co' },
    { TEST_SUPABASE_URL: `http://${ref}.supabase.co` },
    { TEST_SUPABASE_URL: `https://${ref}.supabase.co/path` },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL.replace(ref, 'differentprojectrefxx') },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL.replace('verify-full', 'require') },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL.replace('5432', '6543') },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL + '&host=other-host' },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL + '&sslmode=disable' },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL + '&options=secret' },
    { TEST_DATABASE_URL: `postgresql://postgres:fixture-db-password@db.${ref}.supabase.co:5432/postgres?sslmode=verify-full&sslrootcert=certs/test-ca.pem` },
    { TEST_DATABASE_URL: config().TEST_DATABASE_URL.replace('&sslrootcert=certs/test-ca.pem', '') }
  ]) assert.throws(() => supabaseTestConfig({ ...config(), ...patch }));
});
test('test configuration unit: rejects privileged keys, duplicate identities and malformed URLs without leaking values', () => {
  const key = (role: string) => `header.${Buffer.from(JSON.stringify({ role })).toString('base64url')}.signature`;
  for (const keyValue of ['sb_secret_private', key('service_role')]) assert.throws(() => supabaseTestConfig({ ...config(), TEST_SUPABASE_PUBLISHABLE_KEY: keyValue }));
  assert.doesNotThrow(() => supabaseTestConfig({ ...config(), TEST_SUPABASE_PUBLISHABLE_KEY: key('anon') }));
  assert.throws(() => supabaseTestConfig({ ...config(), TEST_ADMIN_EMAIL: 'reception@example.invalid' }));
  assert.throws(() => supabaseTestConfig({ ...config(), TEST_DATABASE_URL: 'private-password-invalid-url' }), error => {
    assert.ok(!String(error).includes('private-password')); return true;
  });
  const failed = safeFailure(Object.assign(new Error('private-password'), { code: 'EAI_AGAIN' }), 'Fixture connection');
  assert.ok(failed.message.includes('EAI_AGAIN')); assert.ok(!failed.message.includes('private-password'));
});
test('database isolation unit / SQL mock: refuses concurrent runs and existing schema/ledger/public relations before writes', async () => {
  for (const state of [{ app_schema: true }, { ledger: true }, { public_relations: true }]) {
    const sql: string[] = [];
    await assert.rejects(lockEmptyDatabase({ async query(query) { sql.push(query); return { rows: [sql.length === 1 ? { acquired: true } : state] }; } }));
    assert.ok(sql.every(query => query.startsWith('SELECT')));
  }
  let calls = 0;
  await assert.rejects(lockEmptyDatabase({ async query() { calls++; return { rows: [{ acquired: false }] }; } }));
  assert.equal(calls, 1);
  await lockEmptyDatabase({ async query(query) { return { rows: [query.includes('pg_try_advisory_lock') ? { acquired: true } : { app_schema: false, ledger: false, public_relations: false }] }; } });
});
test('schema inspection unit / SQL mock: returns only numeric metadata and refuses misleading or private results', async () => {
  for (const counts of [
    { relations: 0, routines: 0, types: 0, dependencies: 0 },
    { relations: 7, routines: 2, types: 14, dependencies: 23 }
  ]) {
    const result = await readApplicationSchemaObjects({ async query() { return { rows: [{ ...counts, private_value: 'fixture-secret' }] }; } });
    assert.deepEqual(result, counts);
    assert.ok(!JSON.stringify(result).includes('fixture-secret'));
  }
  for (const row of [undefined, { relations: 'fixture-secret', routines: 0, types: 0, dependencies: 0 },
    { relations: 0, routines: 0, types: 0, dependencies: -1 }]) {
    await assert.rejects(readApplicationSchemaObjects({ async query() { return { rows: [row] }; } }), error => {
      assert.equal((error as Error).message, 'Invalid schema metadata counts; details withheld');
      return true;
    });
  }
});
