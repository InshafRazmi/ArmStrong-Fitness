# Acceptance-build delivery

The approved account is `armstrong@gmail.com`, Administrator **ArmStrong**, gym
**ArmStrong Fitness**. Production has onboarding and protocol-2 migrations 1–8.
The installed app covers local gym operations and all-module synchronization,
with seven-day OS-vault offline access and native session renewal. Active
Administrators can edit from any valid enrolled computer. Database migration 5
replaces the old single-writer policy; existing read-only sessions require fresh
online sign-in to receive editing access.

The current source adds Staff with NIC/mobile, fixed monthly salaries,
per-member monthly training fees, trainer selection, combined collection and
salary-plus-collected-fee payouts. Native schema 10 preserves earlier rows and
pending operations. Staff NFC/manual attendance, separate staff dashboard activity,
Male/Female member profiles and daily counts, permanent operational member removal
with retained history, and review/retry for blocked transactions are included.
Login and dashboard have compact layouts with short scrolling at small heights. See [staff workflow](STAFF.md).
Permanent staff deletion works from active/inactive lists, keeps history and
final payouts in Show deleted staff, and prevents reactivation. NFC Attendance
has a larger amber card panel and scan controls. Server migrations 6–8 are
applied to production; existing application data fingerprints are unchanged.
The current Render API still needs the updated source deployed.

This remains an acceptance build. General conflict review,
large-database/legacy bootstrap and real Windows/network/hardware acceptance are
open. See [current status](../STATUS.md).

## Publish the tested source

The current source patch is based on workspace HEAD
`6cb5aef06bdc9a36f53872ff638e9663156c9b6c`. No commit/push is performed by this
update. Tracked `server/.env` is removed;
its historical exposed credentials still require rotation.

The all-module changes are in `Client/armstrong-desktop-changes.zip`, with a
binary-capable Git patch, review copies and SHA-256 manifest. The patch was
checked, applied to that exact baseline and compared with the supplied source.
ZIP CRC and hashes were verified. Extract outside the checkout, preserve other
work, then run:

```sh
git apply --check /path/to/extracted/changes.patch
git apply /path/to/extracted/changes.patch
```

Review, commit and push through the repository's normal workflow. The connected
GitHub app rejects content writes with 403 and the local CLI token is invalid;
no source publication or Windows workflow run is claimed for this change.

The user will handle the commit and push from this workspace. The patch is an
optional transfer/review artifact; do not apply it on top of these same changes.

Rebuild source bundles with `python3 scripts/build-delivery.py`. Private env,
certificates, credentials, SQLite and build outputs are excluded.

## Deploy the API

Before deploying the new Staff API, run `npm run migrate` from the updated
`server` source in a controlled administrative session using the owner connection
and verified TLS. The checksummed runner verifies existing migrations 1–5 and
applies migrations 6–8 atomically. These are already applied to production and
repeat runs verify their exact checksums. Keep owner credentials outside the deployed
service. Startup checks require the new Staff/attendance allowlist, shared card uniqueness and NIC/history guards;
it cannot start this source against the earlier schema. Upgrade all gym desktops
before creating Staff records. Existing older clients retain their queues and
refuse unknown Staff rows.

Deploy the updated source in the existing Render service, root directory
`server`. Preserve the restricted runtime `DATABASE_URL`, verified TLS and
secret CA `/etc/secrets/hi3.crt`.

```text
Build: npm ci --include=dev --ignore-scripts --no-audit --no-fund && npm run build:verify
Start: npm start
AUTOMATIC_DEVICE_ENROLLMENT=true
```

