import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';

// Same transactional/checksummed algorithm used by the CLI and integration tests.
// Caller supplies a dedicated migration-owner connection, never an API transaction.
export async function applyMigrations(client: { query(sql: string, values?: unknown[]): Promise<any> }) {
  await client.query('BEGIN');
  try {
    await client.query('SELECT pg_advisory_xact_lock(714339804)');
    await client.query('CREATE TABLE IF NOT EXISTS public.armstrong_migrations(version integer PRIMARY KEY,sha256 text NOT NULL,applied_at timestamptz NOT NULL DEFAULT now())');
    for (const [version, file] of [[1,'../migrations/001_members.sql'],[2,'../supabase/migrations/20261004213002_desktop_onboarding.sql'],[3,'../supabase/migrations/20261004231412_business_sync.sql'],[4,'../supabase/migrations/20261004234248_business_history_guards.sql'],[5,'../supabase/migrations/20261008015730_administrator_device_access.sql']] as const) {
      const sql = await readFile(new URL(file, import.meta.url), 'utf8');
      const hash = createHash('sha256').update(sql).digest('hex');
      const previous = await client.query('SELECT sha256 FROM public.armstrong_migrations WHERE version=$1', [version]);
      if (previous.rowCount && previous.rows[0].sha256 !== hash) throw new Error('Migration checksum mismatch');
      if (!previous.rowCount) {
        await client.query(sql);
        await client.query('INSERT INTO public.armstrong_migrations(version,sha256) VALUES($1,$2)', [version,hash]);
      }
    }
    await client.query('COMMIT');
  } catch (error) {
    await client.query('ROLLBACK');
    throw error;
  }
}
