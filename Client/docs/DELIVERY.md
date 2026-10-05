# Acceptance-build delivery

The approved account is `armstrong@gmail.com`, Administrator **ArmStrong**, gym
**ArmStrong Fitness**. Production has onboarding and protocol-2 migrations 1–4.
The installed app covers local gym operations and all-module synchronization,
with seven-day OS-vault offline access and native session renewal. One approved
computer edits; additional computers download shared records.

This remains an acceptance build. General conflict review,
large-database/legacy bootstrap and real Windows/network/hardware acceptance are
open. See [current status](../STATUS.md).

## Publish the tested source

Earlier changes are committed on GitHub main at
`ef1faa0a729db3c74aa21c8c4dd5d6a498bbc9df`. Tracked `server/.env` is removed;
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

Deploy the updated source in the existing Render service, root directory
`server`. Preserve the restricted runtime `DATABASE_URL`, verified TLS and
secret CA `/etc/secrets/hi3.crt`.

```text
Build: npm ci --include=dev --ignore-scripts --no-audit --no-fund && npm run build:verify
Start: npm start
AUTOMATIC_DEVICE_ENROLLMENT=true
```

Migrations 3 and 4 are already applied to production, with exact source checksums
and preserved existing records. Startup checks required business tables, narrow
runtime column permissions and immutable guards; it never runs owner migrations.
Real production runtime TLS/catalog checks pass. Independently observed
`/health` returns the existing protocol-1 compatibility body. After deployment,
`/v2/health` must return `{"status":"ok","service":"armstrong-gym-api","protocolVersion":2}`.
This identifies the all-module endpoint; actual login/data acceptance is separate.

`server/armstrong-render-source.zip` is the verified standalone API source
snapshot. It includes migrations 1–4, the business row manifest and vendored
types. Extract its `server/` folder into a separate source checkout if needed.
See [computer setup](../../server/docs/AUTOMATIC_COMPUTERS.md) and
[business protocol](../../server/docs/BUSINESS_SYNC.md).

## Install Arch or build Windows

For Arch x86_64, copy the package and `SHA256SUMS` from `Client/dist-linux/`:

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

For Windows, run **Actions → Windows installer → Run workflow** after publishing
the updated source. Download `ArmStrong-Fitness-Windows-x64` from a successful
run. The workflow runs native business, interface and cross-language contract
checks before building the NSIS setup. No Windows executable exists yet.
See [Windows guide](WINDOWS_INSTALLER.md).

Same-computer backup recovery reconciles an isolated copy during online sign-in
before unlocking. See [recovery and limits](RESTORE_RECOVERY.md).

Complete live native HTTPS/outage/reconnect/restore, general conflict recovery, Windows
install/upgrade, NFC/physical printing and exposed-credential rotation before
calling this a final release.
