import { missingTestKeys, supabaseTestConfig, safeFailure } from '../test/support/supabase-config.ts';
import { databaseOptions } from '../src/database.ts';
const missing = missingTestKeys();
for (const key of missing) console.log(`${key}: missing`);
try {
  databaseOptions(supabaseTestConfig().databaseUrl);
  console.log('Isolated Supabase test configuration present and locally validated; values hidden. No network or database verification performed.');
} catch (error) {
  console.error(safeFailure(error, 'Test configuration').message);
  process.exitCode = 2;
}
