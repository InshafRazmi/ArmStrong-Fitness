# Status — 2026-10-11

## Current update: version 0.1.9 published

Member-table Receive actions match the compact Edit buttons. The payment dialog
uses 16-pixel invoice checkboxes and a bounded invoice list, with amount and method
fields below it. Unnecessary interface instructions were removed. Windows and
Linux now poll synchronization and dashboard snapshots every five seconds while
preserving authentication, offline retry and overlapping-request guards.

Polling tests, rendered interface checks, native payment workflows and process
restart checks pass. Layout checks pass at 960×640 and 1366×768. Windows CI passes
212 native tests, 22 packaged Auth tests, native HTTPS, server contracts and UI
checks. This update uses the existing schema 11 backend and requires no migration.

[Release 0.1.9](https://github.com/InshafRazmi/ArmStrong-Fitness/releases/tag/v0.1.9)
is published as the latest updater release. The signed Windows installer is at
`Client/dist-windows/v0.1.9/ArmStrong.Fitness_0.1.9_x64-setup.exe`, 224,893,054
bytes, SHA-256 `4fd8cc096dd3a396cda776803b0ea15094d20ae8c2c149184b5f34ce479834f6`.
Signature, archive allowlist, x64/static runtime, embedded WebView2, native-only
frontend and public updater manifest checks pass. The Arch Linux package is at
`Client/dist-linux/v0.1.9/armstrong-fitness-0.1.9-1-x86_64.pkg.tar.zst`, 9,289,218
bytes, SHA-256 `ddd6335db75a7b6a98435464e3181b6ba985134c2892924cfefc3c9cdffbe472`.
Archive, version, exact native payload, launcher/icon/README and build-host
dependency checks pass. Both packages, checksums and verification reports are
attached. Physical Windows installation and reader/printer testing remain
separate acceptance checks. Source tag: `401f4cf461a1ccc9cc9bd828ce5044422e7c633d`;
Windows workflow: `38094601082`.

## Current update: Linux version 0.1.8 built and verified (2026-10-11)

The Arch Linux x86_64 package is built at
`Client/dist-linux/v0.1.8/armstrong-fitness-0.1.8-1-x86_64.pkg.tar.zst`
and attached to the existing 0.1.8 GitHub release. It is 9,289,429 bytes, SHA-256
`a9c1c49e9e6f0373271c0a812a0f9e296834afc7c93d16a036de4173686437db`.
The builder now takes its version from package.json and keeps outputs under a
versioned directory, preserving older packages. PKGBUILD and instructions use 0.1.8.
Separate SHA256SUMS-LINUX, VERIFICATION-LINUX.json and BUILD-INFO-LINUX.txt are
attached; the Windows artifacts and updater manifest are preserved.

Locked offline native release compilation and packaged frontend checks pass.
Archive integrity/allowlist, package version/architecture, exact stripped native
payload, launcher/icon/README, build-host library resolution, schema 11/admission
markers and exclusion of demo/smoke hooks pass. Physical package installation,
production keyring login and hardware testing remain separate acceptance.

## Current update: version 0.1.8 published (2026-10-11)

[Signed Windows release 0.1.8](https://github.com/InshafRazmi/ArmStrong-Fitness/releases/tag/v0.1.8)
is published and selected as the latest updater release. Production migration 9
is applied; public API health reports businessSchemaVersion 11. All requested
admission fee, registration invoices, unpaid dues, Receive payment and Pay salary
changes are delivered. Existing rows and pending profile recovery are preserved.

Windows workflow 38088507176 passes. The installer is 224,893,792 bytes, SHA-256
`d3b2bbbf027df7f59a1adb663041eaae6a9885b2fb35eb965b549fd3140767ac`.
Signature matches the app updater key; both updater platforms target the checked
installer. NSIS integrity/eight-file contents, x64/static runtime, exact WebView2,
admission feature and exclusion of browser demo/smoke assets pass. Public feed
matches the verified manifest; the public installer returns HTTP 200/exact size.
Checksums and VERIFICATION.json are attached to the release and saved locally in
Client/dist-windows/v0.1.8. Source tag points to 6b0f10bb18f38f40c7159a62f6bd28e0a1cfd344.

218 native tests, 22 packaged-auth checks, 84 server unit tests, interface and
protocol contracts pass. Real Auth/Postgres financial integration passes with
rollback. Desktop forms, finance, salary payout, reports, backups and separate
process persistence pass. Physical Windows installation, NFC and printing remain
separate hardware acceptance. No Authenticode certificate is configured.
The unrelated AttendancePage formatting change remains uncommitted and preserved.

## Current update: production admission migration applied (2026-10-11)

User-authorized server migration 9 is applied to the ArmStrong production
Supabase project. All prior migration 1–8 checksums matched source before
application; the version-9 application ledger records SHA-256
`336ecfd41446e504bf0f67e47e6bfcdec0f1b277db08ef9d716f79a47cf1c9cf`.
The Supabase migration tool also records the admission_settings change.

Application used an atomic guarded block with the existing migration advisory
lock, a five-second lock timeout, exact ledger/history-function baseline checks,
and an exclusive journal lock. It checked business content and table/function
security fingerprints inside the transaction before recording success.
Post-application read-only verification confirms all 129 business rows retain
fingerprint `62b574c23859e9f45cf1533c0a9bbbc1`. The validated table allowlist
includes admission_settings and RLS remains enabled. No gym/member/payment data
was inserted, edited or deleted.

A temporary-table probe invokes the actual production history trigger: admission
version 1→2 succeeds; stale versions, identity edits and deletion refuse; invoice
amount edits remain refused. Temporary probe rows are discarded at commit.
Security advisors show no new findings. Existing informational private-table
RLS/no-policy findings and pre-existing rls_auto_enable/Auth-password warnings
are unchanged and were not expanded into this migration's scope.

The database migration is complete and public API health now reports schema 11.
Desktop forms and separate-process persistence checks pass. Windows update
packaging/publishing is complete as recorded above.
Existing published installers and the old source ZIP are unchanged.

## Current update: admission charges and payment shortcuts (2026-10-11 candidate)

Settings → Gym profile includes Admission fee (LKR), defaulting to zero until
configured. An independently versioned, synchronized singleton keeps existing
profile rows and frozen sync/recovery envelopes unchanged. SQLite migration 11
preserves existing members, dues, receipts and queues. Server migration 9 adds
the new journal type within existing private gym scope and history/version
guards; API readiness requires it and health reports businessSchemaVersion 11.

Registration now invoices the selected membership for every new member,
including members without a trainer, and invoices admission separately once.
Each charge, its audit and outbox commit with the member. Exact retries retain
original charges; later setting changes affect future registrations only.
A stale reviewed fee refuses before creating records. Existing registrations are
not backfilled because their historical joining/payment intent is unknown.
Training charges retain the existing monthly workflow.

Members show Due to pay and a per-member Receive payment shortcut. Registration
offers Save as unpaid or Save and receive payment. The latter opens the existing
combined payment dialog after registration; canceling leaves durable dues.
Partial payments leave the remaining invoice balance due. If refresh fails,
payment waits for the saved member to load. Staff rows expose Pay salary through
the existing duplicate-protected salary/training-payout workflow.

Checks pass: browser and packaged frontend builds; packaged demo exclusion;
23 adapter tests and interface renders including admission/dues/salary controls;
218 native library tests (three existing environment probes ignored); nine
registration regressions; strict native Clippy and formatting; server typecheck
and 84 unit tests (one existing environment probe skipped); three real-native
SQLite/server protocol contracts. The all-module contract now includes an
admission setting, joining charge, audit and second-device download. Tests cover
partial-payment retries, restart persistence, stale settings/fees, unchanged
historical balances, migration and validated backup/restore.

This is local source work, not a published installer or deployed backend.
Production records were not modified. Live PostgreSQL migration 9, API rollout,
Windows interaction and installer publication remain separate acceptance steps.
Apply migration 9 and deploy the matching API before updating editing desktops
together; older desktop versions cannot import the new admission setting type.
The existing source ZIP and published releases do not contain this candidate.
The user's unrelated AttendancePage formatting remains preserved.

## Current update: first-sign-in retained profile recovery (0.1.7 candidate)

The new Windows screenshot shows Add audit, Add gym settings and Add users.
Version 0.1.6's fixture omitted the normal first-sign-in user reference, so native
eligibility refused this case and the recovery checkbox did not appear.
Version 0.1.7 permits at most one initial inactive/version-1 identity reference,
only for the current verified Administrator's unchanged local ID and Auth
subject. Local activation/version changes normalize exactly as ordinary sync;
name, email, subject, identity updates and additional identities remain refused.
The replacement preserves that exact reference and sign-in audit while omitting
only the installation default profile. Server identity conflicts still refuse
without replacing the original database. Backup, original bytes, real receipts,
stable lost-reply retry, complete download and native commit fences remain.

The native regression now reproduces the screenshot's three-record batch.
It checks backup and original-request preservation, roles/activation retention,
restart and lost-reply behavior, and refusal of edited or mismatched identities.
The full native library passes 216 tests (three existing probes ignored); the
additional server-identity refusal regression also passes. Strict Clippy/fmt,
22 packaged-auth tests, browser/packaged builds, 23 adapters, interface renders
and native desktop launch/restart pass. Server typecheck, 83 unit tests and
three business/one staff/one attendance native contracts pass. Both seed shapes
are included in live isolated Auth/PostgreSQL acceptance, which is in progress.
Signed Windows packaging is in progress; v0.1.5/v0.1.6 tags/assets remain unchanged.
The user's unrelated attendance formatting remains outside this change.
Actual recovery on the affected Windows computer remains an acceptance step.

## Previous update: version 0.1.6 recovery update published

The signed Windows [v0.1.6 release](https://github.com/InshafRazmi/ArmStrong-Fitness/releases/tag/v0.1.6)
is public, non-draft, non-prerelease and latest, published at 08:32:33 UTC.
Its unchanged source tag points to `f9a119a984c3dc54b43085a1b8b411c081e0b1c0`.
The existing v0.1.5 tag and all six public assets retain their recorded hashes.
Frontend, Tauri/native package and both lock files agree on version 0.1.6.

The verified Linux cross-build is saved in `dist-windows/v0.1.6/`:
`ArmStrong-Fitness_0.1.6_x64-setup.exe`, 224,952,155 bytes, SHA-256
`236e433eebc33f26bc093b92d25c55b8ad91e6359ba2ba31d9814c3af88d158d`.
All six uploaded assets match local sizes and hashes. The exact configured public
updater endpoint returns HTTP 200 with version 0.1.6 and the matching signature.
An anonymous full installer download matches that hash and verifies with the
existing embedded key using Tauri's minisign verifier. NSIS integrity/eight-file
allowlist, exact current native/WebView2 payloads, static C runtime, recovery
markers and exclusion of smoke hooks/private signing key pass. A fresh build of
the committed source reproduces identical frontend assets despite the user's
unstaged attendance formatting, which remains preserved.

The [tagged Windows checks](https://github.com/InshafRazmi/ArmStrong-Fitness/actions/runs/37905019831)
passed native business/authorization, real public HTTPS, server/native contracts,
packaged login and interface steps. Its remaining installer/publication work was
cancelled to prevent a second publisher replacing the already verified manifest
and invalidating its checksum record. A separate acceptance-only Windows build
is being started from main; it cannot publish or replace the release assets.
The published installer is the verified cross-build, not that pending CI package.

Install 0.1.6 from **Settings → Application updates**. Sign in online, open
**Server synchronization → Review retained transaction**, confirm **Recover
using server profile** if native eligibility permits it, keep the displayed
backup and sign in online again afterward. See [recovery instructions](docs/RESTORE_RECOVERY.md).
No affected Windows database or production business record was changed remotely.
Actual Windows installation, upgrade and recovery acceptance remain open.

## Previous update: guarded retained default-profile recovery (0.1.6 candidate)

The Windows review contains three Add actions: two audit entries and gym settings
named Armstrong Fitness. This fits an already frozen installation default
colliding with the newer shared profile. Version 0.1.5 prevents this on unused
computers but retains existing conflicts, so its original-request Retry cannot
resolve this case.

Native recovery now requires an online writable Administrator, the original
actor, current enrolled scope, current preview and explicit confirmation. It
checks that the profile is the exact unchanged installation default, the two
retained audits still match, the cursor is zero and there are no other business
records or saved operations. Audit-only previous Retry attempts are preserved.
Edited profiles, money, members, other business data and identity changes refuse
this recovery and remain available for separate reconciliation.

Before network work it writes a validated native backup. An isolated database
rechecks the exact original request: an actual server receipt takes precedence;
only a fresh revision refusal permits a stable audit-only replacement. The
original request stays immutable, refused and without a fake receipt. A separate
append-only recovery operation removes that seed from the active queue. All
original audits remain; confirmation counts include only actual receipts.
Lost replies retry the same replacement operation. Verified ordered server
history must complete before an atomic replacement, with nonce, expiry, native
session and full storage fingerprints rechecked. Cancellation or concurrent
changes leave the original database intact. Successful recovery requires online
sign-in again; replacement and recovery audits await their own real receipts.

The interface offers Recover using server profile only when native checks allow
it, requires a checkbox and displays refusal reasons for other cases. It reports
recovered profiles separately from confirmed server transactions. Eight new
SQLite/mock-transport recovery cases and the recovery dialog render pass. The
full native library passes 214 tests (three existing environment probes ignored);
strict native Clippy and formatting pass. Browser and packaged frontend builds,
23 adapter tests, all interface renders and the desktop webview launch/restart
checks pass. All 22 packaged-auth desktop tests pass. Server typecheck and 83 unit
tests pass; the sandbox-skipped CLI
probe passes separately with all 13 desktop-setup tests. Native business, staff
and attendance contracts pass, including the audit-only replacement.

Live isolated Supabase Auth/PostgreSQL acceptance passes the stale-profile
refusal, absence of an original receipt, exact replacement retry, retained audits,
version-3 server profile and complete ordered download. Synthetic records roll
back. The production backend and affected Windows database were not changed.
The Windows x64 native cross-build compiles; signed NSIS packaging and public
release verification are in progress. Actual Windows recovery acceptance remains
open. Version 0.1.6 is being prepared; the public 0.1.5 installer and tag remain
unchanged. The user's unrelated attendance formatting remains preserved and
outside this patch. See [recovery steps](docs/RESTORE_RECOVERY.md).

## Previous update: version 0.1.5 published

The user-authorized signed Windows [v0.1.5 release](https://github.com/InshafRazmi/ArmStrong-Fitness/releases/tag/v0.1.5)
is public, non-draft, non-prerelease and latest, published at 06:37:35 UTC.
Source commit `f825dab4836670290f7c4352185ab064f057d759` and its `v0.1.5`
tag are pushed; the original tag is unchanged. Frontend, native package, Tauri
and both lock files agree on 0.1.5. It includes the unused-computer initial
download and clearer refusal/update errors described below.

The signed Windows x64 installer in `dist-windows/` and the public release is
`ArmStrong-Fitness_0.1.5_x64-setup.exe`, 224,916,656 bytes, SHA-256
`9ae54e475caf99efd6afaa0338c3e20f239ba474f8ada4c1c97a34f8d97f73d2`.
All six uploaded release assets match local sizes and SHA-256 hashes. The
application's exact configured public endpoint returns HTTP 200 and valid
0.1.5 JSON. Anonymous download of the manifest's installer matches the entire
local package and its signature verifies with the existing embedded updater
public key using Tauri's minisign verifier. NSIS integrity/allowlist, exact
native/WebView2 payloads and initial-download feature markers also pass.
Updater-enabled 0.1.0 installations can use Settings → Application updates →
Check for updates → Install 0.1.5 and restart.

Windows runs pass 200 native tests (one ignored) and the real public Auth/API
HTTPS probe. The corrected [acceptance workflow](https://github.com/InshafRazmi/ArmStrong-Fitness/actions/runs/37894112756)
also passes server/native business, staff and attendance contracts, bundled
login settings and interface checks; its separate Windows installer build is
still running. The published installer is the verified Linux cross-build of
the exact tag. Actual Windows installation and in-app upgrade acceptance remain
open; a successful public download is not runtime installation evidence.

Two earlier tag runs stopped on verification fixtures: Windows CRLF conversion
changed vendored source checksums, and JSON-escaped dotenv values changed
Windows paths. Main now preserves the vendored bytes and uses literal quoted
test values, with a synthetic Windows-path regression. Neither the upstream
checksums nor the runtime's exact-environment guard were weakened. The workflow
supports a tagged rebuild using its corrected checkout logic, while tests still
come from the selected tag. Server typecheck and 83 unit tests pass on Linux
(one sandbox-blocked CLI probe); all 13 desktop-setup tests subsequently pass
outside the sandbox, including actual CLI subprocesses. The original tagged
server fixture remains historical source; it was not changed to publish this
cross-build.

Existing frozen Windows conflicts still need their actual request reviewed;
the update preserves them rather than rewriting or acknowledging their history.
The pre-existing attendance-page formatting remains uncommitted in the workspace.

## Previous update: Windows revision conflict and missing updater feed

The user reports Linux synchronization succeeds after retained-transaction review,
while Windows reports `business_revision_conflict`. Live API health now reports
businessSchemaVersion 10. Read-only production checks show the shared gym profile
at version 3; every unused installation starts with the version-1 default. A
strict two-computer SQLite regression reproduces that first-upload mismatch.
The Windows retained request has not been supplied, so this remains a likely
cause of that computer's specific refusal, not a confirmed repair of its queue.

Source now downloads existing shared records before capturing an unused
computer's installation baseline. Download progress survives bounded runs and
restart; saves during download roll back. Only the unfrozen default profile
journal entry is removed after a verified profile download. Identity/sign-in
audit remain queued, and an empty cloud still receives the first computer's
default. Existing frozen requests and financial conflicts remain guarded.
Revision-conflict text explains why exact retry cannot reconcile changed data.

Before the v0.1.5 publication above, the configured GitHub updater endpoint
returned HTTP 404 and the repository's release collection was empty. Prepared
`dist-windows/latest.json` plus
`RELEASE-UPLOAD.md` for the existing signed 0.1.0 installer. Installer and signature
bytes are unchanged; its checksum and Tauri minisign signature pass, and
SHA256SUMS now also covers the manifest. Publishing the public v0.1.0 release
and those exact assets was the previous proposed remedy, superseded by the
published v0.1.5 release. That manifest described the previous build and did not
include the later synchronization source fix. Update checks
now explain missing/invalid release metadata while retaining other errors.

Checks: browser and packaged builds PASS; 23 adapter tests and all interface
renders PASS; native library 206 PASS, 3 existing environment probes ignored.
Strict native library/test Clippy and whitespace checks PASS.
The regression covers a newer shared profile, bounded download/restart, rollback
of a save during download, empty-cloud bootstrap and preservation of frozen
refused bytes. No installed SQLite, queue, production record, source commit/push
or public release was changed. Windows request inspection, its safe conflict
reconciliation, a rebuilt installer and actual Windows acceptance remain open.

## Previous update: Windows installer with verified updater signature

Built Windows x64 version 0.1.0 from source commit
`03a3bf47060c131251a5c802386c2c3051626e46`, including schema 10 staff deletion,
the larger NFC panel, dashboard attendance counts, retained transaction review
and the signed updater. The installer is
`dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe` with its `.sig`, SHA256SUMS,
BUILD-INFO.txt and VERIFICATION.json. Size: 223,170,297 bytes / 212.83 MiB.
SHA-256: `1b53c4893540b199eb011303e143f33b48a627970dc229c8d0c5e8be297a485a`.

Checks pass: packaged frontend/native bridge requirements, optimized Windows
MSVC x64 cross-build, NSIS integrity and eight-file allowlist, current native
payload (only the expected three-byte Tauri NSIS bundle marker differs from the
unbundled executable), exact cached WebView2 payload, static C runtime, native
staff/NFC/schema markers, embedded updater public key/feed, exclusion of smoke
hooks/private signing key, and updater signature verification using the same
minisign verifier as Tauri. The existing backed-up key was reused.
An attendance-page formatting change during the build was preserved; rebuilding
the current frontend produces byte-for-byte identical packaged assets.

The package is cross-built on Linux and has an updater signature; no Windows
Authenticode publisher certificate is configured. Actual Windows installation,
login/offline/reconnect, data-preserving upgrade, NFC and printing remain open.
No app installation, API deployment, retained-transaction retry, GitHub release,
commit or push was performed. Gym data and queues were not changed.

## Previous update: signed Windows application updater

Windows builds now include Tauri's signed updater runtime and a Settings page
that checks GitHub Releases when opened, shows the installed version and offers
to install/restart only after staff choose. Updater artifacts are restricted to
the Windows release config; the GitHub Actions workflow publishes signed NSIS
packages and `latest.json` on `v*` tags. Local SQLite stays in app-data.

The signing keypair is generated and backed up outside Git at
`/home/prinzz/.tauri/armstrong-fitness/updater.key` and `updater.key.pub`.
The backup directory is mode 700 and both key files are mode 600; their contents
match the originals, and the public key matches the app's `tauri.conf.json`.
This persistent backup survives clearing `/tmp`. Configure the private key
contents as the GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` before pushing
a tagged release. No secret was sent to GitHub, and no commit or push was made.
Existing installations need one manual install of an updater-enabled build
before they can self-update. See [setup steps](docs/WINDOWS_INSTALLER.md).

Validation: frontend production build passes; locked native Rust updater check
passes; Tauri debug build passes with both Windows installer/updater configs
merged and bundling disabled. The later local Windows NSIS cross-build and
signature verification pass as recorded above. A Windows-runner build, actual
Windows installation and live update remain unverified.

## Current update: retained staff transaction deployment diagnosis

The installed app is now schema 10. A read-only snapshot contains six confirmed,
one refused and twenty-one pending business batches. The first refused request
adds staff (`audit`, `trainers`). All 22 retained requests pass the corrected
server's exact row and sequential complete-state validation against current
production gym records. No retained request already has a server receipt.
The restricted runtime login passes current business readiness and verified TLS.
No SQLite records, queue states, frozen requests or production data were changed
by these checks; this is validation, not live receipt confirmation.

GitHub main still points to 6cb5aef06bdc9a36f53872ff638e9663156c9b6c and its
business schema has no Staff tables. Render /v2/health still returns protocol 2
without businessSchemaVersion. Production database migrations 1–8 are already
current; publishing and deploying the prepared server source is the remaining
live fix. Rebuilt source passes typecheck and 82 server unit tests (one existing
probe skipped). The Render plugin is unconnected; no Render CLI/account/API key
or deploy hook is configured locally. Connection and GitHub push status were
requested. The user's existing preference is to handle commit/push.

After publishing, use Render Manual Deploy → Deploy latest commit (a service
restart reuses the existing code). Verify /v2/health includes
businessSchemaVersion 10, then use Settings → Server synchronization → Review
retained transaction → Retry original transaction. Preserve the original request
and review any further server refusal rather than clearing history. See the
current [Render recovery instructions](../server/docs/RENDER_SETUP.md).

## Previous update: permanent staff deletion and larger NFC card panel

Staff → Delete staff is available for active and inactive profiles. An
Administrator must check the aligned confirmation. Native schema 10 adds an
immutable removal marker; deletion deactivates the profile, revokes its NFC card
and clears current member assignments atomically. It disappears from both normal
lists. Show deleted staff retains payroll/training/attendance history and final
earnings payments. Deleted profiles cannot be edited or reactivated. Existing
schema-9 and earlier records, frozen transaction bytes and identities are retained.

NFC Attendance uses a wider reader panel, a 260×166px amber card graphic, larger
type, 48px scan controls and a separate manual selector. Short-window styling
keeps the scan action visible at 1366×681. Existing keyboard/HID scanning, owner
detection and manual attendance use the same native operations.

Checks: browser/native-only packaged builds and 22 adapter tests PASS; all
route/settings/confirmation renders PASS; 203 native library tests PASS, 3
existing environment probes ignored; strict native Clippy PASS. The updated
native two-database test also passes permanent staff deletion download and the
reactivation guard. Native removal envelopes pass the API contract. Real
isolated Supabase Auth/PostgreSQL acceptance passes all eight native transactions,
exact retries, second-device download, retained salary/attendance and API/SQL
immutability guards; all synthetic business rows roll back. Server typecheck
and 82 unit tests PASS, 1 existing subprocess probe skipped. Desktop webview
and restart checks verify the larger NFC controls and deletion authorization.

Production server migrations 6–8 were applied with the checksummed owner runner;
the ledger now contains exact versions 1–8. All ten application-table counts
and fingerprints match the earlier read-only snapshot. Verified TLS and startup
readiness pass using the actual restricted runtime login, without new grants.
Security advisors report the existing private RLS/no-public-policy information,
public rls_auto_enable execute warnings and disabled leaked-password protection;
these pre-existing platform settings were not changed.

Render still returns the older /v2/health body without businessSchemaVersion.
Deploy the prepared current API source; its health reports businessSchemaVersion
10. The Render plugin is available but unconnected and no deployment credential
is available locally. The user handles commit/push. After deployment, review and
retry the original retained transaction in Settings → Server synchronization.
No installed database, frozen requests or queued transactions were changed.
Final read-only inspection sees installed schema 9, six confirmed batches, one
blocked batch and sixteen pending batches. The installed app remains on its
existing build until the user installs the updated package.

Updated Arch package: 7,785,807 bytes / 7.43 MiB; SHA-256
`df3286c3d20fc3e80bc995d023b8e2fa734d6a2bea5bce707db230171bbbf477`.
The package includes schema 10 and the new NFC layout. Archive allowlist, exact
stripped current-release payload, README/launcher/icon, native-only frontend,
schema/removal commands, smoke-hook exclusion, runtime dependencies and checksum
manifest all PASS; adjacent BUILD-INFO and VERIFICATION match. Verified source
ZIPs contain the current API/migrations and desktop patch, with baseline
application, CRC and all file hashes checked. Private env/certificates/databases
and build outputs are excluded. Installation, Windows rebuild,
Render deployment and source commit/push remain separate.

## Previous update: deletion controls, staff removal and visible dashboard counts

Deletion confirmations use an 18px checkbox beside readable text across member,
staff and expense dialogs. Member and staff deletion wait for explicit checkbox
confirmation. Staff now has an Administrator-only Delete action: deactivate the
profile, revoke its active NFC card and clear current trainer assignments
atomically. Inactive profiles remain available for history, final earnings payouts
and reactivation. Existing salaries, training invoices, receipts, attendance,
audit and retry receipts retain their identities. No new database migration is
required for removal; native schema remains 9.

The dashboard puts larger Male/Female counts in the top attendance KPI and the
first row's member card, displays today's activity and stretches its two card
rows to the available window height. The populated fixture has no scrolling at
1366x681 and 1920x993 actual browser viewports; the latter ends with 20px bottom
padding. A 960x553 viewport needs 109px vertical scrolling, with both gender
counts visible at the top. Checkbox geometry confirms 18x18px and 12px separation
from the confirmation text.

The installed Linux database was inspected read-only: its first refused batch
is staff creation (`trainers` and `audit`). The exact frozen request passes the
current local API validator; workspace HEAD lacks the Staff contract. Live
Supabase catalog queries confirm production migrations 1–5 and a business table
allowlist without Staff. Migrations 6–7 and the matching API deployment are still
needed before Settings → Server synchronization → Review retained transaction
can recover the original request. No local application records or queue were
changed. New unsupported tables now receive `unsupported_business_table`; native
refusals point to deployment support and the existing review/retry action.

Checks: browser build and 22 adapter/counting tests PASS; all route/settings/
confirmation renders PASS; 201 native library tests PASS, 3 existing environment
probes ignored; strict native Clippy PASS; server typecheck and 82 unit tests
PASS, 1 existing subprocess probe skipped. The real Linux webview and second
process restart PASS, including checkbox geometry, staff deletion authorization,
inactive-staff navigation and prominent gender counts. Native staff-removal
envelopes pass the server contract, preserving paid amounts and attendance.
An additional focused native check confirms removed staff keep collected unpaid
earnings and can receive a final payout exactly once. The final read-only local
queue check still shows the original 6 confirmed, 1 blocked and 11 pending batches.

Rebuilt `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`:
7,785,106 bytes / 7.42 MiB; SHA-256
`6d8809df653de0a93020b9f461354bd17f7f9d8deb42c551b9ba86212af721a8`.
Package allowlist, exact stripped release payload, current README/launcher/icon,
schema 9 and staff-removal/attendance/retry commands, exclusion of smoke hooks,
declared GTK/WebKit/curl/libsecret dependencies, runtime library resolution and
checksum manifest PASS. Adjacent BUILD-INFO and VERIFICATION match this build.
Production schema/API changes, installation, commit/push and a Windows rebuild
have not been performed for this fix.

## Previous update: Linux package rebuilt with the latest confirmed changes

Built `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst` from the current local source on Arch
Linux x86_64. The package includes compact login/dashboard, staff NFC/manual
attendance and separate dashboard activity, member Male/Female profiles/daily
counts, permanent operational member removal with historical retention, and
blocked-transaction review/exact retry. It also retains Staff/training/billing
features and embeds the approved public login settings. No application behavior
or version changed in this packaging milestone; SQLite migration 9 remains the
normal data-preserving upgrade path.

Package: 7,782,060 bytes / 7.42 MiB; SHA-256
`df4805e5f6d6e7afa57011a419835e8f78997023b223d5ae8d12900aec6b893d`. Adjacent SHA256SUMS,
BUILD-INFO.txt and VERIFICATION.json match the new build.

Measured checks:

- Browser and packaged frontend builds PASS; browser demo remains excluded.
- Locked optimized native Linux compilation with desktop/custom-protocol/
  packaged-auth PASS; no UI smoke commands in the release executable.
- Archive file allowlist and exact stripped current-release payload PASS.
  Packaged README, launcher and icon match their source; launcher validation PASS.
- Full schema 9 plus staff/NFC/recovery commands present. Runtime library
  resolution and declared GTK/WebKitGTK/curl/libsecret dependencies PASS on
  this build host (glibc 2.44, WebKitGTK 4.1).
- Package SHA-256 verification PASS. Earlier business/backend/webview/restart
  checks below still apply and were not repeated for this build-only change.

The package is for Arch Linux x86_64. No installation or production login/keyring/
network/hardware acceptance is claimed. Production remains on migrations 1–5;
new shared records need migrations 6–7 and the matching API. No production schema
change, API deployment or source commit/push was performed. See
[Linux installation](docs/ARCH_LINUX.md).

## Previous update: compact layout, staff NFC, gender counts and permanent removal

Implemented the user-confirmed changes with additive native schema 9 and server
migration 7, preserving saved schema-8/earlier transaction bytes and historical
rows. Login and dashboard use compact layouts. Dashboard has distinct member and
staff attendance cards, and male/female member counts. Add/Edit member requires
a Male/Female choice; earlier unspecified profiles remain explicit. Daily totals
count each person once after a non-voided check-in. Staff supports unique NFC
cards, automatic card-owner routing, manual attendance, daily check-in/out and
two-second duplicate-scan suppression. Durable retries preserve the original
owner even after card reassignment.

Administrator permanent deletion removes active and archived members from both
operational lists. A synchronized immutable removal marker retains the archived
identity and all financial, receipt, membership, training, attendance and audit
references; the active card is revoked. Archive remains separately available.

Business synchronization now shows the actual server refusal and offers a native
Administrator review-and-retry action for the first blocked transaction. A stale
review fails; the original actor is required. Exact frozen requests and operation
IDs are resent after saving review/audit history. This recovers refusals after a
corrected backend deployment/configuration. General stale financial/master
merging remains guarded; retry does not force acceptance or overwrite history.
The specific transaction on the user's installed computer has not been inspected.

Measured checks so far:

- Browser and packaged frontend builds PASS; no demo code in desktop assets.
- 22 adapter/counting tests and ten route/settings/login/receipt renders PASS.
- Native library: 199 PASS, 3 environment probes ignored. New checks cover
  gender/version rollback, cross-category unique cards, staff day rollover,
  duplicate scans, reassigned-card replay/restart, backup/restore, exact blocked
  retry and two-device deletion with immutable billing/attendance/receipt history.
- Strict native Clippy PASS; server typecheck and 82 unit tests PASS (one
  pre-existing sandbox CLI probe skipped).
- Native attendance envelopes match server contract. Real isolated Supabase
  Auth/PostgreSQL acceptance PASS with exact retries, second-device downloads,
  duplicate-card rejection, immutable attendance and historical removal.
  Every synthetic API fixture rolled back. Only isolated migration 7 was applied;
  it extends the existing test schema, which has no application migration ledger.
- Chrome dense fixtures: login fits 960x640; offline login requires 9–10px of
  scrolling; dashboard uses two compact rows and short scrolling at small heights.
  Final populated fixture measures 38px scrolling at 960x640 and 10px at
  1366x768. Actual Linux desktop forms and a second process restart PASS,
  including Male/Female registration, automatic staff NFC routing, manual staff
  check-out, separate dashboard counts and retained financial workflows.
- Windows x64 release/NSIS packaging PASS. Verified archive integrity, exact
  app/WebView2 payloads, x64/static runtime/vault imports, full migration 9,
  recovery IPC and exclusion of UI smoke hooks. Delivery installer is
  222,072,198 bytes / 211.78 MiB, SHA-256
  `3aee104fdcf710c0895dde57736709fc38920b3f66fe046ae8d8c1fb654b4e19`. Adjacent SHA256SUMS and BUILD-INFO match.
  Unsigned Linux cross-build; actual Windows/hardware acceptance remains open.

Production remains on server migrations 1–5. Apply migrations 6–7 with the owner
runner, deploy the matching API and upgrade sharing desktops before using the
new records across computers. The user handles commit/push; no production schema
change, API deployment or source publication was performed. See [staff/attendance
workflow](docs/STAFF.md) and [sync recovery](../server/docs/BUSINESS_SYNC.md).

## Previous update: Windows x64 installer rebuilt with Staff

Rebuilt `Client/dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe` from the
current local source, including Staff/monthly training, combined collection,
salary payouts, Administrator editing access, the Arm logo, larger Quick actions
and scrolling computer registration. The per-user NSIS installer embeds the x64
WebView2 offline installer and public login settings. Existing SQLite records,
identities and pending operations use the normal data-preserving migration path.
No application behavior or version number changed in this packaging milestone.

Installer: 222,043,065 bytes / 211.76 MiB, SHA-256
`b01c893e85c8fcd7d24a7ffdd42d137493de5706de0778be08427962e55aead7`.
The delivery copy, adjacent `SHA256SUMS` and `BUILD-INFO.txt` match this build.

Measured checks:

- Browser and packaged frontend builds PASS; packaged public settings PASS;
  browser demo access/data remain excluded from the desktop assets.
- All 21 adapter tests and ten route/settings/login/receipt renders PASS.
- Windows MSVC x64 optimized compilation and NSIS packaging PASS with Rust
  1.98.0, Tauri CLI 2.12.1, cargo-xwin 0.23.1 and locked Rust dependencies.
  Missing Microsoft SDK PDB warnings do not prevent release linking.
- NSIS integrity and the exact eight expected files PASS. Embedded app bytes
  match the compiled executable with Tauri's NSIS marker patch; embedded
  WebView2 bytes match Microsoft's downloaded runtime. x64 PE, static C runtime,
  Windows Credential Manager/system-directory imports, complete Staff migration
  8 and exclusion of UI smoke hooks PASS. Delivery SHA-256 verification PASS.

This is an unsigned Linux cross-build. Actual Windows installation, login,
offline/reconnect, data-preserving upgrades, NFC and printing remain acceptance
gates. Shared Staff data requires server migration 6 and the matching updated
API; production remains on migrations 1–5. No installation, API deployment or
source commit/push was performed. Prior native business checks are recorded
below and were not repeated for this packaging-only rebuild.

## Previous update: Staff, monthly training and payment UX

The installed application now has Staff profiles with name, mobile, unique NIC,
fixed monthly salary and personal-training fee per member per month. Member
registration/editing selects an active trainer or None. Registration with a
trainer creates the first monthly training invoice and any paid membership
invoice atomically. Future training months are billed explicitly from Payments.
The gym collects selected membership and training invoices in one payment and
saved receipt. Staff payouts record the reviewed fixed salary plus collected,
allocated training fees as a Salary expense; unpaid invoices earn no fee.
Salary is paid once per month and each collected allocation once. See
[staff operation](docs/STAFF.md) for top-ups and correction/refund behavior.

Quick actions have larger cards, text and click targets. Computer registration
has a styled expandable panel, readable setup fields and a sign-in layout that
grows and scrolls when the panel is open.

SQLite migration 8 adds five tables without replacing earlier records,
identities, audit, queues or frozen request bytes. Backups/restores and all-module
sync include the new records. Immutable history, rate/version checks, duplicate
NIC/month/invoice/payout guards and Administrator IPC authorization remain native
and server responsibilities. Trainer profiles do not create login accounts.

CLI-created server migration 6 extends the private business allowlist/history
guards and adds a gym-scoped unique NIC index. It is applied to the isolated
test project only; production remains on migrations 1–5. Apply migration 6,
deploy the updated API and upgrade every gym desktop before using Staff across
computers. Older clients refuse unknown Staff rows while retaining their queues.
No production Staff data, source publication or workspace commit/push is claimed.

Measured checks:

- Full native suite: 192 PASS, three intentionally ignored environment probes.
  An additional two-database Staff download/refund test PASS; 193 distinct native
  tests passed. Schema-7 upgrade preserves frozen queue bytes and reopening;
  complete staff backup/restore, exact retries, stale rates/earnings, combined
  receipts, monthly overlap and payout/refund guards PASS.
- Native Clippy with warnings denied PASS. Native desktop IPC build PASS.
- Server build verification: 82 PASS, one sandbox subprocess skip; strict types
  and pinned declaration checks PASS. Five real native Staff envelopes match the
  server contract; forged amounts, dates, NIC and payout records are refused.
- Real isolated Supabase Auth/PostgreSQL Staff integration PASS: exact retries,
  another Administrator device's download/edit, stale writes, duplicate salary,
  unique NIC and immutable payout checks. All business fixtures roll back.
- Isolated security advisers match the prior baseline: ten private RLS/no-policy
  INFO findings ([guide](https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy))
  and the existing Auth leaked-password-protection WARN
  ([guide](https://supabase.com/docs/guides/auth/password-security#password-strength-and-leaked-password-protection));
  no new finding was introduced.
- UI adapter: 21 PASS; all ten routes, settings, login/startup and saved receipt
  renders PASS. Real Linux webview staff create/NIC, member assignment, monthly
  invoice, collection, salary payout, larger Quick actions, report/backup export
  and process restart PASS against an isolated SQLite database. This is local
  UI acceptance, separate from production login/keyring/offline/reconnect.
- Browser/packaged frontend builds and patch whitespace PASS. Arch release and
  package build PASS. Package checksum, archive contents, exact stripped native
  binary, Staff migration payload, bundled installation instructions, desktop
  entry/icon and dynamic-library resolution PASS.

Rebuilt `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`:
7,747,567 bytes, SHA-256
`eaf9dc29f1e6fbd60be72912e969e482eeb2e6a1db5fe7385a33bd31d30eef8a`.
The adjacent `SHA256SUMS` matches. Reinstall to update the installed app;
installation was not performed. Staff is ready locally; shared Staff sync requires
the migration/API deployment described above. Source delivery bundles were
rebuilt from workspace HEAD; binary patch application and archive/hash checks PASS.

The user handles source commit/push. The Windows rebuild with Staff is recorded
above; the Windows workflow now also verifies the native Staff contract.
Existing Windows/hardware/network release gates remain in PLAN.md.

## Previous update: Administrator editing access from every enrolled device

The user requires Administrator accounts to read and write from any device.
Active Administrators now receive editing access from every valid enrolled
computer. CLI-created PostgreSQL migration 5 removes the single-writer index
and upgrades earlier read-only enrollments during online sign-in. It retains
device IDs, credential hashes, gym boundaries, revocation and active-role checks.
Local API request authorization also derives Administrator write access from
the verified private staff role; other roles keep their operation restrictions.
Manual registration no longer rejects a computer because another writer exists.

Migration 5 was applied first to the isolated test project, then to production
with versions 1–4 checksum guards and the matching version 5 ledger entry.
All ten existing application tables have identical before/after row counts and
fingerprints. The runtime can execute private enrollment but cannot directly
update staff/devices; anon/authenticated cannot execute it. Rolled-back probes
against the existing active Administrator devices confirm write access without
changing identities or credentials. Security advisers match the existing
baseline; no new finding was introduced.

The live enrollment function works with the existing automatic API endpoint,
so the installed client can receive the new grant without an API restart or
reinstallation. Sign out and sign in online to refresh an older read-only
session and its offline grant. The additional role-based API request checks
remain local until the user publishes/deploys this source; no commit/push or
API source deployment is claimed in this update.

Measured checks:

- Server verification: 82 tests PASS, one sandbox subprocess test skipped;
  strict type checking and vendored declaration verification PASS.
- Real PostgreSQL enrollment regression PASS: multiple editing devices,
  legacy grant upgrade, exact retries, unchanged credentials, and invalid
  secret/account/role/revocation/gym/JWT-subject denials. Fixtures roll back.
- Native all-module fixture generator PASS. Live Supabase Auth/PostgreSQL
  integration PASS using those native envelopes: two Administrator computers
  commit changes, exact retries preserve receipts, stale writes conflict, and
  immutable finance/stock/history, gym isolation and revocation checks pass.
  Test data rolls back. This is separate from GUI/offline acceptance.
- Browser/packaged frontend and Arch Linux release/package build PASS. Package checksum,
  exact stripped release-binary match, current Arm-logo/styles and bundled
  installation instructions PASS.

Rebuilt `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`:
7,699,218 bytes, SHA-256
`1dc2a3973f6cb79aa2d3e1fba96a0b99ff5f10269fe7d93ea203f53ffff50098`.
The adjacent `SHA256SUMS` matches. Installation was not performed; the package
includes the larger supplied sidebar logo. Windows was not rebuilt here.
Real multi-computer offline/reconnect, general conflict recovery and existing
Windows/network/hardware acceptance remain release gates in PLAN.md.

## Previous update: larger sidebar logo and rebuilt Linux installer

The sidebar logo is centered and enlarged from 52 px to 144 px in the full
sidebar and 76 px in the compact sidebar. The compact sidebar is now 100 px
wide to accommodate the larger image. Sidebar scrolling keeps navigation
accessible in shorter windows. Login and startup logo sizes remain 52 px.

Browser and packaged desktop builds PASS, including the existing packaged
frontend check. The Arch Linux x86_64 release/package build PASS. Package
checksum, expected archive contents, exact stripped release-binary match and
inclusion of the latest stylesheet/logo assets PASS. Patch whitespace PASS.
This is a CSS-only change; native business/authentication logic is unchanged.

Rebuilt `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`:
7,699,120 bytes, SHA-256
`c22fd233988b94b0679dbdd85736a69fde1f446697745b6c8a473cc2d5d2fce8`.
The adjacent `SHA256SUMS` matches. Reinstall this package and reopen the app to
apply the larger sidebar logo. Installation was not performed in this update;
existing release gates remain in PLAN.md. Source remains local for the user's
commit/push; the Windows installer has not been rebuilt in this update.

## Previous update: supplied Arm logo and rebuilt Linux installer

The sidebar, sign-in screen and desktop session-startup screen now use
`src/img/ArmLogo.png` instead of the letter A. The shared brand styles display
the complete image at its original aspect ratio without the old amber shield
background or clipping, with accessible image text.

Browser and packaged desktop builds PASS. All 21 adapter/calendar tests and
the existing route/settings/login/startup/receipt renders PASS. The Arch Linux
x86_64 release and package build PASS with locked offline Rust dependencies and
`desktop,custom-protocol,packaged-auth`. Package checksum, archive contents,
executable permissions, exact stripped release-binary match, embedded Arm-logo
asset and dynamic-library resolution PASS. The bundled logo matches the supplied
image byte for byte; browser demo login/data remain excluded from the package.

Rebuilt `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`:
7,701,696 bytes, SHA-256
`5a23bf2697ceff4bdea0c645844e2a7db22c4075a0c22f5f91e4038dea395150`.
The adjacent `SHA256SUMS` matches. Install or reinstall this package to see the
logo in the installed Linux app. Installation was not performed in this update;
existing authentication/network/hardware release gates remain in PLAN.md.
No native business/schema change is involved. Source remains local for the
user's commit/push; the Windows installer has not been rebuilt in this update.

## Previous update: remove Attendance and Reports implementation notices

Removed both requested desktop notices from the shared screen-notice component.
The Attendance subtitle now describes NFC/manual check-ins and check-outs rather
than SQLite implementation and unfinished admission/hardware work. Reports keeps
its existing date controls and range-aware CSV behavior. This is a presentation
change; attendance admission/grace/freeze policy and real Windows reader testing
remain unfinished, as documented in the attendance/release milestones.

Browser build and all 21 adapter tests plus route/settings/login/receipt renders
PASS. Packaged desktop build PASS; its assets exclude both notices and the old
Attendance subtitle and retain the Reports date controls. Windows x64 release
and NSIS packaging PASS. Installer archive integrity, exact app/WebView2 payload
hashes, Windows OS/runtime imports and delivery-copy SHA-256 PASS. Patch whitespace
PASS. No native business/authentication code changed in this milestone; the prior
native checks still apply.

Rebuilt `Client/dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe` with these
removals and the previous HTTPS correction: 220,150,558 bytes / 209.95 MiB,
SHA-256 `3437c13232437ea6f3d2428d83f07323274a0d994711cecd9ed9b79445f446e9`.
`SHA256SUMS` and `BUILD-INFO.txt` match this delivery copy. Reinstall to update
the installed Windows app. Existing Windows runtime/hardware release gates
remain open. Source remains local for the user's commit/push.

## Previous update: Windows sign-in connection handling and rebuilt installer

The user reports that the same Administrator can sign in on Linux and that
Windows browser API health works, but the Windows app reports
`Sign-in connection unavailable; values withheld`. That message previously hid
all native transport failures and the failing sign-in stage. The exact failure
on the reporting Windows computer is not yet confirmed by a native probe.

The Windows HTTPS child previously cleared all system variables. It now receives
only `LC_ALL=C` and OS-derived `SystemRoot`/`WINDIR`, retaining the Windows root
needed by loader/security components without inheriting PATH, proxy, certificate
override or TLS key-log variables. The trusted System32 curl path, verified TLS,
no-redirect policy, bounded secret stdin and process deadlines are retained.
Fixed connection messages identify the sign-in/identity/enrollment stage and
missing/incompatible client, DNS, connection, timeout, TLS, invalid-response or
other outage categories. No raw process/provider diagnostics or credentials are
shown. Sync/renewal outages remain transient and failed login grants no access.

Rebuilt `Client/dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe`:
220,137,873 bytes / 209.94 MiB, SHA-256
`c73429246fa744179daee1b80ec2eab380752fefc327a9846f895f45375b2f96`.
The delivery folder includes matching `SHA256SUMS` and updated `BUILD-INFO.txt`.
This supersedes the previous installer at the same path.

Measured checks: 184 packaged-auth native tests PASS, 3 ignored; native Clippy
with warnings denied PASS; browser/packaged frontend builds, all 21 adapter
tests and route/settings/login/receipt renders PASS. Real credential-free public
Auth/API HTTPS and forged-plaintext TLS rejection probes separately PASS on Linux.
Windows MSVC x64 release and NSIS packaging PASS. PE imports include the new
OS Windows-directory lookup and no external Visual C++ runtime dependency.
Installer archive integrity, exact eight files, app/runtime payload hashes and
delivery-copy checksum PASS. Patch whitespace PASS.

Windows Actions now runs the same credential-free native public-service HTTPS
probe before packaging; this workflow has not run for the local change and
depends on public-service availability. Login on the reporting Windows computer
still needs confirmation after reinstalling this build. Account/device setup,
data and credentials were not reset. Source remains local for the user's push.

## Previous update: first Windows x64 NSIS installer built

The first local Windows setup executable was produced at:
`Client/dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe`
(220,129,901 bytes / 209.93 MiB), with `SHA256SUMS` and `BUILD-INFO.txt`.
It includes the recent sign-in diagnostics, membership registration/remaining
days, dropdown theme, notice removals and startup session gate. The per-user
installer embeds Microsoft's x64 WebView2 offline installer. Build outputs and
isolated build tools are ignored; source remains local for the user's commit/push.

Built on Linux with Rust 1.98.0, Tauri CLI 2.12.1 and cargo-xwin 0.23.1, locked
dependencies and `desktop,custom-protocol,packaged-auth`. The initial executable
required Visual C++ runtime DLLs. `Client/.cargo/config.toml` now enables the
static C runtime for Windows x64; the final PE imports only Windows OS libraries.
The isolated NSIS compiler's data path was corrected and packaging completes.
Microsoft library debug-symbol warnings do not prevent release linking.

Measured checks: Windows MSVC x64 release compilation and NSIS packaging PASS;
packaged frontend and all 21 adapter/calendar tests plus route/settings/login/
receipt renders PASS. PE architecture, Credential Manager/System32 HTTPS imports,
registration/login support and exclusion of smoke commands PASS. NSIS archive
integrity and its exact eight expected files PASS. Extracted app bytes match the
compiled executable with Tauri's expected NSIS bundle-marker patch; embedded
WebView2 bytes match the downloaded Microsoft installer. Delivery-copy SHA-256 PASS.

This is an unsigned cross-built acceptance installer. Actual Windows installation,
online/offline login, Credential Manager, reconnect, data-preserving upgrades,
NFC and receipt printing remain unverified. The Windows Actions/native-runtime
and other release gates in PLAN.md remain open.

## Current update: hide the dashboard until the startup session check completes

The desktop shell previously rendered while native auth status was still null,
so its Dashboard heading/navigation could flash before the login screen.
Native startup now shows a full-screen session check using the login theme.
Only a known native access state can show the shell; accounts requiring login
go to the sign-in screen. A failed initial check offers retry and keeps the
shell/member data hidden. Browser demo sign-in state no longer initializes
native access. The explicit local-test build still works after its access
state is known. Native authentication and offline verification remain native.

Frontend and packaged desktop builds, all 21 adapter/calendar tests and route/settings/login/receipt
renders PASS. Startup regression checks cover null/missing auth status even
with a populated snapshot, an initial-check failure, locked/login-required
accounts and the verified-user shell. No browser storage is read during native
startup. Real Linux desktop forms/process-restart checks PASS. The Arch package
is rebuilt and passes SHA-256, archive-content and executable checks. Packaged
assets contain the startup gate and exclude demo auth/the removed notices.
Reinstall `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`
for this startup fix. Source remains local; the user handles commit/push.
No native business/schema change or live production login is involved.

## Current update: remove requested interface notices

Removed the Expenses and Payments implementation notices and the authenticated signed-in/access/
offline-session banner from the shared desktop shell. The normal profile,
login gate, native authorization, sync controls and error reporting remain.
The separate unauthenticated local-test build still identifies itself.

Frontend and packaged desktop builds and all 21 adapter/calendar tests plus route/settings/login/
receipt renders PASS. The existing signed-in render assertion now checks the
clean shell and retained sign-out control. Patch whitespace PASS. The rebuilt
Arch package passes SHA-256, archive contents and executable checks; the
packaged frontend excludes the requested messages. Reinstall it to update
the installed app. No native business or
authentication behavior changed; prior membership checks still apply.
Source remains local; the user handles commit/push.

## Current update: package registration and remaining membership days

Members now shows remaining membership days, including the final valid day,
with zero for expired memberships, a start-date label for scheduled packages,
and no fabricated countdown for unassigned or archived members. The dashboard's
expiring list also shows days left. Counts use the native Asia/Colombo business
date; the separate browser demo uses the same calendar rules.

New-member registration offers active packages with duration and price. It
defaults the start to today and recalculates the inclusive expiry when the
package or start changes. Calendar months use the day before the anniversary,
or the last day of the target month when that anniversary does not exist:
October 5 + one month ends November 4; January 31 ends February 28/29.
An explicit details-only option remains available. Existing recorded dates and
financial history are retained; invoices and payments remain separate actions.

Native registration validates the reviewed package version and derives the
expiry from its stored duration. Member/card history, membership, two audit/
outbox records and one frozen sync batch commit together. Stable operation IDs
make retries after a lost reply/restart safe. Inactive/stale packages, invalid
dates/cards and injected membership/audit/sync/receipt failures preserve all
existing data. No SQLite or cloud schema migration is required. Dropdowns and
native options use dark backgrounds, visible text and amber selection/focus.

Measured checks: **180 packaged native tests PASS / 2 environment probes ignored**;
**21 adapter/calendar tests PASS**, route/settings/login/receipt renders PASS,
frontend and packaged desktop builds PASS, native Clippy with warnings denied PASS, formatting and
patch whitespace PASS. Real Linux webview forms and process restart PASS,
including automatic month-end expiry, package reselection, readable dropdown
colors and remaining-day display. The updated native combined-registration
envelopes also PASS the server's full-state contract/hash check and local
all-module download test. No new live production sync or Windows acceptance
is claimed. The optimized Arch package is rebuilt; SHA-256, archive contents,
registration support in its executable and exclusion of smoke commands PASS.
Reinstall `Client/dist-linux/armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst`
to update an already installed app. Source remains local;
the user handles commit/push. Next release gates remain in PLAN.md.

## Current update: password and computer-enrollment diagnostics

Read-only live diagnosis reached `/v2/health` with HTTP 200 and protocol 2.
The approved Administrator is confirmed, unbanned and active in exactly one gym;
the API runtime has permission to call the enrollment function. This computer's
SQLite metadata has a prepared credential and no server binding/restore guard.
Production had no registered computers for the gym during this inspection.
Auth logs show both successful password/identity checks and subsequent
`invalid_credentials` password failures. These are separate from enrollment
refusals. Render's live automatic-enrollment flag was not inspected, and no
successful production desktop enrollment or synchronization is claimed.

Native sign-in now distinguishes password, online identity, computer enrollment
and renewal refusals. Recognized error codes select fixed actionable messages;
unknown/malformed/oversized provider content remains redacted. Authentication,
scope/device checks and credential preservation remain in force. Tests verify
each refusal stops at its failed step without local users/roles/audit or binding.

Measured checks: **172 packaged native tests PASS / 2 environment probes ignored**;
frontend and packaged desktop frontend builds PASS; Rust formatting and patch
whitespace checks PASS. The optimized Arch package is rebuilt; SHA-256, archive
contents and presence of the new messages in its executable PASS. Source changes
remain local, and the running installed app needs to be closed/reinstalled to
use these messages. No production registration, password, credential or
role was changed by this diagnosis. Use the gym account's Auth password for
sign-in; the computer/keyring unlock password serves local storage only.
See [sign-in diagnosis](docs/NATIVE_SIGN_IN.md#diagnose-a-refused-sign-in).

## Current milestone: all-module synchronization and real desktop checks

The requested final Windows release is still open. This milestone supersedes
the historical member-only status below. The existing single-writer rule remains:
one approved computer edits and additional computers download shared gym records.

Protocol 2 now freezes every business transaction in SQLite with its audit and
outbox IDs, then commits scoped cloud rows, references, ordered history and an
exact request-hash receipt together. It covers members/cards, plans/periods,
attendance, invoices/payments/allocations/receipts/reversals, products/sales/stock,
expenses/voids, profile and audit. Downloaded user rows are inactive historical
references; they cannot grant account access or roles. Logout/session nonce
fencing, bounded native HTTPS, durable retry/backoff and online renewal remain.
Conflicting transactions stay visible and block later uploads without overwriting
financial history. General conflict review remains required. Same-computer
restore recovery now verifies online Auth/possession and reconciles an isolated
copy before atomic replacement/unlock. Lost replies, logout, wrong scope,
read-only approval, expiry and storage races keep the restored database guarded;
both recovery copies remain. See [restore recovery](docs/RESTORE_RECOVERY.md).

SQLite migration 7 preserves local data and queues. CLI-generated PostgreSQL
migrations 3 and 4 are applied to the existing test project and production;
production's ledger checksums match source versions 1–4. Existing unrelated
member/change counts remain 1/3; the approved ArmStrong Fitness gym has no legacy
cloud members. Real restricted-runtime TLS/catalog checks PASS, including narrow
column grants, immutable guards and computer-enrollment permission. Public Data
API access, deletion and direct staff insertion remain denied. API startup now
verifies these capabilities before listening. Legacy member-only gyms require
explicit reconciliation instead of silently creating a second cloud history.

Final measured checks: **171 packaged native tests PASS / 2 default-ignored OS/TLS probes**;
**17 adapter tests PASS**, all route/settings/login/receipt renders PASS, including
the all-module conflict panel; frontend and desktop asset builds PASS; native
Clippy PASS with warnings denied. Backend strict typing/provenance and **77 unit
tests PASS / 1 sandbox process skip**. Ten real native SQLite envelopes pass the
server's full-state contract and canonical hash checks. The isolated integration
suite passes with **real Supabase Auth + PostgreSQL**, exercising exact retries,
read-only download, cross-gym/secret/revocation denial and rollback. Fixtures are
rolled back and never written to production. The final strengthened integration
rerun PASS includes refusing allocations after a saved full reversal.
The separate installed-curl TLS refusal probe PASS. The OS-vault read-only
probe correctly refuses an unavailable/locked Secret Service here; this does
not certify successful credential storage or desktop sign-in.

The real Linux webview form and process-restart checks PASS after correcting the
receipt test to compare immutable snapshot/number separately from current payment
status. The harness now requires explicit webview success on both launches and
cannot report a failed assertion as a pass. This covers local business forms;
production login/keyring/network and Windows hardware acceptance remain separate.

GitHub main advanced to `ef1faa0a729db3c74aa21c8c4dd5d6a498bbc9df`; earlier
onboarding/packaging changes and removal of tracked `server/.env` are committed
there. These all-module changes are local. Connector writes still return 403,
the CLI token is invalid and no Windows workflow run/installer exists. The user
will handle commit/push. The source patch is checked against that baseline, and
the Arch package has been rebuilt with the final recovery changes. Its checksum,
archive allowlist, launcher, all runtime libraries and exclusion of smoke-test
commands are verified. Source bundles have verified CRC and SHA-256 manifests.
Public Render health was independently reached;
no updated deployment is claimed. The updated API adds `/v2/health` to identify
all-module deployment while retaining the original `/health` compatibility body.

Still required: publish/deploy this source; build and accept the Windows installer;
verify native live HTTPS/OS-vault offline/reconnect/restore behavior; provide
general conflict review, legacy-gym migration and large-database bootstrap;
accept NFC/physical printing/upgrades; rotate historically exposed credentials.
Transactions are limited to 1 MiB / 2,000 changed rows; an oversized legacy
baseline is refused atomically. Backups preserve queues and recovery copies;
restored databases stay blocked until successful verified reconciliation. Manual package upgrades
and system-dialog printing remain the implemented options. See
[delivery](docs/DELIVERY.md) and [protocol](../server/docs/BUSINESS_SYNC.md).

## Current milestone: Administrator registered, Arch artifact and offline/member wiring

The latest request is to finish all features and build the final app. That goal
is **not complete**. Windows remains required; Arch is an optional native build.
The paragraphs below this milestone are historical and do not override this
current account/offline/sync state.

The owner confirmed `armstrong@gmail.com`. A real read-only Auth lookup verified
the existing confirmed, unbanned subject; a guarded owner transaction linked it
to **ArmStrong Fitness** as Administrator **ArmStrong**, preserving unrelated
registrations. No computer, credential, synthetic writer or local login was
fabricated. A CLI-generated private computer-enrollment function was applied to
the existing test project and production. Production's application ledger now
records version 2 with the exact source checksum. Runtime EXECUTE is granted,
public/anon/authenticated execution and direct staff/device DML remain denied.
Real PostgreSQL fixture transactions passed first writer, exact retry, second
read-only computer and secret/revocation/role/inactive/unknown-subject refusals;
those fixture writes were rolled back. SQL verification is not a live Auth/API
password sign-in or a newly deployed Render endpoint.

Packaged login now prepares its own persistent OS credential automatically.
Deploy current API source with `AUTOMATIC_DEVICE_ENROLLMENT=true` to enable
account-based computer approval. The flag defaults off; startup checks function
permission. The private prepared Render env has the flag, restricted runtime
connection and `/etc/secrets/hi3.crt`; it is not included in source/artifacts.
The first active computer edits; later computers are read-only. The optional
topology question remains unanswered, so the existing single-writer rule is
retained. Concurrent offline financial writers have not been approved/tested.

Verified online sign-in saves a separate seven-day scoped authorization item in
Windows Credential Manager or Linux Secret Service. Continue offline unlocks
existing records after restart using the OS account, existing credential proof,
scope/subject/active Administrator, expiry/clock and restore checks. The item
contains no bearer, refresh token or password. SQLite holds only its random
marker/expiry. Logout invalidates that marker before OS cleanup. Revocation is
unavailable offline; cached authority expires and is never extended offline.

The native HTTPS member worker now has narrow IPC, a dedicated SQLite connection,
bounded runs, 30-second scheduling, reconnect wakeup and persistent backoff.
Only actual matching server receipts acknowledge member operations. Native epoch
and durable nonce checks prevent queued/late work from committing after logout.
Open-app session renewal rotates credentials in native memory, re-verifies online
identity/enrollment and updates permissions before extending access. Refusals
lock access/remove offline approval; outages do not extend an unverified session.
An offline restart still needs online sign-in to resume sync. All other business
modules and their queues remain local; shared financial/stock/attendance/expense
data download and synchronization are still required for the final release.

The real optimized Arch x86_64 `.pkg.tar.zst` is built with embedded public
settings/frontend, no smoke features, a launcher/icon/README and SHA-256 delivery
manifest. Package archive allowlist and dynamic-library resolution passed.
The separate desktop Vite mode removes browser-demo login/session/storage code
from packaged assets. A build check rejects those markers; a missing/broken
native bridge shows a locked error screen without loading browser demo records.
The final artifact is under `Client/dist-linux/`; build with
`npm run desktop:arch:build`. This is an acceptance build, not the final release.
An isolated launch failed at GTK initialization before creating any app files;
this execution session cannot certify GUI/keyring/login behavior.

Measured final source checks: **161 packaged native tests PASS / 2 OS/TLS probes
ignored**, **17 adapter checks PASS**, all interface route/settings/login renders
PASS, frontend build PASS and native Clippy PASS with warnings denied. Backend
strict typing/provenance and **70 unit checks PASS / 1 sandbox child-process skip**
also passed. Unit/HTTP mocks do not prove live Render/native HTTPS acceptance;
Windows SDK layout tests do not prove real Credential Manager operation.

GitHub main is still `d7e659cccf1f9d19fea5fd4f3f4596645d9f43fa`. Connector writes
remain rejected with 403; no remote source update, Windows workflow run/installer,
PR or Render redeploy has been made. The refreshed desktop patch/source bundle
and Render source archive are prepared for a writable checkout. Next: deploy
this concrete source, enable computer approval, run the Windows workflow, verify
real sign-in/offline/reconnect and complete all-module protocols/download before
calling the build final. NFC/printer/upgrade acceptance and exposed-credential
rotation remain open. See [delivery steps](docs/DELIVERY.md).

## Windows installer/public configuration milestone; full offline onboarding open

User selected an installed app with Windows required and Linux optional, and
provided the repository `InshafRazmi/ArmStrong-Fitness`. Connected GitHub access
confirms its public visibility, default `main` and push permission. Inspected
the immutable base `45117fe1740fe97b763454b257c85112f8a2a930`; applicable AGENTS
and modified native/package sources match the workspace before this change.

The account's push permission does not establish the connector's write scope:
the attempted Git blob creation was rejected with **403 Resource not accessible
by integration**. No remote file, branch, PR or workflow run was created. During
inspection main advanced to `d7e659cccf1f9d19fea5fd4f3f4596645d9f43fa`; preserve
that backend change and its latest milestone documents in the prepared patch.
Windows changes will be delivered as a source patch/bundle for a writable
checkout. A build artifact still requires the Windows workflow to run there.

Prepared `armstrong-windows-changes.zip`: sixteen allowlisted changed sources,
binary-capable Git patch, guide and per-file SHA-256 manifest. Real `git apply
--check` and application against exact baseline files PASS; all resulting files
match staged sources. ZIP CRC/SHA-256 readback PASS. Private env data, old
credential values and the source upload ZIP itself are excluded. The env removal
is a separate `git rm --cached` so deleted passwords do not enter the patch.

Added `packaged-auth`: native builds include exactly the approved public Auth
origin/key and actual Render API origin. Fresh packaged installations require
login without a per-computer configuration file. Existing matching files remain
intact; invalid/conflicting files or bundled values keep access locked rather
than redirecting the gym or bypassing Auth. Compile guards reject production
Windows builds without packaged Auth and reject packaged builds with UI smoke
commands. SQLite/device identities, business data and pending operations are
unchanged. Public values are validated separately from any server env file.

Added an NSIS Windows x64 builder/config using per-user install and an embedded
WebView2 offline installer, a converted existing-brand ICO, pinned Node/Rust/CLI
and action versions, and a Windows Actions workflow that runs core/interface
checks and uploads only the setup executable. No release is published or API
deployed by that workflow. Windows build/installer acceptance and an actual
artifact remain pending until its first remote run. Linux has no Windows target
or Tauri CLI; the local installer command intentionally refuses that host.

Measured locally: **146 core tests PASS / 2 environment probes ignored**;
**14 packaged Auth checks PASS** including the actual bundled origins/login
lock; **16 adapter checks PASS** and interface renders PASS; frontend build PASS;
native packaged desktop `cargo check` PASS; public build preflight PASS.
Native Clippy with warnings denied and workflow YAML parsing also PASS. Default
and packaged suites use SQLite and mocked online enrollment, not real Windows
Credential Manager or live account/device acceptance.

Security inspection found populated database and Administrator probe passwords
plus a privileged Auth key in tracked `server/.env` on the public repository.
Values were withheld. The proposed change removes that tracked file and adds
root ignore rules while preserving the private local copy. Its Git history still
contains exposed credentials: reset the owner database password, Administrator
password/sessions and privileged Auth key through Supabase, then update private
administrative settings. No credentials were rotated by this packaging change;
the separate prepared runtime password was not in the tracked file.

Next: obtain/verify the Windows artifact; finish account-authorized computer
onboarding without server file edits, multi-computer data download, secure offline
session/restart/renewal and real sync scheduling/all-module protocols. The server
still requires existing device approval and one writer; native sessions still
expire/restart online. No guards were removed or full offline/sync release claimed.
Windows fresh-install/upgrade/NFC/printing and actual outage/reconnect acceptance
remain required. See [Windows delivery](docs/WINDOWS_INSTALLER.md).

## Access requirement clarified: any desktop, account login and offline work

The user wants to open the frontend on any desktop, sign in and use it, and
continue working when the network is down with synchronization after reconnect.
This supersedes treating manual per-computer env/configuration steps or a
single approved editing computer as the intended final user experience. An
access-mode question is pending: browser website, installed desktop app or both.
Do not assume an installed app solely from the word desktop.

Inspection confirms browser login is still hardcoded demo access, browser data
is seeded/localStorage-only and browser sync is simulated. Native data uses
transactional SQLite and an outbox, but real member scheduling is disabled,
sign-in depends on manual public settings and approved device registration,
and sessions expire/restart into online sign-in. The API exposes only member
enrollment/push/pull and currently permits one active writer per gym.

Required outcome: one real Administrator account accesses the same gym from
supported computers; onboarding does not require users to edit server files;
first sign-in/data download needs internet; local data and pending changes are
durable during outages; reconnect retries acknowledge actual server commits
without duplicate financial/attendance records and surface conflicting changes.
Offline unlock/restart/expiry must be designed explicitly. Concurrent offline
stock changes need a defined reconciliation policy. Browser delivery additionally
needs a production data/auth provider and offline application/data storage;
hosting the existing demo frontend would not provide the requested behavior.

No authentication, device/writer guard, schema, registration or live-sync behavior
was changed during this inspection. Delivery choice will determine the next
implementation; keep existing records and pending operations intact.

## Render health response confirmed by user; desktop setup next

The user opened `https://armstrong-fitness.onrender.com/health` and supplied the
exact expected body: `{"status":"ok","service":"armstrong-member-api","protocolVersion":1}`.
This is user-reported live process/protocol evidence. This environment could not
independently reach Render because DNS resolution failed, including for
`render.com`; no independently observed HTTP status or deployed revision is
claimed. Current source checks its first database connection before listening,
but health alone does not verify Administrator/device enrollment or member sync.

Saved the confirmed public API origin in ignored `server/.env`, preserving all
existing values and mode 0600. Gym/Administrator names, stable gym UUID, local
database/TLS configuration and Auth probe credentials already validate locally.
Actual desktop SQLite path/native credential hash and native public configuration
remain pending. No registration, business data or sync scheduling changed.

Next: use **Prepare this computer** in the actual desktop, configure its displayed
SQLite path and verified hash locally, review/apply Administrator/device
registration, provision desktop public settings, and verify native sign-in and
enrollment. Synthetic member-sync acceptance, other-module sync, restore/offline
policy and Windows/hardware/installer acceptance remain open.

## Active Render owner URL confirmed; corrected runtime URL supplied

The user supplied Render's actual DATABASE_URL. Its username is
`postgres.<project-ref>`, confirming the exact cause of the repeated production
role refusal. Its certificate setting uses `/etc/secrets/hi3.crt`. Credentials
from the supplied owner URL are deliberately absent from this status/source.

Updated only the certificate path in the private `.env.render` to match `hi3.crt`,
retaining the already-provisioned `armstrong_api` login/password, Session pooler,
project and verified TLS. Full production configuration validation PASS; the
private file is still 0600 and replacement was atomic. The complete corrected
runtime URL was supplied for the user's authorized Render configuration update.
The original administrative `.env`, remote grants/passwords and business rows
were not changed. The local CA upload copy retains its filename but can be added
to Render with secret-file name `hi3.crt`.

Runtime/Render guides and source-only archive are updated to the actual secret
filename. No new backend/frontend behavior or broad tests are needed for this
private setting/documentation correction. Next: replace the active service's
DATABASE_URL with the full runtime connection, confirm the `hi3.crt` secret file,
save/deploy and verify real login/TLS/startup/health. Restore normal `npm start`
if the temporary diagnostic command was used. Actual hosted health remains open.

## Repeated role refusal confirmed in Render logs; active env diagnosis pending

The user confirms the repeated production-role refusal comes from Render's
deployment log. The private prepared settings still pass production validation
with `armstrong_api`, correct project/TLS/CA path and the actual API origin; a
fresh catalog recheck confirms LOGIN enabled and restricted role attributes.
Render's active DATABASE_URL username is requested without its password.

Prepared a temporary inline Start Command that logs only predefined database
role labels (or fixed other/missing/invalid labels), then imports the existing
API entrypoint with all validation intact. It uses only Render's process env
and requires no source upload. Actual local execution with administrative
settings prints `postgres` and reproduces the exact guard error; the prepared
runtime settings print `armstrong_api`, pass that guard and stop on the remotely
mounted CA path missing locally. Malformed/encoded/private username fixtures
confirm credential-safe output. No database connection or health pass is claimed.

No env, credentials, SQL grants or server/frontend behavior changed. Next:
apply the diagnostic Start Command in the service for the actual supplied URL,
redeploy and inspect its new role line alongside the next startup error. Correct
the active service variable/override as needed; restore `npm start` after diagnosis.

## Restricted runtime login provisioned; Render environment update pending

The latest supplied startup error confirms the deployed database role is refused
by the production owner/reserved-role guard. A fresh connected Supabase catalog
query confirmed no `armstrong_api` role, PostgreSQL 17.11, owner-managed six
private RLS tables with no policies and no public business-data grants. Current
official roles/RLS/17 CREATE ROLE docs and changelog were consulted; the markdown
changelog was unsupported by the browser, so its HTML index was used.

Applied the password-free `armstrong_restricted_runtime_role` migration. It
creates `armstrong_api` initially NOLOGIN, NOSUPERUSER/NOCREATEDB/NOCREATEROLE,
NOINHERIT/NOREPLICATION, with BYPASSRLS for the existing private-API access model,
12 connections, 10-second statements and `pg_catalog` search path. Grants are
SELECT on six tables, UPDATE(change_sequence) on gyms, INSERT and updates only
to editable/archive member columns, and INSERT on operations/changes. There are
no inherited role memberships, owned relations, schema CREATE, Auth/Storage
usage or new Data API/public role grants. Existing owner/table/RLS data are intact.

**Real SQL permission verification PASS**: SELECT/row-lock and required zero-row
writes succeed as `armstrong_api`; 16 unauthorized zero-row writes fail, including
staff/device approval, gym creation/name changes, member scope/join-date changes,
history rewriting and deletions. Initial SET ROLE failed because the creator's
automatic ADMIN membership has SET false. The adaptive verification temporarily
enables SET within its transaction and rolls it back; the final catalog confirms
the original SET=false/INHERIT=false membership. No business row was modified.

Then enabled LOGIN with a newly generated 256-bit password through a separate
credential operation, with no password in source/migration SQL or displayed
output. The complete connection is in ignored `server/.env.render` (0600), with
only seven runtime settings and the actual supplied API origin. Its syntax passes
the production guard. Original administrative `.env` was preserved byte-for-byte.
Copied the same readable/unexpired CA to ignored `server/certs/prod-ca-2021.crt`;
the deploy URL points to `/etc/secrets/prod-ca-2021.crt`. First private preparation
needed its missing certificate directory; creation then succeeded without
overwriting an existing environment/certificate file.

Final catalog confirms LOGIN=true and restricted role attributes/grants; all six
RLS flags remain enabled, anon/authenticated schema access remains false and no
private policies were added. Node/pg restricted-login probe with the local CA
still fails `EAI_AGAIN` before TLS/authentication. This is **not** actual login,
deployed certificate or HTTPS/health evidence. User must update Render's current
DATABASE_URL and secret file through its dashboard; no connected Render tool is
available here. No new publication approval is needed for the selected setup.

Security advisors returned their existing observed October 4 findings: seven
no-policy INFOs (intentional private tables/ledger), three
[mutable trigger search paths](https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable),
public `rls_auto_enable()` definer execution warnings for
[anon](https://supabase.com/docs/guides/database/database-linter?lint=0028_anon_security_definer_function_executable)
and [authenticated](https://supabase.com/docs/guides/database/database-linter?lint=0029_authenticated_security_definer_function_executable),
and [Auth password protection](https://supabase.com/docs/guides/auth/password-security#password-strength-and-leaked-password-protection).
These are recorded as release-review work; no clean/fresh security audit is claimed.
No functions, public grants or Auth settings were changed during this role setup.

Updated runtime/Render documentation and refreshed the allowlisted, hash-verified
source zip with private settings/CA excluded. Existing strict/unit/frontend
evidence is retained; no server/frontend behavior changed in this milestone.
**Next:** copy prepared runtime DATABASE_URL, mount the prepared CA, save/deploy
and verify actual HTTPS/TLS/startup/health. Native registration/enrollment, member
scheduling and remaining release milestones still remain open.

## Render logs confirm build success; safe startup diagnostics fixed locally

Supplied October 4 Render logs show **66 PASS / 0 FAIL / 0 SKIP**, successful
build upload, then `npm start`, the expected optional-env notice, sanitized
`API startup failed (details_withheld)` and exit 1. This resolves the reported
build failure: the actual issue is process startup. The service's current env,
source commit, URL and root are not visible here; the exact remote cause is
still unconfirmed. Variable names only and the repository/branch are requested.

Found a local diagnostic defect: missing runtime credentials, missing production
API origin, invalid port and disabled TLS used generic errors, so startup hid
their fixed safe messages. They now use `ServerConfigurationError`, recognized
by the existing sanitized logger. Only application-authored text and variable
names are exposed; unknown errors, look-alikes and provider payloads stay private.
No runtime permission, CA/TLS validation or build gate was relaxed.

The supplied log lacks the Render-origin test and totals 66 tests; the previous
current source had 67. The older Koyeb bundle also lacks Render URL fallback.
This suggests older source but does not establish the deployed commit. Setting
the actual canonical HTTPS `PUBLIC_API_ORIGIN` explicitly works with both source
versions. Missing env is another possible startup cause; neither is claimed as
the confirmed cloud failure without the settings/source evidence.

Measured: strict typing/provenance and **68 PASS / 1 sandbox subprocess skip**
(**69 tests total**); frontend build **PASS**, 54 modules. A direct real
`src/main.ts` invocation with all three required variables unset exits 1 and
prints only `Configure DATABASE_URL; environment value is missing or still a
placeholder`, before network access. Checks use local Node 26.10.0; actual
Render Node version/HTTPS/runtime database remain unverified. Refreshed the
source-only Render archive with per-file hashes/readback and private exclusions;
the deploy must receive the updated repository commit to use this correction.
The Render guide/README now include startup troubleshooting and actual log evidence.

**Next:** verify runtime variable names and actual source/commit, set the real
HTTPS origin explicitly if deploying older code, complete restricted runtime
login/grants and mounted CA, deploy current source and verify HTTPS startup.
No populated env, SQL grants/records, Auth accounts or native configuration changed.
Native enrollment, member scheduling and remaining release milestones stay open.

## Reported Render build failure — local package passes; cloud error pending

The user reports `build:verify` failed on Render. The first actual error and
configured Root Directory have been requested. Render logs/source branch are
not available here: its integration is not confirmed connected, and an accessible
GitHub repository search for ArmStrong returned no matches. No cloud cause or
fix is claimed before that evidence arrives.

Fresh current-source `npm run build:verify` **PASS**, with strict/provenance
checks and **66 PASS / 1 sandbox subprocess skip**. The Render zip's hash manifest
matches both its extracted bytes and current source; it contains `build:verify`,
Node `24.x`, the local pg declarations and all nine vendored files.

A separate `/tmp/armstrong-render-repro-*` extraction also completes a clean
locked install with `NODE_ENV=production`, `--include=dev --ignore-scripts` and
offline copies of 77 already cached public locked tarballs (**78 packages
installed**). Its strict/unit build also **PASS**, 66 checks and one subprocess
skip, with synthetic Render metadata and no populated env or real SQL/Auth.
Both checks use local Node **26.10.0**, npm **12.2.0**; Node 24/actual Render
remains unverified. The install's Node-engine warning is expected for this local
version and is not the user's unknown cloud error. No verification gate,
provenance hash, compiler setting, runtime secret or active dependency changed.

**Next:** inspect the first Render error and actual build root/source/version.
If it specifically reports a missing `build:verify` script, the direct equivalent
is `npm run typecheck && npm test` after the existing locked install; this keeps
both checks and does not establish that source/settings match. Missing files,
compiler errors, failed tests and runtime startup errors require their actual
log and appropriate correction. Hosting/runtime-role/CA/native enrollment and
the remaining release gates are still open.

## Render selected — Free/Singapore preparation verified; deployment pending

The user switched from Koyeb to Render after discussing free hosting and is now
entering Render's build settings. Free in Singapore is the stated testing target;
the current selection supersedes the prior hosting decision gate for this setup.
No service or actual HTTPS origin has been created/verified by Codex.

Updated the selected [Render guide](../server/docs/RENDER_SETUP.md), Blueprint,
README and env example. `build:verify` is the provider-neutral strict/unit build
command; the Koyeb command remains a compatible alias. The Blueprint selects
Singapore/Free, disables automatic deploys and omits the initial explicit API
origin because runtime startup can now use Render's actual `RENDER_EXTERNAL_URL`
when `RENDER=true`. Canonical HTTPS/onrender.com/no non-default-port validation
applies to that fallback. Explicit origin/custom domain still takes precedence,
and local native setup still requires the actual copied `PUBLIC_API_ORIGIN`.
Owner-role refusal, verified database TLS and private settings remain enforced.

Measured: `npm run build:verify` **PASS** (full strict typing/provenance and
**66 PASS / 1 sandbox subprocess skip**, 67 checks total); frontend build **PASS**,
54 modules. Added origin tests reject missing/non-Render metadata, plaintext,
credentials, paths, queries/fragments, non-default ports and substituted domains,
and retain explicit-origin precedence and owner-login refusal. Checks ran on
local Node 26.10.0; real Render/Node 24/HTTPS/database/restart acceptance is pending.
A fresh source-only `armstrong-render-source.zip` includes the Blueprint and
current startup changes, with source hashes/zip readback and private-file
exclusions checked. The Koyeb zip is an older snapshot; use the Render bundle.

The Render integration was discovered and suggested but is not confirmed
installed/connected. No Render CLI/API token is configured here. Repository URL
has been requested; the local `.git` still has no repository metadata/remote.
The user can continue in Render's dashboard while connection/source access is
resolved. The last real catalog query still reports no `armstrong_api` login;
the existing owner connection cannot run production. Actual CA mounting,
restricted login/grants and deployment need completion. No populated env,
remote grants/records, Auth accounts or native device settings were changed.

**Next:** use the current source/build settings; configure restricted runtime
SQL access and the Render CA secret file; deploy and verify the actual HTTPS
origin; save the origin locally; finish native preparation, Administrator/device
registration and enrollment. Optional external `/health` monitoring is documented
as an inference from idle traffic rules, without uptime guarantees or a created
monitor. Scheduling and remaining sync/restore/Windows/hardware milestones stay
open. No paid compute or third-party monitoring service was created.

## Koyeb selected — deployment preparation verified; account/runtime access pending

The user chose Koyeb and explicitly requested its setup. This supersedes the
previous hosting decision gate for this Koyeb setup. The selected target is
Singapore Eco Micro, one fixed instance, with Supabase retained for PostgreSQL
and Auth. No Koyeb service has been created and no live origin exists yet.

Prepared [Koyeb setup](../server/docs/KOYEB_SETUP.md), Node `24.x` in the package
and lockfile, `build:koyeb` for strict typing/unit checks, and a production
`Procfile`. The guide supplies work directory, port/route/health settings,
environment interpolation of the actual public domain, restricted SQL and
runtime CA mounting. The env example now describes Koyeb; ignored source-upload
artifacts and SQLite/native config are excluded from Git. A source-only zip is
prepared for private GitHub upload because this workspace has no configured Git
repository/remote. No populated runtime/test env was changed.

Measured checks: `npm run build:koyeb` **PASS** (vendored declaration provenance,
full strict typing, **65 PASS / 1 subprocess test skipped**); frontend build
**PASS**, 54 modules. These ran on local Node 26.10.0; actual Node 24 buildpack,
hosted memory, HTTPS, database connection and restart remain unverified. The
source zip is checked against an allowlist and per-file hashes; env/CA/native
database/config/dependency directories are excluded. No container/buildpack or
public deployment pass is inferred from these checks.

A fresh real read-only Supabase catalog query confirms six private tables,
zero active Administrators and no `armstrong_api` login. Local runtime settings
still use the administrative owner, which production startup refuses. The local
CA is readable, but it must be mounted separately on Koyeb. There is no installed
Koyeb CLI, connected Koyeb tool or configured token. The user has been asked for
Koyeb account/GitHub repository readiness; passwords/tokens should stay private.
No SQL grants/records, Auth accounts or device registrations were changed.

**Next:** account/source access; provision and verify the restricted runtime
login; configure the Koyeb certificate/env/service and deploy; verify the actual
HTTPS origin and save it locally; complete native preparation, Administrator
registration and real enrollment. Scheduling remains disabled; other-module
sync, restore reconciliation and Windows/NFC/printing/installer acceptance remain
open. API health does not prove ongoing database availability or enrollment.

## Installation details supplied; Render recommendation prepared

The user supplied gym name **ArmStrong Fitness** and Administrator display name
**ArmStrong**, and asked whether Render should host the API. These names and one
stable new gym UUID were appended privately to ignored `server/.env`, preserving
every other existing setting. No active gym UUID was replaced; the existing
fixture-named server gyms remain untouched. This prepares local registration
configuration only and does not insert staff/gym/device records or grant access.

Fresh `setup:check`: approved gym/Administrator configuration now **PASS**;
database URL/CA and probe credential presence still validate locally. API origin
and actual prepared SQLite path/hash remain **PENDING**. Full local readiness
is still **INCOMPLETE**. Only env values and deployment documentation changed;
the previous 65-check/type/build evidence was not rerun unnecessarily.

Current official Render docs were checked for web-service setup, managed TLS,
free cold starts, pricing, Node version and secret-file mounting. Render is
suitable for the API; paid compute is recommended for normal daily operation
because the documented free cold start can exceed native request deadlines.
The existing free Blueprint is now labelled as a testing template; no paid plan,
region, provider service or public deployment was selected/created.
See [Render preparation](../server/docs/RENDER_SETUP.md).

**Next:** restricted API runtime login/grants and deployed CA preparation;
actual OS device preparation and Administrator registration; authorized hosting
decision/creation, actual API origin, native configuration and real enrollment.
The prior no-public-deployment instruction persists: asking whether to use Render
is a recommendation request, not publication authorization. Live sync remains
disabled and the remaining project milestones remain open.

## API/desktop setup continuation — tooling complete; installation settings pending

Added `server` commands `setup:check`, `api:check`, `desktop:config:check` and
`desktop:config:write`. Readiness reports all local gaps without network writes
or secret output. Public config provisioning validates existing SQLite metadata
read-only: device UUID, native preparation hash, saved scope and restore marker.
It writes only Auth/API origins and public key beside the resolved existing
database. Complete flushed bytes publish atomically without replacing a file;
matching retries preserve bytes. Invalid/different/oversized/duplicate-field/
symlink/directory targets, missing databases and unreconciled restores fail.
No credential generation, new database, migration or privilege grant occurs.

The credential-free HTTPS API probe requires canonical origin, verified TLS,
no redirects, a ten-second deadline and a bounded 4 KiB response. `/health` now
returns fixed Armstrong service/protocol metadata as well as `status: ok`.
This remains process/protocol evidence, not DB/Auth/device/sync acceptance.
Shared public Auth validation now applies to API configuration, password-account
probes and desktop provisioning. Production API startup refuses owner/reserved
database roles; administrative registration/probes retain controlled owner use.
Custom role attributes/grants still require real verification.

Measured checks:

- Full strict server typing: **PASS**. Final measured local suite: **65 PASS,
  1 subprocess check skipped** (66 total). The sandbox returns `EPERM` for Node
  child execution; it is not counted as a CLI or live-service pass.
- Commands executed directly against an isolated `/tmp` synthetic SQLite/env
  fixture: review `missing`, write `created`, matching retry `existing`, and all
  six local readiness stages **PASS**. Database SHA-256 remained identical,
  exactly three public fields were written and no temporary file remained.
  This is genuine local CLI/filesystem verification, not enrollment/OS-vault proof.
  A direct production `src/main.ts` run with the fixture's owner connection
  exits 1 with the controlled restricted-login message before opening a pool.
- Frontend build: **PASS**, 54 modules. Native/SQLite schema was not changed;
  previous native/core/GUI evidence remains separate and was not rerun.
- Actual runtime `setup:check`: **INCOMPLETE**. Database URL/CA and probe
  credential presence validate locally; API origin and all registration settings
  remain absent. `api:check` and `desktop:config:check` stop before network/writes
  at the missing origin. No populated runtime/test env changed.
- **Real read-only SQL through the connected Supabase app: PASS.** In the
  project matched from runtime Auth configuration, gyms named `Test gym` and
  `Unrelated gym` already exist; there are zero active Administrators, two active
  devices/writers (one per gym), and one member. All six private tables have RLS
  enabled; `anon`/`authenticated` lack schema usage. The `armstrong_api` login
  is absent and local runtime settings currently use the administrative owner.
  No fixture was reset, no account/role/device was registered, and no SQL record,
  grant or schema changed. Test Auth targets a different project. These connector
  queries do not verify native HTTPS, the Node pooler/TLS connection or enrollment.

Awaiting the existing approved API HTTPS URL and installation gym/Administrator
display names. Optional offline restart policy remains pending; restart currently
requires online sign-in. Actual native device preparation/credential possession,
Administrator registration, restricted runtime login/grants and enrollment remain
required. Member scheduler/live acceptance, other-module sync, restore
reconciliation and Windows/NFC/printing/installer acceptance remain unfinished.
No public deployment occurred and live sync stays disabled.
See [setup guide](../server/docs/LOCAL_SETUP.md).

## Backend typing milestone — completed with pinned upstream declarations

Full strict server NodeNext typing now passes. The registry install of
`@types/pg` still failed with `EAI_AGAIN`, so genuine DefinitelyTyped pg 8.23
sources were retrieved through the read-only GitHub connection and pinned to
commit `97ba786e647c0899a2dd6d1b6807896762e725fa`. Four declaration files and the
upstream package metadata are unchanged and verified against their Git blob
hashes. The MIT license, owner attribution, source URLs and SHA-256 checksums
are retained in `server/vendor/types-pg`. The source version `8.23.9999` is a
repository snapshot, not a claimed published npm version.

A local file dev dependency and npm-generated lockfile make the declarations
installable offline. Packaging preserves upstream CommonJS/ESM exports and
uses the existing Node/pg-protocol/pg-types dependencies. `types:verify` checks
source/license hashes and packaging before the normal typecheck. No ambient
`any` shim, `skipLibCheck`, runtime driver replacement or relaxed strictness was
introduced. Actual declaration checking exposed generic-Duplex TLS accesses and
an undeclared internal driver property. The probes now narrow an actual Node
`TLSSocket` before reading verified status, and the construction-only driver
fixture validates its actual `ConnectionParameters` instance without assertions
that change upstream declarations. Certificate/hostname/CA requirements remain.

Measured checks:

- `cd server && npm run typecheck`: **PASS** for source, scripts and tests.
- `npm test`: **48/48 PASS**. The added socket-classification check rejects
  plaintext/look-alike/unauthorized sockets; its positive flag fixture is
  synthetic and is not TLS-handshake evidence.
- A separate `/tmp/armstrong-types-ci-*` source checkout, with no env files,
  completes clean locked `npm ci --offline --include=dev --ignore-scripts
  --no-audit --no-fund`, full typecheck and **48/48** tests: **PASS**. Its first
  install reported uncached xtend; seeding only the 77 public package tarballs
  already cached for this lockfile allowed the clean retry. No other-project
  files, private registry content or active node_modules were replaced.
- `cd Client && npm run build`: **PASS**, 54 modules. Native/SQLite code was not
  changed; the preceding 142-core/16-UI/native-build evidence remains historical.
- `npm run db:check`: **FAIL (EAI_AGAIN)** before TLS/SQL, with redacted output.
  No real certificate/session/query success is claimed for this run.

Checks ran with Node **26.10.0** and pinned Node 24 declarations. Actual Node 24
hosting runtime and security audit are still unverified. Supabase changelog,
Session-pooler docs and Node TLS docs were checked; no Supabase API, schema,
grant, environment, registration or public deployment was changed. The existing
runtime API origin and all six relevant registration settings remain missing
(only presence flags were inspected; no values printed). No account credentials,
device secret or live device credential were created or disclosed.
See [declaration provenance](../server/vendor/types-pg/README.md).

**Next:** approved API/native configuration, actual OS device preparation and
Administrator registration/enrollment, real synthetic member sync acceptance and
scheduler, other-module sync/restore reconciliation, offline restart policy and
Windows desktop/reader/printing/installer acceptance. Current restart still
requires online sign-in; the optional first-release policy question is pending.
Full production completion remains open, and live sync remains disabled.


## Previous member conflict review milestone — implemented; live sync still gated

Desktop Settings now offers native Administrator review of member conflicts,
showing the current local record and recorded server version. Use-server applies
that reviewed member and discards covered rejected/unsent edits; keep-local
retains active member details and creates one fresh retry against the recorded
server revision. Neither choice claims a server acknowledgement. A reason,
confirmation, current native writer permission and unchanged review fingerprint
are required. No IPC accepts server records, actor IDs or permission grants.

SQLite **schema 6**, migration `006_member_conflicts.sql`, adds append-only
resolution records and links to the covered conflicts/operations. Old outbox
payloads, frozen requests, delivery states, conflicts, audit and financial
history remain. Pending/order/pull checks exclude explicitly retired operations;
acknowledgement counts stay separate. A late receipt cannot confirm a retired
operation. Member replacement/retry, resolution links, actor audit and local
idempotency receipt commit atomically. Card changes retain assignment history;
server archive actors import no privileges.

Requests with an unconfirmed outcome, incomplete/inconsistent server snapshots,
joined-date differences, card/identity collisions, hard deletions, device/scope
mismatches and archive reactivation remain blocked as applicable. Keep-local is
limited to active local/server members; archived intent needs server review.
Restore retains the ledger and still blocks sync. Old backups migrate in
isolation. The first verification run caught and fixed runtime review permission
entering restore storage fingerprints; existing recovery tests now pass.
See [MEMBER_CONFLICT_REVIEW.md](docs/MEMBER_CONFLICT_REVIEW.md).

Measured checks for this milestone:

- Core: **142 PASS, 2 OS/network probes explicitly ignored** (144 total).
  Thirteen added checks cover rejected and unsent edits, frozen request retention,
  stale review/role/writer/expiry/scope/restore denial, NFC/archive safeguards,
  original payment/receipt/membership history, immutable links, rollback,
  schema-5 upgrade/backup restore, and a real-SQLite/mock-worker fresh retry.
  The mock worker sends only the new operation, acknowledges its matching
  receipt, advances the pull cursor and never resends retired requests.
- **16/16 UI adapter checks**, all desktop routes/settings, conflict comparison,
  Administrator/blocked-review renders and separate browser demo: **PASS**.
- Frontend build, Rust fmt and desktop/custom-protocol/all-target Clippy with
  `-D warnings`: **PASS**. Normal desktop compilation with desktop/custom-protocol,
  offline locked dependencies and two jobs: **PASS**. No smoke hooks enabled.
- Backend was not changed in this milestone; its previous 47 local/mock passes
  and missing `@types/pg` limitation remain separate evidence.

No live settings, device credentials, enrollment, remote schema or public
hosting changed. Production member scheduling remains disabled pending real
API/HTTPS/device acceptance. GTK/OS/network restrictions and Windows reader,
printer and installer acceptance remain open; no new GUI/live acceptance claimed.

**Next milestones:** finish backend strict typing with `@types/pg`; provision the
approved API/native configuration and verify
actual device preparation/Administrator enrollment; complete real synthetic
member sync acceptance and scheduler; extend sync to the remaining business
modules and implement restore reconciliation; settle offline restart access;
complete Windows desktop/NFC/printing/installer checks. The entire production
project is still unfinished.


## Previous native login/device milestone — wired; real OS/API/GUI acceptance pending

Implemented actual native device preparation, OS credential storage, bounded
HTTPS transport, login/logout/status IPC and the desktop login screen. This
supersedes the earlier source-only native protocol milestone below. Windows
uses Credential Manager for the current user; Linux development invokes the
installed Secret Service utility, explicitly selecting its persistent service
backend and refusing the automatic Flatpak file backend. Secrets never go in
SQLite, backup bytes, webview responses, command arguments, env or temp files.
Only the existing SQLite device UUID/path and verified SHA-256 hash are returned
for administrative registration. OS randomness is the pinned cached getrandom
0.4.3 crate; Cargo generated the direct-dependency lock change offline.

Retries preserve the device/secret/hash; missing/replaced/corrupt credentials,
unavailable storage and restored databases fail without rotating credentials.
SQLite serializes preparation across app instances; readback precedes the hash
commit. A failed local hash save can reuse the already persisted OS credential.
The server registration CLI now requires that actual native preparation hash.
It still reads the database only, grants no client authority and performs no
implicit replacement. No real device credential or server registration was
created by Codex in this session.

Native settings load from bounded, provisioned `desktop-auth.json` beside the
database, with canonical distinct HTTPS origins/public Auth key and no caller
roles/IDs. Windows networking invokes the OS System32 curl.exe; Linux invokes
/usr/bin/curl. No shell or user PATH lookup, curlrc, inherited curl configuration,
redirects, proxy credentials, insecure TLS flags or uncontrolled output. TLS
uses system trust and TLS >=1.2; native subprocess I/O and deadlines are bounded.
Actual certificate/hostname/TLS acceptance remains unverified in this sandbox.

Configured or previously bound databases require a verified native Administrator
for every business IPC, including reads/exports, with additional writer checks
for writes. A durable auth requirement prevents deleting configuration to return
to test mode. Existing unconfigured test databases retain their explicit warning.
Role/active/expiry are checked in Rust, independent of browser/demo state. Logout
and later login invalidate pending replies. Restart and expiry require online
sign-in; no automatic refresh, offline restart grant or PIN policy was invented.
Only the device secret persists in OS storage; account tokens stay in native
process memory. Local logout does not revoke other Supabase sessions.

New financial records, receipt actor labels, audit actor FKs and outbox provenance
use the verified actor through a private connection-local context, preserving
historical rows and the original local-test actor. Authenticated restore preserves
the auth requirement and restoration actor, refuses a backup lacking that actor
mapping without replacing records, and locks the session after successful
replacement. Existing recovery copies/cursor/scope/reconciliation safeguards
remain. Schema version and migrations are unchanged. Member API transport is
retained privately after enrollment, but no production sync scheduler/command
is enabled; `memberSync.available` remains false.

Checks measured during this milestone:

- Full core suite: **129 PASS, 2 explicitly ignored OS/network probes** (131 total).
  Real temporary SQLite and mocked Auth/vault tests cover scope/role/session,
  restart/expiry, cancellation, read-only denial, actual backup bytes,
  payment/receipt/audit attribution and safe restore. These are not live Auth,
  TLS, Windows or OS credential persistence acceptance.
- **15/15 UI adapter tests**, all nine routes/seven settings, native login/config
  gate/user shell and separate demo render checks, plus frontend build: **PASS**.
- Core Clippy with `-D warnings` and native normal desktop compilation: **PASS**.
  Final normal rebuild after the smoke retry using `CARGO_INCREMENTAL=0 cargo
  build --manifest-path src-tauri/Cargo.toml --features desktop,custom-protocol
  --offline --locked --jobs 2`: **PASS**. Current binary has no smoke hooks.
  Rust fmt check **PASS**. After pinning the Linux Secret Service backend,
  focused credential checks **12 PASS, 1 OS probe ignored**.
  Final Clippy covering desktop/custom-protocol and all targets with warnings
  denied **PASS**. Read-only OS-store probe rerun after backend selection still
  observes unavailable/locked storage with a redacted error.
- Backend **47/47 local/mock/temporary-SQLite tests PASS**. Focused strict typing
  of the changed registration helper/tests **PASS**. Full server typecheck still
  **FAILS** with the same eight `TS7016` diagnostics for missing `@types/pg`.
- Real installed OS-store read-only probe: ran without creating a credential;
  observed unavailable/locked storage and verified a redacted failure.
- Real installed-curl TLS rejection probe: **FAIL to start**, loopback listener
  returns `Operation not permitted`. It is now explicitly ignored in the normal
  suite and documented for a normal terminal; no TLS acceptance PASS claimed.
- Retried `CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 npm run
  test:desktop`: frontend and native smoke compilation **PASS**, execution
  **FAIL** at GTK initialization before any UI/restart check. No OS packages changed.

Fresh registry access still fails `EAI_AGAIN`; runtime API origin and all approved
registration settings remain absent. No populated env, account credentials,
live schema, real enrollment, public deployment or synchronization was changed.
Setup instructions: [NATIVE_SIGN_IN.md](docs/NATIVE_SIGN_IN.md), including the
placeholder-only config, native preparation and administrative review/apply steps.

**Next:** privately provision the existing HTTPS API/public Auth settings, prepare
the actual computer in a working credential/GUI session, review/apply approved
gym/Admin/device registration and run real synthetic Auth/API/desktop acceptance.
Resolve pg typing; then connect/accept the sync scheduler. Conflict/restore
reconciliation UI, other-module sync, offline restart policy and Windows
reader/printer/installer acceptance remain unfinished. No full release completion
is claimed. The earlier no-public-deployment instruction remains in force.

## Date-filtered reports milestone — checks/native builds pass; GUI blocked

Added native inclusive date ranges and exact minor-unit report summaries, exposed
through narrow Tauri commands and the existing Reports page. CSV and totals use
the same filtering rules: saved Colombo business dates for attendance/cash,
Colombo-converted UTC audit timestamps, overlapping membership periods, and
explicitly current inventory. Reversals count on their own posting dates; voided
expenses retain original CSV amounts/history and contribute zero effective
expense. Invalid ranges and unsafe aggregate values fail without rounding or
changing records. UI waits for native totals and discards stale responses.

Fresh verification: **103/103 Rust/SQLite tests**, **13/13 adapter tests**,
all route/settings/demo/receipt/removal render checks, frontend build, Rust fmt,
core Clippy with warnings denied and smoke-script syntax **PASS**. The native
Auth/SQLite mocks now include read-only upload denial; these are not live Auth
or HTTPS evidence. Backend **46/46 local/mock tests PASS**; strict server typing
still fails the same eight missing-pg-declaration errors. Runtime config presence
check confirms API origin and all REGISTRATION settings remain missing; no
values were printed or modified, and no live registration was attempted.

`CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 npm run test:desktop` built the frontend
but failed while compiling the native smoke binary: **No space left on device**.
No window or interactive/restart acceptance ran. The full disk then prevented
all sandbox command launches (`bwrap` could not create its workspace .git mount).
Direct cache removal was automatically rejected; the first safe Cargo cleanup
also failed before execution. Command access then recovered. Successfully ran
`cargo clean --manifest-path src-tauri/Cargo.toml -p armstrong-desktop --offline`,
which removed 4310 generated files / 2.6 GiB. No application database/backup was
removed. The normal-terminal cleanup request is superseded.

Retried `CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 npm run
test:desktop`: frontend and native smoke binary **compile PASS**, but launch
**FAIL**, exit 101 at `Failed to initialize GTK` / `Failed to initialize gtk
backend!`, before any window/harness/restart check. No Xvfb utility is available
here; no OS package change was made. Rebuilt the normal binary without smoke
hooks using `CARGO_INCREMENTAL=0 cargo build --manifest-path
src-tauri/Cargo.toml --features desktop,custom-protocol --offline --locked
--jobs 2`: **PASS**. Current binary is the normal desktop build. These Linux
compilations do not prove Windows, hardware or interactive acceptance.

**Next:** run the expanded desktop smoke in a usable GUI session and rebuild the
normal desktop afterward. Resolve pg typing, approved registration and HTTPS
endpoint; finish production native HTTPS/credential storage/login UI and actual
synthetic sync acceptance. Windows, reader/printer checks remain open; live sync
is still disabled. Full project completion is not established.


## Native protocol/login continuation — implemented internally, 2026-10-04

Completed the previously unfinished `member_http.rs` adapter and its missing
test file. Eight HTTP-mock checks pass, including real temporary SQLite receipt
acknowledgement through the existing push-before-pull engine. The unchanged
77 core tests also passed at that point (85 total). Added private `native_auth.rs`
for password Auth, separate online subject verification and authoritative
device enrollment; eleven focused HTTP-mock/SQLite checks now pass. This is
internal tested logic, not a connected desktop login or production TLS client.

Local identity/role reconciliation, immutable API/gym/device binding and sign-in
audit save in one transaction. Existing actor IDs are retained; collisions,
restored/mismatched devices, expiry and failed saves remain locked. Server role
demotion replaces stale local roles; read-only enrolled devices cannot remove
members or void expenses. No IPC accepts caller identity/roles. Tokens/passwords/
device secrets are absent from SQLite, snapshots and actual backup bytes. No
schema or migration changed. The cached `base64` 0.22.1 crate is now a pinned
direct dependency for rejecting privileged legacy Auth keys; Cargo generated the
lockfile offline. A decoded key is never used to authenticate a user.

Fresh frontend build and 46 backend local/mock tests pass. Full backend typing
still fails with the same eight missing-pg-declaration diagnostics. Fresh npm ping
fails `EAI_AGAIN`; curl cannot resolve supabase.com. Production Rust HTTPS/keyring
dependencies are not cached. No installation from network, live Auth/SQL request,
registration apply, credential generation, database binding, deployment or sync
enabling occurred. The existing HTTPS API address and user-terminal pg type
installation are requested while independent work continues. Docs were verified
through the Supabase docs tool and official Auth OpenAPI; changelog markdown
fetch failed, so its HTML page was reviewed.

This internal milestone preceded the completed reports/final checks above.
**Next:**
Complete audited HTTPS transport/OS credential storage and native commands/UI,
approved gym/Admin/device registration and synthetic real API/desktop acceptance.
Windows, reader and printer acceptance remain release gates; live sync is disabled.

Auth protocol references: [official Auth REST OpenAPI](https://github.com/supabase/auth/blob/master/openapi.yaml),
[getUser](https://supabase.com/docs/reference/javascript/auth-getuser) and
[user sessions](https://supabase.com/docs/guides/auth/sessions). The native
exchange must still enforce verified TLS, no redirects, deadlines and streaming
response limits; the injected mocks prove none of those production conditions.


## Current focused milestone — backend typing repaired; administrative registration prepared, 2026-10-04

Continued the agreed backend-readiness and single-Administrator registration
work. Fixed the `parseEnv` result type, Fastify unknown-error narrowing, HTTP-mock
header access, typed JSON injection in enrollment/live fixtures, integration
callback types and live Auth response-object validation. Existing bearer checks,
member protocol and redacted failures remain intact. The original 79 full-server
diagnostics reduced to seven missing-pg-declaration errors; the new registration
CLI adds one instance of that same missing declaration, for **eight remaining**.
No `any` module shim, typing suppression or relaxed compiler option was added.

Attempted once from `server/`:

```sh
npm install --save-dev --save-exact @types/pg@8.16.0 --ignore-scripts --no-audit --no-fund --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

**FAIL**, exit 1; stopped dependency installation. Full output:

```text
npm error code EAI_AGAIN
npm error syscall getaddrinfo
npm error errno EAI_AGAIN
npm error request to https://registry.npmjs.org/@types%2fpg failed, reason: getaddrinfo EAI_AGAIN registry.npmjs.org
npm error A complete log of this run can be found in: /tmp/armstrong-npm-cache/_logs/2026-10-04T01_03_39_405Z-debug-0.log
```

No dependency/driver/lockfile change occurred. Keep the existing Fastify/pg and
user-added `@supabase/server` dependency. Install compatible 8.x pg declarations
with `--save-exact` in the working terminal and rerun full typecheck; missing
declarations can conceal further driver-typing issues, so a future full PASS
must be measured. Render preparation now installs dev dependencies during build
and runs full typecheck before unit tests; no deployment was attempted.

Added `src/registration.ts`, `src/registration-config.ts` and
`scripts/register-administrator.ts` with these runtime-only commands:

```sh
npm run registration:check
npm run registration:apply
```

Check is read-only. Apply is a separate administrative command that rechecks
approved settings and inserts missing registrations transactionally under
advisory/gym locks. Matching retries preserve records; conflicts fail without
updates, deletion, promotion, reactivation or secret rotation. The identity comes
from real password sign-in/online verification and must be confirmed in the same
SQL project's Auth records. Only the owner SQL connection may provision; API
runtime grants are not widened. No client gym ID, supplied user UUID or role
claim grants access. No new account or member data is created by this tooling.

Three optional local settings are required for gym/Admin registration:
`REGISTRATION_GYM_ID`, `REGISTRATION_GYM_NAME`, `REGISTRATION_ADMIN_NAME`.
These use the one existing Administrator account and its private Auth probe
credentials. Device approval is optional and separately needs
`REGISTRATION_SQLITE_PATH` plus `REGISTRATION_DEVICE_SECRET_SHA256` from the
actual native credential. The tool reads only metadata from that existing
SQLite file in read-only mode, retains its device UUID, refuses restored/mismatched
scope and writes only the hash server-side. Native credential generation/storage
is still pending; omit device entries until it exists. It does not bind a desktop,
verify secret possession or enable sync. Placeholder-only `.env.example`, README,
STAFF_DEVICE_SETUP and SUPABASE_TESTING document the commands and boundaries.
No populated `.env`/`.env.test`, credential or certificate was changed/exposed.

Added eight explicitly SQL-mock registration tests and five local/temporary-
SQLite configuration checks. Extended the isolated real SQL/Auth suite to test
registration review/retry and denied role/gym/hash replacement using its own
synthetic fixtures. That new live step was **not run**; the existing test-schema
guard and separation from runtime `.env` remain unchanged.

Fresh checks:

- `cd server && npm test`: **PASS**, 46/46 local/driver/SQL/Auth-mock and temporary-SQLite tests; no live service connection.
- Focused strict tsc of registration helpers/tests plus app/enrollment/Auth fixture changes: **PASS**, using `--noEmit --strict --target ES2023 --lib ES2023 --module NodeNext --moduleResolution NodeNext --types node --allowImportingTsExtensions --erasableSyntaxOnly --verbatimModuleSyntax`.
- `cd server && npm run typecheck`: **FAIL**, exit 2, exactly eight `TS7016` missing-pg-declaration errors; no other diagnostics. This is not a full typing PASS.
- `node --check scripts/register-administrator.ts` and `node --check test/supabase.integration.ts`: **PASS**.
- `cd Client && npm run build`: **PASS**, 53 modules.
- `cd Client && cargo test --manifest-path src-tauri/Cargo.toml --offline --locked`: **PASS**, 77/77 real local SQLite/core tests, including 14 explicitly labelled mock-server sync-engine cases. These do not prove live backend sync.
- `cd server && npm run registration:check`: **FAIL local setup**, exit 2: `Configure REGISTRATION_GYM_ID, REGISTRATION_GYM_NAME, REGISTRATION_ADMIN_NAME locally; values withheld`. Stopped before Auth/SQL; no live registration review occurred.

No registration apply, migrations, live integration suite, API startup or public
deployment ran. No native/Rust/client behavior, schema/checksum, actor FK, stable
ID, retry/push-before-pull/cursor/restore safeguard or availability flag changed.
The user's earlier live SQL/TLS/schema and Auth PASS remain separate evidence.

**Next:** privately configure the three approved registration settings, run
read-only review in the working terminal, then apply and confirm matching records
through another read-only check. Resolve missing pg declarations and review
restricted runtime grants. Complete native HTTPS/login/enrollment/secure storage,
the verified HTTPS API endpoint and real synthetic member-sync acceptance before
enabling sync. Do not fabricate device credentials or redirect the isolated
integration suite to the runtime project. Live sync remains disabled.

## Current enable request — blocked by missing desktop transport/enrollment and HTTPS API, 2026-10-04

The user requests enabling live sync. Inspected the current implementation and
runtime configuration using only presence/validation booleans; no populated env
values, credentials, URLs, keys, tokens or IDs were printed or changed.
The existing runtime database/Auth settings are populated, `sslmode=verify-full`
is configured and the CA file exists. `PUBLIC_API_ORIGIN` is not configured.
A clarification about an existing HTTPS Fastify endpoint is pending; this request
does not establish one or supersede the earlier prohibition on public deployment.

The native engine remains internal: `member_worker.rs` has no production
`MemberTransport`, native sign-in/enrollment, OS credential storage or registered
sync IPC. `member_sync.rs` reports `available: false`; the desktop provider's
`syncNow` retains pending operations and the Settings action is disabled.
The worker checks the persisted device UUID, verified session subject and bound
HTTPS server/gym/device scope. Those checks must remain intact. A Supabase
database or Auth origin cannot substitute for the Fastify member API endpoint.
The user-terminal real Auth and runtime schema PASS remain valid separate
evidence; gym/Administrator/device registration and end-to-end sync are unverified.

Fresh Codex commands from `server/`:

```sh
npm ping --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

**FAIL**, exit 1:

```text
npm notice PING https://registry.npmjs.org/
npm error code EAI_AGAIN
npm error syscall getaddrinfo
npm error errno EAI_AGAIN
npm error request to https://registry.npmjs.org/-/ping failed, reason: getaddrinfo EAI_AGAIN registry.npmjs.org
```

```sh
npm run db:check
```

**FAIL**, exit 2:

```text
Runtime database probe failed (EAI_AGAIN); connection values and driver details withheld
TLS certificate and hostname verification: not established
Verified database connection established: no
```

This database command used runtime `.env`/`DATABASE_URL` only and failed before
SQL; no data was accessed or changed. No dependency installation, migration,
integration suite, database provisioning or deployment was attempted. No source,
manifest/lockfile, environment, certificate, SQLite safeguard or sync enable flag
changed. Only STATUS, PLAN and the relationship map were updated. No local/mock
tests were rerun because application behavior is unchanged; previous passes and
the failing full server typecheck retain their historical status.

**Next:** obtain the existing verified HTTPS Fastify endpoint (or resolve the
deployment prerequisite), complete approved single-Administrator/gym/device
registration and native login/enrollment/secure credential storage/HTTPS transport,
then run real synthetic-data member-sync acceptance. Enable only after those
checks pass. Live sync remains disabled; no successful live sync is claimed.

## Current identity result — real Supabase password login and online verification pass, 2026-10-04

The user supplied the normal-terminal `cd server && npm run auth:check` result:
**PASS** for real Supabase password sign-in and separate online identity
verification, with values withheld. The reported command also explicitly says
gym/role/device registration and desktop login were **not checked**, and no
application database changes were made. This is user-terminal live Auth evidence,
not a Codex-executed request, a local configuration check or an HTTP mock.
It supersedes the initial missing-probe-credentials failure recorded below;
it does not establish that Codex network access has recovered.

The preceding user-terminal PostgreSQL/TLS/migration metadata PASS and this Auth
PASS verify different parts of the backend. Neither proves approved gym access,
Administrator permissions, device enrollment, member API acceptance or desktop
sync. A verified account becomes the application's Administrator only through
its approved server-side `armstrong.staff` record; no second login account is
needed. The API must continue deriving gym access from authenticated identity
and approved device records rather than client-supplied gym IDs or role claims.

**Next:** prepare the registration step for one stable gym, this same verified
Auth identity with the Administrator role, and the desktop's existing persisted
device UUID. Inspect existing registrations before writes; do not replace IDs,
reassign permissions or rotate an existing device secret implicitly. The device
secret's SHA-256 convention is documented in
`server/docs/STAFF_DEVICE_SETUP.md`; real desktop enrollment also needs protected
native credential storage. Verify actual `/v1/enrollment`, then synthetic member
API and desktop offline/restart/reconnect/pull/retry/conflict acceptance. Native
HTTPS/login/session binding, reviewed runtime grants and full server typing
remain pending. Attendance sync is outside the current member-sync protocol.

Updated STATUS, PLAN, the relationship map, server README and the setup guide
to record this result and its limits. No source, dependency, environment,
certificate, migration, permission, registration or SQLite safeguard changed.
No tests, Auth/database requests, migrations, integration suite or deployment
were rerun in this documentation step. The previous 33 local/mock test PASS and
failing full typecheck remain historical results. The isolated live integration
suite retains its nonempty-schema blocker. Live sync remains disabled.

## Preceding identity direction — one Administrator login; private Auth check prepared, 2026-10-04

The user clarified that one Administrator account logs in and operates attendance
and all controls. This supersedes the preceding no-staff-login/device-only
interpretation and its unanswered choice. Use one Supabase Auth login mapped
internally to the gym's Administrator row in `armstrong.staff`; that row is an
authorization/audit record, not a second account. No extra operational staff or
Reception login is needed. Existing test identities remain synthetic fixtures,
not required production accounts. Attendance permissions are part of the single
Admin model; the current online sync protocol remains member-only.

Added `cd server && npm run auth:check`:
`node --env-file-if-exists=.env scripts/check-admin-auth.ts`. It uses runtime
`.env` only and optional local `AUTH_CHECK_EMAIL`/`AUTH_CHECK_PASSWORD`, rejects
inherited target/credential overrides, and validates the existing canonical
runtime configuration. `src/auth-check.ts` performs real password sign-in followed
by the existing online user verifier, compares verified subjects, refuses weak
origins/privileged keys and redacts provider/network failures. Tokens remain in
memory, never returned or printed. It performs no database operation, account
creation, role grant, device approval or native enrollment. Actual Administrator
authority still comes from approved server records. Added three explicitly
HTTP-mock tests, placeholder-only optional `.env.example` comments and guide/README
instructions. No populated env file was changed or exposed.

Corrected `src/auth.ts`'s unknown-JSON TypeScript access with an object check before
the existing UUID validator; verified identity behavior remains unchanged.
No schema/migration/checksum, pg driver, dependency, member protocol, native
SQLite safeguard or sync availability changed. Real native login/credential
storage and authoritative private-session binding are still pending; this
command does not unlock native forms or grant a demo Administrator.

Fresh results: `cd server && npm test` **PASS**, 33/33 unit/driver/SQL/Auth mocks;
frontend `cd Client && npm run build` **PASS**, 53 modules; new Auth helper/CLI
`node --check` **PASS**. Focused strict tsc of `src/auth.ts`, `src/auth-check.ts`,
`scripts/check-admin-auth.ts` and `test/auth-check.test.ts` **PASS** with
`--noEmit --strict --target ES2023 --lib ES2023 --module NodeNext --moduleResolution NodeNext --types node --allowImportingTsExtensions`.
`cd server && npm run typecheck` still **FAILS**, exit 2, 79 existing pg/parseEnv/
Fastify/fixture/JSON/header errors; no errors in the new/updated Auth-check files.
Final focused HTTP-mock tests also cover rejection during online verification
after a successful password response. None of these local checks proves live Auth.

`cd server && npm run auth:check` **FAILS local setup**, exit 2:
`Configure AUTH_CHECK_EMAIL, AUTH_CHECK_PASSWORD locally for the Admin Auth check; values withheld`.
No Auth/network request or database write occurred; verified live account login
is **no**. Do not describe this as invalid credentials or successful live login.
The prior user-terminal runtime PostgreSQL/TLS/schema/checksum/RLS PASS remains
separate actual database evidence.

**Next:** configure the optional probe entries privately for the ONE Administrator
Auth account in this same project and run `npm run auth:check`. After real Auth
verification, prepare approved gym/Admin/device provisioning and enrollment;
complete native HTTPS/login/secure session storage, restricted runtime grants,
member API/desktop acceptance and full typing. The isolated integration suite
retains its nonempty-schema blocker. No deployment or real member data was used;
live sync remains disabled.

## Preceding authentication interpretation — no separate staff login requested, 2026-10-04

The user clarified that a staff account for authentication is not needed.
Do not continue requesting staff account creation. Runtime PostgreSQL/TLS,
version 1 checksum, six required tables and their RLS flags already PASS in the
user terminal; those results remain valid. The backend authentication choice is
now enrolled-device authentication without staff login, or deferral while sync
stays disabled. A concise choice question is pending; elapsed time is not approval
to change the API authorization model.

Inspected current dependencies: routes verify a Supabase bearer identity;
service authorization joins active staff/device records and derives gym scope;
operation actors and archive actors reference `armstrong.staff`. Native SQLite
archive/receipt checks also preserve authenticated actor relationships. A
device-only implementation therefore needs explicit device permissions and an
audit-actor migration/reconciliation, not removal of bearer checks or a fabricated
staff identity. Preserve the applied checksummed `001_members.sql`; any schema
change must use a new migration and retain existing history/IDs.

Updated the staff/device guide and README to mark their staff-based requirements
as current-implementation reference, rather than an instruction to create an
account. No account/registration, route, migration, grant, environment value or
SQLite behavior changed. No tests, SQL/Auth requests or deployment were rerun.
Prior 30 local/mock checks, failing full server typecheck and user-supplied live
metadata results retain their separate evidence boundaries. Live sync is disabled.

**Next:** settle device authentication versus deferral, then update the server
permission/actor contract and corresponding native enrollment plan before
implementation. Device access must be bound by server-side records to one gym;
privileged database credentials remain only in `server/.env`. HTTPS, secure
native device-secret storage and actual end-to-end acceptance remain necessary
for enabling live sync. No staff sign-in setup is required while that choice is
pending.

## Preceding runtime schema verification — user terminal passes live SQL/TLS and migration metadata, 2026-10-04

The user supplied `cd server && npm run db:verify` with all checks **PASS**:
real runtime PostgreSQL connection and TLS verification; version 1 migration
checksum; required private tables **6/6**; required table RLS flags **6/6**;
read-only schema verification with no gym/member/staff/device rows accessed.
This is actual SQL/catalog/ledger evidence from the user's working terminal,
not a SQL mock or a Codex-executed connection. The preceding user-reported
migration now has a matching ledger/checksum and verified required-table/RLS
metadata. It does not verify every constraint/grant, Supabase Auth, enrollment,
member API or desktop sync.

Reviewed the runtime Auth verifier, enrollment route/service and registration
requirements. The API verifies tokens online and derives gym/role/device access
from approved database records. No automatic account creation or staff/device
approval endpoint exists. Added `server/docs/STAFF_DEVICE_SETUP.md` and its README
link, covering confirmed test staff in the same runtime project, owner-approved
gym/staff/device relationships, stable desktop device UUID and hashing the
64-character device-secret string as UTF-8 (matching the existing API). No new
registration, token, secret, SQL grant or member data was created or read.

**Next:** confirm whether a confirmed test staff Auth account already exists in
the runtime project's Authentication dashboard. Keep its credentials local.
Prepare/review registration provisioning with that online-verified user subject,
approved role/gym and device, then perform real Auth/enrollment acceptance.
Restricted runtime grants, full server typing, native HTTPS/login/enrollment/
credential storage and actual desktop/Render/Windows acceptance remain. The
isolated integration suite retains its separate nonempty-schema blocker;
runtime metadata success is not a passing integration result. Live sync remains
disabled; no deployment occurred.

Only documentation changed in this result-recording step; no env, source,
migration, validator, driver or SQLite safeguard changed. No tests/network/
database operations were rerun. Prior 30 local/mock tests and frontend/helper
checks remain historical, as does the failing full server typecheck. Codex's
preceding verifier failed `EAI_AGAIN`; the user's PASS does not establish Codex
network recovery. STAFF_DEVICE_SETUP documents the remaining enrollment step.

## Preceding runtime migration — user reports success; read-only schema verifier prepared, 2026-10-04

The user reports the authorized `cd server && npm run migrate` succeeded in the
normal terminal against the existing runtime project in `.env`. Record this as
user-reported migration success, separate from Codex execution. Read-only live
verification of the resulting six tables, ledger/checksum and RLS flags is the
next step; migration success alone is not Auth/enrollment/member-sync acceptance.

Added `cd server && npm run db:verify`:
`node --env-file-if-exists=.env scripts/verify-runtime-schema.ts`. It requires the
loaded runtime DATABASE_URL to match the local file, uses existing pg/verified CA
and checks the authorized TLS socket plus `SELECT 1`. It then verifies the
recorded version 1 checksum against unchanged `001_members.sql` and the six
expected private tables/RLS flags inside a repeatable-read, read-only transaction.
Only catalogs and migration metadata are read; no gym/member/staff/device rows,
env/credential/hash values, migrations, seed data or repairs are used. Failures
are controlled/redacted. Added `src/schema-verification.ts`, the CLI, three
explicitly local catalog/ledger-mock tests and README instructions. No driver,
dependency, env, migration SQL, application/SQLite safeguard or sync behavior
changed; test commands still use only the isolated `.env.test` configuration.

Fresh results: `cd server && npm test` **PASS**, 30/30 local/unit/driver/SQL/Auth
mock tests; probe/helper `node --check` **PASS**; focused strict TypeScript check
of `src/schema-verification.ts` and `test/schema-verification.test.ts` **PASS**
with `--noEmit --strict --target ES2023 --lib ES2023 --module NodeNext --moduleResolution NodeNext --types node --allowImportingTsExtensions`.
`cd Client && npm run build` **PASS**, 53 modules. `cd server && npm run typecheck`
still **FAILS**, exit 2, on missing pg declarations and existing parseEnv/Fastify/
JSON/header/fixture typing; the new CLI shares the missing pg declaration blocker.
Its newly introduced parseEnv annotation was corrected without changing behavior.
Strictness remains enabled; the focused pass does not replace the full failure.

`cd server && npm run db:verify` **FAILS in Codex**, exit 2:
`Runtime schema verification failed (EAI_AGAIN); connection values and driver details withheld`.
TLS/hostname verification is **not established**, and no schema query ran here.
No second migration, Auth request, fixture write or deployment ran. These local
tests and failed Codex connection are not live verification of the user's schema.

**Next:** run `npm run db:verify` in the same working normal terminal and share
only its safe result. After it passes, provision approved staff/device records
using verified Supabase Auth subjects and restricted runtime grants. Real Auth/
member API acceptance, native HTTPS/enrollment/credential storage, desktop
reconnect/pull/retry/conflict and Render/Windows checks remain; the isolated test
schema is still untracked/nonempty. Live sync remains disabled.

## Preceding runtime table-creation request — existing .env project authorized; Codex DNS blocks execution, 2026-10-04

The user explicitly chose the existing runtime project in `server/.env` and
authorized creation of the planned tables there without creating a fresh project.
This supersedes the earlier recommendation to wait for isolated integration
acceptance before runtime migration. It does not authorize dropping existing
objects, adopting an untracked schema, using real member data, redirecting
integration tests to runtime, deployment or live-sync enabling.

Read STATUS/PLAN/relationship map, migration SQL, runner, CLI and shared pg/TLS
options. The existing `npm run migrate` command loads only `.env`, uses only
`DATABASE_URL`, and applies checked-in `migrations/001_members.sql` through the
transactional/advisory-locked/checksummed runner. It creates private tables
`armstrong.gyms`, `staff`, `devices`, `members`, `member_operations` and
`member_changes`, with the existing constraints, indexes, triggers, RLS and
public-access revocations. No SQL, driver, validator or environment change is
needed for this requested setup. Existing untracked objects are not deleted or
silently adopted; SQL/checksum errors roll back the transaction.

Attempted a one-off read-only preflight with
`cd server && node --env-file-if-exists=.env --input-type=module` (inline script).
The script checks that the loaded database target matches local `.env`, uses
the existing verified-CA pg options, and would read only `SELECT 1`, schema/ledger
presence and the migration checksum. It **FAILS**, exit 2, before SQL:
`Runtime migration preflight failed (EAI_AGAIN); connection values and driver details withheld`.
No runtime schema/ledger result or TLS connection was established in Codex.
`npm run migrate` was **not run** after that failure. No tables were created,
no data accessed and no schema reset performed. Earlier supplied test-project
object counts cannot establish runtime-project schema contents. The normal
terminal's preceding runtime SQL/TLS PASS remains separately attributed evidence.

**Next authorized action:** from `server/` in the user's working normal terminal,
run `npm run migrate` and share its controlled result only. After success, verify
the six private tables, recorded migration/checksum and RLS metadata before
provisioning staff/devices. Keep `.env.test` separate; its nonempty/untracked
schema blocker and real SQL/Auth acceptance remain unresolved rather than being
counted as passed. Native HTTPS/sign-in/enrollment/credential storage, runtime
least-privilege grants and desktop/Render/Windows acceptance remain required;
live sync is disabled. Only documentation changed; no local/mock tests were
rerun and no deployment occurred.

## Preceding isolated-schema inspection — verified live catalog counts; populated schema has no migration ledger, 2026-10-04

User-supplied `cd server && npm run test:inspect` confirms matching local test
target, real Session pooler connection, authorized TLS and `SELECT 1`. The
read-only catalog query reports application schema present, public migration
ledger absent and public application relations absent. Application schema
counts: **15 relation objects, 3 routines, 12 types, 9 dependency records**.
The empty-database prerequisite fails; no changes were made. These are live
PostgreSQL metadata results from the user's terminal, not SQL mocks or a
Codex-executed connection. Relation objects include indexes as well as tables;
these are not member row counts. Schema data contents remain unknown.

Inspected the checked-in migration and runners: `001_members.sql` defines six
tables, nine primary/unique indexes and three trigger functions, consistent
with these relation/routine counts. Counts alone do not prove schema definitions,
constraints, grants or migration provenance. The runner records its checksum in
`public.armstrong_migrations` transactionally. The observed schema without that
ledger is therefore untracked; no successful runner history can be inferred.
Do not insert a guessed ledger/checksum, edit the applied SQL or bypass the
existing-schema guard to adopt it. The real SQL/Auth integration suite correctly
stops before Auth sign-ins/migrations on this target.

**Next:** choose preservation with a fresh disposable test project, or review an
explicit cleanup of this isolated project's populated Armstrong schema. A
nonempty-schema cleanup would delete objects/data and needs explicit approval;
none is authorized or executed by this inspection result. Preserve runtime
`.env`, use only test `.env.test` for the suite, and do not manually migrate or
run the SQL/mock-Auth suite before the live suite. Once the test target meets
the empty prerequisite and its read-only verified TLS/SQL probe passes, run
`npm run test:integration:supabase`. Main-project migration, deployment and live
sync remain pending the existing acceptance gates.

This step only records supplied evidence and inspects source; no env, application,
SQLite, schema or validator changed, and no tests/network/database operations
were rerun. The 27 local/mock test and frontend results below remain historical.

## Preceding runtime PostgreSQL verification — user terminal passes SQL and verified TLS, 2026-10-04

The user supplied `cd server && npm run db:check`: **PASS**, real runtime Session
pooler connection, authorized TLS and `SELECT 1`; no application data accessed.
This is verified connection evidence from the normal terminal, not a
Codex-executed result. It establishes runtime PostgreSQL/TLS access only; no
Supabase Auth, staff/device enrollment, application schema or desktop sync
acceptance is demonstrated by that query.

After the user configured `server/.env`, a read-only local Node check parsed the
file without displaying values, rejected inherited overrides for its configured
runtime keys, and ran the existing `config()` and `databaseOptions()` validators.
Required runtime settings, project-matched Session pooler/5432, `sslmode=verify-full`,
readable PEM CA and mandatory certificate verification **PASS locally**.
Publishable/legacy-anon key classification **PASS** (format only; no Auth request).
The runtime and isolated test database usernames identify different projects in
the local comparison. No environment file was modified or values exposed.

Codex's preceding `cd server && npm run db:check` **FAIL**, exit 2: redacted `EAI_AGAIN`.
TLS certificate/hostname verification: **not established**. Verified database
connection: **no**. No migration, integration suite, API listener or deployment
ran; no database change or live-sync success is claimed. The user's subsequent
runtime success does not establish that Codex DNS is repaired.

Keep `server/.env` for runtime credentials, `server/.env.test` for separate
ignored test credentials, and `.env.example` as a placeholder-only setup template.
Next run the updated read-only `npm run test:inspect` from `server/` in the normal
terminal to obtain the isolated test schema's object counts. Preserve existing
objects while assessing the blocker; complete isolated real SQL/Auth acceptance
before main-project migration. Live sync remains pending HTTPS, staff/device
enrollment and actual desktop end-to-end acceptance.
No application/SQLite source changed, so the preceding local test results remain
historical and were not rerun for this configuration/probe documentation step.

## Preceding isolation diagnosis — verified user connection; application schema alone blocks initialization, 2026-10-04

User-supplied `cd server && npm run test:inspect` output confirms local test target
settings match `.env.test`, a real Session pooler connection with authorized TLS
and `SELECT 1`, application schema present, migration ledger absent, public
relations absent. The empty-application-database prerequisite fails with no
changes made. This latest inspected state does not establish that the schema
contains tables or fixtures; the earlier SQL/mock-Auth suite's retained state
must not be assumed to describe the current target. No real Auth acceptance or
desktop sync success is established.

Extended the same read-only `test:inspect` command to count application-schema
relations, routines, types and namespace dependency records from PostgreSQL
system catalogs. It prints only fixed labels and validated nonnegative numeric
counts, never names, member rows or environment/credential values. Existing
schema rejection, migration SQL, pg driver, TLS verification and test-only target
settings remain intact. No env file, runtime/test database, SQLite behavior or
sync availability was changed; no migration/integration/deployment ran.

Fresh local commands: `cd server && npm test` **PASS**, 27/27 unit/driver/SQL/Auth
mock tests; `node --check scripts/check-database.ts` and
`node --check test/support/isolated-database.ts` **PASS**;
`./node_modules/.bin/tsc --noEmit --strict --target ES2023 --lib ES2023 --module NodeNext --moduleResolution NodeNext --types node --allowImportingTsExtensions test/support/isolated-database.ts`
**PASS** (focused helper check, not the historically failing whole-server
typecheck). `cd Client && npm run build` **PASS**, 53 modules. These fresh checks
make no Supabase connection; the new catalog counts have not been verified live.
Codex's preceding read-only inspection attempt failed with redacted `EAI_AGAIN`;
the current real connection evidence is supplied by the user's normal terminal.

**Next:** run the updated `npm run test:inspect` from `server/` in that normal
terminal and share only its safe counts. Inspect before proposing any removal;
do not drop/reset or bypass the guard. Complete one isolated real PostgreSQL/Auth
integration run before applying the existing reviewed migrations to the intended
runtime project. Separate test and runtime settings remain required. Native
HTTPS/sign-in/enrollment/credential storage, runtime grants, real desktop
reconnect/pull/retry/conflict and Render/Windows acceptance remain; live sync is
disabled.

## Preceding real PostgreSQL verification — user terminal passes SQL/mock-Auth; live Auth blocked by retained schema, 2026-10-04

The user supplied these results from the normal VSCodium terminal. They are real database evidence from that terminal, not Codex-executed results or local/mock-only claims. Credentials, URL/project/account values and tokens are withheld.

| Command from `server/` | Supplied result | Evidence boundary |
| --- | --- | --- |
| `npm run test:db` | **PASS**: real Session pooler connection, authorized TLS and `SELECT 1`. | Actual read-only PostgreSQL/TLS success; no Auth, application migration or desktop sync acceptance. |
| `npm run test:integration:postgres` | **PASS**, 1 pass, 0 fail, 0 skipped; 46.5 seconds. | Real PostgreSQL transactions/migrations and Fastify injection with an explicitly **mocked Auth verifier**. Source covers enrollment replay, authorization assertions, simultaneous duplicates, conflicting edits/card conflicts, archive roles/history, ordered pulls and revocation. Does not verify Supabase Auth. |
| `npm run test:integration:supabase` | **BLOCKED**, 0 pass, 1 parent failure, 0 skipped; 3.2 seconds. | `TestSetupError: Refusing existing application schema, migration ledger or public relations; use a fresh disposable test project/database`. Reached the read-only isolation check; no live Auth sign-in, migration or fixture case ran in this suite. |

Inspected both integration suites and `test/support/isolated-database.ts`. The passing PostgreSQL/Auth-mock suite deliberately retains its `armstrong` schema, migration ledger and synthetic fixtures. The live suite requires an empty application database and rejects that retained state **before** authenticating accounts or applying migrations. The observed rejection is the intended isolation safeguard, not evidence of a broken SQL connection. No data/schema reset, deletion or guard weakening is authorized or performed. No runtime database access, env change, server/application/schema/SQLite change, deployment or live sync enabling was performed in this result-recording step.

Separately, Codex's most recent attempt at `npm run test:integration:supabase` exited 1 with redacted `EAI_AGAIN` (0 pass/1 setup failure), before connection/migrations. The user terminal's successful probe/SQL suite does not imply Codex DNS is repaired. Prior 26-unit/mock tests and frontend/syntax results below remain historical; no tests were rerun for this documentation-only update.

**Next:** retain the existing SQL/mock-Auth test project for inspection. Configure a **fresh disposable Supabase test project** locally in ignored `server/.env.test`, using only its Session pooler, verified CA/TLS, project-matched Auth settings and exactly three confirmed synthetic email/password Auth accounts. Keep it distinct from runtime; never copy runtime credentials. Do not migrate it manually or run `test:integration:postgres` there first. From the normal terminal, run `npm run test:config`, then `npm run test:db`, then, only if the verified TLS/`SELECT 1` probe passes, `npm run test:integration:supabase`. That live suite owns isolated migration execution. Share only redacted case results. Supabase Auth acceptance, restricted runtime grants, native HTTPS/sign-in/enrollment/credential storage and desktop/Render/Windows end-to-end checks remain; live sync stays disabled.

## Preceding database authentication diagnosis — user-reported 28P01; no successful SQL probe at that stage, 2026-10-04

The normal VSCodium terminal's `npm run test:db` result supplied by the user is **FAIL**, exit 2, SQLSTATE `28P01` (invalid password). This reaches a server authentication response, unlike the preceding Codex DNS `EAI_AGAIN` failures. It does not establish authenticated PostgreSQL access or successful `SELECT 1`. TLS verification was not separately reported by that older probe output, so no verified TLS result is claimed from it.

Privacy-safe local inspection confirms the test URL's Session pooler/5432 owner matches its configured test project, a password is present, and percent decoding is valid. Investigating copied placeholder formatting without displaying the URL, password, project/account values, keys or tokens. Asked whether outer bracket characters belong to the actual password before making any credential change; no credential or runtime env file changed in this diagnosis. Do not guess a replacement password, disable verification, or run migrations/integration after failed login.

Updated `server/scripts/check-database.ts` to report the actual TLS socket's authorized/encrypted state separately from authenticated SQL success when login fails, and give controlled `28P01` guidance. Updated `server/docs/SUPABASE_TESTING.md` with isolated database password/placeholder/encoding troubleshooting. Neither query, connection driver/options, certificate verification nor configuration guards changed. Runtime/SQLite safeguards and disabled sync remain intact.

Fresh verification: `cd server && npm test` **PASS**, 26/26 local/mock tests; `cd server && node --check scripts/check-database.ts` **PASS**; `cd Client && npm run build` **PASS**, 53 modules. These checks do not contact Supabase. No migration or integration suite ran; no new live connection result is claimed. Next confirm the credential formatting, correct only the test URL if appropriate and retry only the read-only `npm run test:db`. Keep integration gated on verified TLS plus successful `SELECT 1`.

## Current Session pooler/verified TLS connection setup — local tests pass; real connection blocked, 2026-10-04

Inspected the existing server connection and test setup, STATUS, PLAN and relationship map. Retained installed **pg 8.23.0** and Fastify; no driver was added/replaced and no dependency installation was needed. Backend changes remain in `server/`. Local `.env` and `.env.test` exist, have separate URL settings and target different projects. Their contents were never printed or written. No production member data, migration, remote fixture write, API listener, deployment or live sync was used.

### Changes

- Added shared `src/database.ts` validation and pg options for **Supabase Session pooler only**, explicit port 5432, `postgres`, role/project-matching login, credentials present, `sslmode=verify-full` and mandatory readable PEM CA through `sslrootcert`. Reject direct/transaction hosts, duplicate/unknown query overrides and global TLS disabling. Provide parsed pg fields and `ssl: { ca, rejectUnauthorized: true }` without a connection string that can replace explicit SSL options; retain Node hostname verification. Runtime supports a restricted custom role; isolated tests require `postgres.<test-project-ref>`.
- Fastify startup and the migration CLI retain the existing pg driver and use these options. Startup checks `SELECT 1` before listening. Startup/idle/shutdown/cleanup failures are sanitized, without raw URL, credentials or driver payloads. Staff/device/gym authorization, idempotency and conflict handling are unchanged.
- Added `npm run db:check`: `.env`/`DATABASE_URL` only, read-only authorized TLS socket and `SELECT 1` probe. Added `npm run test:db`: `.env.test`/`TEST_DATABASE_URL` only, isolated project guards and the same probe. Invalid probe modes fail. Neither probe migrates, accesses member data or substitutes a mock. `test:config` now also checks local PEM readability; it still performs **no network** access.
- Both PostgreSQL suites now use shared verified pg options and test-only Session pooler/project guards; neither falls back to runtime DATABASE_URL. Retained the live SQL/Auth suite versus the separately labelled real-SQL/mock-Auth suite and the empty-database/disposable-project protections. No applied migration SQL or isolation guard was weakened.
- Added three explicitly local/driver tests using a temporary **public** trust root and synthetic credentials; pg construction checks mandatory verification/CA and unchanged hostname-verification behavior without connecting. Tests cover TLS/URI overrides, wrong project, direct/transaction endpoints, missing/unreadable/malformed CA and sanitized failures. Updated synthetic configuration fixtures and placeholder-only `.env.example`; ignored root-level certificate files as well as `certs/`. Updated server README/test setup and PLAN/relationship map. SQLite/Client application/native transport safeguards are untouched; sync remains disabled.

### Exact fresh commands/results

| Command | Result | Actual evidence |
| --- | --- | --- |
| `cd server && npm test` | **PASS**, exit 0; 26/26. | Local/unit/driver/SQL/Auth mocks only; no Supabase connection. |
| `cd server && npm run db:check` | **BLOCKED**, exit 2: `Database requires sslmode=verify-full and sslrootcert; no other or duplicate query parameters allowed`. | Runtime URL is a Session pooler, but required TLS settings are missing. Verified connection established: **no**; stopped before network. |
| `cd server && npm run test:config` | **BLOCKED**, exit 2: `Test database requires the isolated project Session pooler and owner username`. | Test URL now parses but remains direct; no local validation pass or network access. |
| `cd server && npm run test:db` | **BLOCKED**, exit 2 with the same Session pooler requirement. | Verified connection established: **no**; stopped before network. |
| `cd server && npm run test:integration` | **BLOCKED**, exit 1; 0 pass, 1 parent failure, 0 skipped; same Session pooler requirement. | No actual PostgreSQL/Auth/migration/fixture case ran. The separately named Auth-mock PostgreSQL suite was not rerun. |
| `cd server && npm run typecheck` | **FAIL**, exit 2. | Existing missing pg declarations, unknown JSON/error and test request/header errors remain; the newly added pg consumers also lack declarations. Strict checks stay enabled. |
| `cd server && ./node_modules/.bin/tsc --noEmit --strict --target ES2023 --lib ES2023 --module NodeNext --moduleResolution NodeNext --types node --allowImportingTsExtensions src/database.ts src/config.ts src/protocol.ts` | **PASS**, exit 0. | New connection validation/configuration and protocol typing only; does not replace the failing whole-server check. |
| Recursive `node --check` through server `src/`, `test/`, `scripts/` | **PASS**, exit 0; 23/23 files. | Syntax only; not typecheck or connection proof. |
| `cd Client && npm run build` | **PASS**, exit 0; 53 modules. | Frontend baseline only. Prior Rust/SQLite checks remain historical. |

### Local settings still needed

Keep the runtime URL in ignored `server/.env` and the isolated test URL in ignored `server/.env.test`. Add `sslmode=verify-full&sslrootcert=./prod-ca-2021.crt` to the runtime Session pooler URL when that moved certificate is the runtime project's correct CA. Runtime also lacks `SUPABASE_URL`/`SUPABASE_PUBLISHABLE_KEY`, needed before Fastify Auth can start. For the test project, copy its actual **Connect > Session pooler** hostname, use `postgres.<TEST_SUPABASE_PROJECT_REF>:<percent-encoded-password>` on port 5432 and set the correct CA path. The presently configured test CA path does not exist from `server/`. Do not copy runtime credentials or guess the test pooler hostname from the runtime host/region. Requested only the nonsecret test Session pooler hostname; no env values were exposed or changed.

After local correction, run `npm run db:check`, `npm run test:config`, `npm run test:db`, then the live `npm run test:integration` against the disposable empty test project. Integration owns migration execution; do not migrate test schema beforehand. Resolve strict pg/JSON/request typing separately. Real SQL/Auth acceptance, restricted runtime grants, native HTTPS/login/enrollment/credential storage, Render CA/HTTPS provisioning and desktop/Windows end-to-end checks remain. No connection or live-sync success is claimed.

## Preceding local URL diagnosis and Buffer typing — 2026-10-04

Read the status, plan and database relationship map. No environment file, certificate, validator, database migration, SQLite safeguard or sync behavior was modified. No remote integration, deployment or live sync was attempted. The missing-configuration results in the preceding milestone are historical: the local test env file now exists, but its database URL is malformed.

- `server/test/support/supabase-config.ts:24` fails to parse `TEST_DATABASE_URL`; line 25 produces the value-withheld error. The Auth URL parses successfully. Privacy-safe inspection detected one embedded `TEST_DATABASE_URL=` assignment plus quote wrappers inside the database value. The original value remains invalid; extracting the embedded URL for inspection was diagnostic only and was never supplied to the validator or driver.
- The embedded URL has the PostgreSQL scheme, direct project-matching host/owner, port 5432, `/postgres`, credentials present, and only `sslmode`/`sslrootcert` parameters with `verify-full`. No credential, token, account or project value is recorded here. The configured certificate path does not exist from the server working directory and does not resolve to the moved certificate, which does exist. URL parsing fails before this separate path problem; `test:config` itself does not inspect certificate files.
- Local correction required: keep a single env assignment, with the URL inside one pair of outer quotes; use `sslrootcert=./prod-ca-2021.crt` for the certificate now in `server/`, and run commands from `server/`. Keep `sslmode=verify-full` and certificate verification. The env file was left for the user to correct; no corrected configuration pass is claimed.
- Fixed `protocol.ts` TS2580 by explicitly importing `Buffer` from `node:buffer`, adding exact dev dependencies `@types/node@24.12.0` and `typescript@5.9.3`, and adding a server-owned strict NodeNext/no-emit `tsconfig.json` with `types: ["node"]` and `npm run typecheck`. Protocol validation is unchanged. The config covers source, scripts and tests, including integration source without executing it.
- Registry installation failed with `EAI_AGAIN` for `registry.npmjs.org`. Copied only relevant verified Node/TypeScript/undici npm cache entries into a writable temporary cache; `npm install --save-dev --save-exact @types/node@24.12.0 typescript@5.9.3 --offline --ignore-scripts --no-audit --no-fund --cache /tmp/armstrong-types-npm-cache` succeeded, adding three packages. This is an offline installation, not evidence of restored network access.

Fresh checks:

| Command | Result | Boundary |
| --- | --- | --- |
| `cd server && npm run test:config` | **FAIL**, exit 2: `Invalid test connection URL; values withheld`. | Original local configuration, unchanged. No SQL/Auth connection. |
| `cd server && npm run typecheck` | **FAIL**, exit 2. Buffer TS2580 is resolved; existing errors include missing `pg` declarations, unknown error/JSON values, and test request/header typing. | New strict whole-server check; no suppression or weakening added. Full typecheck cleanup remains. |
| `cd server && ./node_modules/.bin/tsc --noEmit --strict --target ES2023 --lib ES2023 --module NodeNext --moduleResolution NodeNext --types node --allowImportingTsExtensions src/protocol.ts` | **PASS**, exit 0. | Protocol typing only; does not replace the failing whole-server check. |
| `cd server && npm test` | **PASS**, exit 0; 23/23. | Unit/route/SQL/Auth mocks, not real backend integration. |
| `cd Client && npm run build` | **PASS**, exit 0; 53 modules. | Frontend baseline only. |

Next: correct the local URL assignment and certificate path, rerun local configuration validation, then address the strict server typing errors. Real isolated PostgreSQL/Auth integration, native HTTPS/login/device enrollment/credential storage, Render HTTPS and desktop end-to-end acceptance remain unverified. Live sync stays disabled.

## Current isolated Supabase integration milestone — prepared; blocked by missing test configuration, 2026-10-04

Read STATUS, PLAN and the relationship map first. Safe inspection found **no** `server/.env.test`, **no** `server/.env`, and **no inherited test settings**. No credential values were read into output or printed. Requested a dedicated disposable Supabase project and three confirmed synthetic Auth accounts via ignored `.env.test`; configuration remains missing. No production gym/member data, provider deployment or live desktop sync was used or enabled.

### Changes and configuration

- Exact setup is documented in `server/docs/SUPABASE_TESTING.md` and commented placeholders in `server/.env.example`. Required names: `TEST_PROJECT_IS_DISPOSABLE=true`, `TEST_SUPABASE_PROJECT_REF`, `TEST_DATABASE_URL`, `TEST_SUPABASE_URL`, `TEST_SUPABASE_PUBLISHABLE_KEY`, `TEST_ADMIN_EMAIL`, `TEST_ADMIN_PASSWORD`, `TEST_RECEPTION_EMAIL`, `TEST_RECEPTION_PASSWORD`, `TEST_OTHER_GYM_EMAIL`, `TEST_OTHER_GYM_PASSWORD`. Populate **only** ignored `server/.env.test`, keeping values out of chat/Git. No populated env file was created. Runtime `DATABASE_URL`/`SUPABASE_*` are not test fallbacks. A custom database CA, when needed, is a local ignored certificate referenced through optional `sslrootcert`; no extra env variable is required.
- Added `test:config` for value-free local validation. Project reference must match HTTPS Auth origin and direct SQL host or session-pooler owner username; database TLS is `verify-full` on 5432, with no URI host/user/SSL overrides or global TLS disabling. Reject privileged Supabase keys, duplicate accounts, existing Armstrong schema/migration ledger/public relations and concurrent initialization. Online subjects must match exactly three confirmed Auth accounts in the isolated SQL database before migrations/fixtures. These guards are not provider provisioning or a substitute for configuring a test-only project.
- Added `server/test/supabase.integration.ts`, explicitly **LIVE PostgreSQL + Auth**, using real password sign-ins, the real online identity verifier, pg connections and actual Fastify injection. Prepared cases cover migrations/repeat/checksum drift, constraints/FKs/immutable history/RLS configuration, staff/device enrollment and access, two-gym isolation, simultaneous duplicates, discarded-response retry across app/pool restart, operation/card/revision/date conflicts, role/read-only/revocation denials, ordered pulls/cursors and injected SQL failure/atomic retry. No SQL/Auth mocks in this suite. Injection/reply discard does not prove network-drop recovery, deployed API HTTPS or desktop sync.
- Extracted the existing unchanged checksummed SQL migration algorithm into `server/src/migrations.ts`, used by CLI and both PostgreSQL suites. Migration SQL/schema is unchanged. CLI/test errors suppress credential/provider/driver payloads. Actual migration execution remains unverified because the real suites could not connect without configuration.
- `npm run test:integration` now explicitly invokes the live Supabase suite and **fails on missing configuration**. The prior real PostgreSQL/**mock Auth** suite is retained under `npm run test:integration:postgres`, with migration-runner/isolation guards and explicit mock labelling. It is not part of the live command. Both preserve synthetic fixtures and refuse populated schemas; neither resets a database. Added five unit tests for test configuration and mocked isolation guards, kept separate from real results.
- Updated server README, PLAN and relationship map. No Client application/Rust/SQLite, private-session/binding, retry, pull/cursor, restore or Render deployment changes. Live sync remains disabled.

### Exact commands and fresh results

From `server/`:

```sh
npm run test:config
npm test
npm run test:integration
npm run test:integration:postgres
```

| Check | Actual result | Evidence boundary |
| --- | --- | --- |
| `npm run test:config` | **BLOCKED**, exit 2; `.env.test not found`, all eleven required names missing. | Local presence/validation only; no network. |
| `npm test` | **PASS**, exit 0; 23 unit/route/guard tests. | SQL/Auth fixtures and isolation mocks; no live database/Auth proof. |
| `npm run test:integration` | **BLOCKED before connection**, exit 1; 0 pass, 1 parent-test failure, 0 skipped. | `TestSetupError: Missing isolated Supabase test configuration` lists the eleven names above. No PostgreSQL/Auth/migration/fixture case executed. |
| `npm run test:integration:postgres` | **SKIPPED**, exit 0; 0 pass, 1 skipped. | Missing `TEST_DATABASE_URL`; even when run, Auth is mocked. |
| Recursive `node --check` of server `src/`, `test/`, `scripts/` TypeScript files | **PASS**, exit 0; 20 files. | Syntax only, no backend typecheck or SQL execution. |
| `cd Client && npm run build` | **PASS**, exit 0; TypeScript/Vite, 53 modules. | Baseline interface check, not native/live sync acceptance. |

The live command resolves to `node --env-file-if-exists=.env.test --test --test-isolation=none test/supabase.integration.ts`. Missing setup fails intentionally instead of yielding a misleading successful skip. Prior 77-test SQLite/Rust and UI/desktop/Clippy results below remain historical; they were not rerun for these server-test changes. No new test data or credentials were written remotely.

### Exact next task and remaining HTTPS/enrollment steps

Configure the eleven test variables and **only three confirmed synthetic accounts** in a fresh disposable Supabase project. Run `cd server && npm run test:config`, then `npm run test:integration`; the suite itself performs migrations after its read-only isolation checks, so **do not run `npm run migrate` first**. Keep the test project empty of application relations beforehand. Resolve actual SQL/Auth/TLS/network failures if encountered, record real case results and retain fixtures for inspection. No automatic schema reset or production access is permitted.

Passing these backend tests would still leave restricted runtime database grants, native HTTPS transport, native verified login/enrollment/credential storage and authoritative session/scope binding, desktop offline/restart/reconnect/one-upload and pull-to-SQLite acceptance, real network drops/token lifecycle, Render HTTPS/certificate validation and Windows checks. Continue those only within their authorized scope; **do not deploy or enable live sync in this milestone**. No backend end-to-end/live synchronization success is claimed.

## Preceding backend enrollment continuation — installed packages usable; live sync disabled, 2026-10-04

The requested backend install retry **SUCCEEDED**, exit 0, with `up to date in 1s`:

```sh
cd server
npm install --ignore-scripts --no-audit --no-fund --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

`node_modules/` and `package-lock.json` were already present at this continuation's first inspection. Preserved them and the existing backend source. `npm ls --all --json` exits 0; the lockfile records Fastify 5.12.5, pg 8.23.0 and 62 entries including the root, with HTTPS registry URLs/integrity for both direct packages. This establishes a usable existing installation, not fresh download access, a clean `npm ci`, or a security audit. Node is v26.10.0 here; the prepared Render configuration selects Node 24.

### Changes

- Added `POST /v1/enrollment` in `server/src/app.ts`/`service.ts` with strict protocol/device/secret validation. Supabase-verified bearer subject and owner-provisioned active staff/approved device records determine the gym. No request may submit gym/user/role/approval fields. Permission rechecks under the existing gym lock precede the authoritative gym/staff/role/device-permission reply. Missing/wrong/ambiguous/revoked registration denies access. Repeated verification is read-only and safe after a lost response; it cannot create staff, approve devices or grant roles. Native login/enrollment is still unfinished.
- Existing member endpoints retain validated UUIDs, frozen operation-ID replay, revision/card conflicts and derived gym authorization. Responses now use `Cache-Control: no-store` and `nosniff`; logging remains disabled. Added seven clearly labelled Fastify enrollment/route tests using mock SQL and identity HTTP. Extended the real PostgreSQL integration source with enrollment/retry/spoofed-field/revocation checks; that suite remains skipped.
- Added `server/render.yaml`: service root `server`, Node 24, locked install plus unit checks, process health check, environment-only configuration and automatic deploys off. Production configuration requires canonical HTTPS `PUBLIC_API_ORIGIN` and Supabase origin; updated only `.env.example`, with no credentials. Configuration does not prove TLS connectivity. Blueprint structure was reviewed against official Render documentation; Render validation/deployment was not performed. A local YAML parser check was unavailable (`ModuleNotFoundError: No module named 'yaml'`). Creating a Blueprint would still initially deploy; no service was created.
- Updated server README and this status/plan/relationship map. No Client application, Rust, SQLite schema, migration or credential-store changes. Stable local device/bound gym IDs, frozen retries, push-before-pull gating, atomic pull/history/cursor, retained conflicts and restore locks remain intact. The unbound gym remains unassigned until verified native enrollment. Live synchronization remains unavailable.

### Fresh checks

```sh
cd server
npm test
npm run test:integration
# Syntax: node --check applied to every src/*.ts and test/*.ts file.
cd ../Client
npm run test:ui
npm run build
cd src-tauri
# The following cargo commands used these environment flags:
# CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked
cargo clippy --features desktop,custom-protocol --all-targets --locked -- -D warnings
cargo fmt --check
cargo build --features desktop,custom-protocol --locked
```

- Backend **PASS**: 18 unit/route tests and all TypeScript syntax checks. Actual installed Fastify executes injection; SQL and Supabase HTTP remain mocked. No backend TypeScript typecheck result is claimed.
- PostgreSQL integration **SKIPPED**: 0 passed, 1 skipped; `.env not found` and `TEST_DATABASE_URL` absent. No migration, actual PostgreSQL transaction or live identity verification was run.
- SQLite/Rust **PASS**: all 77 tests, including 14 `sync_engine_mock_server_*` cases. These use real temporary SQLite and a mock transport, not live member sync.
- Frontend **PASS**: 12 adapter/API tests, all route/settings/browser rendering checks, TypeScript/Vite build (53 modules).
- Rust fmt, desktop-feature Clippy with warnings denied and normal desktop/custom-protocol build **PASS**. No native GUI/Windows/hardware acceptance was performed.

### Remaining blockers and exact next task

Fresh development-environment network diagnostics still fail despite successful reuse of installed npm packages:

```sh
# From server/:
npm ping --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

Exit 1, full npm output:

```text
npm notice PING https://registry.npmjs.org/
npm error code EAI_AGAIN
npm error syscall getaddrinfo
npm error errno EAI_AGAIN
npm error request to https://registry.npmjs.org/-/ping failed, reason: getaddrinfo EAI_AGAIN registry.npmjs.org
npm error A complete log of this run can be found in: /tmp/armstrong-npm-cache/_logs/2026-10-03T20_07_46_243Z-debug-0.log
```

`curl --fail --silent --show-error --connect-timeout 5 --max-time 10 https://index.crates.io/config.json` exits 6 with `curl: (6) Could not resolve host: index.crates.io`. An audited native HTTPS/credential-store dependency is still unavailable locally. `podman images --format '{{.Repository}}:{{.Tag}}'` exits 1 with `Failed to obtain podman configuration: mkdir /run/user/1000/libpod: read-only file system`; PostgreSQL server/client binaries are absent. No further dependency install or unverified replacement transport was attempted.

Requested a **test-only** disposable PostgreSQL/Supabase environment via ignored `server/.env` (`TEST_DATABASE_URL`, `DATABASE_URL`, `SUPABASE_URL`, `SUPABASE_PUBLISHABLE_KEY`), with credentials kept out of chat. Configuration is still missing. Next run the existing migration and actual Fastify/PostgreSQL integration on a new disposable database; provision test staff/approved device records and verify live Supabase auth plus enrollment. Restore native dependency download access, implement native HTTPS/credential storage/session binding/transport, then execute offline create/restart/reconnect/one-upload, server-to-SQLite pull, retries/duplicates/auth/conflict tests. Separate fixture/local checks from actual live results. Render HTTPS acceptance and Windows checks remain required. **No live end-to-end result or production sync success is claimed. No public deployment or real member data was used.**

## Preceding real member-sync access check — dependency probes blocked, 2026-10-04

Read this status, PLAN and the database relationship map before checking access. Inspected the existing `server/package.json`, member route/authorization source and client transport/reply seams. Existing backend work remains in `server/`; no application, migration, dependency or deployment changes were made in this continuation.

From `ArmStrong/server/`, the two fresh read-only dependency/network probes were:

```sh
npm view fastify@5.12.5 version --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
npm view pg@8.23.0 version --fetch-retries=0 --fetch-timeout=10000 --cache /tmp/armstrong-npm-cache
```

**Both FAILED, exit code 1.** Exact errors: `npm error code EAI_AGAIN`, `npm error syscall getaddrinfo`, `npm error errno EAI_AGAIN`; requests to `https://registry.npmjs.org/fastify` and `https://registry.npmjs.org/pg` respectively failed with `getaddrinfo EAI_AGAIN registry.npmjs.org`. This confirms dependency access is still blocked in the development environment; it does not establish whether Supabase or Render is reachable. Per the requested access gate, backend installation stopped; no install, lockfile generation or public deployment was attempted.

Current source inspection confirms member routes exist and gym authorization derives from verified staff/device records; a supplied gym ID is only checked against that derived scope. The client still has an internal `MemberTransport` trait and protected acknowledgement/pull entrypoints, with no production HTTPS implementation or authenticated enrollment flow. The existing **Sync unavailable** control and `memberSync.available=false` remain intact. Device identity, verified gym-binding seam, durable retries, push-before-pull gating, transactional pull/cursor and restore safeguards were preserved without code changes. An unbound local database still has no assigned gym; stable enrolled gym identity requires the missing verified enrollment flow.

No backend, SQLite, frontend or Rust suites were rerun in this documentation-only continuation. Results under the preceding offline milestone are historical local/mock evidence, not results of this access check or proof of online connectivity. No test staff/device enrollment, offline/restart/reconnect upload, live server-to-SQLite pull, real auth/conflict/retry test or live backend end-to-end result was produced. No real member data or credentials were used.

**Remaining work:** restore registry/network access and install/review the pinned dependencies; generate the backend lockfile and run migrations/Fastify integration in a disposable test database. Implement authenticated staff/device enrollment, native HTTPS and credential storage, and Render HTTPS configuration with environment-only secrets. Connect the real transport only after verified enrollment; run the requested test-gym end-to-end cases and backend/SQLite/frontend/Rust checks, separately reporting mock and live results. Keep live sync disabled until HTTPS and enrollment succeed; do not claim successful live sync until those end-to-end cases actually pass. No public deployment is authorized.

## Current member-sync engine — offline safeguards verified; live sync disabled

Continued the member-only plan with the user's four verification points. No production HTTPS transport, authenticated native enrollment or deployed provider exists. The 2026-10-03 source/storage milestone below is historical.

### Changes and safeguards

- Device UUID remains stable across restart. A native-only binder persists canonical HTTPS origin/gym/that device UUID in metadata and rejects rebinding, another device, insecure/credential-bearing URLs, restored databases and existing unscoped remote state. It checks the private active staff session/canonical Supabase subject inside its transaction. Fresh databases remain unbound with no invented gym; verified enrollment is still absent. No binding/identity IPC exists.
- Backend source derives gym access from active staff for the verified Supabase subject joined with approved devices and authenticated device secret. Client gym is only a consistency assertion. Permissions are rechecked under the derived gym lock. Added gym authorization unit tests and a PostgreSQL case for an unrelated real gym/device without staff membership; the integration case remains skipped.
- Added internal bounded `member_worker.rs`. Frozen requests survive restart; delivery error and retry deadline commit together with capped backoff. Earlier acknowledgements remain when a later push fails. Transient/auth failures stop the run; rejected/unsupported pending member operations block pull. Nonmember queues are untouched. Partial runs never advance the complete-run success timestamp. Server denial clears the private session.
- Worker page entrypoint checks staff/session and absence of pending member operations inside its IMMEDIATE transaction, then applies page/history/cursor atomically. Edits during pull defer that page/cursor until pushes succeed. Invalid pages roll back and retry from the same cursor. Acknowledgement transactions recheck session identity. Unauthenticated raw receipt/page seams are test-only; no worker/reply IPC is registered.
- New tests are named `sync_engine_mock_server_*` and explicitly use real temporary SQLite plus a mock server transport. They verify orchestration, not HTTPS/JWT/Supabase/PostgreSQL/native UI or live connectivity. Settings retains **Sync unavailable** and the snapshot always reports `memberSync.available=false`; generic failure text does not enable sync. Browser simulation stays separate.
- No SQL schema change was needed. Binding/retry metadata participates in backup confirmation fingerprints and v5 copies; restore preserves it but clears sessions and blocks sync. Added exact cached `url=2.5.8` and updated Cargo.lock offline. No replacement HTTPS/password scheme was introduced.

### Checks

Cargo flags: `CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`.

- Full Rust suite: **PASS**, 77 tests (63 previous + 14 labelled mock-server engine tests). After final transactional session checks, all 14 engine tests passed again (63 filtered).
- New coverage: stable/unbound/canonical scope, dropped response/restart/frozen retry, deadlines, writes during push/pull, partial acknowledgement retention, permanent rejection gating, cursor rollback/retry, bounded pagination, role/session/subject/archive checks, atomic failure-state rollback, reply denial after revocation and restore locks.
- `cd server && npm test`: **PASS**, 10 dependency-free protocol/auth/config/gym unit tests. Identity HTTP and registry inputs are fixtures, not live provider checks.
- PostgreSQL/Fastify integration: **SKIPPED**, one scenario because `TEST_DATABASE_URL` is absent. All server/test TypeScript syntax checks pass. Installed dependency imports, backend TypeScript typechecking and PostgreSQL execution remain unverified.
- `npm run test:ui`: **PASS**, 12 adapter/API tests plus route/settings/browser/receipt/removal/conflict/retry-failure rendering. `npm run build`: **PASS**, TypeScript/Vite, 53 modules.
- Final desktop-feature Clippy `--all-targets --locked -- -D warnings`, `cargo fmt --check`, and normal `cargo build --features desktop,custom-protocol --locked`: **PASS**. No smoke feature or trusted test identities are included in the normal build.
- No native GUI, Windows/hardware, actual HTTPS/auth/provider deployment or production sync was verified. Earlier GTK failure remains the last interactive result. No OS packages changed.

### Remaining access / exact next task

The 2026-10-04 dependency recheck still fails npm registry DNS (`EAI_AGAIN`). No local Fastify/pg installation or PostgreSQL tools were found; Podman's normal runtime path is read-only. HTTPS/credential-store/password-hashing dependencies remain uncached. These are environment limitations, not a pending permission question.

Restore dependency access, install/review backend packages and generate its lockfile, then run actual PostgreSQL/Fastify integration in a new disposable database. Implement audited native HTTPS and authenticated staff/device enrollment, bind only after verified server authorization, store secrets natively, and connect a real transport/scheduler to the tested engine. Provider/offline policy, real role/token/reconnect tests, conflict/restore reconciliation and Windows acceptance remain required. **Do not enable live sync based on mock-server tests.** Current PLAN and server README record this boundary.

## Preceding member-sync source/storage milestone — 2026-10-03

The previous writable-workspace restriction is resolved. Both `Client/` and the requested sibling `server/` are now writable; a reversible server write probe succeeded. No usable Git metadata exists in this mounted project, so no commit/tracked-diff result is claimed. The older inspection section below records the preceding session and is superseded here.

### Changes

- Updated the relationship/protocol map before schema changes. Added all backend files directly to `ArmStrong/server/`: README, ignored environment example/rules, exact direct-dependency pins, PostgreSQL private schema and transactional/checksummed migration runner, Supabase online token verifier, configuration, Fastify routes and member service.
- Backend source supports member create/update/archive only. Every request requires verified Supabase identity plus active gym staff/device and an independently registered device secret. Archive requires Administrator. Permission/device mutations share the gym lock with API operations; current permission checks precede replay. One active writer is enforced pending topology agreement. No role/device/account self-enrollment or demo seeds exist.
- Backend member changes/operation receipts commit together. Same operation ID/content/device/actor replays its original receipt; mismatches, stale revisions, duplicate normalized cards and immutable archives are rejected. Per-gym sequence allocation and commit are serialized to prevent pull cursor skips. Initial bootstrap is retained-history pagination. Credentials and privileged SQL stay out of Client.
- Added explicit SQLite v5 migration `005_member_sync.sql`. Original outbox rows stay immutable and undeleted. Separate frozen requests/acknowledgements, confirmed remote revisions, ordered pull cursor and retained conflicts persist across restart. Only individually matching member receipts remove pending status; nonmember/deletion queues remain pending. Money/membership/attendance/NFC history remains intact.
- Internal pull and cursor commit atomically; pending local changes, card/history collisions and actor mappings retain local rows plus remote conflict snapshots. Remote current-card changes append/revoke existing local NFC history; original attendance card FKs survive. Archive actors map by verified subject to existing local user FKs or new inactive identity references without roles/session grants. Archive acknowledgements must match the original enrolled subject/member details; local archive history is retained alongside remote history.
- Backups/restore recognize canonical v1–v5 and include delivery/head/cursor/conflict state. Restored databases block all internal sync paths for server reconciliation. Settings retains the black/amber interface and shows local/server conflict projections, acknowledgement/conflict counts and unavailable sync. Browser simulated sync is separate.
- These sync methods are **internal storage seams with no IPC endpoints**. No live server is configured and there is **no native HTTPS worker or verified login/enrollment**. No form can grant identity, submit trusted remote pages or acknowledge outbox rows. No production operation has been synchronized. Conflict resolution is displayed as unavailable.

### Verification

Resource flags for Cargo: `CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`.

- `cargo test --manifest-path src-tauri/Cargo.toml --locked`: **PASS**, 63 tests on final source (12 member-sync plus all 51 preceding storage/finance/removal checks). New cases cover frozen retries/new writes during delivery, exact acknowledgement/history retention, wrong receipts, independent versions, pull/cursor rollback, pending/card conflicts, inactive archive references, mapped existing actor FKs, original-actor receipt denial, NFC attendance history, pending deletes, persistent rejections and v5 backup/restore sync locks.
- `cd server && npm test`: **PASS**, eight dependency-free protocol/config/auth-response tests. Supabase responses are test stubs, not live identity verification.
- `cd server && npm run test:integration`: **SKIPPED**, one real PostgreSQL/Fastify integration scenario because `TEST_DATABASE_URL` is absent. It refuses an existing schema and does not substitute a fake database. Real database transaction/concurrency/authorization tests are not claimed.
- All backend/test TypeScript files pass `node --check`. This validates syntax, not installed Fastify/pg imports or TypeScript typechecking.
- `npm run test:ui`: **PASS**, 12 adapter/API tests plus all nine native routes, seven settings panels, separate browser routes, prior receipt/removal checks, and new conflict-projection/disabled-sync rendering.
- `npm run build`: **PASS**, TypeScript/Vite production build (53 modules).
- Desktop-feature Rust Clippy with `--all-targets --locked -- -D warnings`: **PASS** after the final actor mapping change. `cargo fmt --check`: **PASS**. `npm run desktop:build`: **PASS**; after the final source adjustment the normal binary was rebuilt again with `cargo build --manifest-path src-tauri/Cargo.toml --features desktop,custom-protocol --locked`: **PASS**. No smoke features or trusted test identities are included in the normal build.
- Native GUI/IPC, Windows, hardware, real Supabase authentication, provider deployment, HTTPS, disconnected/reconnected worker and conflict resolution acceptance were not performed. Earlier GTK launch failure remains the last native interactive result; it was not repeated for this milestone.

### Remaining blockers / exact next work

`npm view fastify version --fetch-retries=0 --fetch-timeout=10000` fails with DNS `EAI_AGAIN`; offline install fails `ENOTCACHED`. Fastify/pg and a Rust HTTPS client are not cached. PostgreSQL tools are absent. The backend cannot currently install/start, no lockfile has been fabricated, and no actual provider/database/staff/device credentials were supplied. These are dependency/infrastructure limitations, not an outstanding workspace permission request.

Restore dependency/network access and run the server integration suite against a new disposable PostgreSQL database; generate/review the lockfile and fix real SQL/route failures. Then add verified native staff/device enrollment, HTTPS/credential-store/session integration and bind server/gym/device scope before using the internal sync methods. The worker must retry pending frozen operations to obtain exact receipts before pull; implement backoff/status/real conflict handling and restore reconciliation. Provider deployment/region/budget and offline unlock policy still need owner decisions. Keep native sync unavailable until real authentication/server/restart/reconnect tests pass. See `server/README.md` and current PLAN; do not synchronize finance or silently grant local privileges.

## Preceding member-sync inspection — historical backend directory access block

Current requested scope: authenticated two-way synchronization of member creates, updates and archives only. Preserve local SQLite/history and browser separation.

- Confirmed project directory `/home/prinzz/development/Projects/ArmStrong/` contains `Client/` and the requested existing `server/`. The server folder is empty; no existing backend work or secrets were changed. `git rev-parse --show-toplevel` fails: this mounted checkout exposes no usable Git metadata, so a Git repository root cannot be verified.
- Read AGENTS, PLAN, STATUS and relationship map; inspected native member fields/validation, v4 archive/user/card relationships, audit/outbox writes, and desktop synchronization Settings/provider.
- Existing architecture proposal is TypeScript/Fastify, Supabase PostgreSQL/Auth and Render. No actual provider deployment, authenticated backend or staff enrollment exists yet. Keep the existing proposal as the implementation starting point rather than silently substituting another provider.
- Members store UUID, name, phone, optional email/current normalized unique NFC UID, joined business date, local optimistic version, archive instant and actor-user FK. Linked periods/attendance/invoices/payments/NFC history must remain intact.
- Local create/update/archive already commit immutable audit/outbox rows atomically, but there is no member acknowledgement state, server revision, pull cursor, conflict record or native HTTP worker. Settings still correctly reports no configured backend. Browser `services/sync.ts` is simulated and must not be reused in native mode.
- **Environment blocker:** `ArmStrong/server/` is mounted read-only. `test -w server` fails; a reversible write probe fails with `Read-only file system` and creates no file. This session permits writes only under `Client/` and `/tmp`; `cd ../`/a changed command working directory does not extend those roots. Sandbox escalation is unavailable. Backend code and `server/README.md` cannot be created at the required location in this session.
- No backend/client implementation, schema changes, env file, dependency installation or new tests/builds were performed for this sync milestone. Do not interpret the earlier 51-test/build results as sync verification. No native UI or network/server test is claimed.

**Exact next task:** reopen `/home/prinzz/development/Projects/ArmStrong/` as the writable workspace (including both `server/` and `Client/`). Then preserve/reinspect server contents, create its README/env-example/ignore rules, finalize the member schema/protocol in the relationship map, and implement/test authenticated idempotent endpoints plus native durable push/pull/conflict handling. Keep all backend files directly in the existing `ArmStrong/server/`, never inside Client. Live provider provisioning and credentials should be handled through ignored server env/config; no server secrets belong in native/frontend assets.

## Removal milestone — storage/UI implemented; privileged use blocked

This is the preceding removal milestone. Finance and earlier milestone sections below are historical evidence and are superseded here for member/expense removal.

- Audited actual v3 member/expense screens, FK relationships, immutable financial/NFC history, audit/outbox/request receipts and the unauthenticated operator. Updated the relationship map **before** schema changes.
- Added explicit v4 migration `004_removal.sql`. Existing members migrate active with unchanged IDs/versions/fields/history. New archive metadata stores UTC time and a staff-user FK; active lists/counts/search/manual attendance omit archived members. Archived view and membership history remain accessible. Attendance, periods, invoices, payments, financial balances and NFC reservation are retained. Native edits/attendance/new memberships reject archived members. No reactivation/card-release policy was invented.
- Added visible **Archive / Deactivate** with confirmation and **Delete permanently** only for unlinked members. IMMEDIATE transactions recheck version/business links and record user/device/before/after audit, immutable outbox and actor-bound durable request receipt. All membership, attendance, invoice, payment and NFC history links (including revoked cards) prevent permanent deletion. Unlinked deletion retains creation/action audit, pending operations and tombstone; no cascade or history purge.
- Added visible **Void / Reverse** with confirmation and required reason. One immutable `expense_voids` record references the original expense and authenticated actor. Original positive amount/date/method/recorded user remain unchanged and visible. Duplicate void and voiding a legacy reversal are blocked. Effective totals exclude Voided originals and legacy reversing rows; Expense CSV exposes original/effective amounts, status, reason, time and staff identity.
- Valid legacy expense reversals preserve both old rows and receive honestly labeled void metadata; conflicting amounts/details stop migration intact at v3. Canonical v1–v4 backups/restore include archives/voids, insert users before archived member FKs and clear runtime authorization after restore. Runtime session capability is excluded from persisted restore-change fingerprint.
- Existing React black/amber styling and browser demo behavior remain intact. No localStorage fallback, demo administrator privilege, session-grant IPC, seeded user or fake sync/success was added.

### Authorization dependency — live saves deliberately locked

There is **no implemented authenticated native login/enrollment source in this repository**. Consequently no production path can currently establish the private native removal session. The new confirmations are visible, but final saves stay disabled and the Rust commands independently deny the unauthenticated local operator, including direct IPC.

Native removal requires an unexpired verified-session context plus an active enrolled SQLite user with Administrator role, checked **inside the transaction before replay**. Inputs never accept staff identity/role; audit uses the session-derived user FK and label. Deactivation, role revocation and expiry deny later operations. Tests use a trusted test-only fixture to exercise authorized transactions; this is **not completed staff authentication or live authorization enrollment**. Other preexisting test-operator workflows retain their preceding milestone limitations.

### Verified checks

Cargo-backed commands used `CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`.

- `npm run test:core`: **PASS**, 51 tests. Nine removal cases cover all linked-member archive/history/restart/counts, unlinked deletion/audit/tombstone/replay, each business/NFC link preventing deletion, expense reason/void/duplicate/report/audit/immutable rows, unauthorized/expired/inactive/nonadmin/revoked/spoofed denial, stale versions and outbox rollback, valid/conflicting v3 migration, archive-user FK/void backup recovery and session clearing. Prior 42 finance/storage checks remain passing.
- `npm run test:ui`: **PASS**, 12 adapter/API tests; nine routes, seven settings panels, separate browser demo/login, finance receipt checks and new removal confirmations, required reason/checkbox, unauthorized save locks, archived adapter/history/debt preservation, effective expense totals and void history.
- `npm run build`: **PASS**, TypeScript/Vite production build.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: **PASS**.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --features desktop,custom-protocol --all-targets --locked -- -D warnings`: **PASS**.
- `node --check tests/desktop-smoke.js`: **PASS**; native harness expanded for confirmation controls, archived view and direct unauthenticated archive/delete/void IPC denial without business/audit/outbox changes.
- `npm run test:desktop`: frontend/smoke binary compile **PASS**; launch **FAIL** at `Failed to initialize gtk backend!` / `Failed to initialize GTK` before the window/harness executes. Native click/IPC/restart acceptance is **not verified**. No GLIBC or OS package changes.
- `npm run desktop:build`: **PASS**, normal Linux desktop/custom-protocol binary rebuilt after smoke attempt. Binary inspection found no smoke-harness markers or privileged test-fixture identities.

**Exact next task:** integrate an actual verified staff enrollment/login flow in native code with the agreed identity/backend/offline policy; only that flow may establish/clear the private session. Do not expose an IPC that trusts a user ID or allow the browser demo/local operator to self-grant Administrator. Then, in a usable Linux/Windows GUI, run `npm run test:desktop` and add/run authenticated Administrator vs Reception acceptance for archive, linked-delete denial, unlinked deletion and expense void/duplicate/report/audit/restart. Rebuild normal desktop afterward. Real authentication and successful native GUI acceptance remain required before claiming these actions usable by production staff.

## Finance milestone — implemented; native acceptance blocked

This is the preceding finance milestone. Older milestone sections below are historical and are superseded here for finance.

- Inspected actual SQLite v2 tables/triggers, relationship map, payment/date UI, durable request receipts, backup code and tests. Recorded the finance contract in the relationship map **before** changing the schema.
- Added transactionally applied `003_finance.sql` (schema v3). Original received payments, invoice/allocation IDs and values, old audit/outbox and request results survive migration. Legacy receipts are labeled as artifacts generated from historical records; valid old reversals receive releases/reasons without erasing history. Conflicting old reversal amounts/ownership stop migration and leave v2 intact.
- Added native member invoices (optionally linked to a saved membership at its historical price), repeated partial allocations, capped invoice settlement, unallocated overpayment credit and separate member debt/credit/net balances.
- Added explicit-date membership renewal: plan name/price snapshot and membership/invoice commit together. Captured plan version/latest period protect against stale renewal. Dates remain inclusive and nonoverlapping; a successor starts after the latest period. Free plans retain date-only entry with no invented zero-invoice policy. Old periods are never automatically billed.
- Added full received-payment reversal: retain original positive payment and receipt, append matching reversal/reason plus releases for all original allocations, reopen balances, preserve dates and commit audit/outbox atomically. Duplicate reversal and reversal-of-reversal are blocked. No bank/card refund execution, partial refund, invoice cancellation or membership cancellation is implied.
- Added immutable saved payment/reversal receipt snapshots with unique stable device/UUID numbers, preview content and read-only reprinting. Later gym/member/plan/allocation edits do not rewrite issue-time details. React preview and receipt-only print action are present; **native print-dialog/physical printing are not verified** and are not counted as a completed native printing workflow.
- Desktop Payments keeps existing modal/table/buttons and black/amber shell; finance controls use typed Tauri methods only. Success notification follows native commit. Browser demo remains a separate branch/provider. No localStorage/seeded record/fake sync fallback or new Settings configuration was introduced.
- Dashboard/Reports/Income CSV subtract full reversing payments on their own Colombo day; original history stays positive. Allocations/invoices add no cash. Backups/restore include all v3 documents/releases in FK order and retain recovery/validation protections.

### Verified checks

With `CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` for Cargo-backed commands:

- `npm run test:core`: **PASS**, 42 tests (15 finance tests plus previous 27). Finance coverage: partial payments/overpayments/credit and reopen; repeated partial allocation and durable retry; ownership/caps; explicit renewal/stale plan/history/invalid dates/atomic invoice; immutable full reversal/duplicate rejection; injected outbox/receipt failure rollback; v2 migration/original old retry results; valid/conflicting legacy reversals; orphan/document/release and append-only integrity; concurrent allocation limits; saved profile/member receipt reprints; signed income CSV; finance backup/restore.
- `npm run test:ui`: **PASS**, nine adapter/API tests, all nine native route renders and seven Settings panels, separate browser login/demo routes, fail-closed loading/storage/runtime checks, saved receipt preview/legacy/reversed/reversal/escaping checks and reversal-adjusted dashboard income.
- `npm run build`: **PASS**, TypeScript and production Vite build.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: **PASS**.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --features desktop,custom-protocol --all-targets --locked -- -D warnings`: **PASS**.
- `npm run desktop:build`: **PASS**, normal Linux desktop binary (desktop/custom-protocol; no smoke features).
- `node --check tests/desktop-smoke.js`: **PASS**. Harness expanded for actual form/IPC invoice, allocation, partial balances, reversal, explicit renewal, overpayment/credit, saved receipt preview and process restart.
- `npm run test:desktop`: frontend + smoke binary compile **PASS**; native launch **FAIL** at `Failed to initialize GTK` / `Failed to initialize gtk backend!` before a window or harness executes. Interactive native UI, restart harness and system print dialog were **not tested successfully**. Windows build/installer/printer acceptance is unperformed. No GLIBC or OS package changes were made.

### Decisions and limitations

- Automatic renewal remains disabled: gym policy must define automatic start/end/month-end, expired-period gaps/grace, and unpaid renewal admission. Repository rules support staff-selected explicit dates only; receiving money does not auto-renew or authorize admission.
- Confirm whether the gym requires a prescribed receipt sequence/details. Current unique stored identifiers use `AF-R-{device UUID}-{payment UUID}`, wrap in the 80 mm content layout and survive reprinting/restores. Select printer/paper in the real system dialog; no printer selector is faked.
- Full payment accounting reversal is available; partial refunds, invoice corrections/cancellations, membership-date corrections and sale/expense reversals remain outside this milestone.
- Existing unauthenticated local operator and missing authenticated server/roles/sync remain release gates. Durable financial outbox remains pending until a real server confirms it; this milestone implements no network synchronization.

**Exact next task:** In a usable desktop display session, run the expanded `npm run test:desktop` with the resource flags above to verify native forms/IPC and restart. Then open a saved original and reversal receipt, use **Print / system preview**, cancel/reprint and print to PDF/target Windows printer; verify receipt number, issue snapshot and payment/audit/outbox counts stay unchanged. Rebuild `npm run desktop:build` afterward to leave a normal binary. Record actual Windows/native print results and obtain the automatic-renewal/receipt-format decisions before adding any automatic policy.

## Local persistence — milestones B/C: implemented and checked

This describes the preceding local-persistence milestone. Current finance status is above; older coverage/runtime handoffs below are historical evidence. The local storage milestone is implemented; production release gates remain open.

### Changes

- Kept the existing shared React App, nine route components, sidebar/topbar and black/amber styling. Desktop uses only native SQLite methods; browser demo seeds/localStorage/login/simulated sync remain separate. Storage failures fail closed with no demo substitution.
- Added `002_local_operations.sql`: transactional v1 → v2 migration preserves original IDs/device metadata/member/plan/period/history/audit and version-1 pending outbox payloads. New STRICT tables store profile, card assignment history, attendance, received payments, products, stock movements, sales/items and expenses. Empty identity and invoice/allocation tables reserve the documented relationships; their workflows are not implemented.
- Added Rust commands and typed frontend storage methods for the existing forms and four real profile inputs. Money uses integer LKR minor units; native validation, Colombo business dates, FKs/unique constraints/indexes, optimistic master versions and append-only ledgers protect records. Business changes, audit, durable outbox and request receipts commit atomically. Native success messages follow successful commit; failed forms remain open with errors.
- New append operations have persistent request receipts; retrying the same operation cannot duplicate payment/sale/stock/attendance/expense records. Product entry/editing uses existing product fields and opening stock as a ledger movement. Existing member/plan/period creation APIs retain their contract: if a response is lost, refresh before recreating/submitting those forms; there is no automatic retry.
- Pending operations stay pending. No worker, fake server acknowledgement, local outbox deletion, browser-data import or automatic financial conflict overwrite was introduced. Stored operations include card IDs and sale/stock snapshots needed for later real synchronization.
- Six native CSV exports contain actual records, including audit before/after history; CSV fields resist spreadsheet formula injection. Backups use SQLite's backup API, private files and a versioned SHA-256 envelope. Restore validates schema/integrity/FKs in isolation, migrates old backups, previews explicit replacement, rechecks confirmation under an IMMEDIATE writer lock, fsyncs a validated recovery backup, and copies all tables/reinstates history protections in one transaction. Failure rolls back replacement. Restored databases are flagged for future server reconciliation.

### Current screen coverage

| Navigation | Implemented native behavior | Remaining limits |
| --- | --- | --- |
| Dashboard | Stored member/expiry summaries, today's check-ins and payments + sales; real last-28-day attendance chart and pending count. | No admission/financial allocation policy inferred from totals. |
| Members | Add/edit/search, unique normalized current cards plus assignment history, membership status/dates/history. | No archive, freeze or historical correction workflow. |
| NFC Attendance | NFC/manual logs, native same-day in/out toggle, Colombo dates, duplicate HID-scan protection. | Reader hardware unverified; no approved expiry/grace/admission enforcement. |
| Memberships | Add/edit plans, exact prices, derived active counts, explicit immutable period entry/history. | No automatic renewal, date calculator or reversals. |
| Payments | Member, amount and method persist as received-payment records. | No invoice settlement/partial balance, linked renewal, receipt rendering/printing or reversal. |
| Sales & Inventory | Add/edit products, opening stock, +/-1 movements, atomic single-product sales, native prices and no local overselling. | No returns/corrections, multi-product cart or on-screen sale/ledger history. |
| Expenses | Description/category/amount/method persist with native day/unauthenticated actor. | No reversals or date filters. |
| Reports | Real totals and all six CSV exports: attendance, memberships, income, inventory, expenses and full audit. | No report date filters or full on-screen audit/history explorer. |
| Settings | Gym profile and backup actions use native storage; sync counts are native; Windows updater checks GitHub Releases and installs signed updates on request. | Accounts/roles enforcement, certified reader/printer and full authenticated sync remain incomplete. |

### Verification

- `npm run build`: **PASS**, TypeScript + Vite, 48 modules. Final embedded frontend includes all local form/report/settings changes.
- `npm run test:core -- --offline`: **PASS**, 27 tests. Covers populated v1 migration/idempotent reopen, invalid legacy money/orphan rollback, exact money, unique cards/SKUs, FKs/card ownership/invoice-allocation integrity, append-only history, stale versions, settings persistence, file reopen, durable retry receipts, outbox failure rollback, atomic sale/item/stock writes, concurrent sellers, Colombo midnight/debounce, historical prices/costs, checksum/schema/FK rejection, recovery/restore/reconciliation, stale restore confirmation and writer exclusion/rollback after replacement.
- `npm run test:ui`: **PASS**, seven adapter tests plus actual React rendering of all nine desktop destinations, all seven Settings tabs, loading/error/fail-closed states and separate browser demo login/provider/all routes. These verify routing/component/data behavior without a display; they are not clicked native UI acceptance.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: **PASS**.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --offline --locked --all-targets --features desktop,custom-protocol -- -D warnings`: **PASS** on the final Rust changes.
- `npm run desktop:build -- --offline`: **PASS**, final normal Linux debug executable with embedded assets; smoke-test command/path markers absent. No Windows artifact/installer was built.
- `npm run test:desktop`: frontend and `ui-smoke` compilation **PASS**; launch **FAILED before UI execution** with `Failed to initialize gtk backend!` / `Failed to initialize GTK`, exit 101. No actual webview navigation/form/IPC/restart or visual/hardware acceptance is claimed. Both harness scripts pass syntax checks.
- Initial full desktop Clippy build exhausted disk space. Removed only generated `src-tauri/target` artifacts; final Cargo checks/builds used `CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` and offline dependencies. No application databases, GLIBC or OS packages changed.

### Open decisions and exact next task

Local test access remains unrestricted and audit actors are explicitly unauthenticated. Real enrollment/offline authorization, approved topology/hosting and authenticated backend are required before production release. Resolve billing/debt/renewal/reversal/receipt policy and attendance admission before those workflows. No secrets, fake users, fabricated invoices or settings absent from the actual app were added. Copy native backups off-device; this milestone does not include encryption, unattended backup retention or automatic sync reconciliation.

**Exact next task:** run the expanded native navigation/forms/exports/backup/process-restart harness in a Linux GUI session with usable GTK/display access, then rebuild the normal executable; do not change GLIBC or unrelated OS packages. Follow with Windows build/installer and reader/printer acceptance, plus the identity/topology decisions. Native acceptance must pass before claiming those screens were interactively tested.

```sh
CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true npm run test:desktop
CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 npm run desktop:build -- --offline
```

## Local persistence — milestone A: audit and relationship map

- Re-read actual entry points, v1 migration/native storage, typed adapter/context, operational forms, all seven Settings tabs and existing core/adapter/SSR/native-smoke tests. Native tables are exactly metadata/plans/members/membership_periods/audit/outbox; other desktop operational writes and settings are still unavailable. Prior summaries were checked against source.
- Created `docs/DATABASE_RELATIONSHIP_MAP.md` before application changes. It inventories current and required entities, keys/FKs/uniqueness/indexes, history/edit rules, every saved UI field and visible setting, missing storage and stored fields without UI. Reserved billing/identity relationships are explicitly distinguished from working workflows.
- Unresolved: real identity/offline authorization; topology/hosting/server acknowledgements; billing/partial-payment/renewal/receipt/reversal policies; attendance admission and hardware policies; Windows reader/printer verification. Only Gym profile has editable settings fields (name/location/phone/email); no printer/reader/endpoint/update/retention selectors exist. Do not invent those settings or persist fake readiness/account/sync states.
- Proposed bounded local work: additive v2 migration preserving v1 data; native attendance/payment-record/product/stock/sale/expense/gym-profile persistence; real ledger summaries/exports; validated native backups and recovery. Payment records do not automatically renew memberships or settle invented invoices. Attendance logs do not imply an admission policy. Product entry is needed to use a fresh empty inventory; reuse already-visible product fields and existing styling.
- Audit milestone is documentation only. No new build/native acceptance claimed. Next exact task: implement and test the additive local schema/migration against populated v1 fixtures, then wire the existing desktop forms and profile fields, preserving browser separation.

## Interface integration — inspection milestone

- Inspected browser `src/main.tsx` → `GymProvider` → `App` and desktop `src/main.tsx` → `FoundationApp`, native commands, storage schema and smoke harness. The separate desktop shell causes the missing seven screens and layout mismatch.
- Implementation plan: both runtimes render the existing `App`/layout/pages/CSS; a separate desktop provider adapts native SQLite snapshots and commands. Browser demo storage, authentication and simulated synchronization remain browser-only. Preserve native member/plan editing, period history, optimistic versions and commit-before-success behavior.
- SQLite currently implements members/plans/periods only. Expose all nine navigation destinations, with unavailable attendance/finance/inventory/settings operations disabled and clearly described, rather than enabling their browser demo writes. This milestone does not implement synchronization or resolve deployment topology.
- Existing runtime blocker may be resolved: current `ldd --version` reports glibc 2.44. Native launch still requires verification. No implementation or new feature acceptance is claimed at this inspection boundary.

Milestone 0 complete. Milestone 1a local foundation and the shared full-interface integration are implemented. Day 1 remains partial; this is not a production release. Current checks and the sandbox display blocker are recorded below. The older glibc blocker and stop handoff are retained as history.

## Interface integration — implementation and checks

### Changes

- Desktop and browser now render the same `App`, sidebar, topbar, nine page components and existing black/amber styles. Removed the separate two-page `FoundationApp`. One navigation registry supplies both sidebar destinations and screen components; desktop starts on Dashboard.
- Split the shared context contract from `BrowserGymProvider` and added `DesktopGymProvider`. Only the browser entry dynamically loads demo seeds/localStorage and simulated synchronization; desktop uses the existing native SQLite snapshot and commands. Desktop storage failures do not substitute demo data. The SQLite schema, database path and transactions are unchanged; there is no automatic import, replacement or migration of browser data.
- Adapted shared member/plan forms for native asynchronous saves. Preserve exact minor-unit plan prices, unique normalized cards, expected edit versions, visible errors with forms retained, and success only after native commit. Desktop Members shows derived plan/expiry/status and a membership-dates modal with history; new members have no invented plan/expiry. Desktop Memberships supports adding/editing plans and native derived counts. Dates remain explicit and inclusive, with overlap rejection.
- Desktop topbar/dashboard show real pending/audit counts and the Asia/Colombo business date. Removed demo charts, automatic-sync claims and fake administrator identity from desktop. Unsupported financial totals read “Unavailable.” No pending operations are acknowledged without a server.
- All requested screens are exposed. Native operations absent from SQLite are disabled and explained; desktop Settings cannot invoke the browser JSON backup/restore or simulated success flows.

### Screen coverage

| Navigation item | Desktop behavior |
| --- | --- |
| Dashboard | Shared KPI/chart/list/quick-action layout; SQLite member/expiry summaries and pending counts. Attendance/income are unavailable; no seeded trend. |
| Members | SQLite add/edit, search, card linking, derived membership details, explicit membership dates/history. |
| NFC Attendance | Existing scanner/activity layout renders. Recording and manual check-in are disabled; SQLite attendance/admission rules and reader acceptance are missing. |
| Memberships | SQLite plan creation/editing, LKR minor-unit prices, active counts from dates; historical names/prices preserved. |
| Payments | Existing payment table renders. Receiving payments is disabled; payment storage, balances and receipts are missing. |
| Sales & Inventory | Existing product/stock table renders. Sales and adjustments are disabled; products, sales and stock ledgers are missing. |
| Expenses | Existing expense table renders. Expense creation is disabled; native expense storage is missing. |
| Reports | Existing report cards/summary render. Real member/audit counts; unsupported totals labeled unavailable. Detail/export/audit viewer are missing; export buttons disabled. |
| Settings | All seven tabs render with explicit native limitations. Profile persistence, accounts/roles, reader/printer checks, validated SQLite backups, backend sync and updates are missing. |

### Checks

- Final `npm run desktop:build -- --offline`: **PASS**. TypeScript/Vite (47 modules) and normal Linux Tauri executable with embedded assets; final binary does not contain `ui-smoke` hooks. Includes required `npm run build` baseline. No installer or Windows artifact was built.
- `npm run test:core -- --offline`: **PASS**, nine real SQLite business/integrity tests, including reopen persistence, card uniqueness, rollback, stale edits, membership dates/history, minor-unit validation and corrupt/newer storage preservation.
- `npm run test:ui`: **PASS**, four adapter tests plus actual React rendering of every desktop navigation route, all seven desktop Settings panels, loading/storage-error states, fail-closed runtime detection, and separate browser demo login/provider/all nine routes. Tests run without a display; these are route/component rendering checks, not clicked webview acceptance.
- Expanded `tests/desktop-smoke.js` for all nine navigation clicks/settings tabs, browser-storage access rejection, native add/edit/error cases, exact currency, historical snapshots, stale edits across refresh, pending/audit counts and process restart. Both smoke-script syntax checks **PASS**.
- `npm run test:desktop`: frontend and `ui-smoke` executable compilation **PASS**, native launch **FAILS before UI execution**: `Failed to initialize gtk backend!` / `Failed to initialize GTK`, exit 101. This sandbox cannot use the advertised display. Host glibc now reports 2.44, so the previous missing-GLIBC_2.44 error is resolved. A headless Chrome probe also fails on sandbox socket restrictions (`setsockopt: Operation not permitted`); no interactive/visual acceptance is claimed.

### Next steps / limits

Run `npm run test:desktop` in a Linux session with usable GTK/display access, then `npm run desktop:build -- --offline` to restore the normal binary. This verifies real navigation clicks, forms/IPC and restart together. Windows compilation/installer, pixel-level visual comparison and NFC/printer acceptance remain unperformed. Resolve identity/topology/business policy and complete native operations before using real gym data; this UI milestone does not implement synchronization or the remaining business modules. Browser demo persistence, simulated sync and receipt/export/profile toasts remain demo behavior.

## Baseline feature coverage (before milestone 1a)

Evidence paths below are under `Client/src/`; behavior is established by source inspection, not browser/hardware acceptance.

| Area | Present in code | Missing or demo behavior |
| --- | --- | --- |
| Shell/search | React navigation, member search/detail, black/amber styles. | No Tauri/Rust/installer. Version labels disagree: login v0.6, settings v0.4, package 0.1.0. |
| Members | Add/edit, search, plan/expiry fields, card capture input and UI duplicate check (`pages/MembersPage.tsx`). | No database uniqueness, robust validation, archive/history; random member IDs can collide. |
| Memberships | Edit seeded plans/prices/durations/status. | No plan creation or actual renewal workflow/history; members reference plan names; stored active-member counts are not calculated despite UI claim; statuses do not follow dates. |
| Attendance | Card text/HID-style Enter input and manual selection; alternates check-in/out from last event. | Hardware untested; no native reader integration, scan debounce or expired/frozen admission rule. “Today” activity includes older records; UTC day boundaries are wrong for local business dates. |
| Payments | Add member payment, amount/method and list. | Always records new payments as Paid; no invoice/balance allocation, renewal linkage, receipt rendering/printing/reprint or reversal. “Receipt ready” is only a toast. |
| Sales/inventory | Single-product sale, local stock deduction, low-stock labels and ±1 adjustment. | No product creation/editing, sale history UI, stock ledger/reasons, returns, durable transaction or concurrent stock protection. |
| Expenses | Add/list category, amount and method. | Hard-coded actor, no correction/reversal workflow or period filters. |
| Reports/dashboard | Some totals derived from local arrays. | Report clicks only show toasts; no detail/filter/export. Attendance KPI uses fixed 2026-09-01, chart is static, expiry uses stored status; income KPI omits retail sales. |
| Users/security | Hard-coded login check and sessionStorage flag (`App.tsx`, `pages/LoginPage.tsx`). | No real accounts/authentication/authorization; roles are static settings text; actor/profile is hard-coded. |
| Audit | Mutation metadata appended locally in `context/GymContext.tsx`. | No audit viewer, before/after history or trustworthy identity; restores bypass audit. |
| Backups | JSON download and JSON file restore (`services/storage.ts`). | Restore merely casts parsed JSON; no schema/integrity checks, migration, pre-restore recovery, atomic restore or safe sync reconciliation. |
| Storage/sync | Entire state in localStorage; local queue; browser connectivity indicator. | No SQLite/API/cloud. `services/sync.ts` waits 650 ms and returns IDs without network I/O. Records marked synced from connectivity alone. |
| Settings | Tabs and backup/sync controls. | Gym-profile save only toasts; printer/roles are descriptions; reader-ready/update-current claims are not device/server checks. |

## Priority integrity findings

- `GymContext.syncNow` ignores returned acknowledgement IDs and empties the entire current queue after awaiting; edits arriving during sync can be discarded from the queue and falsely marked synced.
- Automatic sync runs on connectivity changes, not each new queue item. Even the simulation does not provide continuous synchronization.
- Persistence happens in a React effect without error handling. `loadData` silently substitutes demo seeds on malformed data; there is no database recovery or migration.
- Money uses unrestricted JS numbers, validation is mostly HTML-only, audit actors are fixed, and stock changes lack a transaction/ledger. UI duplicate-card validation is not a durable constraint; attendance matching also differs in normalization.
- Backups can replace state with arbitrary parsed JSON. Production must not silently import seeds or accept invalid restored records.

## Changes and checks

- Created `AGENTS.md`, `PLAN.md`, `STATUS.md` (now located in `Client/`); application source/config/dependencies unchanged. Existing build regenerated `Client/dist` artifacts.
- `cd Client && npm run build` **PASS**: TypeScript build plus Vite 8.2.2 production bundle, 37 modules. Environment: Node v25.8.2, npm 11.12.1; used existing dependencies, no clean install performed.
- No test script or test suite found. No business tests added for this documentation-only milestone; required test cases are in `PLAN.md`.
- No browser interaction, Windows build, NFC/printer, offline crash recovery or real-backend verification performed. Build success establishes compilation/bundling only.
- `git status --short` failed: supplied workspace has no usable Git repository metadata. No commit or tracked-diff verification possible. Dependency declarations use `latest`; pin a supported toolchain/dependencies when infrastructure work begins.

## Original planning blockers

Await topology, hardware/access, hosting and core business-policy decisions listed in `PLAN.md`. Architecture is proposed, not approved or implemented. Highest priority is whether multiple PCs must edit simultaneously/offline.

Next milestone: resolve those decisions, then implement and test Tauri/SQLite transactions, identity and the first member/membership workflow; establish Windows build and backend access early. Full scope in three days is high risk; release requires the explicit acceptance gates in `PLAN.md`.


## Milestone 1a — local foundation

### Changes

- Added Tauri 2 shell using the existing black/amber CSS and modal components. Desktop opens the foundation UI; ordinary `npm run dev` remains the labeled browser demo. No npm dependencies added; Rust dependencies are pinned with `Cargo.lock`.
- Added normalized SQLite plans, members and membership periods plus device metadata, audit and outbox. Versioned migration, WAL/FULL durability, foreign keys, immediate write transactions and consistent read snapshots. Fresh storage contains no business/demo records; corrupt/newer storage is rejected without fallback.
- Completed desktop forms → typed native commands → SQLite → refreshed lists for plan/member creation and editing and membership-period entry. Save errors remain visible and forms stay open; success follows commit. Member edits use expected versions; card IDs are trimmed/normalized and uniquely constrained. Empty card IDs remain allowed.
- Plan prices use integer LKR cents; membership history snapshots plan name/price. Dates are explicit and inclusive, overlaps are rejected, status and active-member counts are derived using Asia/Colombo dates. No automated renewal or payment creation.
- Audit/outbox writes are atomic with business changes. Outbox remains pending; no sync simulation runs in desktop mode. No browser data import, restore or finance screens are exposed in this desktop slice.
- Added Rust business/integrity tests and an opt-in native webview smoke-test build. Test hooks and isolated data path override are compiled only with `ui-smoke`; normal desktop builds do not contain them.

### Checks

- `npm run build`: PASS (TypeScript + Vite, 40 modules).
- `npm run test:core`: 9 business/integrity tests passed (file reopen persistence; duplicate/blank cards; injected outbox failure rollback; stale edit rejection; overlap/date/foreign-key rejection; plan history snapshots; Sri Lanka midnight and inclusive expiry boundaries; derived counts/validation; corrupt/newer database preservation).
- `cargo clippy --manifest-path src-tauri/Cargo.toml --offline --locked --lib --tests -- -D warnings`: PASS. Rust formatting and both test-script syntax checks passed.
- `npm run desktop:build -- --offline`: PASS. Final normal executable rebuilt without smoke-test hooks or compiler warnings.
- `npm run test:desktop`: frontend and smoke-harness compilation PASS, but native launch **BLOCKED/FAILED before UI execution**. WebKitGTK/JavaScriptCore require `GLIBC_2.44`; host `ldd` reports glibc 2.43 and the loader reports missing `GLIBC_2.44` in `/usr/lib/libm.so.6`. No native form, IPC or process-restart UI acceptance is claimed. The host needs compatible glibc/WebKit packages; no system packages were changed.
- Only `x86_64-unknown-linux-gnu` Rust target is installed. Windows compilation, packaging and hardware checks remain unperformed.
- Dependency download attempt: npm registry DNS `EAI_AGAIN`. Used cached Tauri/SQLite dependencies offline; no weak replacement for unavailable password-hashing dependency.

### How to test

First resolve the host glibc/WebKit mismatch (the current environment cannot launch the app). Then, from `Client/`, run `npm run desktop:run`. This builds and launches the native debug executable with embedded frontend assets; no Vite server or Tauri CLI is required. Rust and Tauri's OS prerequisites are required. GTK 3 and WebKitGTK 4.1 development libraries compiled successfully here, but their runtime dependency mismatch blocks launch.

1. In Memberships, create a test plan with price and duration.
2. In Members, add/edit a test member; enter a card ID. A second member with the same card (including different letter case) must fail without adding a record.
3. Add membership start/end dates. Overlapping periods must fail; plan edits must preserve historical name/price.
4. Close/relaunch the app. Records and pending counts must remain. Use Refresh after a stale-edit rejection.

Database: Tauri app-data directory `lk.armstrong.fitness/armstrong.sqlite3` (Linux normally `~/.local/share/lk.armstrong.fitness/`; Windows normally under `%APPDATA%`). It is separate from browser localStorage. Keep test data only: authentication, encryption and backup/restore are not implemented. Do not copy only the main SQLite file while running; WAL sidecars may contain committed records.

### Remaining limitations / next boundary

- Day 1 is **partial**: no accounts/roles, secure offline unlock, online identity/API/backend skeleton, Windows build/installer or hardware acceptance. The test UI intentionally has no login; operator access is unrestricted and audit actor is explicitly unauthenticated.
- Membership policies and topology remain unanswered. Dates are entered manually; no renewal calculator, freeze, correction/reversal or linked payment. The seven-day expiring indicator is provisional.
- No attendance, payments/receipts, inventory/sales, expenses, reports, audit viewer, backups or real synchronization in this desktop slice. Those legacy browser screens remain demonstrations.
- Stop at this test boundary. Next: repair/provide a compatible desktop runtime, rerun `npm run test:desktop`, then owner tests local member/membership flow and answers deployment/identity/hosting questions; then finish Day 1's remaining gates before starting Day 2. The three-day full scope remains high risk.


## Stop handoff — 2026-10-03

Stopped at the user's request. No operation was running when the stop request arrived. This handoff changes documentation only; no implementation or tests were started or repeated. Completed changes, prior verification results and remaining work are recorded in Milestone 1a above. Day 1 remains partial.

### Exact runtime blocker

The native executable exits with status 1 before the UI starts. The loader reported:

```text
/usr/lib/libm.so.6: version `GLIBC_2.44' not found (required by /usr/lib/libwebkit2gtk-4.1.so.0)
/usr/lib/libm.so.6: version `GLIBC_2.44' not found (required by /usr/lib/libjavascriptcoregtk-4.1.so.0)
```

`ldd --version` reported `ldd (GNU libc) 2.43`. Compilation/linking success does not establish runtime compatibility. No host libraries were modified and no native UI test passed.

### One precise next task and resume commands

**Unblock and verify the existing native UI → SQLite → process-restart smoke test on a compatible Linux runtime; make no new feature changes.** First have the host environment provide mutually compatible glibc, WebKitGTK and JavaScriptCore packages, or use a compatible development environment. The package-manager repair command is intentionally not guessed; do not manually replace or symlink individual system libraries.

When explicitly resumed, run these commands in order. If native launch still fails, stop and record the error. After the smoke test passes, rebuild the normal executable to remove smoke-test hooks, then record the result here.

```bash
cd /home/prinzz/development/Projects/ArmStrong/Client
ldd --version
npm run test:desktop
npm run desktop:build -- --offline
```

Acceptance: both native smoke-test launches exit successfully, verifying form saves, duplicate-card rejection, exact minor-unit price storage, and member/membership/audit/outbox persistence after process restart. Do not repeat already-passing core tests unless code changes justify it. Authentication/backend decisions, Windows verification and remaining features listed above follow this task; they are outside this resume boundary.
