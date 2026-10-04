// Local configuration and REAL temporary SQLite metadata only; NOT live sync,
// PostgreSQL, Auth, secret-store or native desktop enrollment acceptance.
import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
import { mkdtempSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { registrationConfig } from '../src/registration-config.ts';

const gymId = randomUUID(), deviceId = randomUUID();
const settings = () => ({ REGISTRATION_GYM_ID: gymId, REGISTRATION_GYM_NAME: 'Synthetic gym', REGISTRATION_ADMIN_NAME: 'Synthetic Admin' });
function fixture(run: (path: string, database: DatabaseSync) => void) {
  const dir = mkdtempSync(join(tmpdir(), 'armstrong-registration-'));
  const path = join(dir, 'fixture.sqlite3');
  const database = new DatabaseSync(path);
  database.exec('CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL)');
  database.prepare('INSERT INTO metadata VALUES(?,?)').run('device_id', deviceId);
  database.prepare('INSERT INTO metadata VALUES(?,?)').run('native_device_secret_sha256', 'a'.repeat(64));
  try { run(path, database); }
  finally { database.close(); rmSync(dir, { recursive: true, force: true }); }
}
const deviceSettings = (path: string) => ({ ...settings(), REGISTRATION_SQLITE_PATH: path, REGISTRATION_DEVICE_SECRET_SHA256: 'a'.repeat(64) });

test('registration config / local: Admin-only setup and missing names use no default identity or device', () => {
  assert.deepEqual(registrationConfig(settings()), { gymId, gymName: 'Synthetic gym', adminName: 'Synthetic Admin' });
  assert.throws(() => registrationConfig({}), /REGISTRATION_GYM_ID, REGISTRATION_GYM_NAME, REGISTRATION_ADMIN_NAME/);
  for (const patch of [{ REGISTRATION_GYM_ID: 'bad' }, { REGISTRATION_ADMIN_NAME: ' ' }, { REGISTRATION_GYM_NAME: 'x'.repeat(121) }]) assert.throws(() => registrationConfig({ ...settings(), ...patch }));
  for (const patch of [{ REGISTRATION_SQLITE_PATH: 'private-path' }, { REGISTRATION_DEVICE_SECRET_SHA256: 'a'.repeat(64) }, { REGISTRATION_SQLITE_PATH: 'private-path', REGISTRATION_DEVICE_SECRET_SHA256: 'private-value' }]) {
    assert.throws(() => registrationConfig({ ...settings(), ...patch }), error => {
      assert.ok(!String(error).includes('private-path') && !String(error).includes('private-value')); return true;
    });
  }
});

test('registration config / real temporary SQLite: preserves persisted device ID and database bytes', () => {
  fixture((path) => {
    const before = readFileSync(path);
    const registered = registrationConfig(deviceSettings(path));
    assert.deepEqual(registered.device, { id: deviceId, secretSha256: 'a'.repeat(64) });
    assert.deepEqual(readFileSync(path), before);
  });
});

test('registration config / real temporary SQLite: restored or invalid identity cannot be approved', () => {
  fixture((path, database) => {
    database.prepare('INSERT INTO metadata VALUES(?,?)').run('restore_requires_reconciliation', 'true');
    assert.throws(() => registrationConfig(deviceSettings(path)), /reconciliation/);
    database.prepare("DELETE FROM metadata WHERE key='restore_requires_reconciliation'").run();
    database.prepare("UPDATE metadata SET value=? WHERE key='device_id'").run('fixture-private-invalid-id');
    assert.throws(() => registrationConfig(deviceSettings(path)), error => {
      assert.ok(!String(error).includes(path) && !String(error).includes('fixture-private-invalid-id')); return true;
    });
  });
});

test('registration config / real temporary SQLite: existing scope must match HTTPS API, gym and persisted device', () => {
  fixture((path, database) => {
    const origin = 'https://api.example.invalid';
    const scope = { serverOrigin: origin, gymId, deviceId };
    const put = (value: unknown) => database.prepare("INSERT INTO metadata VALUES('member_sync_scope',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value").run(JSON.stringify(value));
    put(scope);
    assert.equal(registrationConfig(deviceSettings(path), origin).device!.id, deviceId);
    for (const bad of [{ ...scope, gymId: randomUUID() }, { ...scope, deviceId: randomUUID() }, { ...scope, serverOrigin: 'https://other.example.invalid' }, { ...scope, role: 'Administrator' }, null]) {
      put(bad); assert.throws(() => registrationConfig(deviceSettings(path), origin));
    }
    put(scope); assert.throws(() => registrationConfig(deviceSettings(path)), /reconciliation/);
  });
});

test('registration config / local SQLite: missing path never creates a database or prints its path', () => {
  const dir = mkdtempSync(join(tmpdir(), 'armstrong-registration-missing-'));
  try {
    const path = join(dir, 'missing.sqlite3');
    assert.throws(() => registrationConfig(deviceSettings(path)), error => { assert.ok(!String(error).includes(path)); return true; });
    assert.equal(existsSync(path), false);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('registration config / real temporary SQLite: device hash must have been verified natively', () => {
  fixture((path, database) => {
    const before = readFileSync(path);
    assert.throws(() => registrationConfig({...deviceSettings(path),REGISTRATION_DEVICE_SECRET_SHA256:'b'.repeat(64)}), /verified by native device preparation/);
    assert.deepEqual(readFileSync(path),before);
    database.prepare("DELETE FROM metadata WHERE key='native_device_secret_sha256'").run();
    assert.throws(() => registrationConfig(deviceSettings(path)), /verified by native device preparation/);
    database.prepare('INSERT INTO metadata VALUES(?,?)').run('native_device_secret_sha256','private-invalid-hash');
    assert.throws(() => registrationConfig(deviceSettings(path)), error => {
      assert.ok(!String(error).includes('private-invalid-hash') && !String(error).includes(path)); return true;
    });
  });
});
