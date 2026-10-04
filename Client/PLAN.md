# Core release plan

## Koyeb setup selected — local preparation complete

User selection/request now authorizes Koyeb setup, superseding the earlier
hosting decision gate for this provider. Target Singapore Eco Micro with one
fixed instance; keep Supabase PostgreSQL/Auth and the desktop SQLite application.
Node `24.x`, a production Procfile, strict/unit build command, private source
upload bundle and [Koyeb guide](../server/docs/KOYEB_SETUP.md) are prepared.
Strict typing/provenance, 65 local checks (one sandbox subprocess skip) and the
frontend build pass; source packaging is verified without env/CA/native data.

Account/repository access is pending: there is no configured Koyeb connection,
CLI/token or local Git remote. Fresh read-only SQL confirms the restricted
`armstrong_api` login is absent; owner credentials cannot run production.
Complete restricted access and Koyeb CA/env/service provisioning, then actual
hosted Node 24/HTTPS/database/restart checks and native enrollment. Account signup
and connection require the user's account access, not another publication
confirmation. No service, remote record/grant, registration or env credential
was changed by this preparation. Member scheduling and the remaining release
milestones stay open; see STATUS for measured evidence and limits.

## Installation names supplied; hosting decision pending

User-provided names **ArmStrong Fitness** / **ArmStrong** and a stable new gym
UUID are saved privately for registration; existing settings/remote records are
preserved. Readiness now validates those names/UUID locally; API origin and actual
device path/hash remain missing. The prepared Render guide covers Node service
settings, runtime role/CA prerequisites and Free versus paid timeout behavior.
No service, paid plan, region, registration or public deployment was created.
Recommendation discussion does not revoke the earlier no-public-deployment
instruction. Complete restricted runtime access/CA and actual device preparation
before authorized API hosting/enrollment and the remaining sync/release work.

## Current continuation — API/desktop setup tools and runtime guards, 2026-10-04

Local setup inspection, credential-free HTTPS process/protocol probe and separate
public desktop-config review/provisioning are implemented. Provisioning preserves
existing files and SQLite/device history. Public Auth settings share canonical
origin/key validation; production API startup refuses owner/reserved roles while
administrative commands keep owner access. The role-name guard does not verify
custom role grants. Commands and limits are in
[LOCAL_SETUP.md](../server/docs/LOCAL_SETUP.md).

Strict typing, 65 local checks (one Node subprocess skipped due sandbox EPERM),
direct isolated CLI review/write/retry/readiness and frontend build pass.
Read-only connector SQL reached the configured runtime project, confirming two
fixture-named gyms, no active Administrator, existing devices/member, private RLS
and no `armstrong_api` login. No records, env values, credentials, grants, schema
or hosting changed. Source and real-service evidence are separate in STATUS.

**Next:** obtain the existing approved API origin and installation display names;
preserve the existing fixture records and approve a stable installation gym;
prepare the actual OS device, register Administrator/device, provision and verify
restricted runtime access, native config and enrollment; finish member scheduling
and real synthetic sync acceptance. Other modules, restore reconciliation,
offline restart policy and Windows/hardware/installer acceptance remain open.
Retain the no-public-deployment instruction and disabled live sync.

## Previous milestone — full backend typing and clean install pass, 2026-10-04

Missing pg declarations are resolved with unchanged official DefinitelyTyped
source at an immutable commit, licensed and hash-verified in `server/vendor`.
The npm-generated lockfile installs that local dev package offline. Full strict
NodeNext typing now passes without shims or compiler suppressions. Probe TLS
streams and the driver-construction fixture have runtime narrowing compatible
with the official declarations. Runtime pg and Supabase schema/grants are intact.

Full server typing, 48 local/mock checks, a separate clean offline locked install
plus its typing/48 tests, and frontend build pass. Actual database probe still
fails DNS before TLS/SQL; no live test or deployment pass is claimed. Current
measurements use Node 26.10.0 against Node 24 declarations. STATUS records source
provenance, the local-cache recovery and remaining actual Node 24 acceptance.

**Next:** approved HTTPS/API/native configuration, real OS device preparation
and approved Administrator registration/enrollment; synthetic live member sync
acceptance and scheduler; remaining-module sync/restore reconciliation; offline
restart decision; Windows/hardware/printing/installer acceptance. The API origin
and registration settings are still absent. Retain no-public-deployment and all
live acceptance gates. Current restart requires online sign-in while the optional
first-release policy question remains pending.


## Previous milestone — member conflict review implemented, 2026-10-04

Native Administrator review now supports use-recorded-server and keep-current-
local choices for eligible member conflicts. Schema 6 preserves original edits,
requests, conflicts and financial history through an append-only resolution
ledger. Keep-local creates a new update UUID based on the reviewed remote
revision. Native role/writer/scope/restore checks, stale-review protection and
idempotency are enforced at commit. Unconfirmed requests and exceptional archive,
identity, card, deletion or history cases retain reconciliation gates.

142 core checks pass with two environment probes ignored; 16 UI adapter checks,
route/settings/conflict review renders, frontend build, fmt and full desktop
Clippy and normal desktop compilation pass. See current STATUS
and [review behavior](docs/MEMBER_CONFLICT_REVIEW.md).

**Next:** backend strict typing (`@types/pg`), approved API/native configuration,
actual OS/device registration and
Administrator enrollment; real synthetic sync acceptance and scheduler; remaining
module sync/restore reconciliation; offline restart policy; Windows/hardware/
printing/installer acceptance. Preserve the no-public-deployment instruction.
Do not claim a complete production release from local/mock checks.


## Previous native milestone — login/device wiring implemented, acceptance pending

Native OS credential storage/device preparation, real OS HTTPS transport,
Administrator login/logout/status and UI are now connected. Every configured
business IPC checks native identity, role, expiry and writer permission. Verified
actors are recorded for new business writes/receipts/audit/outbox; restart/logout
locks permissions, and restore preserves its actor and auth requirement. No
schema migration changed. Only device UUID/path/hash are exposed for server
registration, which now checks the native readback hash. Setup is documented in
[NATIVE_SIGN_IN.md](docs/NATIVE_SIGN_IN.md); STATUS separates implementation from
actual OS/TLS/API/GUI/Windows acceptance.

