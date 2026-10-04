# Armstrong member API

Backend source for the existing `ArmStrong/server/` folder, beside `Client/`.
The selected architecture is Fastify on Render, with Supabase PostgreSQL/Auth.
See [Render setup](docs/RENDER_SETUP.md) for the Singapore/Free testing target,
build settings, certificate mounting and remaining live deployment requirements.
The protocol covers **member creates, updates and archives only**.
It does not synchronize payments, memberships, attendance, inventory, expenses,
NFC assignment history or member hard deletions.

## Current verification boundary

The latest setup milestone adds read-only local readiness and credential-free
HTTPS API checks, plus separate native public-config review/provisioning. The
write command validates actual prepared SQLite identity/scope and never replaces
an existing file. Production API startup refuses owner/reserved database roles;
shared Auth settings reject privileged keys and require canonical distinct
origins. See [commands and limits](docs/LOCAL_SETUP.md).

Read-only queries through the connected Supabase app reached the configured
runtime project: two fixture-named gyms, no active Administrator, two active
writers (one per gym), one member, six RLS-enabled private tables, no `anon`/
`authenticated` schema usage, and no `armstrong_api` login. No registration or
schema changed. Connector queries do not prove this Node driver's TLS/session,
deployed API, native credentials or enrollment. Current `.env` still has no API
origin; installation names and a stable gym UUID are now saved locally, while
actual prepared-device settings remain missing. Driver DNS limitations remain separate from
successful connector SQL access.

The user reports runtime `npm run db:verify` PASS for real PostgreSQL/TLS,
migration checksum and six required table/RLS flags, followed by
`npm run auth:check` PASS for real Supabase password login and online identity
verification.
These are separately attributed normal-terminal results. The Auth check makes no
application database changes and explicitly leaves gym/role/device registration
and desktop login unchecked. Approved registration, actual enrollment and real
member API/desktop acceptance remain pending; live sync is disabled.

Pinned Fastify/pg packages and the generated lockfile are present. Full strict
NodeNext typing now passes, including source, scripts and tests. Missing pg
declarations were resolved with unchanged official DefinitelyTyped source files
pinned to an immutable commit in [vendor/types-pg](vendor/types-pg/README.md),
installed as a local dev dependency. The source/license hashes are checked by
`npm run types:verify` before typechecking. Runtime pg remains 8.23.0; no ambient
`any` shim, relaxed compiler setting or driver upgrade was introduced.

An isolated source checkout completed `npm ci --offline --include=dev
--ignore-scripts --no-audit --no-fund`, full typecheck and all **48** local tests.
This used existing cached public packages, without copying any env files or
changing the active installation. Fresh npm registry access still fails
`EAI_AGAIN`; security audit and deployment acceptance remain unverified. These
checks ran on Node 26.10.0 against pinned Node 24 declarations; actual Node 24
runtime/hosting acceptance is still required.

Protocol/configuration/Supabase response tests run using Node's built-in test runner.
The tests stub identity HTTP responses; they do not establish a live Supabase login.
New enrollment tests execute installed Fastify routes with mock SQL/identity HTTP;
their names and header explicitly identify those mocks. They are not database or
live sync evidence.
The PostgreSQL/Fastify suite with a **mock Auth verifier** is separately invoked by
`npm run test:integration:postgres` and skipped without `TEST_DATABASE_URL`.
The new `npm run test:integration` suite uses real Supabase PostgreSQL **and Auth**
and fails before connection when its isolated test configuration is absent. See
[exact variables, commands and isolation rules](docs/SUPABASE_TESTING.md).
No configured API listener or deployment has
been verified here; `/health` is tested only through Fastify injection.

SQLite v6 has native HTTPS transport, OS credential preparation,
Administrator login/logout and explicit member conflict review. The bounded
member engine retains its database scope, frozen requests, durable retries and
atomic pull cursors. Eligible conflict review preserves original operations and
history; keep-local creates a new operation instead of rewriting the old request.
See [native setup](../Client/docs/NATIVE_SIGN_IN.md) and
[conflict review](../Client/docs/MEMBER_CONFLICT_REVIEW.md).

