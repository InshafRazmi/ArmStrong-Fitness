import pg from 'pg';
import { runtimeConfig } from './config.ts';
import { createApp } from './app.ts';
import { createVerifier } from './auth.ts';
import { MemberService } from './service.ts';
import { databaseOptions, databaseFailure } from './database.ts';
let pool: InstanceType<typeof pg.Pool> | undefined;
let app: ReturnType<typeof createApp> | undefined;
try {
  const c = runtimeConfig();
  pool = new pg.Pool({ ...databaseOptions(c.databaseUrl), max: 10 });
  pool.on('error', (error: unknown) => console.error(databaseFailure(error, 'Idle database connection')));
  app = createApp(new MemberService(pool), createVerifier(c.supabaseUrl, c.publishableKey));
  // A process health response must not precede the first real database connection.
  await pool.query('SELECT 1');
  for (const signal of ['SIGINT','SIGTERM']) process.once(signal, async () => {
    try { await app!.close(); await pool!.end(); }
    catch (error) { console.error(databaseFailure(error, 'API shutdown')); process.exitCode = 1; }
  });
  await app.listen({ host: c.host, port: c.port });
} catch (error) {
  console.error(databaseFailure(error, 'API startup'));
  if (app) { try { await app.close(); } catch {} }
  if (pool) { try { await pool.end(); } catch {} }
  process.exitCode = 1;
}