129 core checks pass with two environment-dependent probes explicitly ignored;
15 UI adapter checks, all routes/settings/native login render checks, frontend
build, core Clippy and normal desktop build pass. Backend has 47 local/mock checks
and focused registration typing passing; full typing still needs @types/pg.
The actual Linux credential service is unavailable/locked, the TLS probe cannot
bind its loopback listener, and GTK still blocks desktop execution. No actual
device credential, registration, live login or sync was enabled by Codex.

**Next:** approved HTTPS/API config and actual native device preparation,
administrative registration review/apply, real synthetic Auth/API/desktop tests,
then sync scheduler acceptance. Conflict/restore reconciliation, other-module
sync, offline restart policy and Windows reader/printer/installer acceptance
remain. Preserve no-public-deployment instruction. Full completion remains open.

## Current report milestone — implemented; native builds pass, GUI blocked

Native date-filtered summaries/CSV and Reports controls are complete locally.
103 Rust/SQLite tests, 13 adapter tests, interface rendering, frontend build,
fmt and core Clippy pass. Cash reversal posting dates, void history, Colombo
midnight, overlapping periods, invalid bounds and exact aggregate limits are
verified. Inventory is explicitly current stock. STATUS records details.

Recovered the full-disk build failure with targeted Cargo cleanup (2.6 GiB of
generated artifacts). Retried with incremental compilation disabled: frontend
and smoke binary compile, but GTK fails before UI execution. The normal desktop
binary has been rebuilt successfully without smoke hooks. Run the expanded
smoke in a usable graphical session, then rebuild normally. API origin and
registration settings are missing, pg typing needs installation, and native
production TLS/keyring/login UI remain unfinished. Live synthetic sync and
Windows/hardware acceptance are still required. Sync remains disabled; no
deployment or live application/database binding occurred. STATUS records commands.


## Current continuation — native protocol/login logic and report filters, 2026-10-04

Completed the unfinished HTTP adapter (eight mock/SQLite checks, 85 total tests
at that milestone) and private Auth/online-identity/enrollment plus atomic local
role/scope binding (eleven focused checks). No production HTTP/keyring transport
or login IPC/UI is connected; sync remains disabled. Cached base64 is pinned and
locked. Frontend build and 46 backend checks pass; full typing still needs pg
declarations and network DNS remains blocked. Actual API address and working-
terminal dependency installation are requested. STATUS records boundaries.

Reports/CSV and final local build/core/UI/lint checks are now complete above.
**Next:** native TLS/keyring/commands/UI,
approved registration and real synthetic sync acceptance. Retain no-public-
deployment instruction and Windows/hardware release gates. Do not label this
source-only internal milestone a live login or finished production release.


## Current focused milestone — registration tooling ready; live review needs local settings, 2026-10-04

Prepared runtime-only `registration:check` (read-only review) and
`registration:apply` (explicit administrative new-record writes). Identity comes
from the one real Auth login and must be confirmed in the SQL project. Matching
retries preserve IDs/permissions; conflicts refuse implicit replacement. Gym and
Administrator can be registered first; optional device approval reads the existing
SQLite device UUID and accepts only a hash of a native-stored credential. Secure
native generation/storage is still pending; do not invent a device ID or secret.
No account, role/device registration or SQLite binding was applied by Codex.

Fixed the original non-driver TypeScript diagnostics without suppressions or
relaxed checks. Full typing now reports eight missing-pg-declaration errors,
including the new CLI. A single `@types/pg` install attempt failed `EAI_AGAIN`;
dependency installation stopped with the exact command/output recorded in STATUS.
The existing driver/dependencies/lockfile are preserved. Render's prepared build
now includes dev dependencies and full typecheck before local tests; not deployed.

Fresh 46 local/mock/temporary-SQLite tests, focused strict typing, script/live-suite
syntax, frontend build and 77 Rust/SQLite core tests pass. Registration review
stops before network at three missing local `REGISTRATION_*` settings. The new
registration step in the real SQL/Auth suite is prepared but unrun; existing
isolation guards remain intact. None of these checks proves live enrollment/sync.

**Next:** configure approved gym UUID/name and Administrator display name privately
in `.env`, run `registration:check`, review/apply, then recheck existing records.
Install compatible pg declarations in the working terminal and measure full
typecheck; review least-privilege runtime access. Complete native login/enrollment/
OS credential storage/HTTPS transport and the verified API endpoint, then real
synthetic-data sync acceptance. Keep tests isolated and live sync disabled until
these prerequisites pass. See the updated server setup guide and STATUS.

## Current enable request — prerequisites remain incomplete, 2026-10-04

The user requests live sync. Inspection confirms no native production HTTPS
transport, login/enrollment, secure credential storage or sync IPC exists.
Runtime `PUBLIC_API_ORIGIN` is missing; an existing HTTPS Fastify endpoint is
being clarified. Do not substitute the Supabase Auth/database origin or bypass
persisted device/session/scope checks. The earlier no-public-deployment constraint
remains in force. One real Auth account has passed sign-in/online identity checks;
approved gym/Administrator/device records and real member-sync acceptance remain
unverified. No second operational account is required.

Fresh Codex npm ping exits 1 with `EAI_AGAIN`; runtime `npm run db:check` exits 2
with `EAI_AGAIN` before connection/TLS/SQL. Exact commands/output are in STATUS.
No installation, migrations, integrations, database writes, env/source changes
or deployment occurred. Only documentation changed; local tests were not rerun.

**Next:** establish the verified HTTPS API endpoint, complete approved enrollment
and native transport/login/credential storage, and pass synthetic-data real
member API plus desktop offline/restart/reconnect/pull/retry/conflict checks.
Preserve stable IDs, durable retries, push-before-pull and atomic pull/cursor
transactions. Live sync remains disabled until these prerequisites pass.