Migrations 1–8 are already applied to production, with exact source checksums
and preserved existing records. Migration 5 updates the enrollment function used
by the existing automatic endpoint, so Administrator permission refresh needs
online sign-in rather than an API restart. The additional role-based request
checks remain in local source until it is published/deployed. Startup checks
required business tables, narrow
runtime column permissions and immutable guards; it never runs owner migrations.
Real production runtime TLS/catalog checks pass. Independently observed
`/health` returns the existing protocol-1 compatibility body. After deployment,
`/v2/health` must return `{"status":"ok","service":"armstrong-gym-api","protocolVersion":2,"businessSchemaVersion":10}`.
The older response without businessSchemaVersion does not establish support
for current Staff/attendance/removal rows. Render remains on that older response.
After the new deployment, open Settings → Server synchronization, review the
retained transaction and retry its original request. Actual desktop acceptance is separate.

`server/armstrong-render-source.zip` is the verified standalone API source
snapshot. It includes migrations 1–9, the admission settings business row manifest and vendored
types. Extract its `server/` folder into a separate source checkout if needed.
See [computer setup](../../server/docs/AUTOMATIC_COMPUTERS.md) and
[business protocol](../../server/docs/BUSINESS_SYNC.md).

## Install Arch or build Windows

The Arch x86_64 package in `Client/dist-linux/` was rebuilt on 2026-10-08 with
all the latest confirmed changes, including schema 10, permanent staff deletion,
larger NFC card/controls, staff NFC attendance,
gender counts, permanent operational removal and compact layouts. Package
contents, current-release payload, library resolution and checksum are verified.
Copy the package and `SHA256SUMS` from that directory:

```sh
sha256sum -c SHA256SUMS
sudo pacman -Syu
sudo pacman -U ./armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst
armstrong-fitness
```

Use an unlocked persistent Secret Service collection. First login/download needs
internet and the updated API. Continue offline uses the OS user's cached grant
for up to seven days; logout removes it. An offline restart requires online
sign-in before synchronization resumes. Linux webview local forms and restart
passed; production login/keyring/network acceptance remains separate.

For Windows, the current local installer is
`Client/dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe`, rebuilt on
2026-10-08 with Staff and the latest interface changes. Copy it with the adjacent
`SHA256SUMS` and `BUILD-INFO.txt`. This unsigned Linux cross-build includes
WebView2 for offline installation; Windows runtime acceptance remains open.
The existing Windows installer predates the new schema-10 staff deletion and
larger NFC layout. Rebuild it from current source to include these changes.
Shared Staff/attendance/profile/removal synchronization requires migrations 6–8
(already applied to production) and the matching updated API.

For a Windows-runner build, run **Actions → Windows installer → Run workflow**
after publishing the updated source. Download `ArmStrong-Fitness-Windows-x64`
from a successful run. The workflow runs native business, interface and
cross-language contract checks before building the NSIS setup, including the
native Staff contract. See [Windows guide](WINDOWS_INSTALLER.md).

Same-computer backup recovery reconciles an isolated copy during online sign-in
before unlocking. See [recovery and limits](RESTORE_RECOVERY.md).

Complete live native HTTPS/outage/reconnect/restore, general conflict recovery, Windows
install/upgrade, NFC/physical printing and exposed-credential rotation before
calling this a final release.

## Admission-fee candidate (2026-10-11)

Source now supports a once-only configured admission charge at registration,
automatic membership invoices without a trainer, member dues/payment shortcuts
and the staff Pay salary shortcut. Existing registrations and invoices are not
backfilled or repriced. Zero remains the admission default until configured.

Before rolling out this candidate, apply server migration 9, deploy the matching
API (health reports businessSchemaVersion 11), and package/update editing
desktops with SQLite schema 11. Keep older recovery transaction bytes intact.
The regenerated standalone API source ZIP contains this candidate. Published
installers remain on the previous release until signed packaging and API rollout
are verified.
Live PostgreSQL migration, deployment, Windows interaction and publishing are
separate acceptance steps; local financial, restore and protocol checks do not
claim those steps.

Production admission migration 9 was applied and verified on 2026-10-11. The
129 existing business records retained their exact pre-migration fingerprint;
RLS, grants and history protections were preserved. The remaining rollout steps
are deploying API schema 11 and packaging/updating the editing desktops.
