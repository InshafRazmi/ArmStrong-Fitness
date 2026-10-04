# Armstrong database relationship and UI storage map

Audit date: 2026-10-03. This file was created **before application/schema changes** from the actual code. The baseline and implementation target are distinguished below. This is a local persistence contract; it does not approve a synchronization topology, admission policy or authentication design.

## Backend typing continuation — 2026-10-04

Latest setup continuation adds only server configuration/probe/file-provisioning
tools; native SQLite v6 and SQL relationships below remain intact. Provisioning
reads actual device/preparation/scope/restore metadata without writes and creates
only the three public native config fields beside the existing database. No
fixture, financial/audit/outbox row, role, credential or sync cursor is changed.

Real read-only connector queries now reach the matched runtime Supabase project:
two fixture-named gyms, zero active Administrators, two active devices/writers,
one member, six RLS-enabled private tables and no `armstrong_api` login. Public
Auth/database roles lack private schema usage. This supersedes a blanket SQL
access limitation; actual Node-driver TLS, native enrollment and live sync remain
unverified. The API origin/installation settings are still pending. Production
startup rejects owner/reserved logins without changing grants. See current STATUS
and [setup limits](../../server/docs/LOCAL_SETUP.md).

Full strict server typing and a separate clean locked offline installation now
pass with official unchanged pg declarations pinned in `server/vendor/types-pg`.
Their upstream commit/blob/license/SHA-256 metadata is retained and checked.
Runtime pg, SQL schema/migrations, grants and native SQLite relationships did not
change. Read-only probes now validate an actual Node `TLSSocket` before accepting
its encryption/authorization state. The installed-driver fixture narrows its
private connection-parameter object at runtime without changing upstream types.

48 local/mock checks and frontend build pass. The real runtime probe still fails
DNS before TLS/SQL; no actual enrollment, registration, API deployment or sync
was enabled. No populated env, secret, private cache or owner permission changed.
See [source provenance](../../server/vendor/types-pg/README.md) and current STATUS.

## Member conflict review contract — 2026-10-04

SQLite v6 adds explicit conflict review through `006_member_conflicts.sql`.
Native preview/resolve IPC requires an enrolled, current Administrator writer
and the unchanged saved server/gym/device binding. A fingerprint covers the
actor/scope, local member, active conflicts, remote head and pending original
payloads/requests. Resolve repeats those checks inside an IMMEDIATE transaction.

| Relationship | Purpose |
| --- | --- |
| `member_conflict_resolutions.actor_user_id → users.id` | Immutable actor, reason, choice and complete reviewed state. |
| `member_resolved_conflicts.conflict_id → member_sync_conflicts.id` | Links retained conflicts to their review; `member_active_conflicts` excludes covered records without deleting them. |
| `member_resolved_operations.operation_id → outbox.id` | Explicitly discards/supersedes same-member rejected/unsent operations. SQL guards refuse other members/entities and pending/acknowledged deliveries. |
| Resolution links → `member_conflict_resolutions.id` | All links, member application or fresh retry, audit and local idempotency receipt commit together. |

Use-server applies the recorded valid member and retains NFC assignment history.
Keep-local is limited to active versions and queues a new update UUID against the
recorded server revision. Original outbox/delivery/receipt/conflict history stays
immutable. Pending/order/pull queries exclude retired operations, while only real
matching receipts increase acknowledgement counts. Financial, attendance,
membership and receipt snapshot history is preserved. Server archive actors are
inactive identity references unless already mapped to a verified local subject.

Unconfirmed requests, invalid/inconsistent snapshots and protected archive/card/
identity/joined-date/deletion cases remain gated. No caller supplies server data or
actor identity. Backups include all three ledger tables; older backups migrate in
isolation and restore continues to require reconciliation. Runtime permission is
excluded from restore storage fingerprints. No live sync scheduler or remote
schema changed. See [review details](MEMBER_CONFLICT_REVIEW.md) and current STATUS.

## Previous native login/enrollment continuation contract — 2026-10-04

Current continuation: native HTTPS/OS credentials and login/logout/status commands
are now wired through the desktop. All configured business IPCs require the
memory-only verified Administrator; write commands also check the enrolled device
writer grant. Unconfigured test mode stays explicitly labelled. No migration or
schema version changed; no actual runtime device was enrolled by Codex.

| Native state | Storage / relationship |
| --- | --- |
| `metadata.native_device_secret_sha256` | SHA-256 of the native OS-store secret's UTF-8 hex string, written only after exact readback. Backups contain the hash, never the credential. Registration now requires this same hash. |
| `metadata.native_auth_required` | Durable requirement after native config is provisioned; deleting configuration cannot turn the database back into unrestricted test mode. Preserved by authenticated restore. |
| `temp.native_actor` | Private connection-local verified user ID/label for a protected command; cleared at logout/denial, excluded from backup. |
| `audit.actor_user_id` / financial actor labels | New configured business writes point to the verified existing `users.id` and snapshot its display label; historical actor rows remain. |
| `outbox.payload_json.actorUserId` | New authenticated operations retain local actor provenance; existing member protocol maps that actor to the verified Auth subject. |

Windows secrets use Credential Manager; Linux development uses the persistent
Secret Service. Credentials/tokens are not webview configuration. Native auth
settings are provisioned outside the webview beside SQLite; only device UUID,
path and hash are returned for registration. Account tokens remain native memory;
restart/expiry/logout lock access. Authenticated restore requires an existing
matching actor mapping in the backup and preserves the auth requirement before
replacement. OS/HTTPS/GUI/API/Windows acceptance and actual sync scheduling remain
pending; sync is disabled. See [native setup](NATIVE_SIGN_IN.md) and current STATUS.

Before implementing native identity binding: reuse v5 `users`, `roles`,
`user_roles` and `metadata`; no SQLite or server migration is needed. A private
native login must verify the Auth subject online, then verify the existing API's
enrollment response for the same subject and persisted `metadata.device_id`.
Only that response supplies gym and role. Importing an inactive remote archive
actor is an identity reference and grants no session until this verification.

Commit local identity/role reconciliation and the immutable HTTPS/gym/device
binding in one transaction. Preserve existing local actor IDs and all history;
refuse subject/email collisions and a mismatched binding, restored database or
unscoped remote state. Role changes must replace prior role assignments from
the authoritative enrollment. Set the memory-only session only after commit;
failure, expiry and logout must leave privileged access locked. Persist no token,
password or device secret in SQLite, snapshots, audit/outbox or backups. Native
HTTP adapter/Auth mocks remain distinct from production TLS and live enrollment.

Implemented this private contract with twelve Auth/enrollment/SQLite mock checks
and eight HTTP-adapter checks. Read-only devices also cannot transmit queued
member changes. Sessions remain memory-only and lock on restart. There was no
production login command at the source-only milestone; the continuation above adds
login command wiring. No actual runtime SQLite database was enrolled.

Date-filtered report summaries and CSV now share a consistent native snapshot:
inclusive saved business dates for cash/attendance, Colombo dates for UTC audit
instants, membership interval overlap and explicitly current stock. These are
read-only additions; no schema/relationship/migration changed. Reversal posting
dates and expense void history remain intact; unsafe aggregate amounts fail
without rounding. 103 core tests, UI checks/build and core lint pass. Desktop
build passes after generated-cache cleanup; GTK blocks UI acceptance.
STATUS/PLAN record pending external
setup and live acceptance. Sync remains disabled.

## Current registration contract — administrative review/apply prepared, 2026-10-04

New runtime-only administrative tooling maps the one online-verified, same-project
confirmed Auth UUID to the approved gym's `armstrong.staff` Administrator row;
there is no additional login account. `registration:check` reads state only.
`registration:apply` inserts missing approved gym/staff/device records inside a
transaction with advisory/gym locks; exact retries preserve IDs, roles and hashes.
Conflicting mappings, names, revocations or writers fail without updates or
deletion. The runtime API still derives access from approved records and does
not expose role/device approval or trust client-supplied gym authorization.

Device approval is optional. When native credential storage exists, the tool
reads the existing SQLite `metadata.device_id` through a read-only connection,
rejects restored/mismatched scope, and writes only its native-secret hash to the
server. It creates no SQLite database or replacement device, applies no SQLite
migration, grants no local session/role and saves no sync binding. Gym/Admin-only
setup does not fabricate a device. Existing actor FKs and authenticated-subject
reconciliation remain unchanged; no migration/checksum/schema change was needed.

New SQL-mock and temporary-SQLite metadata checks pass; all 77 existing Rust/
SQLite core tests pass, including the 14 mock-server engine cases. Full server
typing still needs missing pg declarations. Actual registration review stopped
before Auth/SQL at missing local settings; no runtime registration was applied.
The new isolated real SQL/Auth registration case remains unrun. No populated env,
secret, local history, retry/push-before-pull/cursor/restore safeguard or native
availability flag changed. STATUS/PLAN record exact results. Live sync is disabled.

## Current enable request — no verified desktop binding or transport yet, 2026-10-04

The user requests live sync, but native production HTTPS transport, login,
enrollment and secure credential storage are absent and runtime HTTPS API origin
is not configured. Fresh Codex npm/database probes fail DNS with `EAI_AGAIN`;
the database probe never establishes verified TLS or accesses SQL. No account,
role/device registration, schema/migration, env, local data, secret or binding
changed. The preceding user-terminal real Auth/schema passes remain separate
evidence, not live member-sync acceptance.

