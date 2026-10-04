// Local configuration/driver tests only. No real PostgreSQL or Supabase access.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { rootCertificates, TLSSocket } from 'node:tls';
import { Socket } from 'node:net';
import pg from 'pg';
import ConnectionParameters from 'pg/lib/connection-parameters';
import { authorizedDatabaseTls, databaseOptions, sessionDatabaseUrl, databaseFailure } from '../src/database.ts';

const fixture = (cert: string) => `postgresql://postgres.abcdefghijklmnopqrst:synthetic%40password@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres?sslmode=verify-full&sslrootcert=${encodeURIComponent(cert)}`;

test('database local/driver: CA and mandatory verification survive pg option parsing; no network', () => {
  const dir = mkdtempSync(join(tmpdir(), 'armstrong-ca-test-'));
  try {
    const cert = join(dir, 'fixture.pem');
    writeFileSync(cert, rootCertificates[0]); // Public trust root; no private key.
    const options = databaseOptions(fixture(cert));
    const client = new pg.Client(options); // Construction only; never connect.
    // Runtime inspection of the actual installed driver's private property.
    // Keep this test independent of additions to upstream public declarations.
    assert.ok('connectionParameters' in client && client.connectionParameters instanceof ConnectionParameters);
    const parameters = client.connectionParameters;
    assert.ok(parameters.ssl && typeof parameters.ssl === 'object');
    assert.equal(options.ssl.rejectUnauthorized, true);
    assert.equal(parameters.ssl.rejectUnauthorized, true);
    assert.equal(parameters.ssl.ca, rootCertificates[0]);
    assert.equal(parameters.ssl.checkServerIdentity, undefined);
    assert.equal(parameters.port, 5432);
    assert.equal(parameters.password, 'synthetic@password');
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('database local socket classification: rejects absent, plaintext, look-alike and unauthorized sockets; no TLS handshake', () => {
  const plaintext = new Socket();
  const tls = new TLSSocket(new Socket());
  try {
    for (const stream of [undefined, null, {}, {encrypted:true,authorized:true}, plaintext]) {
      assert.equal(authorizedDatabaseTls(stream), false);
    }
    Object.assign(plaintext, {encrypted:true,authorized:true});
    assert.equal(authorizedDatabaseTls(plaintext), false);
    assert.equal(authorizedDatabaseTls(tls), false);
    // Explicit synthetic status fixture: this is classification, not live TLS evidence.
    tls.authorized = true;
    assert.equal(authorizedDatabaseTls(tls), true);
    tls.authorized = false;
    assert.equal(authorizedDatabaseTls(tls), false);
  } finally { plaintext.destroy(); tls.destroy(); }
});

test('database local: rejects TLS weakening, URI overrides, direct/transaction endpoints and wrong project', () => {
  const value = fixture('fixture.pem');
  for (const invalid of [
    value.replace('verify-full', 'require'), value.replace('5432', '6543'),
    value + '&sslmode=disable', value + '&sslrootcert=other.pem',
    value + '&host=other', value + '&user=other', value + '&uselibpqcompat=true',
    value.replace('aws-0-ap-southeast-1.pooler.supabase.com', 'db.abcdefghijklmnopqrst.supabase.co'),
    value.replace('&sslrootcert=fixture.pem', ''), 'malformed-synthetic-password'
  ]) assert.throws(() => sessionDatabaseUrl(invalid));
  assert.throws(() => sessionDatabaseUrl(value, 'differentprojectrefxx', true));
  assert.doesNotThrow(() => sessionDatabaseUrl(value, 'abcdefghijklmnopqrst', true));
  assert.doesNotThrow(() => sessionDatabaseUrl(value.replace('postgres.', 'runtime_role.'), 'abcdefghijklmnopqrst'));
});

test('database local: unreadable or malformed CA fails without disclosing URL, credentials or file path', () => {
  const dir = mkdtempSync(join(tmpdir(), 'armstrong-invalid-ca-'));
  try {
    const cert = join(dir, 'invalid.pem'); writeFileSync(cert, 'not a certificate');
    for (const path of [cert, join(dir, 'missing.pem')]) assert.throws(() => databaseOptions(fixture(path)), error => {
      const message = databaseFailure(error, 'Local fixture');
      assert.ok(message.includes('readable PEM certificate'));
      assert.ok(!message.includes(path)); assert.ok(!message.includes('synthetic'));
      return true;
    });
    const failure = databaseFailure(Object.assign(new Error('synthetic-secret'), { code: 'EAI_AGAIN' }), 'Fixture');
    assert.ok(failure.includes('EAI_AGAIN')); assert.ok(!failure.includes('synthetic-secret'));
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
