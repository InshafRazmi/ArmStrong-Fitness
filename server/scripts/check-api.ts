import { ApiCheckError, checkApiEndpoint } from '../src/api-check.ts';
import { DesktopSetupError, localSetupEnvironment } from '../src/desktop-setup.ts';

try {
  if (process.argv.length !== 2) throw new ApiCheckError('Use npm run api:check without additional arguments');
  const env = localSetupEnvironment(process.env);
  if (env.PUBLIC_API_ORIGIN === env.SUPABASE_URL) throw new ApiCheckError('Configure the Armstrong API origin separately from Supabase Auth');
  await checkApiEndpoint(env.PUBLIC_API_ORIGIN);
  console.log('Approved HTTPS API process/protocol endpoint: PASS (real HTTP request; values withheld)');
  console.log('No credentials or application data sent. Database readiness, Auth, device enrollment and live sync: not verified');
} catch (error) {
  console.error(error instanceof ApiCheckError || error instanceof DesktopSetupError ? error.message : 'API endpoint check failed; values and provider details withheld');
  console.error('API endpoint check: FAIL; no enrollment or sync verified');
  process.exitCode = 2;
}