Preserve SQLite's existing `metadata.device_id`, immutable bound
server/gym/device scope, authenticated actor subject, archive/receipt actor
relationships, retries, push-before-pull and transactional pull/cursor saves.
Server enrollment must continue deriving gym access from approved identity/device
records. Complete the HTTPS endpoint and native login/enrollment/secure storage/
transport, then verify real synthetic-data member-sync acceptance before enabling
availability. Only documentation changed in this step; no mocks were rerun or
presented as live evidence. Live sync remains disabled.

## Current identity verification — live Auth passes; authorization relationships still pending, 2026-10-04

The user-terminal runtime `npm run auth:check` **PASSES** real Supabase password
sign-in and online subject verification. Gym/role/device registration and desktop
login were explicitly not checked; no application database changes were made.
This supersedes the initial missing local probe settings but does not establish
Administrator permissions or enrolled SQLite scope.

The intended relationship remains one Supabase Auth identity mapped internally
to its approved `armstrong.staff` Administrator row, one stable gym and an approved
device. This is not a second login account. For desktop enrollment, retain the
UUID saved in SQLite `metadata` under `device_id`; the worker rejects a binding
whose device differs from that persisted identity. Server enrollment derives gym
access from approved identity/device records. Existing operation/archive actor
FKs, verified-subject reconciliation, durable retry state, push-before-pull and
transactional pull/cursor persistence remain intact. Auth verification creates
none of these registration records or SQLite bindings.

Only documentation changed in recording the user-reported live Auth result;
no schema, migration/checksum, environment, permission, device secret or local
data changed. No network/database requests or tests were rerun. STATUS and PLAN
record pending registration, native HTTPS/login/secure storage, real enrollment
and member-sync acceptance. Live sync remains disabled; member-only scope is
unchanged.

## Preceding identity contract — one authenticated Administrator, existing actor relationships retained, 2026-10-04

The user's clarification is one Administrator login for attendance and all
controls. Authenticate that single account through Supabase Auth; its verified
user UUID maps to the approved gym/Administrator row in `armstrong.staff`. This
is one account plus an internal authorization/audit record. Existing operation/
archive actor FKs and native subject reconciliation remain applicable; no device-
only actor migration, fabricated identity or extra operational staff account is
needed. The earlier interpretation/choice is superseded. Device permissions and
gym access remain derived from approved server records. Member-only sync scope
does not expand to attendance because of this account decision.

New runtime-only `auth:check` performs password sign-in and online subject
verification, using optional local probe credentials with redacted output and
no database registration/role/device write. Auth response JSON typing is corrected
before the existing UUID validation. Schema, migration/checksum, FK, driver, env
values, audit receipts and SQLite binding/retry/pull/cursor/restore safeguards are
unchanged. Fresh 33 local/mock checks, focused Auth typing and frontend build pass;
full server typing remains blocked. The live command stops at missing local probe
credentials, so no actual Auth request has passed yet. Runtime SQL/TLS/table/RLS
metadata PASS is separately user-supplied. STATUS/PLAN contain the next Admin
verification/provisioning/native login steps; live sync remains disabled.

## Preceding identity interpretation — no staff login; actor contract decision pending, 2026-10-04

The user declines staff-account authentication. Verified runtime SQL/TLS, version
1 checksum and six private-table/RLS flags remain unchanged. Enrolled-device
authentication versus deferral is pending; no new authorization or schema
relationship is implemented yet. Existing `member_operations.actor_user_id` and
`members.archived_by_user_id` still reference `armstrong.staff`, and the native
SQLite receipt/archive paths preserve enrolled actor subjects and local history.

A device-only design must introduce an explicit approved device permission/audit
actor contract and reconcile those FKs/native relationships while preserving
historical actors, immutable receipts and stable database IDs. Server records
must derive gym access; client gym IDs cannot grant access. Keep the applied
checksummed migration intact and use a new migration for any change. The existing
staff guide is marked implementation reference. No route, FK, env, driver,
registration, grant, SQLite safeguard or live-sync availability changed; no new
test/database/deployment operation ran. STATUS/PLAN record the pending choice.

## Preceding runtime migration metadata verified — staff/device relationships still require provisioning, 2026-10-04

The user supplied runtime `npm run db:verify` with real PostgreSQL/TLS, recorded
version 1 checksum, six required private tables and six RLS flags all PASS. This
verifies runtime catalog/ledger metadata without accessing gym/member/staff/device
rows. Full constraint/grant, real Auth/enrollment/member API and desktop acceptance
remain separate. Codex's preceding verifier still failed DNS (`EAI_AGAIN`).

Reviewed relationships for the next step: a confirmed same-project Auth account
must be verified online, then administratively mapped to an approved gym/role
in `staff`; the approved device maps to that gym, preserving actual desktop
device UUIDs. The API derives scope from those active rows and the device secret,
not client gym/role claims. Its secret hash uses the 64-character lowercase hex
string as UTF-8. The new `server/docs/STAFF_DEVICE_SETUP.md` documents preparation;
no account/registration, grant, FK, schema, migration, env, driver or SQLite
relationship changed. Test readiness/approval information and actual Auth,
native HTTPS/enrollment/credential storage and live-sync acceptance remain open;
sync is disabled. STATUS/PLAN record the supplied result and remaining work.

## Preceding runtime migration reported applied — metadata verification pending, 2026-10-04

The user reports the existing six-table runtime migration succeeded from the
normal terminal. Its checked-in relationships remain unchanged. The new
runtime-only `npm run db:verify` checks the migration ledger/checksum and required
private-table/RLS catalog metadata inside a read-only transaction after verified
TLS/`SELECT 1`. It reads no member/gym/staff/device rows and changes no relationships,
schema, grants, env, pg driver, Auth authorization or SQLite safeguards. It does
not certify full constraints/grants, enrollment or live member sync.

Fresh local/catalog-mock tests pass 30/30; focused helper typing, syntax and
frontend build pass. Full server typecheck remains blocked; Codex's verifier
fails `EAI_AGAIN` before a live connection/catalog query. Obtain the normal-
terminal verification result before approved staff/device provisioning. Test
isolation and real Auth/HTTPS/native enrollment/desktop acceptance remain
required; live sync is disabled. STATUS/PLAN contain exact commands and results.

## Preceding requested runtime provisioning — existing project, unchanged six-table migration, 2026-10-04

The user authorizes creation of the existing planned schema in the runtime
project configured by `.env`, without creating a fresh project. The unchanged
transactional/checksummed migration creates `armstrong.gyms`, `staff`, `devices`,
`members`, `member_operations` and `member_changes`, their gym/member/staff/device
FKs, unique keys, immutable history/archive triggers and private RLS boundary.
No source relationship or SQLite safeguard changed. This request supersedes the
earlier recommendation to gate runtime schema setup on isolated test acceptance;
it does not authorize replacing an existing schema or skipping Auth/HTTPS gates
for live sync. Integration tests retain test-only configuration.

Codex's read-only runtime preflight fails DNS (`EAI_AGAIN`, exit 2) before any
catalog query or verified TLS; no migration or remote schema change occurred.
The user's earlier normal-terminal runtime `SELECT 1`/TLS PASS is separate.
Run the already available `npm run migrate` there, then verify six tables,
migration ledger/checksum and RLS metadata. Existing test-project catalog counts
do not describe runtime schema contents. Full commands/results and remaining
staff/device provisioning, restricted grants and live-sync gates are in STATUS.

## Preceding isolated schema contains objects without a ledger — 2026-10-04

User-terminal read-only inspection verifies test SQL/TLS and reports 15 schema
relation objects, 3 routines, 12 types and 9 dependency records. Public migration
ledger and public application relations are absent. Relations are catalog
objects, not member rows; data contents and exact definitions are unknown.
The six tables/nine indexes/three functions in the checked-in migration are
consistent with these counts, but no checksum/application provenance is proved.
Do not adopt the schema by inventing a ledger or weakening the guard. The live
Auth suite correctly refuses existing application objects before initialization.

No schema/FK, migration, driver, gym authorization, TLS, env or SQLite relationship
change occurs in recording this evidence. Preserve this project and use a fresh
isolated test target, or review explicitly authorized cleanup of the test-only
schema. Keep runtime separate, complete real SQL/Auth acceptance and retain live
sync/HTTPS/enrollment gates. STATUS/PLAN record exact results and the unresolved
preservation/cleanup decision; no database operation or new test ran in this step.

## Preceding runtime PostgreSQL/TLS evidence — no schema relationship change, 2026-10-04

User-supplied runtime `npm run db:check` passes real Session pooler connection,
authorized TLS and `SELECT 1`. The query accesses no application data and does
not establish migration state, Auth, staff/device/gym relationships or desktop
sync. Codex's preceding probe separately failed DNS (`EAI_AGAIN`). Recording the
normal-terminal result changes no schema/FK, migration, pg driver, authorization,
env file or SQLite safeguard. Keep runtime and isolated test projects separate;
inspect the isolated schema blocker and complete real SQL/Auth acceptance before
main-project migration. No deployment or live-sync enabling occurred. See STATUS
and PLAN for exact commands/results and remaining HTTPS/enrollment gates.

## Preceding schema-only isolation blocker — read-only metadata diagnosis, 2026-10-04