## Current milestone — real Supabase login verified; registration and enrollment next, 2026-10-04

The user-terminal `cd server && npm run auth:check` now **PASSES** real password
sign-in and separate online identity verification. This supersedes the initial
missing local probe settings; it does not verify gym/role/device registration or
desktop login. No application database changes occurred. Runtime SQL/TLS/schema
metadata and live Auth results are separately verified; end-to-end member sync
has not passed. No second staff login account is required.

**Next:** prepare and inspect approved registration for the one gym, this same
verified account's internal Administrator row, and the actual desktop's persisted
device UUID. Preserve existing registrations and IDs. Protect the device secret,
then verify `/v1/enrollment` with real Auth; gym/role access must still come from
server records. Complete native HTTPS/login/credential storage/session binding
and real member API/desktop offline/restart/reconnect/pull/retry/conflict checks.
Review restricted runtime grants and finish full server typing. Keep synthetic
acceptance data, isolated test configuration and existing SQLite safeguards;
attendance sync remains outside the member protocol. No public deployment or
live-sync enabling yet. Exact reported output and evidence limits are in STATUS.

Only documentation changed in recording this PASS; no tests, network/database
requests, registrations, migrations, environment edits or deployment occurred.

## Preceding account model — one Administrator login for all controls, 2026-10-04

The user clarifies one Administrator account handles attendance and all controls.
Keep Supabase Auth for that one login; map its verified subject internally to the
gym's Administrator permission/audit row. No extra operational staff/Reception
account, device-only identity migration or removal of API authentication is
needed. The earlier device-only/deferral choice is superseded. Existing member-
only sync scope and offline safeguards remain intact.

Prepared runtime-only `npm run auth:check` with optional private local probe
credentials. It signs in and verifies the identity online, with HTTPS/no redirects,
public Auth key only, no token/credential output, database writes or role grants.
Fresh 33 local/mock tests, focused Auth typing/syntax and frontend build pass;
full server typing still fails 79 existing errors. The actual command stops at
missing local `AUTH_CHECK_EMAIL`/`AUTH_CHECK_PASSWORD` before any network request.
Live SQL/TLS/schema evidence remains the user's preceding normal-terminal PASS.

**Next:** privately configure/run the ONE Admin Auth check, then prepare approved
gym/Admin/device records, authoritative native login/session and enrolled scope.
Verify real enrollment/member API behavior and desktop offline/reconnect/pull/
retry/conflict acceptance. Review runtime grants, finish typing and HTTPS/secure
credential storage before deployment or sync enabling. The account decision does
not implement attendance synchronization. Exact commands/results are in STATUS
and `server/docs/STAFF_DEVICE_SETUP.md`; no live Auth or desktop success is claimed.

## Preceding authentication interpretation — staff login declined; device option pending, 2026-10-04

The user declines staff-account authentication. Runtime database/TLS/migration
metadata verification remains complete in the user terminal. Replace the next
staff-account-creation step with a choice of enrolled-device authentication
without staff login, or deferred authentication with sync disabled. A choice
question is pending; no API authentication change is yet implemented.

For the device option, design server-derived gym scope, explicit approved device
permissions and device audit actors. Existing staff-referencing operation/archive
FKs and native actor reconciliation must migrate without fabricated identities or
history loss. Keep applied migration SQL/checksum immutable, preserve stable gym/
device/member IDs, durable retries, push-before-pull and atomic cursor/page saves.
Database credentials stay server-only; native enrollment must obtain scoped device
credentials over HTTPS and store them securely. Implement and verify the selected
contract before enabling sync. No staff sign-in setup, registration writes or
deployment occurs in this documentation step; exact boundaries are in STATUS.

## Preceding runtime SQL/TLS/schema milestone — normal-terminal verification passes, 2026-10-04

User-supplied `npm run db:verify` passes real runtime PostgreSQL/TLS, version 1
checksum, six required tables and six RLS-enabled flags. These are live SQL
metadata results, separately attributed from the local mocks and Codex DNS
failures. No Auth/enrollment/member API or desktop-sync pass is established.
No application, schema, env, migration or SQLite change occurs in recording them.

**Next:** establish whether a confirmed test staff Auth account exists in the
same runtime project. Prepare owner-approved gym/staff/device registration using
its online-verified subject and explicit role/device approval; use synthetic
acceptance data. Follow `server/docs/STAFF_DEVICE_SETUP.md`, including the existing
API's string-hashing convention and stable desktop device identity. Verify real
Auth and enrollment, then member API behavior; review least-privilege runtime
grants and resolve strict server typing. Keep integration tests separate and
complete native HTTPS/credential storage/scope binding plus actual desktop and
deployment acceptance before enabling live sync. No public deployment or real
member data is authorized. Exact results and boundaries are in STATUS.

## Preceding runtime migration reported successful — verify metadata before enrollment, 2026-10-04

The user reports `npm run migrate` succeeded against the authorized existing
runtime `.env` project. Added runtime-only `npm run db:verify` to check authorized
TLS/`SELECT 1`, version 1 ledger/checksum and the six private tables/RLS flags in
a read-only repeatable-read transaction. It exposes only status/counts and reads
no gym/member/staff/device rows; no repair, seed, Auth or migration runs. Migration
SQL, driver, env files, test isolation and SQLite safeguards remain unchanged.

Fresh checks pass 30 local/mock tests, new helper/CLI syntax, focused helper/test
typing and frontend build. Full strict server typecheck remains blocked by pg
declarations and existing typing; the new CLI also needs those declarations.
Codex `npm run db:verify` exits 2 with `EAI_AGAIN` before connection/catalog queries.
User-reported migration success and these local mocks are separate evidence.

**Next:** run the verifier in the working normal terminal. On metadata/TLS pass,
provision approved staff/device mappings with verified Auth subjects and least-
privilege runtime grants, then establish actual Auth/member API acceptance.
Keep integration tests isolated; do not drop/reset existing schemas. Native
HTTPS/login/enrollment/credential storage and actual desktop/Render/Windows
acceptance remain. No deployment or live-sync enabling; exact results are in STATUS.

