import pg from 'pg';
import { authorizedDatabaseTls, databaseOptions, databaseFailure, DatabaseConfigurationError } from '../src/database.ts';
import { supabaseTestConfig, safeFailure } from '../test/support/supabase-config.ts';
import { readIsolationState, readApplicationSchemaObjects } from '../test/support/isolated-database.ts';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';

// Read-only probes: no migration, fixtures, member queries or API listener.
const inspect = process.argv[2] === 'test-schema';
const test = process.argv[2] === 'test' || inspect;
let client: InstanceType<typeof pg.Client> | undefined;
let connected = false;
try {
  if (process.argv.length > 3 || (process.argv[2] !== undefined && !['test', 'test-schema'].includes(process.argv[2]))) throw new DatabaseConfigurationError('Invalid probe mode; use npm run db:check, npm run test:db or npm run test:inspect');
  if (inspect) {
    let local: ReturnType<typeof parseEnv>;
    try { local = parseEnv(readFileSync('.env.test', 'utf8')); }
    catch { throw new DatabaseConfigurationError('Cannot inspect local .env.test; values withheld'); }
    for (const key of ['TEST_DATABASE_URL', 'TEST_SUPABASE_PROJECT_REF', 'TEST_SUPABASE_URL']) {
      if (local[key] !== process.env[key]) throw new DatabaseConfigurationError(`${key} does not match local .env.test; inherited override or missing setting; values withheld`);
    }
    console.log('Test target settings match local .env.test: PASS (values withheld)');
  }
  const value = test ? supabaseTestConfig().databaseUrl : process.env.DATABASE_URL;
  if (!value) throw new DatabaseConfigurationError('Configure runtime DATABASE_URL in server/.env');
  client = new pg.Client(databaseOptions(value));
  await client.connect();
  const stream = client.connection.stream;
  if (!authorizedDatabaseTls(stream)) throw new DatabaseConfigurationError('Database probe requires an authorized TLS socket');
  connected = true;
  const result = await client.query('SELECT 1 AS connected');
  if (result.rows[0]?.connected !== 1) throw new DatabaseConfigurationError('Database probe did not return the expected result');
  console.log(`${test ? 'Isolated test' : 'Runtime'} PostgreSQL probe PASS: real Session pooler connection, authorized TLS and SELECT 1. No application data accessed.`);
  if (inspect) {
    const state = await readIsolationState(client);
    console.log(`Application schema present: ${Boolean(state.app_schema)}`);
    console.log(`Migration ledger present: ${Boolean(state.ledger)}`);
    console.log(`Public relations present: ${Boolean(state.public_relations)}`);
    if (state.app_schema) {
      const counts = await readApplicationSchemaObjects(client);
      console.log(`Application schema relation objects: ${counts.relations}`);
      console.log(`Application schema routine objects: ${counts.routines}`);
      console.log(`Application schema type objects: ${counts.types}`);
      console.log(`Application schema dependency records: ${counts.dependencies}`);
    }
    const empty = !state.app_schema && !state.ledger && !state.public_relations;
    console.log(`Empty application database prerequisite: ${empty ? 'PASS' : 'FAIL (existing database objects; no changes made)'}`);
    if (!empty) process.exitCode = 2;
  }
} catch (error) {
  console.error(test ? safeFailure(error, 'Isolated test database probe').message : databaseFailure(error, 'Runtime database probe'));
  // PostgreSQL can reject login after a successful TLS handshake. Report the
  // socket's actual state independently from authenticated SQL success.
  const stream = client?.connection.stream;
  const tlsVerified = authorizedDatabaseTls(stream);
  console.error(`TLS certificate and hostname verification: ${tlsVerified ? 'PASS' : 'not established'}`);
  if ((error as { code?: unknown })?.code === '28P01') console.error(`Database authentication failed; check the ${test ? 'isolated test' : 'runtime'} project database password and percent encoding. Credential values withheld.`);
  console.error(`Verified database connection established: ${connected ? 'yes' : 'no'}`);
  process.exitCode = 2;
} finally {
  if (client) { try { await client.end(); } catch { console.error('Database probe cleanup failed; details withheld'); process.exitCode = 2; } }
}
