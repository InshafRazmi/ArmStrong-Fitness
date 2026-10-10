# Armstrong gym API

Fastify on Render, with Supabase PostgreSQL and Auth, serves the installed Tauri
application. Protocol 2 synchronizes members/cards, plans/periods, attendance,
invoices/payments/allocations/receipts/reversals, products/sales/stock,
expenses/voids, gym profile and audit. One approved computer edits; additional
approved computers download records. Historical user references cannot grant
roles. See [business protocol](docs/BUSINESS_SYNC.md).

## Current verification

Strict typing/provenance and 84 unit tests pass, with one sandbox subprocess
skip. Ten real native SQLite transaction envelopes pass the shared row contract
and request hash checks. The isolated all-module integration test passes with
real Supabase Auth and PostgreSQL, including exact receipts/retries, ordered
read-only download, stale edits, closed-payment refusal, immutable history,
cross-gym/secret/revocation denial and rollback. All fixtures are rolled back;
test credentials never fall back to production.

The approved Administrator/gym and migrations 1–9 are applied to production.
Existing unrelated records are preserved. Actual restricted-runtime TLS/catalog
checks pass for business and computer-enrollment permissions. API startup
verifies private tables, restricted grants and immutable guards before listening.

The public Render health endpoint was independently reached. Updated protocol-2
source still needs publication/deployment and live native acceptance; health
alone does not verify the deployed revision. Windows installer/hardware
acceptance, general conflict review, legacy/large-data bootstrap
and historically exposed-credential rotation remain open.
See [current status](../Client/STATUS.md) and [delivery](../Client/docs/DELIVERY.md).

## Build and run

Use Node 24 LTS. From `server/`:

```sh
npm ci --include=dev --ignore-scripts --no-audit --no-fund
npm run build:verify
cp .env.example .env
```

Set canonical distinct HTTPS Auth/API origins and a public publishable or legacy
anon key. Never supply a secret/service-role key for user verification.
`DATABASE_URL` requires the Supabase Session pooler on port 5432, a separate
restricted runtime login and `sslmode=verify-full` with a readable CA file in
`sslrootcert`. The driver verifies certificate and hostname; production rejects
owner/reserved login names. `AUTOMATIC_DEVICE_ENROLLMENT=true` enables the
private account-authorized computer approval function. Its default is false.

Apply checksummed, advisory-locked migrations with `npm run migrate` using an
owner connection in a controlled administrative session. Never use that owner
connection in the deployed service. The private schema has no public Data API
policies/grants. Runtime has only the required row/column grants; staff/device
administration, history deletion and migrations remain denied.

```sh
npm start
```

Build verification neither loads credentials nor runs database migrations.
The API sanitizes startup/driver errors, bounds requests and sends no-store
responses. Real Auth identity, approved gym/staff, device possession and current
writer permission are rechecked under the gym lock for every transaction,
including an exact retry.

See [Render setup](docs/RENDER_SETUP.md), [runtime database](docs/RUNTIME_DATABASE_SETUP.md),
[computer approval](docs/AUTOMATIC_COMPUTERS.md) and
[administrative registration](docs/STAFF_DEVICE_SETUP.md).
The basic `db:verify` command checks migration 1 and six legacy tables; startup
additionally verifies protocol-2 capabilities through the restricted connection.

## Tests

`npm test` uses local/unit fixtures without env files. For isolated live testing,
configure `.env.test` as described in [Supabase testing](docs/SUPABASE_TESTING.md).
The all-module suite requires native envelopes first:

```sh
# From Client: synthetic native data only.
ARMSTRONG_BUSINESS_FIXTURE_PATH=/tmp/armstrong-business-fixture.json npm run test:core
# From server:
ARMSTRONG_BUSINESS_FIXTURE_PATH=/tmp/armstrong-business-fixture.json npm run test:native-contract
ARMSTRONG_BUSINESS_FIXTURE_PATH=/tmp/armstrong-business-fixture.json ARMSTRONG_BUSINESS_LIVE_AUTH=true npm run test:integration:business
```

The live business test uses only the existing disposable test project, with
transactional fixture rollback. Do not point it at a production database.
Windows CI generates its own native fixture and checks the contract without
loading any account/database credentials.

## Compatibility and limits

`POST /v1/enrollment` remains the authenticated onboarding route.
`POST /v2/business/push` returns an exact committed request receipt.
`GET /v2/business/changes?after=0` returns ordered atomic transaction pages.
Authenticated business requests require bearer, gym/device IDs and the native
device secret in headers; caller IDs alone cannot grant access.
`GET /health` retains the protocol-1 compatibility response.
`GET /v2/health` identifies the all-module deployment with protocol version 2 and
`businessSchemaVersion: 11` for staff, attendance, removal and admission settings.
Apply migration 9 (`20261010211757_admission_settings.sql`) before deploying this
API version. It adds the admission-settings journal type and preserves gym scope,
restricted grants and version/history guards. Update editing desktops together;
older desktop builds cannot import the new settings row type.

Legacy member-only routes remain for older clients. After protocol-2 history
exists, new protocol-1 writes are refused. Existing legacy cloud members require
explicit reconciliation before protocol-2 use; unrelated history is preserved.

Each transaction is bounded to 1 MiB and 2,000 changed rows. Initial large legacy
databases require a bootstrap path; oversized transactions roll back locally.
Server validation currently reads the gym's complete business state under its
lock. Same-computer backup restore reconciles an isolated copy at verified online
sign-in before unlocking; general financial/master conflicts remain guarded.
This implementation does not authorize concurrent offline
financial writers or establish final release acceptance.
