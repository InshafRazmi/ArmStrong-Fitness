# Isolated Supabase backend integration

The live suite now includes the administrative registration helper against its
own synthetic fixtures: read-only review, exact apply retry, and refusal of
implicit role/gym/device-hash replacement. This step uses the suite's already
online-verified subjects and isolated SQL owner; it never loads runtime `.env`
or calls the runtime registration CLI. It has been syntax/type-reviewed but has
not passed live execution. New `registration*.test.ts` tests use SQL mocks and
real temporary SQLite metadata only, and must not be reported as this live pass.

Use a **new disposable Supabase test project**, never a production project or real
gym/member data. No project creation, deployment or desktop sync is performed by
these commands. The suite keeps synthetic fixtures for inspection and refuses to
reuse an existing application database; it does not reset/drop a schema.

## Local configuration

Create `server/.env.test` in your editor and restrict its permissions with
`chmod 600 server/.env.test` from the repository root. `.gitignore` excludes this
file and every populated `.env.*`; only `.env.example` is allowed. Do not paste
credentials in chat, commit them, or run commands that print the env file.
Runtime `.env` and runtime `DATABASE_URL`/`SUPABASE_*` are never test fallbacks.

Required variables, all in `.env.test`:

| Variable | Meaning |
| --- | --- |
| `TEST_PROJECT_IS_DISPOSABLE` | Exactly `true`, asserting a dedicated disposable test project. |
| `TEST_SUPABASE_PROJECT_REF` | Reference of that test project; matched against Auth and SQL connections. |
| `TEST_DATABASE_URL` | Test-project **migration-owner** Session pooler URL on 5432, database `postgres`, `sslmode=verify-full` and a readable PEM CA in `sslrootcert`. Percent-encode the password in the URI. |
| `TEST_SUPABASE_URL` | Exactly `https://<test-project-ref>.supabase.co`, with no credentials/path/query/fragment. |
| `TEST_SUPABASE_PUBLISHABLE_KEY` | Test project's `sb_publishable_...` key or legacy `anon` key; never a secret/service-role key. |
| `TEST_ADMIN_EMAIL` | First synthetic, confirmed test Auth account. |
| `TEST_ADMIN_PASSWORD` | Its password. |
| `TEST_RECEPTION_EMAIL` | Second synthetic, confirmed test Auth account. |
| `TEST_RECEPTION_PASSWORD` | Its password. |
| `TEST_OTHER_GYM_EMAIL` | Third synthetic, confirmed test Auth account. |
| `TEST_OTHER_GYM_PASSWORD` | Its password. |

Create those three accounts in the isolated project's Auth dashboard beforehand.
They must be distinct, confirmed email/password accounts suitable for password
sign-in. The project must contain **only those three Auth accounts**. The suite
does not create or delete Auth users, require an Auth administrative key, change
production authentication policy, or read/print account emails from SQL. It
assigns Administrator/Reception roles in synthetic server fixtures using their
online-verified subjects; client JWT role metadata cannot grant permissions.

Placeholder layout for the local file (replace values **locally**, never in chat):

```dotenv
TEST_PROJECT_IS_DISPOSABLE=true
TEST_SUPABASE_PROJECT_REF=REPLACE_PROJECT_REF
TEST_DATABASE_URL="postgresql://postgres.REPLACE_PROJECT_REF:REPLACE_ENCODED_PASSWORD@REPLACE_TEST_SESSION_POOLER_HOST:5432/postgres?sslmode=verify-full&sslrootcert=./prod-ca-2021.crt"
TEST_SUPABASE_URL=https://REPLACE_PROJECT_REF.supabase.co
TEST_SUPABASE_PUBLISHABLE_KEY=REPLACE_PUBLISHABLE_KEY
TEST_ADMIN_EMAIL=REPLACE_TEST_ADMIN_EMAIL
TEST_ADMIN_PASSWORD="REPLACE_TEST_ADMIN_PASSWORD"
TEST_RECEPTION_EMAIL=REPLACE_TEST_RECEPTION_EMAIL
TEST_RECEPTION_PASSWORD="REPLACE_TEST_RECEPTION_PASSWORD"
TEST_OTHER_GYM_EMAIL=REPLACE_TEST_OTHER_GYM_EMAIL
TEST_OTHER_GYM_PASSWORD="REPLACE_TEST_OTHER_GYM_PASSWORD"
```

