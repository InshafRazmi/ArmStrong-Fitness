import { DesktopSetupError, localSetupEnvironment, provisionDesktopConfig } from '../src/desktop-setup.ts';
import { RegistrationError } from '../src/registration.ts';
import { PublicAuthConfigurationError } from '../src/public-auth-config.ts';

try {
  const mode = process.argv[2];
  if (process.argv.length !== 3 || !['check', 'write'].includes(mode)) throw new DesktopSetupError('Use npm run desktop:config:check or npm run desktop:config:write');
  const state = provisionDesktopConfig(localSetupEnvironment(process.env), mode === 'write');
  console.log(`Desktop public configuration: ${state} (beside the verified existing database; paths and values withheld)`);
  console.log(mode === 'check' ? 'Configuration review: PASS (read-only; no file was created or replaced)' : 'Configuration provisioning: PASS (matching file retained; existing files never replaced)');
  console.log('Restart loads native sign-in settings. OS credentials, Administrator/device approval, real login and live sync: not verified');
} catch (error) {
  console.error(error instanceof PublicAuthConfigurationError || error instanceof RegistrationError ? error.message : 'Desktop configuration failed; paths and values withheld');
  console.error('Configuration command: FAIL; no live enrollment or sync verified');
  process.exitCode = 2;
}
