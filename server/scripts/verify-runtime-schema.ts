import pg from 'pg';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';
import { authorizedDatabaseTls, databaseOptions, databaseFailure, DatabaseConfigurationError } from '../src/database.ts';
import { verifyRuntimeSchema, SchemaVerificationError } from '../src/schema-verification.ts';

// Runtime .env only. Never loads .env.test or creates/changes application data.
let client: InstanceType<typeof pg.Client> | undefined;
let transaction = false;
try {
  if (process.argv.length !== 2) throw new DatabaseConfigurationError('Runtime schema verification accepts no arguments');
  let local: ReturnType<typeof parseEnv>;
  try { local = parseEnv(readFileSync('.env', 'utf8')); }
  catch { throw new DatabaseConfigurationError('Cannot read local runtime .env; values withheld'); }
  if (!local.DATABASE_URL || local.DATABASE_URL !== process.env.DATABASE_URL) {
    throw new DatabaseConfigurationError('Runtime database setting missing or overridden; values withheld');
  }
  client = new pg.Client(databaseOptions(local.DATABASE_URL));
  await client.connect();
  const stream = client.connection.stream;
  if (!authorizedDatabaseTls(stream)) throw new DatabaseConfigurationError('Runtime schema verification requires authorized TLS');
  if ((await client.query('SELECT 1 AS connected')).rows[0]?.connected !== 1) throw new DatabaseConfigurationError('Runtime connection probe returned an unexpected result');
  console.log('Runtime PostgreSQL connection and TLS verification: PASS');
  await client.query('BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY');
  transaction = true;
  const result = await verifyRuntimeSchema(client);
  await client.query('COMMIT');
  transaction = false;
  console.log('Runtime migration version 1 checksum: PASS');
  console.log(`Required private tables: PASS (${result.tableCount}/6)`);
  console.log(`Required table RLS: PASS (${result.tableCount}/6)`);
  console.log('Runtime schema verification: PASS (read-only; no gym/member/staff/device data accessed)');
} catch (error) {
  console.error(error instanceof SchemaVerificationError ? error.message : databaseFailure(error, 'Runtime schema verification'));
  const stream = client?.connection.stream;
  console.error(`TLS certificate and hostname verification: ${authorizedDatabaseTls(stream) ? 'PASS' : 'not established'}`);
  process.exitCode = 2;
} finally {
  if (client) {
    if (transaction) { try { await client.query('ROLLBACK'); } catch { console.error('Runtime verification transaction cleanup failed; details withheld'); process.exitCode = 2; } }
    try { await client.end(); } catch { console.error('Runtime verification connection cleanup failed; details withheld'); process.exitCode = 2; }
  }
}
