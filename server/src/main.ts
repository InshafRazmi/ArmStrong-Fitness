import pg from "pg";
import { runtimeConfig } from "./config.ts";
import { createApp } from "./app.ts";
import { createVerifier } from "./auth.ts";
import { MemberService } from "./service.ts";
import { databaseOptions, databaseFailure } from "./database.ts";
let pool: InstanceType<typeof pg.Pool> | undefined;
let app: ReturnType<typeof createApp> | undefined;
try {
  const c = runtimeConfig();
  pool = new pg.Pool({ ...databaseOptions(c.databaseUrl), max: 10 });
  pool.on("error", (error: unknown) =>
    console.error(databaseFailure(error, "Idle database connection")),
  );
  app = createApp(
    new MemberService(pool, c.automaticEnrollment),
    createVerifier(c.supabaseUrl, c.publishableKey),
  );

  await pool.query("SELECT 1");
  if (c.automaticEnrollment) {
    const permission = await pool.query("SELECT has_function_privilege(current_user,'armstrong.enroll_desktop(uuid,uuid,text)','EXECUTE') AS allowed");
    if (permission.rows[0]?.allowed !== true) throw new Error('Automatic device enrollment schema or runtime permission is missing');
  }
  for (const signal of ["SIGINT", "SIGTERM"])
    process.once(signal, async () => {
      try {
        await app!.close();
        await pool!.end();
      } catch (error) {
        console.error(databaseFailure(error, "API shutdown"));
        process.exitCode = 1;
      }
    });
  await app.listen({ host: c.host, port: c.port });
} catch (error) {
  console.error(databaseFailure(error, "API startup"));
  if (app) {
    try {
      await app.close();
    } catch {}
  }
  if (pool) {
    try {
      await pool.end();
    } catch {}
  }
  process.exitCode = 1;
}