## Preceding authorized runtime schema setup — use existing .env project, 2026-10-04

The user explicitly requests the existing runtime project in `server/.env` and
creation of the checked-in schema, without a fresh project. Proceed with that
authorized migration; the preceding isolated-test-before-runtime recommendation
does not block it. Keep test configuration separate and report outstanding real
Auth integration honestly. No deletion/reset, untracked-schema adoption, real
member data, deployment or live-sync enabling is authorized by this request.

Existing `cd server && npm run migrate` uses only runtime DATABASE_URL, verified
CA/TLS and the pg driver. Its transaction, advisory lock and checksum ledger
protect the unchanged six-table private migration. Codex attempted a read-only
runtime schema/ledger preflight but failed before SQL with redacted `EAI_AGAIN`
(exit 2); migration was not attempted here. Normal-terminal runtime TLS/SQL PASS
remains user-supplied. No source/env/database changes or new tests occurred.

**Next:** run `npm run migrate` in the working normal terminal, record its real
result, then verify table/ledger/checksum/RLS metadata. Provision approved staff
and devices with verified Auth subjects and restricted runtime grants as a later
step. Resolve real SQL/Auth acceptance, HTTPS/enrollment/native credential storage
and actual desktop sync before enabling synchronization. STATUS contains exact
execution boundaries and the retained isolated-test blocker.

## Preceding isolated schema is populated and untracked — resolve target before real Auth integration, 2026-10-04

User-terminal read-only `npm run test:inspect` passes target matching, real
Session pooler TLS and `SELECT 1`; catalogs report 15 application relation
objects, 3 routines, 12 types and 9 dependency records. The schema exists with
no public migration ledger or public application relations. It is populated;
row contents are unknown. Counts are consistent with the existing six-table,
nine-index, three-function migration, but do not verify definitions or prove
successful checksummed application. The existing isolation rejection is correct.

**Next:** preserve the current project and configure a fresh disposable test
project, or explicitly review/approve cleanup confined to this isolated test
project. No cleanup/reset, invented ledger, guard bypass or runtime migration is
authorized by the reported counts. Keep `.env` and `.env.test` separate and TLS
verified. After the empty prerequisite and read-only probe pass, execute the
real SQL/Auth suite, which owns migration execution. Live sync remains disabled
pending real Auth, HTTPS, staff/device enrollment and desktop acceptance. This
documentation update reruns no local or live test; exact evidence is in STATUS.

## Preceding runtime connection milestone — user terminal verifies PostgreSQL/TLS, 2026-10-04

User-supplied `cd server && npm run db:check` **PASS**: real runtime Session pooler,
authorized TLS and `SELECT 1`, no application data accessed. This verifies the
runtime database connection from the normal terminal; Auth, enrollment, schema
provisioning and desktop sync remain unverified. Codex's preceding runtime probe
still failed with redacted `EAI_AGAIN`; do not merge the two environments' results.
No migration or backend/application change was made while recording this result.

**Next:** resolve the isolated test schema blocker using the updated read-only
`npm run test:inspect` in the normal terminal. Inspect counts before any removal
decision; retain isolation and verified TLS. Complete the real PostgreSQL/Auth
suite on that isolated project before main-project migrations. Runtime and test
env files remain separate. Live sync stays disabled pending HTTPS, authenticated
staff/device enrollment and real desktop acceptance. Exact evidence is in STATUS.

## Preceding test-schema diagnosis — inspect contents before runtime migration, 2026-10-04

Latest user-terminal `npm run test:inspect` verifies matching local test target,
real Session pooler TLS and `SELECT 1`. Only the application schema presence
condition is true; ledger/public relations are absent. Schema contents remain
unknown. The updated command reads only system catalogs and adds numeric
relation/routine/type/dependency counts without names, data or credentials.
Fresh local checks pass: 27 unit/mock tests, probe/helper syntax, focused helper
typing and frontend build. These are not live catalog-query or real Auth results;
the whole-server typecheck remains historically blocked as recorded in STATUS.

**Next:** use the normal terminal's updated read-only inspection to assess the
current schema. Preserve the isolation guard and data; require an explicit
reviewed decision before removal. Complete one real SQL/Auth integration run in
the isolated project before applying the existing versioned migration to the
intended runtime project. Keep env files separate, verified TLS intact and live
sync disabled until HTTPS, enrolled identity/device and actual desktop
end-to-end acceptance. No runtime database operation or deployment is authorized
by this inspection milestone.

## Preceding verified SQL milestone — user terminal passes real PostgreSQL/mock Auth; live suite needs fresh project, 2026-10-04

User-supplied terminal evidence: `npm run test:db` verifies real Session pooler TLS and `SELECT 1`; `npm run test:integration:postgres` passes 1/1 with real SQL/Fastify but mocked Auth. The following `npm run test:integration:supabase` reaches PostgreSQL but correctly fails its empty-application-database guard (0 pass/1 parent failure) because the SQL/mock-Auth suite retained its schema, ledger and fixtures. Live Auth sign-ins and real-Auth integration cases remain unexecuted. Codex's separate live-suite attempt still fails DNS `EAI_AGAIN` before connection. Source inspection confirms this distinction; full results are recorded in STATUS.

**Next:** preserve existing fixtures/guards and configure a fresh disposable Supabase project, distinct from runtime, using ignored `.env.test` and exactly three confirmed synthetic Auth accounts. Use that project's Session pooler/CA/project/Auth settings; keep credentials private. In the normal terminal run test configuration, read-only test probe, then the **live Supabase** suite only after verified TLS/SQL success. Do not run manual migration or the mock-Auth PostgreSQL suite first in the new project. No automatic reset/deletion, production data, deployment or sync enabling. Complete real Auth, remaining strict backend typing, native HTTPS/enrollment/credential storage, desktop reconnect/pull/retry/conflict and Render/Windows checks before claiming live sync.