Production scheduling remains disabled pending real HTTPS/API/device acceptance.
Mock transports verify local orchestration, not online synchronization. No IPC
accepts trusted acknowledgements, server pages or actor/role grants. Offline
restart unlock is unimplemented; current native restart requires online sign-in.
No production record is marked synced from a local review.

## Local setup when dependency/network access is available

`npm run typecheck` first verifies the pinned upstream declaration snapshot,
then runs the server's strict NodeNext config and pinned Node 24 types over source,
scripts and tests. It neither executes the application nor loads env files.
Full typing and the local/mock suite pass; this does not establish real backend,
TLS, Auth, enrollment or desktop integration. Current measurements are in
[Client/STATUS.md](../Client/STATUS.md).

Use Node 24 LTS (`24.x`). From this directory:

```sh
npm ci --include=dev --ignore-scripts --no-audit --no-fund
npm run typecheck
npm test
cp .env.example .env
```

Populate `.env` locally. `DATABASE_URL` requires a PostgreSQL login and
the Supabase Session pooler on port 5432, `sslmode=verify-full` and a readable PEM
CA path in `sslrootcert` for runtime connections. Copy the hostname from that
project's Connect dialog; custom restricted runtime roles use `role.project-ref`
as the pooler username. The existing pg driver supplies the CA with mandatory
certificate/hostname verification. Keep Supabase/database credentials
and device secrets out of Client, browser bundles, logs and Git. The publishable
key plus access token is sent only to the configured HTTPS Supabase Auth endpoint.
No Supabase service-role key is needed for token verification.

Run `npm run db:check` from `server/` for a read-only real runtime PostgreSQL probe;
it loads `.env` only, uses `DATABASE_URL` only and queries `SELECT 1` after verifying
an authorized TLS socket. `npm run test:config` validates test settings and the CA
locally; `npm run test:db` probes only the isolated `TEST_DATABASE_URL` from
`.env.test`. Both probes print controlled status/errors without connection values.
The API performs `SELECT 1` before listening; startup and idle-driver errors are
sanitized. These checks do not prove live Auth, enrollment or desktop sync.