Use the test project's dashboard **Connect > Session pooler** URL, owner username
`postgres.<test-project-ref>`, and port 5432. Copy the actual hostname: do not infer
it from the runtime project or region. The suites reject direct hosts and
transaction pooling (6543). Download the correct database CA locally into ignored
`server/certs/`, or use the moved `server/prod-ca-2021.crt` if it is that project's
CA. Set `sslrootcert=certs/test-ca.pem` or `sslrootcert=./prod-ca-2021.crt`, respectively.
Paths resolve from `server/` when running npm there. No extra CA env variable is
required. Only `sslmode` and mandatory `sslrootcert` URI query parameters are
accepted, preventing host/user/SSL overrides. The shared pg options explicitly
set `rejectUnauthorized: true` and retain Node's hostname verification; no SSL
connection-string parameters can replace those options. Never disable TLS verification.
These settings follow [Supabase connection guidance](https://supabase.com/docs/guides/database/connecting-to-postgres)
and [node-postgres SSL guidance](https://node-postgres.com/features/ssl).

## Commands and evidence boundaries

From `server/`:

```sh
npm run test:config
npm run test:db
npm test
npm run test:integration
```

`test:config` checks names/configuration and local PEM certificate readability only, outputs no values and exits 2 for
missing/invalid setup. It makes no network request. `test:integration` invokes
`test:integration:supabase`, which runs:

```sh
node --env-file-if-exists=.env.test --test --test-isolation=none test/supabase.integration.ts
```

Missing setup **fails** that command with exit 1 before database connection; it
does not skip or count as a successful live test. Do **not** run `npm run migrate`
first: the suite must start on an empty application database and runs the actual
shared migration runner itself. Auth/PostgreSQL/network failures are sanitized
to controlled step labels/status/error codes without connection strings, tokens,
passwords, provider payloads or driver details. Fastify request logging is off.

| Command | Actual SQL? | Actual Supabase Auth? | Scope |
| --- | --- | --- | --- |
| `npm test` | No | No | Unit/route/guard tests; SQL and identity fixtures explicitly labelled. |
| `npm run test:db` | Yes, if locally validated | No | Read-only real test Session pooler TLS connection and `SELECT 1`; no migrations/data. Fails before network when settings are invalid. |
| `npm run test:integration:postgres` | Yes, when configured | **No: Auth verifier mocked** | Older real PostgreSQL/Fastify suite, separately invoked; skipped without `TEST_DATABASE_URL`. |
| `npm run test:integration` / `test:integration:supabase` | Yes, when setup passes | Yes, when setup passes | New live Supabase suite; never substitutes mocks or skips missing configuration. |

Runtime `npm run db:check` loads only `.env` and uses only `DATABASE_URL` for a
read-only TLS/SQL probe. It never loads `.env.test`. Conversely, test commands load
only `.env.test` and never fall back to runtime `DATABASE_URL`. Runtime Fastify uses
the same pg options, verifies `SELECT 1` before listening, and also needs its own
`SUPABASE_URL`/`SUPABASE_PUBLISHABLE_KEY`. A database probe is not Auth/enrollment,
Render HTTPS or desktop sync acceptance. Real synthetic fixtures remain restricted
to the explicitly disposable isolated project.

If the read-only test probe reports `28P01`, the server rejected database login:
check the isolated project's database password, not an Auth account password or
API key. Dashboard placeholder brackets are not part of the replacement password;
retain brackets only if they are actual password characters. Percent-encode the
password once in the URL. The probe now reports the socket's TLS verification
separately from authenticated database/SQL success, including after failed login.
An authentication error is not a successful `SELECT 1`; stop before integration
or migrations. After correcting local credentials, retry only `npm run test:db`.
Keep all values out of terminal output and chat. See
[Supabase authentication troubleshooting](https://supabase.com/docs/guides/troubleshooting/fatal-password-authentication-failed).

Do not run the two database suites against the same project one after the other:
the first retains its schema, and the second will refuse it. Both require explicit
disposable configuration and an empty application database. The PostgreSQL/Auth-
mock suite may also use a new local PostgreSQL database; it is not live Auth proof.

If `test:integration:postgres` passes and `test:integration:supabase` then reports
`Refusing existing application schema, migration ledger or public relations`, the
live suite has reached PostgreSQL and intentionally stopped at its read-only
guard. The earlier suite retained the application schema/ledger and fixtures.
No real Auth sign-in or new migration runs after that rejection. Preserve the
existing database for inspection; configure a fresh disposable test project with
exactly three confirmed synthetic Auth accounts for the live suite. Run
`test:config`, then `test:db`, then `test:integration:supabase` only after the probe
passes. Do not run the SQL/mock-Auth suite or a manual migration there first.
Never bypass the guard or automatically drop/reset the prior database.

To identify which condition blocks the live suite, run `npm run test:inspect`
from `server/` in the same normal terminal. It first refuses inherited target
settings that differ from local `.env.test`, then performs the authorized TLS and
`SELECT 1` probe and the same read-only catalog checks as the isolation guard.
Output contains only fixed labels, schema/ledger/public-relation presence
booleans and numeric catalog counts for the application schema's relations,
routines, types and dependency records. Relation counts include indexes,
sequences and views as well as tables; they are not member row counts. No
environment values, URLs, credentials, identifiers, object names or member data
are displayed. It does not acquire an advisory lock, migrate, drop, reset or
sign in to Auth. Existing objects cause exit 2. The live guard now includes those
same presence labels in its rejection, without weakening the refusal.

If only the application schema is present, inspect these counts before choosing
a fresh project or proposing cleanup. An existing schema is still refused even
when every displayed count is zero. Counts are diagnostics, not authorization
to remove a schema. No automatic cleanup is provided. PostgreSQL's
[dependency catalog](https://www.postgresql.org/docs/current/catalog-pg-depend.html)
records object dependencies; the schema counts query reads only system catalogs.
Complete the real SQL/Auth suite on the isolated project before applying the
reviewed migrations to the intended runtime project. This follows Supabase's
[separate staging and production environment guidance](https://supabase.com/docs/guides/deployment/managing-environments).

A populated schema without `public.armstrong_migrations` is untracked. Matching
object counts do not prove matching definitions, permissions or migration
history. Do not fabricate a migration-ledger checksum, manually apply the SQL
again or bypass the guard to make this project appear fresh. Preserve it and
use a new isolated target, or obtain explicit approval for a reviewed cleanup
confined to the disposable test project. Cleanup is not part of the test command.

## Real suite behavior

Before writes, the suite validates matching project URLs and TLS, takes a session
advisory lock, and refuses any `armstrong` schema, public migration ledger or
public application relation. It signs in the three real test accounts via Auth
over HTTPS and checks each token using the production online verifier. SQL must
contain the same three confirmed subjects in `auth.users` and no additional Auth
users. This guards against mismatched SQL/Auth projects. It then:

- Runs the CLI's shared, transactional/checksummed migration runner, checks
  repeatability and rejects a changed checksum. Migration SQL remains unchanged.
- Provisions two synthetic gyms, three staff mappings and writer/read-only
  devices. Verifies authoritative registration, roles, bad tokens/secrets,
  client-supplied role/gym denial and staff/device revocation.
- Exercises real one-writer/NFC/date/revision/archive-pair/actor-FK constraints,
  append-only history and archived-row triggers, and checks RLS configuration.
- Sends simultaneous identical pushes; checks one member/receipt/change. Retries
  discarded replies across Fastify/pool restart against retained PostgreSQL state.
  Discarding an injection response simulates loss; it is not a real network drop.
- Tests reused operation IDs, card collisions, concurrent/stale edits, immutable
  joined dates, Administrator-only archive and isolation with reused IDs/cards
  across two gyms and forged gym/device assertions.
- Verifies ordered pulls, repeatable cursors and ahead-cursor denial. Injects a
  temporary SQL change-insert failure, verifies full rollback and one subsequent
  retry commit, then removes only that temporary test trigger/function.

Failures stop dependent cases and retain fixtures for inspection. Do not point the
suite at a populated database to rerun it. Inspect/dispose the dedicated test
project externally before preparing another fresh test environment.

Even a passing suite verifies backend routes through **Fastify injection**, real
SQL and real Auth. It does not verify a deployed listener, Render certificates,
restricted runtime-role grants, native sign-in/enrollment/credential storage,
desktop offline/restart/reconnect, SQLite pull application or Windows/hardware.
Live desktop sync remains disabled, and deployment remains unauthorized.

Primary Auth references: [password sign-in](https://supabase.com/docs/reference/javascript/auth-signinwithpassword)
and [online user verification](https://supabase.com/docs/reference/javascript/auth-getuser).
