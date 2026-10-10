import { readFile, mkdir, copyFile, mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';

const client = fileURLToPath(new URL('../', import.meta.url));
function run(command, args, cwd = client, extra = {}) {
  const result = spawnSync(command, args, { cwd, env: { ...process.env, ...extra }, stdio: 'inherit' });
  if (result.error || result.status !== 0) throw new Error('Arch build command failed; no final release is claimed.');
}
try {
  if (process.platform !== 'linux' || process.arch !== 'x64') throw new Error('Build this package on Arch Linux x86_64.');
  const os = await readFile('/etc/os-release', 'utf8');
  if (!/^ID=arch$/m.test(os)) throw new Error('Build this package on Arch Linux x86_64.');
  const { version } = JSON.parse(await readFile(resolve(client, 'package.json'), 'utf8'));
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error('Invalid package version.');
  run(process.execPath, ['scripts/windows-installer.mjs', '--check']);
  run('npm', ['run', 'build:desktop']);
  const config = JSON.parse(await readFile(resolve(client, 'src-tauri/tauri.linux.conf.json'), 'utf8'));
  run('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--release', '--locked', '--features', 'desktop,custom-protocol,packaged-auth'], client,
    { TAURI_CONFIG: JSON.stringify(config), CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS || '2', CARGO_INCREMENTAL: '0' });
  const output = resolve(client, 'dist-linux', `v${version}`);
  await mkdir(output, { recursive: true });
  const staging = await mkdtemp(join(tmpdir(), 'armstrong-arch-'));
  for (const [source, target] of [
    ['src-tauri/target/release/armstrong-desktop', 'armstrong-fitness'],
    ['packaging/arch/PKGBUILD', 'PKGBUILD'], ['packaging/arch/armstrong-fitness.desktop', 'armstrong-fitness.desktop'],
    ['src-tauri/icons/icon.png', 'icon.png'], ['docs/ARCH_LINUX.md', 'README.md'],
  ]) await copyFile(resolve(client, source), join(staging, target));
  run('strip', ['--strip-unneeded', 'armstrong-fitness'], staging);
  run('desktop-file-validate', ['armstrong-fitness.desktop'], staging);
  const hashes=[];
  for (const name of ['armstrong-fitness','armstrong-fitness.desktop','icon.png','README.md']) hashes.push(createHash('sha256').update(await readFile(join(staging,name))).digest('hex'));
  const recipe = (await readFile(join(staging,'PKGBUILD'),'utf8')).replace(/^pkgver=.*$/m, `pkgver=${version}`).replace(/^sha256sums=.*$/m,`sha256sums=(${hashes.map(value=>`'${value}'`).join(' ')})`);
  await writeFile(join(staging,'PKGBUILD'),recipe);
  run('makepkg', ['--nodeps', '--clean', '--force'], staging, { PKGDEST: output });
  const name = `armstrong-fitness-${version}-1-x86_64.pkg.tar.zst`;
  const bytes = await readFile(join(output, name));
  await writeFile(join(output, 'SHA256SUMS'), `${createHash('sha256').update(bytes).digest('hex')}  ${name}\n`);
  console.log(`Built ${name}. This is an acceptance build; remaining release gates are documented in ARCH_LINUX.md.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : 'Arch build failed; details withheld.');
  process.exitCode = 1;
}
