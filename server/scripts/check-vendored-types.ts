import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

const base = new URL('../vendor/types-pg/', import.meta.url);
const manifest = JSON.parse(await readFile(new URL('SOURCE.json', base), 'utf8'));
const upstream = JSON.parse(await readFile(new URL('UPSTREAM.package.json', base), 'utf8'));
const packaged = JSON.parse(await readFile(new URL('package.json', base), 'utf8'));
const files = ['index.d.ts', 'index.d.mts', 'lib/connection-parameters.d.ts', 'lib/type-overrides.d.ts', 'UPSTREAM.package.json'];

assert.equal(manifest.repository, 'https://github.com/DefinitelyTyped/DefinitelyTyped');
assert.match(manifest.commit, /^[a-f0-9]{40}$/);
assert.deepEqual(manifest.files.map((entry: {file: string}) => entry.file).sort(), [...files].sort());
for (const entry of manifest.files) {
  const bytes = await readFile(new URL(entry.file, base));
  assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, `Changed source: ${entry.file}`);
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'), entry.gitBlob, `Git blob differs: ${entry.file}`);
}
const license = await readFile(new URL('LICENSE', base));
assert.equal(createHash('sha256').update(license).digest('hex'), manifest.license.sha256);
assert.equal(packaged.name, '@types/pg');
assert.equal(packaged.version, upstream.version);
assert.equal(manifest.upstreamVersion, upstream.version);
assert.equal(packaged.license, 'MIT');
assert.equal(packaged.types, 'index.d.ts');
assert.deepEqual(packaged.exports, upstream.exports);
assert.deepEqual(packaged.dependencies, upstream.dependencies);
assert.equal(packaged.scripts, undefined);
assert.equal(packaged.devDependencies, undefined);
console.log('PASS pinned pg declaration sources, license and packaging');
