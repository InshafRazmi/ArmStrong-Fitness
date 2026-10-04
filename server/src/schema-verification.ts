import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';

export class SchemaVerificationError extends Error {}

const requiredTables = ['gyms', 'staff', 'devices', 'members', 'member_operations', 'member_changes'];
type MetadataClient = { query(sql: string, values?: unknown[]): Promise<{ rows: Record<string, unknown>[] }> };

// Catalog and migration metadata only. The caller holds a read-only transaction;
// no application records, schema changes, fixture writes or enrollment occur.
export async function verifyRuntimeSchema(client: MetadataClient) {
  const state = (await client.query(`SELECT
    EXISTS(SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname='armstrong') AS app_schema,
    EXISTS(SELECT 1 FROM pg_catalog.pg_class c
      JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
      WHERE n.nspname='public' AND c.relname='armstrong_migrations'
        AND c.relkind='r') AS ledger`)).rows[0];
  if (state?.app_schema !== true || state?.ledger !== true) {
    throw new SchemaVerificationError('Runtime application schema or migration ledger missing; no changes made');
  }
  const expectedHash = createHash('sha256').update(await readFile(new URL('../migrations/001_members.sql', import.meta.url), 'utf8')).digest('hex');
  const migration = (await client.query('SELECT sha256 FROM public.armstrong_migrations WHERE version=1')).rows;
  if (migration.length !== 1 || migration[0]?.sha256 !== expectedHash) {
    throw new SchemaVerificationError('Runtime migration version 1 missing or checksum mismatch; values withheld; no changes made');
  }
  const tables = (await client.query(`SELECT c.relname AS name, c.relrowsecurity AS rls
    FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
    WHERE n.nspname='armstrong' AND c.relkind='r' AND c.relname=ANY($1::text[])`, [requiredTables])).rows;
  const byName = new Map(tables.map(row => [row.name, row.rls]));
  if (tables.length !== requiredTables.length || byName.size !== requiredTables.length || requiredTables.some(name => !byName.has(name))) {
    throw new SchemaVerificationError('Runtime required private tables missing; no changes made');
  }
  if (requiredTables.some(name => byName.get(name) !== true)) {
    throw new SchemaVerificationError('Runtime required table RLS disabled; no changes made');
  }
  return { tableCount: requiredTables.length, checksumMatches: true, rlsEnabled: true };
}
