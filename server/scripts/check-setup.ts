import { DesktopSetupError, localSetupEnvironment, setupReadiness } from '../src/desktop-setup.ts';
import { PublicAuthConfigurationError } from '../src/public-auth-config.ts';

try {
  if (process.argv.length !== 2) throw new DesktopSetupError('Use npm run setup:check without additional arguments');
  const checks = setupReadiness(localSetupEnvironment(process.env));
  for (const check of checks) console.log(`${check.step}: ${check.status} — ${check.detail}`);
  const ready = checks.every(check => check.status === 'PASS');
  console.log(`Local setup configuration: ${ready ? 'PASS' : 'INCOMPLETE'} (read-only; values withheld)`);
  console.log('Runtime database grants, real TLS/OS credential storage, Auth, API enrollment and live sync: not verified');
  process.exitCode = ready ? 0 : 2;
} catch (error) {
  console.error(error instanceof PublicAuthConfigurationError ? error.message : 'Setup inspection failed; details withheld');
  process.exitCode = 2;
}
