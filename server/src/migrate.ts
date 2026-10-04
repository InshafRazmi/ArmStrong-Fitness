import pg from 'pg';
import { applyMigrations } from './migrations.ts';
import { databaseOptions } from './database.ts';
let client: InstanceType<typeof pg.Client> | undefined;
try {
  if (!process.env.DATABASE_URL) throw new Error('DATABASE_URL required for migrations');
  client = new pg.Client(databaseOptions(process.env.DATABASE_URL));
  await client.connect();
  await applyMigrations(client);
  console.log('Migrations applied or already current');
} catch (error) {
  // Driver errors may include connection credentials. Never print their payloads.
  const code = (error as any)?.code;
  const safe = typeof code === 'string' && /^[A-Z0-9_]{3,32}$/.test(code) ? code : 'configuration_or_migration_error';
  console.error(`Migration failed (${safe}); credentials and driver details withheld`);
  process.exitCode = 1;
} finally {
  if (client) { try { await client.end(); } catch { console.error('Migration connection cleanup failed; details withheld'); process.exitCode = 1; } }
}