## Current Session pooler and verified TLS — implemented; local configuration blocks real connection, 2026-10-04

Keep the existing pg 8.23.0 driver. Shared server connection options now require Session pooler/5432, a project-matching role, `sslmode=verify-full`, a readable PEM CA and mandatory certificate/hostname verification. Runtime Fastify/migrations read DATABASE_URL from `.env`; isolated suites/probe read only TEST_DATABASE_URL from `.env.test`. No runtime fallback or driver replacement. Read-only `db:check`/`test:db` separate actual TLS/SQL connection evidence from `test:config` local checks; startup queries `SELECT 1` before listening and driver failures remain value-withheld.

Fresh results: 26 local/mock/driver tests and 23 server syntax checks pass; frontend build passes. Runtime probe stops before network for missing TLS URL settings; test config/probe/live integration stop because the test URL is direct, not the required Session pooler. Live integration exits 1 with 0 pass/1 parent failure, not a successful skip. Strict whole-server typecheck still fails on missing pg declarations and existing request/JSON/error typing. No actual database/Auth connection, migration, fixture write, API listener or deployment occurred.

**Next:** locally complete runtime TLS/CA and Supabase Auth settings; obtain the test project's actual Session pooler hostname from its Connect dialog, preserve isolated test credentials and fix its CA path. Rerun runtime probe, test local validation, test probe, then the live isolated SQL/Auth suite. Never guess the test endpoint or use runtime credentials for tests. Do not migrate test schema first. Full details/results and unchanged offline SQLite safeguards are recorded in STATUS. Keep deployment and live desktop sync disabled until actual HTTPS/Auth/enrollment/end-to-end acceptance; strict server typing and CA provisioning for deployment remain open.

## Current isolated Supabase integration — tests prepared; configuration missing, 2026-10-04