Latest user-terminal inspection verifies test TLS/`SELECT 1` and local target
matching; application schema is present, public migration ledger and public
relations are absent. The schema's contents are not yet known. The earlier
SQL/mock-Auth suite's retained fixtures cannot be assumed to describe this
latest target. `server/scripts/check-database.ts` now also obtains numeric counts
of application-schema relations, routines, types and namespace dependency
records via `readApplicationSchemaObjects`. Only PostgreSQL system catalogs are
read; no member rows or object/credential names are displayed. No FK, schema,
migration, driver, TLS, gym authorization or SQLite relationship changes occur.
The guard still rejects any existing application schema, including an empty one.

Local verification passes 27 unit/mock tests, helper typing, probe/helper syntax
and frontend build; new catalog SQL has not run against a live database in Codex.
Inspect those counts in the user's working terminal before considering any
cleanup. Keep test/runtime separation and complete isolated real Auth/SQL
acceptance before main-project migration; no deletion, reset, guard weakening,
deployment or live-sync enabling. STATUS/PLAN record exact commands and evidence.

## Preceding real PostgreSQL evidence and isolation guard — 2026-10-04

User terminal results: read-only test probe verifies real Session pooler TLS and `SELECT 1`; PostgreSQL/**mock Auth** integration passes 1/1, exercising existing migration/member/receipt/change relationships. That suite retains its application schema, public migration ledger and synthetic fixtures. The subsequent **real Supabase Auth** suite reaches PostgreSQL and rejects this existing application state at `lockEmptyDatabase` before Auth sign-ins or migrations (0 pass/1 parent failure). These are separately attributed results, not live Auth or desktop sync success.

No schema, FK, driver, authorization, migration, SQLite relationship/safeguard or environment change was made while recording these results. Preserve fixtures and the rejection guard; do not delete/reset or bypass it. The next real-Auth run requires a fresh disposable Supabase test project, exactly three confirmed synthetic Auth accounts, isolated project-matched settings and a successful read-only TLS/SQL probe before the live suite owns migrations. Do not run the PostgreSQL/mock-Auth suite there first. Codex DNS remains separately blocked (`EAI_AGAIN`); normal-terminal results are recorded in STATUS/PLAN. Native HTTPS/enrollment/credential storage, desktop/Render/Windows acceptance and full strict server typing remain open; live sync remains disabled.

## Preceding Session pooler connection continuation — 2026-10-04

Backend connections retain pg 8.23.0 and the existing private PostgreSQL/member schema. `server/src/database.ts` supplies shared Session pooler/5432 options with an explicit local CA, mandatory certificate verification and unchanged Node hostname checks. Runtime `.env`/DATABASE_URL and isolated `.env.test`/TEST_DATABASE_URL never fall back to each other; tests match the disposable project owner and retain empty-database/migration guards. Read-only probes query `SELECT 1`; startup checks SQL before listening. No schema, FK, migration, native SQLite binding/retry/pull/cursor/restore or gym authorization relationship changed.

Fresh evidence: 26 local/mock tests and 23 server syntax checks pass; runtime/test probes and real SQL/Auth integration are blocked before connection by local Session pooler/TLS settings. No real SQL/Auth or live sync result. Runtime Auth configuration, test pooler host/correct CA, strict server typing, deployment HTTPS/CA provisioning and native enrollment/transport remain open; see current STATUS/PLAN for exact commands and results.

## Preceding isolated Supabase integration contract — 2026-10-04

Before test-runner changes: use a separate ignored `server/.env.test`, exclusively `TEST_*` configuration, and an explicitly disposable Supabase project. Match HTTPS Auth origin/project reference to the direct database host or session-pooler username; require verified database TLS and reject connection-string overrides. Refuse existing Armstrong schema/migration ledger or public application relations before any fixture writes. Hold a dedicated session advisory lock against concurrent suite initialization. Do not reset an existing database or use production data. Configuration must identify exactly three confirmed synthetic Auth accounts; compare verified online subjects against the same database's Auth records before application writes.

Extract the unchanged checksummed migration algorithm into a callable runner so both CLI and real integration tests execute it. The new suite must use actual Supabase password sign-ins and online user verification plus real PostgreSQL/Fastify, without SQL or Auth mocks. Provision only synthetic gym/device/staff mappings for three pre-created test Auth accounts, matching their verified subjects against the database Auth records. Test constraints, registration/access/roles/revocation, two-gym isolation, operation retries/conflicts, pull cursors and transactional rollback. Retain fixtures for inspection; test configuration missing is an explicit failure, never a passing live result. Existing PostgreSQL/Auth-mock and SQL/Auth-mock suites remain separately named/commanded. No Client/SQLite schema, session, binding, retry or pull behavior changes; no deployment or live desktop sync enabling.

Implementation/source now follows that contract: shared `server/src/migrations.ts`, live `test/supabase.integration.ts`, separately invoked PostgreSQL/Auth-mock suite, configuration/isolation helpers and exact environment/setup documentation in `server/docs/SUPABASE_TESTING.md`. Static SQL migration relationships are unchanged. A temporary failure-injection trigger/function exists only during the isolated suite and is removed after its rollback test; no persistent application schema change is introduced.

Fresh results: `cd server && npm run test:config` exits 2 (eleven required variables missing); `npm run test:integration` exits 1 at preflight (0 pass, 1 parent failure, no real SQL/Auth/migration/fixture case executed). `npm run test:integration:postgres` has 0 pass/1 skip and mocks Auth. `npm test` passes 23 unit/mock tests; 20 TypeScript syntax checks and `cd Client && npm run build` pass. Local env files/inherited test settings are absent; no credentials or remote data were used. This is prepared integration source, not successful Supabase/backend verification. Next configure the isolated project/ignored env and rerun the live command, which performs migrations itself; preserve disabled sync and all SQLite safeguards. STATUS records exact commands/errors and remaining native HTTPS/enrollment/Render acceptance work.

## Backend enrollment verification continuation — 2026-10-04

### Backend enrollment continuation contract — 2026-10-04

Backend npm installation now succeeds using the existing installed packages and lockfile. `POST /v1/enrollment` accepts only protocol version, the stable local device UUID and its secret. Supabase verifies the bearer subject; owner-provisioned active staff and approved device records determine one gym. The endpoint returns the authoritative gym/staff role/device permissions after rechecking under that gym's transaction lock. No request supplies a gym, staff identity, role or approval grant; no endpoint creates staff or approves a device. Repeated verification is read-only and safe after a dropped response. Database owner provisioning remains a prerequisite, and native login/enrollment/credential storage remains a separate unfinished step. Existing SQL migrations and SQLite relationships stay unchanged. Fresh network downloads still fail DNS, so installed-package and mock route results must not be reported as live PostgreSQL/Supabase sync.

Command: `cd server && npm install --ignore-scripts --no-audit --no-fund --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache` (exit 0, `up to date in 1s`). New code is backend-only, with environment-only Render configuration prepared and responses uncached. Production requires canonical HTTPS API/Supabase origins. Neither configuration nor a mock authoritative profile grants a native session or enables desktop sync.

Fresh results: `npm test` passes 18 backend tests (real Fastify injection, mock SQL/identity); `npm run test:integration` has 0 pass/1 skip because `TEST_DATABASE_URL` is absent. All 77 SQLite/Rust tests, frontend UI/build, Rust formatting/Clippy/normal desktop build pass. No live enrollment/upload/pull result. Render Blueprint provider validation and deployment are unperformed; a local YAML parser is unavailable. Full commands/results are recorded in STATUS.

Fresh `npm ping --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache` exits 1 with `getaddrinfo EAI_AGAIN registry.npmjs.org`. `curl --fail --silent --show-error --connect-timeout 5 --max-time 10 https://index.crates.io/config.json` exits 6 with `Could not resolve host: index.crates.io`. PostgreSQL tools/test configuration and cached native HTTPS/credential-store dependencies remain missing. Next configure a disposable test database/Supabase project, execute real migration/auth/enrollment/API checks, implement native HTTPS/credential/session/transport and run the requested desktop test-data scenarios. Keep the existing transactional storage, retries, stable identity/binding and restore contracts intact; no public deployment or live success claim without actual acceptance.

## Preceding blocked dependency probes — 2026-10-04

No relationship, schema or application changes in this continuation. Fresh probes from `server/`:

```sh
npm view fastify@5.12.5 version --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
npm view pg@8.23.0 version --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

Both exited 1 with `npm error code EAI_AGAIN`, `syscall getaddrinfo`, `errno EAI_AGAIN`; requests to the respective npm registry package endpoints failed with `getaddrinfo EAI_AGAIN registry.npmjs.org`. Backend installation stopped at the user's access gate. No fresh test-suite or live end-to-end result was produced; earlier local/mock results remain historical evidence only.

Preserved contract: stable generated SQLite device UUID; no invented gym for an unbound database; immutable enrolled server/gym/device scope set only through a protected native seam after verified enrollment. Server authorization uses verified staff joined to approved device records to derive gym access, never the supplied gym ID as a grant. Frozen member deliveries retain durable retry state, pending pushes block pull, and pulled changes/history/cursor commit together. Restore still blocks sync and cannot restore a session. Member IDs and linked attendance, membership, financial, card and audit history remain unchanged.

Authenticated enrollment, native HTTPS transport/credential storage, Render HTTPS configuration and live test-gym acceptance are still required. Resume dependency installation and disposable PostgreSQL migration/integration before these steps. The real path must verify enrollment, offline create/restart/reconnect with one upload, server-change pull, retries/duplicates, denied auth and conflicting edits, then run backend/SQLite/frontend/Rust checks with mock and live results explicitly separated. Keep live sync disabled until HTTPS/enrollment succeed; no public deployment or real member data, and no live-sync claim without passing real end-to-end tests.

## Member-sync inspection — recorded before any sync schema changes

### Resumed implementation contract — 2026-10-03

### Worker/binding continuation contract — 2026-10-04

Before adding worker behavior: persist a canonical HTTPS server origin, gym UUID and the existing local device UUID in metadata, containing no tokens/device secrets. Only verified native enrollment may call the internal binding method; no IPC exposes it. Reject a different scope, restored databases and any existing unscoped remote state. Binding and retry status participate in backup confirmation fingerprints and backup copies; restore still blocks sync. No SQL schema changes are required for metadata keys.

An internal worker uses a transport trait until an audited HTTPS implementation and enrollment exist. Check the unexpired private native session, active local Administrator/Reception role and transport's verified subject; archive transmission additionally checks its original enrolled actor before I/O. Freeze/push pending operations before pulling; stop on transient/auth failures so a dropped push response cannot become a self-conflict during pull. Preserve acknowledgements already committed when a later request fails. Retry deadlines/failure counts persist with capped exponential backoff; bounded runs yield without claiming complete sync. Pull pages/cursors remain atomic. Clear runtime sessions on server authorization denial, and recheck identity after I/O before storing replies. Keep success timestamps unchanged until an entire bounded run reaches the end of pull pagination. Production has no transport/session source yet and remains unavailable; transport doubles test orchestration only, never live network acceptance.

The parent workspace is now writable. Member protocol v1 uses explicit `operationId`, `deviceId`, `memberId`, `action` (create/update/archive), independent `expectedRevision` (0 for create), and a normalized member projection. The immutable legacy outbox remains the source; a separate durable delivery table freezes each transmitted request before I/O. Only the oldest pending operation per member may be prepared. A matching authenticated server receipt marks that one operation acknowledged; original outbox rows remain as history and nonmember/delete operations stay pending. Local optimistic versions and server revisions are never interchangeable. Historical member deletes block transmission for that member pending reconciliation; this milestone has no remote hard-delete endpoint.

PostgreSQL keeps gym/staff/device registration, members, operation receipts and ordered changes in a private schema. API verifies bearer tokens with Supabase Auth and checks active staff role/device registration inside every transaction. Each gym row is locked before allocating a change sequence, so a late commit cannot fall behind a pull cursor. Same operation ID with different content/device/actor is rejected; successful retries return the original receipt. NFC is normalized ASCII with per-gym uniqueness including archived members. Archive actor comes from authenticated staff; archive is Administrator-only. Native archive receipts must match the original local actor's enrolled Supabase subject, and local archive timestamp/FK history is preserved alongside the confirmed server projection. Pulled actors map to existing subject/FK records when present, otherwise to inactive identity-only references. No API enrolls a device or grants a role; the owner provisions these records. Deployments can register one writer until topology is settled.

SQLite v5 adds frozen deliveries/acknowledgements, confirmed remote heads, an ordered cursor and retained conflicts. Pull is transactional with cursor advancement, defers overwrites when local operations exist, and records card/archive collisions. Archive actor references are imported as inactive identity records without granting roles or sessions. NFC assignment rows remain local history; remote current-card changes append/revoke local assignments using existing triggers. Remote event payloads remain in heads/conflicts/audit; financial/membership/attendance/history rows are never replaced. Backups include all sync state; a restored database remains blocked for reconciliation. No UI IPC accepts arbitrary acknowledgements or trusted remote pages.

Dependency downloads currently fail with DNS EAI_AGAIN; PostgreSQL and a Rust HTTPS library are unavailable locally. Tests of protocol/storage do not establish live authentication, HTTPS, PostgreSQL concurrency or provider deployment. Native network/session integration must remain disabled until those dependencies and verified enrollment are available.

Requested backend location is the existing empty `ArmStrong/server/` alongside `Client/`. Existing architecture proposal: Fastify + Supabase PostgreSQL/Auth on Render; there is no implemented deployment/auth service. This section is inspection/design boundaries only, not completed synchronization.

Current member projection: `members.id/name/phone/email/nfc_id/joined_on/version/archived_at/archived_by_user_id`. Native commands validate fields, normalize unique current card IDs and reject stale edits. Archive actor is a FK to users; remote authenticated actor enrollment/import must precede applying archive metadata. Existing periods, attendance, invoices, payments and NFC assignments retain their original member IDs and history. Member-only sync does not imply that these other modules or their histories are transferred.

Current audit/outbox is immutable and transactionally written for member create/update/archive. A future versioned sync schema needs durable operation state/attempt/error/server acknowledgements, a separate confirmed remote member revision, ordered pull cursor and retained conflicts. Local optimistic versions cannot be assumed equal to server revisions. Preserve legacy pending operations/payloads and nonmember queues; never delete/ack them on connection alone. A confirmed member archive must not reactivate a member or purge linked records. Unique card collisions and pending local edits must produce explicit conflicts, not silent overwrite.

The original inspection stopped before implementation because server was read-only in the preceding Client-only workspace. That restriction is now resolved and the resumed contract above supersedes this historical inspection boundary. Browser simulated sync remains separate. Real network/provider/native enrollment acceptance remains unperformed; see current STATUS/PLAN and server README.

## Member archive/deletion and expense void contract — recorded before schema changes

Audit: current v3 has immutable attendance/membership/invoice/payment/NFC history, mutable member masters, positive append-only expenses with a reserved unique self-reversal FK, and audit/outbox transactions. Members/Expenses have no removal controls. Every existing write identifies an unauthenticated local test operator; reserved users/roles are empty, and there is **no authenticated native staff session**. The browser demo administrator is not an authorization source.

| Change | SQLite relationship / constraints | UI and historical behavior |
| --- | --- | --- |
| Member archive | Add nullable `members.archived_at` and `archived_by_user_id FK users`; both must be set together, with index on archive state/name. Increment existing expected version. All existing members migrate active. | Visible Archive/Deactivate + explicit confirmation. Active lists and normal member pickers exclude archived members; archived list/history remains accessible. No attendance/period/invoice/payment/card history is removed. |
| Member deletion | Native IMMEDIATE transaction verifies expected version and no FK-linked membership, attendance, invoice, payment or NFC assignment (even revoked), then deletes only the member. FKs and a deletion guard reject linked rows. | Visible permanent Delete only for members without business links, with irreversible confirmation. Creation/audit/outbox/request history uses polymorphic IDs and is retained; it does not prevent deletion of an otherwise unlinked member. Append a deletion tombstone with full before snapshot, actor/device, audit and durable outbox. No cascade/history purge. |
| Archive effects | Current NFC association remains reserved/history intact; native scans/manual attendance, member edits and new membership periods/renewals reject archived members. Active-plan counts exclude them. | Archive does not cancel dates, invoices or debts, refund payments or free the card automatically. Financial account/history and debt settlement remain available. No reactivation policy/workflow is invented in this milestone. |
| Expense void | New STRICT `expense_voids`: `expense_id PK/FK expenses` (one void), required reason, UTC created_at, actor snapshot, optional `actor_user_id FK users`, optional UNIQUE `legacy_reversal_id FK expenses`. New commands require an authenticated actor ID; null actor only for honestly labeled migrated reversals. Append-only. | Visible Void/Reverse + original amount/details, required reason and confirmation. Never delete/update the original expense. Duplicate void and voiding an old reversal are rejected. This is accounting removal from effective expenses, not execution of a bank refund. |
| Expense totals/history | View `expense_history` derives Recorded/Voided/Reversal, saved reason/void actor/time and `effective_amount_minor`. Voided originals and legacy reversing rows have effective amount 0; untouched originals retain their full amount. | UI/history shows original positive amount, status, actor/reason/time. Reports sum effective amounts and CSV exposes both original and effective LKR amounts/status, preventing hidden history or double subtraction. |
| Legacy expense reversals | Validate that existing original/reversing rows match amount/title/category/method and are not reversals of reversals. Preserve both original rows/IDs/amounts and append labeled legacy void metadata. Conflicts stop migration intact. | No invented historical reason/user; capture reversing row's existing actor and instant with explicit reason-unavailable label. |
| Authorization/audit | Native authenticated-session context is in memory, never supplied by a form/IPC actor field or restored from a backup. Require unexpired session, active enrolled SQLite user and Administrator role, checked inside every removal transaction **before** replay. Audit FK actor_user_id plus saved identity, device and before/after; durable UUID command replay; permission changes/expiry deny later actions. | App currently has no login/enrollment source: these protected actions must stay locked for the unauthenticated local operator. Visible controls explain this rather than granting demo/OS identity administrator privileges. Verified authentication integration is a remaining dependency, not a completed auth feature. Test-only trusted sessions exercise the transaction/authorization branches; no session-grant command is exposed to the webview. |
| Backup/recovery | Explicit v4 migration, canonical v1–v4 validation, copy new void table after users/expenses; preserve archive metadata, pending operations and recovery checks. | Restoring cannot restore authenticated sessions. No browser import, fake success/sync or new settings. |

Implementation must preserve the black/amber controls and browser demo behavior. Only the native committed result can cause a success notification. These operations are separate from invoice/payment reversals and never clear audit/outbox. User identity/enrollment integration remains an explicit blocker for live privileged use; no permissive fallback is authorized.

## Version-4 removal implementation check

The removal contract above is implemented in `004_removal.sql` and `removal.rs`, registered as `archive_member`, `delete_member` and `void_expense`, and exposed through typed desktop-only API/provider methods. UI controls open `MemberRemovalDialog` / `ExpenseVoidDialog` with required confirmation, stable request UUID, captured member version and required expense reason. Provider success follows commit. **Live removal remains locked because there is no implemented native login/enrollment flow**; no session-grant command or hard-coded administrator was added. The native-only session seam is empty after open/restart/restore. Authorized branches are tested through `#[cfg(test)]` fixture setup only.

| Current field/control | Actual SQLite or runtime mapping |
| --- | --- |
| Member active/archived lists and status | `members.archived_at IS NULL` derives active; archived name/phone/card/period history remains in the full snapshot. Default adapter, dashboard/search, manual attendance and normal member selectors use active members; explicit archived view uses retained records. |
| Archive / Deactivate and confirmation | `members.archived_at` UTC, `archived_by_user_id FK users`, expected `version` increment. Pair/update triggers protect archive metadata. Confirmation is transient UI, not a setting. |
| Delete permanently visibility / confirmation | `canDelete` derived from absence of period/attendance/invoice/payment/NFC links (including revoked cards). Native transaction and FK/delete trigger recheck; UI visibility is not authorization. Delete only the unlinked master, retain audit/outbox/request receipts. |
| Membership/card behavior after archive | Plan active counts exclude archived users. Native scan/manual lookup and membership insert guards reject new attendance/periods; member edits are rejected. Current NFC reservation stays intact; no invented reactivation or automatic card-release policy. Existing account/debt/payment/receipt history remains available, with archived financial targets labeled explicitly. |
| Expense Void / Reverse / required reason | `expense_voids.expense_id PK/FK`, `reason`, `created_at`, `actor`, `actor_user_id FK users`. Native checks original Recorded status and enforces one unique void. Original `expenses.*` row is untouched. |
| Expense history status / reason / actor / timestamp | `expense_history.status/void_reason/voided_by/voided_by_user_id/voided_at` joins immutable void metadata. Original amount/date/method/actor remain visible. Legacy reversal rows stay visible as Reversal with zero effective amount and metadata identifying the original's void. |
| Effective Expenses / Reports / Expense CSV | `expense_history.effective_amount_minor`: amount for Recorded originals, zero for Voided originals and legacy reversing rows. CSV retains original and effective amounts, status, reason, void time/user and original recorded user; report summaries never sum discarded/voided amounts. |
| Protected action actor/permission | In-memory native session + current active `users` / `user_roles` / Administrator `roles` inside transaction. Expiry/deactivation/role revocation denies removal and even replay; caller-supplied actor fields are rejected. `removalAuthorization` snapshot is derived runtime information, not persisted authorization. |
| Audit and durable operation | Append `audit.actor_user_id/actor/device_id/action/entity/entity_id/before_json/after_json/created_at`; append outbox archive/delete/void payload (schema 4), and `local_operations` receipt bound to actor and request UUID. No audit/outbox deletion or false sync. |
| Legacy void metadata | `legacy_reversal_id UNIQUE FK expenses`; preserves original/reversing records. Unknown old reason is labeled; old actor/time are copied honestly, not asserted as an authenticated identity. Null actor-user FK is permitted only for these legacy metadata rows. |
| Migration/backups | V1–V4 canonical schema recognized. Existing member/version/history, money and pending payloads stay unchanged. Users are restored before archived-member actor FKs; voids after expenses/users. Runtime session is excluded from storage confirmation fingerprint and cleared after successful restore. |

`expense_voids` is STRICT, indexed by actor/time and protected against UPDATE/DELETE. New raw self-reversal expense inserts are rejected in favor of the reasoned void workflow. No editable setting or browser demo removal behavior was invented.

51 SQLite tests (nine removal cases plus prior 42), 12 adapter/API tests, all-route/settings/browser/receipt/removal-confirmation render checks, frontend build, Rust fmt/Clippy and Linux native compilation pass. Native GTK initialization prevents actual window/form/IPC acceptance. A smoke harness now verifies confirmation controls, archived-view selection and direct unauthorized IPC denial when run in a usable GUI; no privileged test identity is injected into that binary. Authentication integration and successful native acceptance remain open; STATUS lists the exact next task.

## Finance milestone contract — recorded before schema changes

Inspected v2 migrations, Rust payment/idempotency/backup code, payment and membership date forms, browser provider, and tests. Existing received amounts and payment IDs are authoritative; existing reserved invoices/allocations must also survive migration. No code defines automatic renewal dates, month-end rollover, expiry-gap/grace, debt admission, or payment-triggered activation. Existing periods have explicit inclusive dates, no overlaps, and snapshot the chosen plan price/name.

| Entity/change | Keys, constraints and relationships | UI mapping / history |
| --- | --- | --- |
| Existing `invoices` | Retain original id/member/optional period/sale/amount/issue date/instant unchanged. Native new invoices require a member; a linked period must belong to that member and use its saved price. | New invoice: member, optional existing membership, description, amount. Generic charges use staff-entered amount; linked membership charges use the historical period price. |
| New `invoice_details` | `invoice_id PK/FK invoices`, `number UNIQUE`, description, member name snapshot, actor, legacy flag. Immutable one-to-one document metadata. | Stable invoice identifier, saved description/member label; migration labels old metadata honestly instead of inventing old invoice descriptions. |
| `payment_allocations` | Preserve id/payment/invoice/amount rows; replace unique payment/invoice pair with an index to permit repeated partial allocations. Positive amounts; aggregate payment/invoice limits and same-member ownership; reject reversal or reversed-payment funding. | Receive payment optionally applies up to outstanding amount to one selected invoice in the same transaction. Allocate existing credit explicitly using payment, invoice and amount. No automatic allocation across invoices. |
| New `payment_reversal_details` | `payment_id PK/FK payments` referencing the reversing row; nonblank reason. Existing `payments.reverses_id UNIQUE FK payments` protects one full reversal per original. New reversals must copy the original member, amount and method; cannot reverse a reversal. | Reverse payment: select saved original and enter reason; confirm full amount. Original remains immutable. This is a local accounting reversal, not bank/card execution or membership cancellation. |
| New `allocation_reversals` | `id PK`, `allocation_id UNIQUE FK payment_allocations`, `payment_reversal_id FK payments`, timestamp. Must match that allocation's original payment. Immutable full release of an allocation. | Full payment reversal appends all releases, reversing payment, reason, receipt, audit and outbox atomically. Originals/allocations are never deleted or edited. |
| New `payment_receipts` | `payment_id PK/FK payments`, `number UNIQUE`, immutable JSON snapshot, issue instant and legacy flag. One receipt per received/reversing payment, issued within the financial transaction. | Preview/reprint saved gym profile, member, amount, method, dates, actor, invoice allocations at issue, and reversal reference/reason. Later allocations/reversed status are shown separately; original receipt contents and number remain stable. |
| Balances | Derived from original invoices and effective allocations (exclude allocation releases and reversed payments). Invoice outstanding = amount − effective paid; status Unpaid/Partial/Paid. | Member debt, available unallocated credit, and net balance displayed separately. Excess payment is credit, never a negative invoice balance or discarded amount. |
| Renewal | Create explicit nonoverlapping successor period and linked invoice/details in one transaction; compare captured latest period ID and active plan version/price before saving. | Member, plan, explicit start/end dates; plan price read-only. No automatic dates or payment-triggered renewals. Existing date-only entry remains available and is not retroactively billed. |

New document numbers use `AF-I-{device_id}-{invoice_id}` / `AF-R-{device_id}-{payment_id}`: UUID-backed stable identifiers without inventing a gym receipt sequence policy. A restored old backup cannot reuse a numeric counter; new UUIDs produce distinct numbers. Number-format/thermal-printer acceptance remains an owner release decision. Migration generates receipt artifacts for old payments from saved amounts/names/methods/instants and the profile at receipt issue; these are labeled legacy receipts, not claims of historical printing. It never fabricates matching invoices or allocations for unallocated received amounts. Old outbox payloads and durable command receipts stay unchanged.

Cash summaries/Income CSV must treat a reversing payment as a negative cash entry on its own Colombo business date; the original remains positive on its original date. Allocation adds no cash income. Payment reversal reopens invoices but never cancels or shifts membership dates. Historical generic/membership invoice correction, partial refunds, sales returns were outside this payment-finance milestone. Expense voids are implemented separately in v4 above.

All native financial commands use IMMEDIATE SQLite transactions, native validation, durable operation UUID replay, audit and immutable outbox. Failed transaction yields no success message. Receipt read/reprint does not create a payment, allocation, receipt number, audit or outbox operation. Backups/restore must recognize v1/v2/v3 and copy the new finance tables with FK order intact.

Receipt delivery will be a real saved-data React preview with a print-dialog action and print CSS. Rendering and data/reprint tests are separate from actual Tauri print-dialog/Windows printer acceptance; do not mark hardware printing verified without native execution.

Outstanding policy decision: define automatic start/end/month-end/expiry-gap rules, unpaid renewal admission and whether receipt numbering must follow a prescribed gym sequence. Until then use explicit dates and no automatic renewals. No tax/discount/gateway/printer configuration settings are invented.

## Version-3 finance implementation check

Implemented the contract above in `003_finance.sql`, `finance.rs`, the six Tauri commands, typed desktop API/provider, `DesktopPaymentsPage.tsx` and `ReceiptView.tsx`. Browser payment behavior remains in its separate provider/page branch. Native payments never load demo/localStorage. The v1/v2 sections below are historical baselines; this section supersedes their reserved-invoice/reversal limitations.

`Store::open` applies 001 → 002 → 003 in one transaction before returning a store. V3 rebuilds only the allocation table to remove the pair uniqueness restriction and copies every original allocation ID/payment/invoice/amount. Received payments, original invoices, history, old outbox payloads and durable command results are unchanged. Matching legacy reversals receive explicitly labeled reasons/releases/receipt artifacts; inconsistent full reversals abort migration and retain v2 for recovery/decision. No retroactive debt is invented. Backup validation recognizes the exact v1/v2/v3 schemas, including v3 views; restore copies document/reversal tables in FK order.

Important v3 indexes: `allocation_payment_invoice(payment_id,invoice_id)`, `allocation_invoice(invoice_id)`, `allocation_release_payment(payment_reversal_id)`, plus unique invoice/receipt numbers and unique allocation releases. Existing invoice-member and unique period/sale indexes, payment member/date, business date and unique original reversal references remain intact. FKs restrict orphaning. New document/release tables and allocation rows are STRICT and append-only.

| Actual desktop control / saved field | SQLite column / native behavior |
| --- | --- |
| New invoice: Member | `invoices.member_id`; native requires a saved member, snapshots `invoice_details.member_name`. Nullable member is retained only for preexisting generic invoices. |
| New invoice: Membership period (optional) | `invoices.membership_period_id` (FK/UNIQUE). Native verifies same member and exact saved `membership_periods.price_minor`; already billed periods are not offered. |
| New invoice: Description / Amount | `invoice_details.description` / `invoices.amount_minor`. Linked-period amount is read-only; generic amount is explicit, positive integer LKR minor units. |
| Invoice Number / Date / Amount / Paid / Outstanding / Status | Saved `invoice_details.number`, `invoices.issued_on/created_at/amount_minor`; paid/outstanding/status derived by `invoice_balances`. |
| Receive: Member / Amount / Payment method | `payments.member_id/member_name/amount_minor/method`; native supplies Colombo `business_on`, UTC `created_at`, actor and UUID. |
| Receive: Invoice (optional) | `payment_allocations.payment_id/invoice_id/amount_minor` in the payment transaction; amount = min(received, selected invoice outstanding). With no selected invoice, all received money remains credit. |
| Allocate: Received payment / Invoice / Amount | `payment_allocations.payment_id/invoice_id/amount_minor`. Repeated partial entries are separate immutable rows; native triggers enforce member ownership, credit and invoice caps. |
| Renew: Member / Plan / Start date / Last valid day | `membership_periods.member_id/plan_id/starts_on/ends_on`, saved plan name/price, and linked invoice/details. Explicit inclusive successor dates only; no duration-based date calculation. |
| Renew: displayed price / expected plan version / latest membership | Saved plan price becomes period/invoice price. Captured `plans.version` and latest period ID are validated inside the transaction and retained in durable operation request/audit; background refresh cannot alter the captured agreement. |
| Reverse: Original payment / Reversal reason | New `payments.reverses_id` (UNIQUE) plus full original member/name/amount/method; `payment_reversal_details.reason`. No original update or deletion. |
| Reversal allocation effects | `allocation_reversals.id/allocation_id/payment_reversal_id/created_at`. All original allocation releases, receipt, reason, audit/outbox and command receipt commit together. Duplicate reversal/partial reversal/reversal-of-reversal is rejected. |
| Member Invoice outstanding / Unallocated credit / Net balance | Sum of `invoice_balances.outstanding_minor`, sum of eligible `payment_balances.unallocated_minor`, difference. Positive net is due; negative net is credit. Neither aggregate is an editable saved field. |
| Payment Cash effect / Allocated / Credit / Status | `payment_balances` view. Original amount stays positive; reversing row's cash effect is negative on its own day. Reversed payments and their releases provide no spendable credit. |
| Receipt Number / Preview / Reprint | `payment_receipts.number/snapshot_json/issued_at/legacy`. JSON snapshots saved gym name/location/phone/email, received fields, issue-time allocations/outstanding/credit and optional reversal number/reason. Original reprint reads this document, never a regenerated live receipt. |
| Original receipt reversed warning / reference | Separate live lookup of reversing payment/receipt; original snapshot is untouched. This warning does not assert a refund or membership cancellation. |
| Print / system preview | `window.print()` plus receipt-only print CSS and a body portal. No settings columns, printing success toast or printer-readiness state. Native dialog/hardware acceptance remains unverified. |
| Request UUID / retries / audit / server pending | `local_operations.request_id/request_json/result_json`, append-only `audit`, version-3 `outbox` payloads in the same transaction. Receipt reading/preview/reprinting writes none of these. |

`effective_payment_allocations` excludes allocation releases, reversing rows and originals that have been reversed. `invoice_balances` and `payment_balances` derive financial state from history; no cached editable outstanding field exists. Income dashboard/report/CSV uses signed payment cash effects plus sales; allocation and renewal add no cash. Negative numeric CSV values retain the existing spreadsheet-safety quote prefix.

Receipt rendering is tested from saved data, including historical gym/member values, escaping, legacy/reversal labels and stable reprint contents. Normal Linux desktop compilation passes; actual native window launch fails at GTK initialization in this environment, before receipt preview or the print dialog can be exercised. Printing is **not verified as a completed native workflow**. The 80 mm receipt content layout uses the system dialog's selected paper/printer; no printer configuration control was invented.

Historical records with no editor remain: invoice correction/cancellation, membership correction/freeze, partial refunds and sales returns. Expense voids are implemented in v4 above. Authenticated roles/backend/sync remain outside this finance milestone. Exact source/verification and unresolved automatic-renewal/receipt-format decisions are recorded in STATUS.

## Evidence and baseline at audit

- Entry: `src/main.tsx` selects `DesktopGymProvider` or dynamically loads `BrowserGymProvider`. Both render `App` and `pages/navigation.tsx` (nine routes), existing styles and layout.
- SQLite at audit: `src-tauri/migrations/001_foundation.sql`, `src-tauri/src/lib.rs`; `Store::open` supports version 1 only. Native commands in `src-tauri/src/main.rs`: `foundation_snapshot`, `save_member`, `save_plan`, `add_membership_period`.
- Native adapter at audit: `src/desktop/api.ts`, `adapter.ts`, `DesktopGymProvider.tsx`. Only members/plans/periods are populated. Attendance/payment/sale/expense/stock/restore methods only notify that they are unavailable.
- Forms: `MembersPage`, `MembershipsPage`, `MembershipDatesModal`, `AttendancePage`, `PaymentsPage`, `InventoryPage`, `ExpensesPage`. Desktop native operational actions are disabled in the audited baseline. Inventory has a sale form and +/-1 controls but **no product creation/edit form**.
- Settings: `SettingsPage.tsx` and `DesktopSettingsPanel.tsx`. Seven tabs. Browser profile has four inputs; its save only toasts. Desktop has two disabled profile inputs. Other tabs contain static descriptions or actions, not editable configuration fields.
- Reports/dashboard read arrays; native financial/attendance data are unavailable. Browser report exports are only toasts. Browser settings user rows and login credentials are demonstrations, not persisted accounts.
- Tests: nine SQLite core tests, four native-adapter tests, SSR route/settings/demo isolation checks and an opt-in real Tauri UI/restart smoke harness. SSR is not interactive IPC acceptance.

## Original version-1 tables (exact existing constraints)

| Table | Primary key / columns | Relationships, uniqueness and indexes |
| --- | --- | --- |
| `metadata` | `key TEXT PK`, `value` | `device_id` generated on first creation. No registered-device table/API. |
| `plans` | `id TEXT PK`; `name`, `duration_months`, `price_minor`, `active`, `version` | `name UNIQUE COLLATE NOCASE`; name 1–80 trimmed chars; months 1–60; price 0–100,000,000,000 minor units; active boolean; version >0. |
| `members` | `id TEXT PK`; `name`, `phone`, `email`, nullable `nfc_id`, `joined_on`, `version` | Nonblank name/phone checks. Nonempty `nfc_id UNIQUE COLLATE NOCASE`, <=128 chars; blank becomes NULL. Rust normalizes ASCII UID and validates email/card. No membership columns. |
| `membership_periods` | `id TEXT PK`; `member_id`, `plan_id`, `plan_name`, `price_minor`, `starts_on`, `ends_on`, `created_at` | FKs to members/plans, default RESTRICT/NO ACTION; index `membership_member_dates(member_id,starts_on,ends_on)`; insert trigger rejects overlaps for a member. Historical plan name/price snapshot. Inclusive dates validated in Rust. No correction UI. |
| `audit` | `id TEXT PK`; `actor`, `device_id`, `action`, `entity_id`, `before_json`, `after_json`, `created_at` | Legacy actor is explicitly unauthenticated. Polymorphic entity reference; not a misleading FK to just one table. No viewer/index/immutability triggers in v1. |
| `outbox` | `id TEXT PK`; `device_id`, `entity`, `entity_id`, `action`, `expected_version`, `payload_json`, `schema_version`, `created_at`, `attempts` | Polymorphic entity reference. Audit/outbox/business records share an IMMEDIATE write transaction. All rows pending; no server acknowledgements or sync implementation. |

WAL, `synchronous=FULL`, foreign keys and a busy timeout are set in Rust. Snapshot reads use one transaction. New stores start without business/demo records. Database corruption/newer versions are rejected; migrations must preserve existing rows and device IDs.

## Version-2 local schema (implemented after audit)

UUID text keys are used for command-created records; migrated card links retain identifiable migration IDs. Money is integer LKR minor units, maximum 100,000,000,000 per amount. Instants are UTC; event business dates are generated in Rust in Asia/Colombo. Persisted name/price snapshots keep history readable after master data edits.

| Entity/table | Keys and columns | Constraints / important indexes / relationships |
| --- | --- | --- |
| Plans, members, subscriptions | Existing tables above | Retain IDs, prices, versions, dates, audit and pending outbox rows. Add period-by-plan/date and audit/entity/time/outbox/time indexes. A subscription is the existing explicit `membership_periods` record, not a newly invented automatic renewal. |
| NFC assignment history: `nfc_cards` | `id PK`, `member_id FK`, `uid`, `assigned_at`, nullable `revoked_at` | Unique active UID (NOCASE) and unique active member using partial indexes; member/date index. `members.nfc_id` remains the current-card compatibility projection. Transactional DB triggers open/close historical links when it changes, including migration of current assignments. No deletion of past links. |
| `attendance` | `id PK`, `member_id FK`, optional `card_id FK nfc_cards`, `member_name`, `card_uid`, `kind`, `source`, `business_on`, `occurred_at`, optional `voids_id FK attendance UNIQUE` | NFC/manual source; check-in/out kind; index `(member_id,business_on,occurred_at)` and business date. Native toggle uses last event on the same business day in the write transaction. UID is a historical snapshot. Recording is logging, not an approved admission decision. A two-second duplicate-scan window protects HID repeats. A DB trigger validates card/member/UID ownership. |
| `payments` | `id PK`, `member_id FK`, `member_name`, `amount_minor`, `method`, `business_on`, `created_at`, `actor`, optional `reverses_id FK payments UNIQUE` | Positive integer amount, Cash/Card/Transfer. Member/date and business-date indexes. Standalone received amounts: the current form has **no invoice, amount-due or subscription selection**. No automatic renewal or fabricated paid invoice/printed receipt. |
| `invoices` (reserved, no current entry UI) | `id PK`, optional `member_id FK`, optional `membership_period_id FK UNIQUE`, optional `sale_id FK UNIQUE`, `amount_minor`, `issued_on`, `created_at` | Period/sale relationships mutually exclusive; membership invoices require a member. Invoice balances derive from allocations. No automatic invoices until billing/receipt policy is decided. |
| `payment_allocations` (reserved) | `id PK`, `payment_id FK`, `invoice_id FK`, `amount_minor`; unique `(payment_id,invoice_id)` | Positive integer amount; invoice/payment indexes. Amount/member ownership limits enforced by triggers; no allocation command/UI in this milestone. Partial payments/debt remain a separate decision. |
| `products` | `id PK`, `name`, `sku`, `cost_minor`, `price_minor`, `reorder_level`, `version` | SKU unique NOCASE, nonblank name/SKU, nonnegative integer cost/price/reorder level. Stock is derived from movements, never replaced from a UI numeric projection. No seeded products. The native product editor exposes the current product fields; opening stock writes a movement so a fresh empty inventory can be used. |
| `sales` | `id PK`, `total_minor`, `method`, `business_on`, `created_at`, `actor`, optional `reverses_id FK sales UNIQUE` | Cash/Card/Transfer; positive integer total; date index. Retail sales currently have no member field. |
| `sale_items` | `id PK`, `sale_id FK`, `product_id FK`, `name`, `sku`, `quantity`, `price_minor`, `cost_minor` | Unique `(sale_id,product_id)`; positive integer quantity; snapshotted unit price/cost/name/SKU. Sale total is calculated natively, not trusted from the frontend. |
| `stock_movements` | `id PK`, `product_id FK`, optional `sale_item_id FK UNIQUE`, `delta`, `kind`, `reason`, `created_at`, `actor` | Nonzero integer delta; Sale negative, Opening positive, Adjustment signed. Product/time index. Insert trigger prevents negative resulting stock. Sale/item/movement/audit/outbox commit together. +/-1 UI has no reason input; commands record an explicit manual +/-1 reason. |
| `expenses` | `id PK`, `title`, `category`, `amount_minor`, `method`, `business_on`, `created_at`, `actor`, optional `reverses_id FK expenses UNIQUE` | Operations/Utilities/Maintenance/Salary/Other; Cash/Card/Bank; positive integer amount; business-date/category indexes. Actor supplied by native session/test context, never hard-coded frontend “Prinzz”. |
| `gym_settings` | singleton `id=1 PK`, `name`, `location`, `phone`, `email`, `version` | Exactly the four current profile fields. Required name/location; validated optional phone/email; optimistic version. Defaults match the current visible gym name/location; these are configuration defaults, not business demo records. |
| `users` (reserved) | `id PK`, `subject UNIQUE`, `email UNIQUE NOCASE`, `display_name`, `active`, `version` | No plaintext passwords, hard-coded users or fake authentication. Server subject/enrollment and offline credential policy remain unresolved. |
| `roles`, `user_roles` (reserved) | role `id PK`, `name UNIQUE`; join `(user_id FK,role_id FK) PK` | Current UI describes Administrator/Reception but has no assignment/editor. Do not seed demo accounts or claim access restrictions. Future actor-user links can be nullable FKs; legacy actors remain intact. |
| `audit` | Existing columns + `entity`, optional `actor_user_id FK users` | Backfill entity from existing action; append-only update/delete protection; entity/time index. Local test actor remains unauthenticated until identity is actually implemented. |
| `outbox` | Existing columns retained | Keep old schema-version-1 payloads unchanged; new records declare their operation schema version. Immutable payload/identity; attempts may change. No deletion or “synced” status without an authenticated server acknowledgement. Server protocol remains unimplemented. |
| `local_operations` | `request_id PK`, `command`, `request_json`, `result_json`, `created_at` | Durable idempotent receipt for new append operations. Same request and payload returns its previous result; reused ID with another payload is rejected. Receipt, business rows, audit and outbox share one transaction. This is local command idempotency, not server sync acknowledgement. |
| Backup/restore | Native SQLite snapshot + versioned checksum envelope outside the live DB | Includes all business/settings/audit/outbox tables consistently. Validate checksum, recognized schema, full integrity and FKs; preview then explicitly confirm replacement. Recheck confirmation and capture recovery under an IMMEDIATE write lock; replace all tables and reinstate append-only triggers in one transaction. Mark restored storage as requiring server reconciliation. Do not use browser GymData JSON to replace SQLite. |
| Reports and UI state | Derived queries, no report/search/modal table | Totals and lists read SQLite snapshots. Search text, open forms, selected tab, scan input, notification messages and connectivity are transient UI state. Exports must write real files, not success-only toasts. |

```mermaid
erDiagram
  members ||--o{ membership_periods : has
  plans ||--o{ membership_periods : snapshots
  members ||--o{ nfc_cards : assignments
  members ||--o{ attendance : logs
  nfc_cards o|--o{ attendance : scanned_with
  members ||--o{ payments : receives
  members o|--o{ invoices : billed
  membership_periods o|--o| invoices : optional_billing
  sales o|--o| invoices : optional_billing
  payments ||--o{ payment_allocations : allocates
  invoices ||--o{ payment_allocations : settled_by\n  invoices ||--o| invoice_details : document\n  payments ||--o| payment_receipts : saved_receipt\n  payments ||--o| payments : full_reversal\n  payments ||--o| payment_reversal_details : reason\n  payment_allocations ||--o| allocation_reversals : released_by\n  payments ||--o{ allocation_reversals : reversal_effects
  sales ||--|{ sale_items : contains
  products ||--o{ sale_items : snapshots
  products ||--o{ stock_movements : ledger
  sale_items o|--o| stock_movements : deducts
  users ||--o{ user_roles : assigned
  roles ||--o{ user_roles : grants
  users o|--o{ audit : actor\n  users o|--o{ members : archived_by\n  expenses ||--o| expense_voids : void_metadata\n  expenses o|--o| expense_voids : legacy_reversal\n  users o|--o{ expense_voids : staff_actor
```

Audit/outbox polymorphic references cover members, periods, plans, settings, attendance, payments, products/stock, sales and expenses; a single FK cannot point to all these tables. They commit with each command. Device identity currently lives in metadata, not an invented enrolled-device record.

## History, edits and reversals

- Mutable masters: members, plans, products and gym profile use expected versions; stale edits fail, not last-write-wins. NFC relinking revokes the old link and appends the new one atomically.
- Historical periods keep plan name/price and explicit dates. Neither browser date fields nor payment receipt imply a renewal. Period correction/freeze/reversal policy has no UI and stays unresolved.
- Attendance, payments, expenses, sales/items, stock movements and audit are append-only. Their original rows cannot be destructively edited/deleted. Future correcting/reversing operations reference originals and must include compensating stock/payment effects and audit/outbox atomically; the v3 full payment reversal is described above; other correction workflows remain absent.
- Stock edits never change past movements or sale prices. Opening stock is a movement. Sale deduction cannot exceed stock even if two connections submit concurrently.
- Outbox identity/payload is immutable. A local write stays pending until real server acknowledgement. Backup restore is explicit, recoverable replacement and requires future sync reconciliation; it is not a financial correction or conflict-resolution algorithm.

## Version-2 implementation check (preceding milestone)

The original baseline columns below remain historical audit evidence; their target mappings now apply to v2. `Store::open` applies migrations 001 and 002 transactionally and rejects corrupt/newer/unrecognized data and noninteger legacy plan/period prices without replacing records. New native methods are in `operations.rs` and `recovery.rs`; all are bound in `main.rs` and typed in `src/desktop/api.ts`.

All nine route components use SQLite snapshots in desktop mode. The operational forms and the four Gym profile fields now persist. The native product form adds Name, SKU, Cost, Selling price, Reorder level and Opening stock (new products only); these map to `products.*` and an Opening `stock_movements` entry. No new Settings options were introduced.

Attendance/payment/product/sale/stock/expense writes include a durable UUID request receipt, audit and outbox in one transaction. Profile/master edits use expected versions. Existing member/plan/period creation commands retain their original contract; a lost IPC response on those commands requires refreshing before reopening/submitting again. No automatic retry or sync acknowledgement is claimed.

Native reports write six real CSV exports. The Audit CSV exposes before/after history, actor and entity; there is no full on-screen audit/filter editor. Backup exports are private files beside the database in `armstrong.backups/`; replacement requires a validated preview and a fsynced pre-restore recovery file. Restore never merges newer financial records automatically. A populated database rejects cross-device replacement; a fresh empty installation can explicitly adopt a backed-up device identity and is flagged for reconciliation.

Reserved `users/roles/user_roles/invoices/payment_allocations` are empty on fresh installation. No form exposes enrollment, role assignment, invoice allocation or reversals; schema constraints and references are not those workflows. NFC/printer readiness, updater checks and authenticated sync remain unavailable. Full runtime/Windows/hardware acceptance remains separate from storage and route tests.

## Every saved UI field and visible setting

Baseline state: **stored v1**, **missing**, **derived**, **demo-only**. Implementation-target columns are named explicitly; no invented settings are implied.

| Screen/control/field | Baseline | Database mapping / native behavior |
| --- | --- | --- |
| Member Full name, Phone, Email | stored v1 | `members.name`, `phone`, `email` |
| Member NFC card ID / Scan card | stored v1 / input action | `members.nfc_id`; normalized unique current link + `nfc_cards.uid/member_id` history. Scan captures input; it does not prove reader hardware readiness. |
| Member ID, joined date, initials | stored / derived | `members.id`, `joined_on`; initials computed from name. UUID generated by native storage. |
| Browser member Plan/Expiry edit | demo-only | Browser name/date fields stay localStorage; desktop uses explicit `membership_periods.plan_id/ends_on` and never silently writes these legacy projections. |
| Desktop Membership dates Plan, Start date, Last valid day | stored v1 | `membership_periods.plan_id`, `starts_on`, `ends_on`; immutable `plan_name/price_minor` from selected plan. |
| Member displayed plan, expiry, status; membership history | derived/stored | Current/relevant period, native status by Colombo date, all periods. Scheduled/no membership are not active membership. |
| Package name, Duration in months, Price, Status | stored v1 | `plans.name`, `duration_months`, `price_minor`, `active`; LKR conversion at typed bridge. |
| Active members | derived, disabled | Distinct members with inclusive current periods per plan; no saved editable count. |
| Attendance Scan or type NFC ID | missing | Resolve current normalized UID to member/card; `attendance.card_id/card_uid/source`; native generated instant/day/type. |
| Attendance Manual member check-in select | missing | `attendance.member_id`, source Manual; label/name snapshot, generated kind/day/time. No membership admission setting exists. |
| Attendance listed member, time, date, check-in/out, source, sync | missing | `attendance.member_name/member_id/occurred_at/business_on/kind/source`; local records remain pending. |
| Payment Member, Amount, Payment method | missing | `payments.member_id`, `amount_minor`, `method`; native snapshots name and supplies `business_on`, instant and actor. No invoice/renewal picker exists. |
| Payment displayed Invoice column, ID, Date, Paid/Partial | demo-only/missing | Existing “Invoice” actually shows a generated payment ID. Native UI must identify it as Payment; “Recorded” amount is not proof of invoice settlement. Future invoice/partial state derives from invoices/allocations. |
| Inventory displayed Product, SKU, Cost, Selling price, reorder limit | demo-only | `products.name/sku/cost_minor/price_minor/reorder_level`; product master fields currently have no entry form. Add a native editor using these fields to enable empty installations. |
| Inventory Stock; low-stock status | demo-only | `SUM(stock_movements.delta)`; compare with `products.reorder_level`, never silently clamp a negative adjustment to zero. |
| Inventory +/-1 buttons | missing | Native manual movement +/-1, reason recorded by command, durable local request ID, audit/outbox. |
| Sale Product, Quantity, Method | missing | `sale_items.product_id/quantity`, `sales.method`; native master snapshots/unit price/cost and calculated total. |
| Sale date/items/total (stored browser, no list UI) | demo-only | `sales.business_on/created_at/total_minor`, `sale_items`; report view/export can expose history. No current customer/discount/tax fields. |
| Expense Description, Category, Amount, Method | missing | `expenses.title/category/amount_minor/method`; exact current choices only. |
| Expense Date, Recorded by, ID, sync | missing | Native business date, `expenses.actor/id`; pending outbox. Ignore frontend actor/date assertions in desktop commands. |
| Settings Gym profile: Gym name, Location, Phone, Email | missing; browser toast only | Singleton `gym_settings.name/location/phone/email/version`. Desktop must expose all four existing fields, save in SQLite, and reload them after restart. |
| Settings Users & roles: Prinzz/Admin, Reception descriptions | static/demo | Future `users`, `roles`, `user_roles`. No username/role editing controls exist; no fake populated accounts. Actual records may be displayed read-only. |
| Browser login Username/Password; Show/Hide; Sign out | demo-only/transient | Browser-only demonstration. No password copied into SQLite, no production demo authentication. Real enrollment/unlock is unresolved. |
| Settings NFC reader: mode Keyboard/HID, Ready label, Test reader | static/action | No editable setting. Current input path is HID text, not native hardware detection. Do not persist a fake Ready state. Test action directs actual scan input and must report hardware unverified. |
| Settings Receipt printing: Windows default printer, 80 mm/Default | static | No printer or size selector/save control. Do not invent printer-name/paper-size settings. Printer integration is unresolved. |
| Settings Backup & restore: Export backup, Restore backup, hidden file picker | missing native, browser JSON demo | Native versioned validated SQLite backup/restore operations; paths/checksum/preview/confirmation are action results, not configuration fields. No visible retention/destination setting exists. |
| Settings Server synchronization: Connection, Pending operations, Sync now | native pending only / browser simulated | Connection is runtime status, pending is native outbox count; no saved endpoint/token input exists. No real sync/ack implementation until topology/hosting/auth agreed. |
| Settings Application updates: Installed version, Current, Check for updates | static/demo | Installed version comes from build metadata; no update URL/channel setting exists. Never claim Current/no-update without a server response. |
| Reports Attendance, Membership, Income, Inventory, Expense, Audit; summary totals | derived/missing | Read native snapshot/ledgers/audit. No date filters/save fields exist; native file exports must correspond to real records. |
| Dashboard member/attendance/expiry/income KPIs and chart/list links | derived/missing | Actual SQLite summaries and Colombo business dates. Static browser chart remains a demo; native bars/totals derive from records. |
| Global Search, page selection, tab selection, modal selection, scan buffer, toast | transient | No database columns needed; do not confuse ephemeral UI state with business persistence. |

## Stored fields with no current editor/UI

V1: metadata/device ID; expected versions; period creation instant; historical period price; full audit before/after JSON and device; outbox operation payload/action/schema/attempts. Current UI shows only counts/derived summaries for most of these.

V2: command idempotency receipts; card assignment/revocation timestamps; UTC event instants and actors; historical unit cost/SKU; reversal/void references; invoice/allocation balances and links; identity subjects/roles; restore-reconciliation metadata. These are integrity/history fields, not implied settings or completed workflows. No secret-bearing auth storage is introduced.

## Unresolved decisions and bounded implementation

1. Identity/enrollment, allowed roles, offline credentials and revocation are not agreed. Keep unauthenticated test status explicit; do not claim production authorization.
2. Deployment topology/hosting and authenticated server acknowledgements are not agreed. Local outbox persists; no network sync, clearing or simulated synced state in desktop.
3. Current payment UI can safely record received amounts without allocating them or renewing membership. Billing, debt/partial payments, receipt numbering and reversal workflows need decisions. Reserve relationships; do not fabricate invoices from a form with no amount-due/period.
4. Attendance is event logging only. Admission, expiry grace/freeze and approved debounce policy need acceptance; duplicate technical scans must not double-toggle events.
5. Printer and reader models/Windows acceptance remain unknown; static UI descriptions are not persisted fake hardware configuration.
6. Backup replacement must preview changes, preserve a validated recovery copy, reject incompatible/corrupt backups and stale confirmation, and flag restored state for future server reconciliation. No unattended backup retention/destination setting is present.
7. The existing empty inventory needs a real product-entry path. Expose only currently displayed product fields in the existing modal styling; do not seed products or import browser demo stock.
8. This schema does not add workouts/body tracking, tax/discount/customer fields, billing schedules, printer selectors, automatic retention, sync endpoint controls or any other settings absent from the application.
