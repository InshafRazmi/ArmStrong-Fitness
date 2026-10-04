import { TestSetupError } from './supabase-config.ts';

export async function readIsolationState(client: { query(sql: string): Promise<any> }) {
  return (await client.query(`SELECT
    EXISTS(SELECT 1 FROM pg_namespace WHERE nspname='armstrong') AS app_schema,
    to_regclass('public.armstrong_migrations') IS NOT NULL AS ledger,
    EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
      WHERE n.nspname='public' AND c.relkind IN ('r','p','v','m','f')) AS public_relations`)).rows[0];
}

// Metadata only: count schema objects without selecting application rows or
// exposing object names. Dependency records include objects beyond these three
// catalog categories; a zero count never bypasses the existing-schema guard.
export async function readApplicationSchemaObjects(client: { query(sql: string): Promise<any> }) {
  const row = (await client.query(`SELECT
    (SELECT count(*)::integer FROM pg_catalog.pg_class c
      JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
      WHERE n.nspname='armstrong') AS relations,
    (SELECT count(*)::integer FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='armstrong') AS routines,
    (SELECT count(*)::integer FROM pg_catalog.pg_type t
      JOIN pg_catalog.pg_namespace n ON n.oid=t.typnamespace
      WHERE n.nspname='armstrong') AS types,
    (SELECT count(*)::integer FROM pg_catalog.pg_depend d
      JOIN pg_catalog.pg_namespace n ON n.oid=d.refobjid
      WHERE d.refclassid='pg_catalog.pg_namespace'::regclass
        AND n.nspname='armstrong') AS dependencies`)).rows[0];
  const counts: Record<'relations' | 'routines' | 'types' | 'dependencies', number> = {
    relations: row?.relations, routines: row?.routines,
    types: row?.types, dependencies: row?.dependencies
  };
  if (Object.values(counts).some(value => !Number.isSafeInteger(value) || value < 0)) {
    throw new TestSetupError('Invalid schema metadata counts; details withheld');
  }
  return counts;
}

// Hold this connection until the suite ends. The lock prevents two test commands
// from racing their empty-database check. No schema reset or data deletion.
export async function lockEmptyDatabase(client: { query(sql: string): Promise<any> }) {
  const lock = await client.query('SELECT pg_try_advisory_lock(714339805) AS acquired');
  if (!lock.rows[0].acquired) throw new TestSetupError('Another Armstrong integration suite holds the test database lock');
  const state = await readIsolationState(client);
  if (state.app_schema || state.ledger || state.public_relations) throw new TestSetupError(`Refusing existing application schema, migration ledger or public relations; use a fresh disposable test project/database (application schema: ${state.app_schema ? 'present' : 'absent'}, migration ledger: ${state.ledger ? 'present' : 'absent'}, public relations: ${state.public_relations ? 'present' : 'absent'})`);
}