Prepared a real PostgreSQL/**real Auth** suite in `server/test/supabase.integration.ts` and a shared migration runner; migration SQL and all Client/SQLite safeguards remain unchanged. Separate ignored `server/.env.test` and eleven `TEST_*` settings are required, with a fresh disposable project and exactly three confirmed synthetic Auth accounts. Project/TLS/empty-schema/account checks run before any fixture writes. No runtime credential fallback, SQL/Auth mocks, automatic schema reset, deployment or sync enabling. Exact variables and safe setup are in `server/docs/SUPABASE_TESTING.md`/`.env.example`.

Fresh results: `npm run test:config` exits 2 (all settings missing); `npm test` passes 23 unit/mock tests; `npm run test:integration` exits 1 at missing-configuration preflight (0 pass, 1 parent failure, no real SQL/Auth cases run). The separately named `test:integration:postgres` is real-SQL/**mock Auth** and remains skipped. Twenty TypeScript syntax checks and the baseline frontend build pass. Earlier SQLite/Rust/desktop results remain historical, not fresh live evidence.

**Exact next task:** configure a new test-only Supabase project and local `.env.test`, then run `cd server && npm run test:config` and `npm run test:integration`. That command runs migrations itself after isolation guards; do not migrate manually first. Execute/resolve real constraints, staff/device authorization, gym isolation, retries/duplicates/conflicts and pulls, then record actual results separately from mocks. Retain fixtures for inspection; no reset of existing data. This milestone explicitly forbids deployment and live sync enabling. Native HTTPS/credential storage/sign-in/enrollment/session binding, real desktop reconnect/pull acceptance, runtime least-privilege grants, Render HTTPS and Windows verification remain subsequent work.

## Preceding backend enrollment continuation — installed packages usable, 2026-10-04

The retry `cd server && npm install --ignore-scripts --no-audit --no-fund --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache` succeeded (exit 0, `up to date in 1s`) using existing installed Fastify/pg and a real lockfile. Fresh `npm ping --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache` still fails (exit 1, `getaddrinfo EAI_AGAIN registry.npmjs.org`); the crates.io HTTPS probe also fails DNS (exit 6). Installation success does not establish fresh download or live backend access.

Implemented backend enrollment **verification** for already owner-approved staff/devices, deriving gym/role/permissions from verified server records with gym-lock rechecks and read-only retries. No self-approval/role-grant endpoint. Prepared `server/render.yaml` and canonical production HTTPS-origin validation with environment-only secrets; no public deployment or Render validation. Client/SQLite behavior and all existing safeguards remain unchanged; native login/enrollment/credential storage/HTTPS transport are still missing and sync stays unavailable.

Fresh checks: 18 backend unit/real-Fastify-route tests pass with SQL/identity mocks; TypeScript syntax passes. All 77 SQLite/Rust tests (14 labelled mock-engine cases), frontend UI/build, Rust fmt/desktop Clippy/normal desktop build pass. PostgreSQL integration is **skipped** (0 pass, 1 skip, `TEST_DATABASE_URL` absent); neither live Supabase enrollment nor real desktop end-to-end sync has passed. Full commands/results/errors are in current STATUS. PyYAML is absent, so no local parser/provider validation result is claimed for the prepared Blueprint.

**Exact next task:** configure a new disposable test PostgreSQL database and Supabase Auth project in ignored `server/.env`, keeping credentials out of chat; run migration/actual SQL integration and live test-staff/device enrollment. Restore Rust dependency access and implement audited native HTTPS, credential storage and authoritative session/scope binding, then connect a real transport on a dedicated connection. Verify offline create/restart/reconnect with one upload, remote change pull, retries/duplicates, denied auth and conflicting edits using test gym/member data. Complete HTTPS/provider/Windows acceptance before enabling live sync; never report mock tests as online proof. No public deployment or real data is authorized.

## Preceding real member-sync access check — dependency probes blocked, 2026-10-04

Fresh probes from `server/` both failed with exit code 1:

```sh
npm view fastify@5.12.5 version --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
npm view pg@8.23.0 version --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

Exact blocker: `npm error code EAI_AGAIN`, `syscall getaddrinfo`, `errno EAI_AGAIN`; both registry requests failed with `getaddrinfo EAI_AGAIN registry.npmjs.org`. Backend installation stopped as requested. Only documentation changed; no install, migration, application change, deployment or fresh test-suite result is claimed. The existing server source and offline SQLite engine remain intact, with live sync unavailable. Earlier passing mock/local checks below do not establish real backend connectivity.

Resume at dependency installation once access works. Keep backend files in `server/`; review installed pins, generate the lockfile and verify migrations/endpoints against a disposable PostgreSQL database. Implement verified staff/device enrollment with gym authorization derived from authenticated server records, native HTTPS/credential storage and Render HTTPS configuration using environment secrets. Supply the real desktop transport only after verified enrollment. Use test gym/member data for enrollment, offline creation/restart/reconnect with one confirmed upload, server-change pull, retries/duplicates, auth failures and conflicting edits. Run backend, SQLite, frontend and Rust checks and report mock tests separately from real end-to-end results. Preserve durable identity/binding, frozen retries, push-before-pull, transactional page/cursor and restore safeguards. Keep sync disabled until HTTPS/enrollment succeed and claim live success only after actual end-to-end tests pass. No public deployment or real member data is authorized.

## Current continuation — sync engine safeguards verified offline, 2026-10-04

The internal bounded engine now persists canonical server/gym/local-device binding and durable retry state. Device UUID survives database restart; gym identity is assigned only through a protected native enrollment seam and is never invented for an unbound database. There is no actual enrollment flow yet. Server source derives authorized gym access from verified staff/device records; the client gym ID is only a consistency assertion.

All pending member pushes must succeed before pull. Failed errors/deadlines are transactional and survive restart; prior acknowledgements remain intact. Permanent conflicts/unsupported pending member operations block pull. New writes during a pull defer the page inside its transaction without advancing its cursor. Reply commits recheck staff/session; page/history/cursor remain atomic. Fourteen explicitly labelled mock-server sync-engine tests pass; they do not verify live HTTPS/auth/backend connectivity. Settings and the native snapshot keep synchronization unavailable.

Verified: full 77-test Rust suite, final 14-test engine rerun, 10 backend unit tests, UI/render checks, frontend and normal desktop compilation, formatting and Clippy. Actual PostgreSQL/Fastify integration is skipped without a disposable test database. Dependency DNS still fails; no Fastify/pg/PostgreSQL/HTTPS installation is available.

**Exact next task:** restore dependency access, install/review backend packages and generate its lockfile; run actual PostgreSQL/Fastify integration against a new disposable database. Then implement audited native HTTPS, authenticated staff/device enrollment and native credential storage, call the existing protected scope binder only after verified server authorization, and supply a real transport/scheduler to the tested engine on a dedicated connection. Provider/offline-policy decisions, real token/role/reconnect tests, conflict/restore reconciliation and Windows acceptance remain required. Do not enable sync based on mock-server engine tests. No worker or identity-grant IPC is registered.

## Preceding member-sync source/storage milestone — 2026-10-03

The first backend source now lives in `/home/prinzz/development/Projects/ArmStrong/server/`, alongside Client. It retains the Fastify/Supabase PostgreSQL/Auth/Render proposal. No provider deployment or verified native staff enrollment exists yet.

Inspect/map → server README/config/migrations/authenticated member endpoints → durable SQLite member push/pull/conflicts → verification is the implementation order. Member IDs/NFC normalization, local/remote versions, archive actor references and history must be reconciled explicitly. Preserve nonmember outbox/history; acknowledge only server-confirmed operation IDs. Do not reuse browser simulated sync or synchronize other modules in this milestone.

The previous server-directory write restriction is resolved. README/ignored environment configuration, checksummed PostgreSQL migration, Supabase verification and staff/device authorization, idempotent member endpoints and ordered pull source are present. SQLite v5 implements internal frozen delivery, matching acknowledgement, independent remote revisions, atomic pull cursors, retained conflicts and backup compatibility. Settings displays conflicts and keeps sync disabled. Local archive actor subjects are checked against server receipt actors; pulled actors map to existing local subject/FKs or inactive references with no role/session grants. Other queues and all original outbox/history remain intact.

**Current blockers:** npm registry DNS fails (`EAI_AGAIN`), Fastify/pg are not cached (`ENOTCACHED`), PostgreSQL is absent, and the Rust HTTPS dependency is not cached. Backend syntax/protocol/auth-response tests and real SQLite tests pass; Fastify/PostgreSQL/live Supabase/HTTPS tests are unperformed. Native HTTP worker, verified sign-in/enrollment, OS credential storage, backoff/reconnect, actionable conflict resolution, server/gym binding, and restore reconciliation are not yet implemented. No server-confirmed production operation exists.

**Exact next task:** restore dependency/network access, install/review the pinned backend dependencies and generate its lockfile; provision a new disposable PostgreSQL database and run `cd server && npm test && npm run test:integration` with `TEST_DATABASE_URL` (without it the integration test is explicitly skipped). Fix actual SQL/route integration failures before deployment. Then integrate an audited Rust HTTPS client and verified native staff/device enrollment: bind configured server/gym/local device to this database, keep credentials native and out of backups/webview storage, recheck roles/expiry, establish the private session only from verified identity, retry frozen member requests before pulling, and wire these internal storage methods to real replies. Obtain provider/account/offline-policy decisions rather than granting demo identity. Add real dropped-response/restart/reconnect and token/role tests, implement authorized conflict/restore reconciliation and validate Windows/native GUI before enabling sync. Follow server/README for configuration and least-privilege provisioning; never add backend files inside Client.

## Preceding removal milestone — 2026-10-03

The user-authorized removal work adds member archive/unlinked deletion and reasoned expense voids while retaining history and existing styling. Audit/relationship map was updated before schema changes. V4 native operations, confirmation/history controls, derived effective expenses, audit/outbox/replay and backup compatibility are implemented and tested.

51 SQLite tests, 12 adapter/API tests, route/settings/browser/receipt/removal render checks, frontend build, Rust fmt/Clippy and Linux native compilation pass. Native GTK fails before window execution. More importantly, the current app has no verified staff login/enrollment, so protected saves remain locked and native commands deny the unauthenticated local operator. Test-only trusted sessions are transaction/permission tests, not a production login.

**Exact next task:** connect agreed native verified staff identity/offline enrollment to the private session, retaining active Administrator checks/expiry and no webview identity grant. Then verify native Administrator/Reception removal behavior in a usable GUI, including archive history, linked deletion denial, unlinked deletion, expense void/reason/duplicate/report/audit/restart. Run normal desktop build afterward. No synchronization/identity mock, demo privilege or OS package changes are authorized by this removal work.

## Preceding local finance milestone — 2026-10-03

The user-authorized finance milestone follows the completed interface/local-persistence work. Preserve the existing React interface, separate browser demo and native SQLite storage. The relationship map was updated before schema changes; STATUS contains current evidence and gaps.

| Milestone | Outcome |
| --- | --- |
| F1 — focused finance audit/map | Completed: inspected actual v2 schema, payments, explicit membership dates, migration/recovery and tests; documented current rules and decisions. |
| F2 — invoices/allocations/balances | Schema v3 and transactional commands implemented; member/optional membership invoices, repeated partial allocations, capped settlement and overpayment credit. Existing financial history and old request receipts retained. |
| F3 — renewal/reversal | Explicit successor dates + price-snapshotted membership/invoice transaction; captured plan/history guards. Full payment reversal appends matching cash/reason/releases/audit/outbox without deleting originals; duplicate protection tested. |
| F4 — saved receipts/interface | Immutable unique numbered saved payment/reversal snapshots, historical reprints, desktop finance forms, preview component and real system print action. Browser stays separate; signed cash reports and v3 backups/restore implemented. |
| F5 — verification/native acceptance | 42 core tests, nine adapter/API tests, route/settings/demo/receipt render checks, TypeScript/Vite build, Rust formatting/Clippy and normal Linux desktop build pass. GTK fails before window launch; native interactive form/restart and print-dialog/Windows printer acceptance remain unverified. |

No automatic renewal policy was invented. Dates remain staff-selected, inclusive and nonoverlapping; payment does not trigger activation/renewal. Membership correction, invoice cancellation, partial refunds and sale/expense reversals remain outside this bounded milestone. No tax/discount/printer/sync configuration options were added.

**Exact next task:** run expanded `npm run test:desktop` in a usable display session, then validate original/reversal receipt system preview/cancel/reprint/PDF/Windows printer without new financial writes. Run `npm run desktop:build` afterward to restore the normal binary. Use offline/resource flags in STATUS. Resolve automatic dates/month-end/gaps/grace/debt-admission and receipt format before extending policy. Do not change GLIBC/unrelated OS packages or implement synchronization before deployment topology/identity are agreed.

The remaining release plan is a proposal. Day 1's identity/backend/Windows gates are still open; local persistence and Linux compilation do not establish a production release or a three-day full-scope promise. Preserve the interface; exclude workouts/body progress.

## Architecture proposal

| Layer | Concrete choice |
| --- | --- |
| Desktop | Tauri 2, existing React/TypeScript/Vite UI; narrow Rust commands for validated business operations and authorization. No arbitrary SQL exposed to the webview. |
| Local storage | Rust `rusqlite`, bundled SQLite in app-data, versioned migrations, foreign keys, WAL, `synchronous=FULL`, busy timeout and serialized writes. One local database per device, never a shared network-drive database. |
| Online service | TypeScript/Fastify HTTPS API on Render; managed Supabase PostgreSQL and Supabase Auth. Desktop accesses business data through the API; database credentials and privileged keys stay on the server. Hosting/project creation still needed. |
| Authentication | Admin-provisioned Supabase accounts; API verifies access tokens and current gym/role membership. Rust stores refresh credentials in the OS credential store, enforces local roles and owns sessions. No demo credentials. |
| Offline access | After first online enrollment, a server-signed, device/user-bound role grant plus a separately enrolled local PIN hashed with Argon2id permits local unlock. Proposed expiry: seven days, pending owner decision. Rate-limit failures; keep grant/verifier protected locally. Revocation cannot take effect while disconnected until grant expiry; expired grants require online reauthentication. |
| Delivery | Linux development; Windows CI build and Windows hardware acceptance. NSIS installer with offline WebView2 provisioning; agree signing/release ownership. |

### Data and synchronization contract

- Normalize members, plans, membership periods, invoices/payment allocations, receipts, attendance events, products, sales/lines, stock movements, expenses, users/roles, audit, outbox and sync cursors. Use UUIDs internally, stable plan IDs, unique normalized nonempty NFC UIDs, positive quantities and integer minor-unit amounts. Snapshot sold prices and receipt details. Derive status/counts from records.
- Every business command commits its records, stock/ledger effects, actor/device audit and outbox operation in one SQLite transaction. Acknowledge success to the UI only after durable commit. Use UTC instants and explicit Asia/Colombo business dates; renewal dates are calendar dates.
- Rust worker pushes immutable operations with operation ID, device ID, schema version and expected entity version. API transaction validates permissions/business rules, records operation ID uniquely, applies all effects and appends a sequenced change record. Duplicate delivery returns the original result.
- Remove only individually acknowledged outbox IDs; retain new, rejected and failed operations. Retry transient failures with backoff. Show actual server acknowledgement and last successful sync; connectivity alone never means synced.
- Pull paginated changes after a durable server cursor; apply each page and cursor atomically. Include tombstones and initial bootstrap. Preserve unacknowledged edits during pulls. Conflicting versions become visible reconciliation tasks; do not use silent last-write-wins for money, memberships or stock.
- Recommend one authorized editing device for the three-day core, with backend-enforced writer registration. Design protocol for future devices, but defer multi-writer release unless explicitly required. Concurrent offline sellers cannot guarantee globally available stock without allocating stock per device or requiring online authorization; owner decision is a release gate.
- Posted financial records use linked reversals, not destructive edits. Current device/transaction-UUID receipt identifiers avoid offline collisions; a prescribed numeric sequence remains an owner receipt-format decision. Audit includes real user/device and before/after or event details; local audit alone is not tamper-proof against a machine administrator.
- Create versioned SQLite snapshots using the backup API, checksum manifest and restricted file access; include outbox/cursor consistently and exclude authentication secrets. Validate schema/integrity in a temporary database, take a pre-restore backup, then replace safely. Old-backup restores require server reconciliation before sync resumes so acknowledged operations are not duplicated. Keep an off-device copy and test recovery; cloud sync is not backup.
- Fresh production installs start empty. Import existing browser data only through an explicit validated migration, never automatically importing demo seeds.

References checked: [Tauri Windows packaging](https://v2.tauri.app/distribute/windows-installer/), [SQLite WAL](https://www.sqlite.org/wal.html), [SQLite backup API](https://www.sqlite.org/backup.html), [Supabase Auth](https://supabase.com/docs/guides/auth). The offline grant and synchronization protocol above are proposed application work, not built-in Supabase features.

## Three-day milestones and gates

| Milestone | Work | Required evidence |
| --- | --- | --- |
| Day 1: durable foundation | Tauri shell; migrations and transactional commands; empty production data; members/plans/membership periods; identity/roles; backend skeleton and early Windows build. | Build on Windows; restart persistence; rollback/constraint tests; role denial and offline unlock tests; renewal month-end/expiry tests. |
| Day 2: operations and real sync | Manual/HID attendance; membership renewal and payment allocation; receipt rendering/reprint; simple sales/stock, expenses; deployed API push/pull and visible failures. | Duplicate scan handling; atomic sale/payment tests; no oversell locally; real server persistence; dropped response/retry without duplicates; concurrent queue additions retained; disconnect/restart/reconnect. |
| Day 3: recovery and release | Date-filtered daily cash/attendance/membership/stock/expense reports and CSV; audit viewer; validated backup/restore; Windows installer, reader and printer acceptance. | Totals reconcile to ledgers; corrupt backup rejected; restore drill; pending writes survive forced restart; clean offline Windows install; receipt reprint does not create another payment; end-to-end staff walkthrough. |

Target a limited pilot: one editing PC, Admin/Reception roles, basic plans/renewals, simple retail sales, recorded payment methods (no card processing integration), essential reports and manual backups. Proposed deferrals: advanced analytics, unattended updates, complex promotions/freezes/refunds UI, multi-device offline editing and non-HID reader integrations. Owner must agree scope; do not claim deferred items complete.

The entire requested system is not credibly guaranteed tested in three days from this prototype. Identity, synchronization, Windows/hardware access and financial rules are substantial risks. Start the clock after critical decisions/access; retain failed release gates as blockers, never replace them with mocks. If time runs out, report a narrower pilot or delay release.

## Owner decisions needed

1. **Topology:** one or multiple editing PCs? Must each edit offline? If multiple, agree stock allocation/conflict ownership and revise schedule.
2. **Hardware/release:** Windows version/architecture, access to a Windows test PC, reader model and HID vs PC/SC mode, printer model/paper width, signing ownership.
3. **Hosting:** approve proposed providers/budget or identify existing infrastructure; nominate account owner, region and backup retention/location.
4. **Business rules:** renewal start/end and month-end handling, grace/freeze policies, partial payments/debt, discounts/refunds, receipt numbering/details and expired-member attendance behavior.
5. **Access/data:** role permissions, offline login duration and account recovery, existing real data/import requirements, expected member volume, and minimum day-three acceptance scope.


## Milestone 1a — local foundation test boundary

Implemented the decision-independent part of Day 1: Tauri shell, version-1 SQLite migration, empty database, member/card add/edit, plan add/edit and explicit membership-period entry/history. Each native command commits its record, audit and outbox together. The existing browser prototype remains separate and explicitly labeled; it is not migrated or silently imported.

This is a **local test build**, not a production core release. Authentication/roles could not be implemented as planned: online enrollment/hosting decisions remain unanswered and dependency downloads fail with DNS `EAI_AGAIN`; the proposed design's password-hashing library is not cached. No substitute password scheme or fake authorization was added. The native UI states that anyone operating this computer can edit these test records. Audit identifies an unauthenticated local test operator, not a staff account.

Policy pending: membership start/end dates are entered explicitly, with inclusive end date and no overlapping periods. The seven-day “Expiring” display is a test convention, not an approved admission policy. Plan duration is descriptive until renewal calculation is agreed. No payment is implied by membership creation. Periods cannot yet be corrected or reversed.

Verified: frontend build, Linux desktop compilation, nine SQLite tests and Rust core lint. Native UI/restart acceptance is blocked before launch by the host WebKitGTK/glibc mismatch (`GLIBC_2.44` required; host 2.43). Windows is untested. Details and test instructions are recorded in `STATUS.md`. Stop here for owner testing; do not start Day 2 or synchronization. Remaining Day 1 work: resolve topology/identity/hosting decisions, implement authenticated enrollment and roles, backend skeleton, approved renewal calculations and Windows compilation/acceptance. Do not mark Day 1 complete from Linux checks alone.

## Shared-interface integration — 2026-10-03

The subsequent user request authorizes reusing the complete existing React interface in Tauri. Both runtime entry paths now render the same nine-page app with one navigation registry. Desktop uses an independent provider for the existing SQLite commands; browser demo storage/sync remains separate. Native members/plans/period workflows are retained, while screens lacking native operations display their gaps and disable writes. No Day 2 business modules or synchronization were implemented.

Frontend/desktop compilation, core SQLite tests and all route/component rendering checks pass. The former glibc mismatch is resolved (host now 2.44), but actual native webview acceptance fails at GTK initialization in this sandbox. Next acceptance remains the expanded native UI/restart smoke test in a usable display session, then the normal build, Windows/hardware verification and the remaining Day 1 decisions/gates. See the current integration section in `STATUS.md` for screen-by-screen coverage.