After `npm run migrate`, run `npm run db:verify` from `server/`. This loads only
runtime `.env`/`DATABASE_URL`, refuses a target overridden by the shell, and uses
the existing pg driver and mandatory CA/certificate/hostname verification. It
checks the version 1 migration checksum, the six required private tables and
their RLS-enabled flags in a repeatable-read, read-only transaction. It reads
only catalogs and the migration ledger, never gym/member/staff/device records;
output contains fixed PASS/error labels and counts, no environment or hash
values. The RLS flag comes from
[PostgreSQL's table catalog](https://www.postgresql.org/docs/current/catalog-pg-class.html);
the transaction uses PostgreSQL's
[read-only access mode](https://www.postgresql.org/docs/current/sql-set-transaction.html).
This verifies basic migration metadata, not every constraint/grant, actual Auth,
enrollment, deployed HTTPS or desktop sync. It never migrates, repairs or seeds.

Local certificate files are ignored. Provision the CA file separately in a future
deployment and point `sslrootcert` to its deployed location; no deployment occurred.

Production (`NODE_ENV=production`) requires a canonical HTTPS API origin with no
credentials, path, query or fragment. Set `PUBLIC_API_ORIGIN` explicitly, or let
Render supply its actual `RENDER_EXTERNAL_URL` while `RENDER=true`; the automatic
fallback accepts only HTTPS `onrender.com` origins without non-default ports.
Local setup/native provisioning still require explicit `PUBLIC_API_ORIGIN`.
Supabase must likewise
use a canonical HTTPS origin distinct from the API. Native and server settings
share public publishable/legacy-anon key validation; privileged keys are refused.
Production startup rejects owner/reserved role names while administrative
registration/probes retain owner access. A custom name alone does not verify
role attributes or grants. Configuration validation does not prove a working
TLS endpoint; real certificate/connectivity acceptance is still required.

Apply `npm run migrate` with a migration-owner connection in a controlled session.
The migration is transactional, advisory-locked and checksummed; a changed applied
migration stops. It creates a private `armstrong` schema. Configure a separate
server-only runtime role with `USAGE` on that schema, `SELECT` on its tables,
`UPDATE(change_sequence)` on `gyms`, `INSERT/UPDATE` on `members`, and `INSERT` on
`member_operations/member_changes`. It needs RLS bypass **only in combination with
these restricted grants**; do not grant access to public/anon/authenticated Data API
roles. The runtime role must not administer staff/devices, delete history, create
schemas or run migrations. No public RLS policies are supplied.

Normal operation uses **one Administrator login** for attendance and all existing
controls. The account is authenticated through Supabase Auth and internally mapped
to its approved gym/Administrator permission record. No additional staff login is
required. Before API enrollment, provision the approved records through an
administrative SQL connection:

See [single Administrator login and device setup](docs/STAFF_DEVICE_SETUP.md)
for `npm run auth:check`, its optional private local credentials, registration
relationships and the exact device-secret hashing convention. Auth check success
does not itself grant permissions or implement the native desktop login.

Administrative registration is now prepared as `npm run registration:check`
(read-only review) and the separate `npm run registration:apply` (transactional
new approved records only). Both use runtime `.env`, real verified Auth, owner
SQL and mandatory CA/TLS; they never load `.env.test`. They refuse conflicting
existing registrations and exact retries preserve IDs/roles/hashes. Start with
gym and Administrator only; device approval is optional and requires its existing
SQLite metadata plus the hash of a native-stored credential. No runtime records
have been created by Codex. See the guide for local `REGISTRATION_*` settings.

The latest `npm test` passes **65** local/mock and temporary-SQLite checks;
one subprocess check is skipped because of sandbox EPERM, with the commands
verified separately through direct isolated CLI runs. Registration SQL cases
remain explicitly mocked. Full strict typecheck and an
isolated clean install now pass with the pinned official declaration snapshot.
The socket-status helper verifies a real `TLSSocket` instance before accepting
its encryption/authorization flags; a local classification fixture is not a
live TLS handshake. The current Codex runtime database probe still fails DNS
before TLS or SQL. No registration or production sync was performed.

- One gym UUID/name in `armstrong.gyms`.
- Supabase Auth accounts and their **verified user UUIDs** in `armstrong.staff`,
  with Administrator or Reception and explicit active status.
- Approved device UUIDs and a SHA-256 hash of each independently generated
  32-byte secret represented as 64 lowercase hex characters in `armstrong.devices`.
  Keep the plaintext secret for later native credential-store enrollment. Never
  send it via URL/query string. The API compares its hash in constant time.
- At most one active `can_write` device. This restriction enforces the proposed
  pilot boundary pending the owner's topology decision.

No seeded accounts, passwords, gym IDs or API role/device approval routes exist.
`POST /v1/enrollment` verifies an already approved registration; it cannot create
staff, approve a device or assign a role. Run `npm start` only after configuration
and provisioning. `/health` checks
process health; it does not certify database access or successful sync.

Pinned direct dependencies follow the verified [Fastify version](https://www.npmjs.com/package/fastify?activeTab=versions)
and [pg version](https://www.npmjs.com/package/pg?activeTab=versions) metadata checked
for this milestone. Transitive versions are recorded in the supplied lockfile;
the earlier isolated clean install passed; a security audit and actual deployment
runtime acceptance remain open.

## Protocol v1

Server source derives authorized gym access from verified staff/approved-device
records and an authenticated device secret. `X-Gym-Id` is only a consistency
assertion; it cannot grant access to an unrelated gym. Permissions are rechecked
under the derived gym lock. Ambiguous registrations fail closed. Authorization
unit tests use registry fixtures; real PostgreSQL and live identity proof remain
required.

Each local device UUID is stable from database creation. Server/gym/device binding
is persisted through a protected native enrollment seam with no secrets; unbound
databases never invent a gym ID. Native login/enrollment wiring is implemented;
actual HTTPS/device enrollment acceptance remains pending. The backend route
alone cannot bind a desktop database or enable sync.
The internal engine pushes before pull, persists retry state transactionally and
blocks pull for failed/conflicted/unsupported pending member operations. Edits
arriving during a pull defer the page inside its transaction without moving the
cursor. Session identity is checked before I/O and in reply/page commits. Binding
and retry metadata survive backup/restore, but restored databases stay locked for
reconciliation. The future HTTPS scheduler must use a dedicated connection and
release UI locks over I/O. There are no worker/identity-grant IPC commands.

Authenticated requests require `Authorization: Bearer <Supabase access token>`,
`X-Gym-Id`, `X-Device-Id`, and `X-Device-Secret` headers. The API verifies identity
online with Supabase `/auth/v1/user` on every request. It checks active staff,
role, approved device and secret inside the database transaction, including retry
requests. Unavailable identity verification fails closed. Server actors come from
verified identity, never request role/actor fields.

`POST /v1/enrollment` requires the bearer token and an exact JSON object containing
`protocolVersion: 1`, `deviceId` (the existing local UUID), and `deviceSecret`
(64 lowercase hex characters). It needs no gym header. Active server staff/device
records plus the secret determine a single gym; missing/ambiguous registration,
wrong secrets or revocation deny access. It returns `protocolVersion`,
`gym: {id, name}`, `staff: {id, name, role}`, and `device: {id, canWrite}` after
rechecking permissions under the same gym lock used for writes. Role/identity/gym
fields in the request are rejected. Repeated verification does not write records
or rotate credentials, so a dropped response can retry safely. Device approval and
staff provisioning remain owner-only prerequisites. A native client must verify
HTTPS and this authoritative reply before creating its private session/binding;
it must not trust an IPC-supplied role/subject or restore credentials from backup.
All responses use `Cache-Control: no-store`; request logging remains disabled.

`POST /v1/members/push` accepts one bounded operation:

```json
{
  "protocolVersion": 1,
  "operationId": "11111111-1111-4111-8111-111111111111",
  "deviceId": "22222222-2222-4222-8222-222222222222",
  "memberId": "33333333-3333-4333-8333-333333333333",
  "action": "create",
  "expectedRevision": 0,
  "member": {
    "name": "Example member",
    "phone": "0771234567",
    "email": "",
    "nfcId": null,
    "joinedOn": "2026-10-03"
  }
}
```

Create requires revision 0. Update requires the last confirmed remote revision;
`joinedOn` is immutable. Archive requires Administrator, a positive confirmed
revision and `member: null`; server time and verified actor are saved. Native
archive delivery additionally requires the original local actor's Supabase subject
to match the server receipt actor; local archive history/actor/timestamp remain
intact alongside the remote projection. Cards are trimmed, ASCII-normalized to
uppercase and unique within the gym, including archived members.

Success returns `protocolVersion`, `operationId`, `memberId`, `revision`, `sequence`
and the canonical `member`, including `archivedAt/archivedBy`. Reusing an operation
ID with different normalized content/device/actor returns 409; identical retry
returns the original receipt after current permission checks. Entity, receipt and
change row commit together. Conflicts return 409 and retain the local operation.
Authentication/device/permission failures use 401/403, validation 400, provider or
service failures 503. Server errors do not expose secrets or PostgreSQL details.

`GET /v1/members/changes?after=0` bootstraps an empty client from retained changes.
It returns `protocolVersion`, `after`, `nextCursor`, `hasMore`, and up to 100 ordered
`{sequence, operationId, member}` changes. Keep requesting pages until `hasMore`
is false. Each gym's row lock serializes sequence allocation **and commit**, so a
late transaction cannot appear below an already advanced cursor. No pruning is
implemented; an invalid/ahead cursor must be reconciled explicitly.

SQLite freezes requests before I/O and retries the same bytes/operation ID after
restart. Its independent remote revision does not reuse local optimistic versions.
Only a matching server receipt acknowledges that operation; original outbox rows
remain immutable history, and nonmember operations remain pending. Pull page and
cursor commit atomically. Pending edits, card collisions, immutable archive/history
and identity conflicts retain local rows plus remote conflict snapshots. Remote
archive actors are imported as inactive identity references without roles or
sessions. Existing NFC history is retained; current-card changes append/revoke local
assignments. Conflict resolution and post-backup server reconciliation are still
required work; restored databases cannot sync automatically.

## Verification and deployment gates

Configure only a new isolated Supabase test project in ignored `.env.test`, using
the eleven `TEST_*` variables in [SUPABASE_TESTING.md](docs/SUPABASE_TESTING.md).
Runtime `.env` credentials are never a fallback. From this directory:

```sh
npm run test:config
npm test
npm run test:integration
```

The live suite executes the shared checksummed migration runner, actual Auth
sign-in/online verification, real PostgreSQL constraints and Fastify routes. It
checks enrollment, duplicate/restarted retries, conflicts, two-gym isolation,
roles/revocation, ordered pulls and SQL failure rollback. It refuses existing
application schemas, migration ledgers/public relations and mismatched project
connections, requires exactly three pre-created confirmed test Auth users, and
retains synthetic fixtures. Missing configuration fails before connection rather
than producing a passing live result. Do not run migrations manually first or
reset existing data. This suite currently has no real execution result because
test settings are missing.

The older `npm run test:integration:postgres` separately covers real PostgreSQL
with a **mock Auth verifier**. It remains skipped here. `npm test` remains unit/
mock evidence. Neither suite substitutes for desktop/Render HTTPS acceptance;
no API listener/deployment or live desktop sync has been verified.

Render is now the selected hosting provider. `npm run build:verify` runs the strict
typecheck and unit/mock suite; `Procfile` defines the startup command. The package
and lockfile pin the supported runtime major to `24.x`. See
[Render setup](docs/RENDER_SETUP.md) before creating the service. Account/source
access, a separately provisioned runtime database login and the mounted CA are
still required. Supplied Render logs confirm a successful build followed by a
failed process startup; no live API URL has been verified in this workspace.
See the guide's startup troubleshooting for the refreshed source bundle and
safe variable-name diagnostics. Unexpected errors still withhold private details.

The selected `server/render.yaml` uses repository-relative `rootDir: server`, Node 24,
locked `npm ci --ignore-scripts --no-audit --no-fund` plus unit tests, `npm start`,
`HOST=0.0.0.0`, Render's `PORT`, `/health`, and environment-only configuration.
Select that file as the Blueprint path; do not move backend files into Client.
No migrations or database-owner secrets are run from the runtime build. The
template uses Singapore and the free plan, with automatic deploys off. Creating a
Blueprint still causes an initial deployment. The user selected Render setup;
successful startup still needs source/settings and database runtime access. Render handles
edge HTTPS; copy the actual origin into local `PUBLIC_API_ORIGIN` and verify its
certificate before desktop use. This
file has not been validated by Render or deployed. See the official
[Blueprint specification](https://render.com/docs/blueprint-spec) and
[Render TLS documentation](https://render.com/docs/tls).

The Free template is for testing: its cold start can exceed native HTTPS/login
deadlines. Review the compute plan, restricted runtime connection and deployed
database CA before any service creation. The installation names are now supplied
and saved locally; no remote registration was made. See the concrete
[Render preparation guide](docs/RENDER_SETUP.md).

Provider creation, region/budget, restricted database login, TLS acceptance,
deploy/restart/persistence and security review, actual OS credential persistence,
HTTPS enrollment, dropped-response reconnect, real desktop/Supabase integration
and Windows acceptance remain open gates. Local clean installation and typing
pass; they do not close these external acceptance gates.

Primary references used for this source: [Fastify testing](https://github.com/fastify/fastify/blob/main/docs/Guides/Testing.md),
[node-postgres transactions](https://node-postgres.com/features/transactions),
[Supabase verified users](https://supabase.com/docs/reference/javascript/auth-getuser).
