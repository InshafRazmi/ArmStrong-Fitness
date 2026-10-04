# API and desktop setup checks

Run from `server/` with Node 24 or newer. These tools use the ignored runtime
`.env` only and reject inherited overrides of setup settings. Output contains
status labels and setting names, not paths, IDs, keys, passwords or hashes.

```sh
npm run setup:check
```

This read-only command reports every missing or invalid configuration stage in
one run: public HTTPS/Auth settings, Session-pooler URL/readable CA, approved
gym/Administrator details, probe credential presence, prepared desktop metadata,
and installed desktop public configuration. Exit 2 means setup is incomplete.
A local PASS does not certify real TLS, online identity, OS credential possession,
server registration, database grants or sync. It never creates a SQLite database
or loads `.env.test` as a fallback.

Fill `PUBLIC_API_ORIGIN` with the existing approved HTTPS Fastify origin, and
`REGISTRATION_GYM_ID`, `REGISTRATION_GYM_NAME`, `REGISTRATION_ADMIN_NAME` with
approved installation details. Keep the same gym UUID for retries. Preserve
existing registrations, including unrelated and synthetic records.

## Verify the existing HTTPS API

```sh
npm run api:check
```

The probe sends one credential-free `GET /health` to the exact configured HTTPS
origin. TLS verification stays enabled; redirects, credentials, paths, queries,
fragments and noncanonical origins are refused. The request has a ten-second
deadline, the response is limited to 4 KiB, and errors withhold provider bodies.
It requires this source's process/protocol metadata:

```json
{"status":"ok","service":"armstrong-member-api","protocolVersion":1}
```

A PASS certifies that response over HTTPS only. Complete separate database/Auth/
registration checks and genuine native enrollment before claiming backend or
sync readiness. An older status-only health response requires the current API
source before this probe can pass. This command does not publish a server.

## Provision native public configuration

First use **Prepare this computer** in the actual desktop and unlock its OS
credential store. Add its displayed existing SQLite path and verified hash as
`REGISTRATION_SQLITE_PATH` and `REGISTRATION_DEVICE_SECRET_SHA256`. Follow
[Administrator/device registration](STAFF_DEVICE_SETUP.md). On the workstation
with access to that database, review then provision:

```sh
npm run desktop:config:check
npm run desktop:config:write
npm run setup:check
```

The first command validates without writing. `missing` is a valid review result.
The write command creates `desktop-auth.json` beside the actual, resolved SQLite
file with exactly `authOrigin`, `apiOrigin`, `publishableKey`. It copies no other
env values. New files are mode 0600 where supported. Complete temporary bytes
are flushed and published with an atomic exclusive hard link; unsupported
filesystems fail without replacing a file. Matching retries retain original
bytes. Different, invalid, oversized, duplicate-field, symlink or directory
targets fail for review. Temporary files are removed after publication.
Windows filesystem acceptance is still required.

SQLite is opened read-only. Existing device UUID, native preparation hash,
saved API/gym/device scope and restore marker must validate. No substitute device,
credential rotation, migration, registration or reconciliation bypass takes
place. The marker verifies saved native preparation metadata; actual native
enrollment must prove current possession of the OS credential.

Restart the desktop to load the file. Provisioned settings require native sign-in;
deleting the file cannot remove that durable requirement. Online Administrator
sign-in verifies Auth and approved device enrollment. See
[native sign-in](../../Client/docs/NATIVE_SIGN_IN.md).

## Production database connection

API startup in `NODE_ENV=production` refuses migration-owner and reserved
database roles. Use the separately provisioned restricted runtime connection.
Administrative registration/probes keep their owner connection in a controlled
session. The guard checks reserved names; it does not verify actual attributes
or grants of a custom role. Verify those grants and real application access before
deployment, using the requirements in [README](../README.md).

All configuration consumers require canonical distinct API/Auth origins and
public publishable or legacy anon keys. Privileged keys, empty publishable keys,
malformed legacy payloads and header-injection characters are rejected locally.
Legacy key role decoding validates key syntax only and grants no account authority.
See [Supabase API keys](https://supabase.com/docs/guides/getting-started/api-keys).

No public deployment or live synchronization is activated by these tools.
