import { readdir, readFile } from 'node:fs/promises';

const assets = new URL('../dist/assets/', import.meta.url);
const files = await readdir(assets);
const scripts = files.filter(name => name.endsWith('.js'));
if (!scripts.length || files.some(name => /BrowserGymProvider/.test(name))) {
  throw new Error('Packaged frontend must exclude browser demo data and storage.');
}
const bundle = (await Promise.all(scripts.map(name => readFile(new URL(name, assets), 'utf8')))).join('\n');
for (const marker of ['arm123', 'armstrong-demo-auth', 'Browser prototype', 'simulated sync', 'localStorage']) {
  if (bundle.includes(marker)) throw new Error('Packaged frontend contains browser demo access.');
}
if (!bundle.includes('Gym records remain locked') || !bundle.includes('desktop_login')) {
  throw new Error('Packaged frontend requires the native bridge and verified login.');
}
console.log('PASS packaged frontend: native bridge/login required; browser demo excluded.');
