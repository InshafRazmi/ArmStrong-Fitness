# Windows desktop delivery

Windows is the required release platform. Linux is optional for development and
later distribution. The target user flow is install, open, sign in with the one
Administrator account and work with the same gym data. New computers need
internet for first sign-in and initial data download. Network outages must retain
local changes for real synchronization after reconnect.

## Current packaging milestone

Signed Windows version [0.1.9](https://github.com/InshafRazmi/ArmStrong-Fitness/releases/tag/v0.1.9)
is published for the application updater. It adds five-second automatic sync and
dashboard refresh, compact member payment actions, smaller invoice checkboxes
and shorter interface text. It includes configurable admission
fees, automatic registration invoices, member dues/Receive payment shortcuts and
Pay salary. Production migration 9 is applied and API health reports schema 11.
The Windows CI build and installer signature/payload checks pass. The public
updater feed matches the verified manifest. SHA256SUMS and VERIFICATION.json are
attached to the release. Physical Windows installation and hardware interaction
remain separate acceptance; older packages described below are historical builds.

`npm run desktop:windows:check` validates the three public connection fields in
`desktop-auth.production.json`. It refuses SQL credentials, secret/service keys,
unknown or duplicate fields and noncanonical/non-HTTPS origins. The installer
compiles these public settings into native code; users do not create a settings
file on each computer. Existing matching workstation settings are retained;
different or invalid settings keep access locked for Administrator review.

The installer command enables `packaged-auth`, `desktop` and `custom-protocol`.
Production Windows builds without packaged Auth, or combining packaged Auth with
the UI smoke commands, fail compilation. Fresh packaged installs require verified
login before any business reads/writes. No demo account or server password is
packaged. Local SQLite, existing identities and pending operations are preserved.
Installer builds use the desktop Vite mode, excluding browser-demo credentials,
sessions and data storage. The package check rejects demo code in compiled assets;
a missing native bridge leaves access locked with a visible error.

NSIS produces an x64 setup executable for Windows 10/11. The per-user install
uses an embedded WebView2 offline installer, so installation does not depend on
downloading WebView2 on the destination computer. This increases installer size;
the build runner still needs internet. `Client/.cargo/config.toml` links the
Windows C runtime statically so the app does not require a separate Visual C++
redistributable. Verify the finished executable's imports after packaging;
see [Rust's runtime linkage documentation](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes).
See the official
[Windows installer and WebView2 options](https://v2.tauri.app/distribute/windows-installer/).

## Build and download

The repository is
[InshafRazmi/ArmStrong-Fitness](https://github.com/InshafRazmi/ArmStrong-Fitness).
The `Windows installer` Actions workflow uses a Windows runner, Node 24, Rust
1.98.0, the locked frontend/Rust dependencies and Tauri CLI 2.12.1. It runs native
business/authorization and interface checks. Pull requests and manual runs upload
the setup executable as `ArmStrong-Fitness-Windows-x64` for seven days. Pushing
a `v*` tag builds and publishes the signed updater installer and `latest.json`
to GitHub Releases using the configured signing secret. The workflow requests
repository contents write permission for publishing and uses pinned action
commits. It does not deploy the API. Windows installation/runtime acceptance
remains required after the build.

On a Windows x64 developer machine with Rust/MSVC and Node installed:

```powershell
cargo install tauri-cli --version 2.12.1 --locked
npm ci
npm run desktop:windows:check
npm run desktop:windows:build
```

Run these from `Client/`. The setup executable is written to
`src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`. The public settings
are already configured for the selected Render/Supabase endpoints. Ordinary
`desktop:run` is still the explicit local development/test workflow.

The current local source has also produced
`Client/dist-windows/v0.1.6/ArmStrong-Fitness_0.1.6_x64-setup.exe`, built on
2026-10-09 (224,952,155 bytes / 214.53 MiB), with its updater `.sig`,
`SHA256SUMS`, `BUILD-INFO.txt` and `VERIFICATION.json`. Its SHA-256 is
`236e433eebc33f26bc093b92d25c55b8ad91e6359ba2ba31d9814c3af88d158d`.
Source commit: `f9a119a984c3dc54b43085a1b8b411c081e0b1c0`.
This Linux cross-build adds guarded recovery for an unchanged retained
installation profile. It also includes schema 10 staff deletion/history, the larger NFC
panel, male/female attendance counts, retained transaction review/retry and the
signed Windows updater, plus the existing membership/finance/sign-in features.
The updater signature is verified with Tauri's minisign verifier. NSIS integrity,
the eight-file payload allowlist, the current app after the expected three-byte
NSIS bundle marker patch, exact cached WebView2 payload, x64 PE/static runtime,
native feature markers and exclusion of smoke hooks/private signing key pass.
No Windows Authenticode publisher certificate is configured. Windows installation
and runtime acceptance remain open. Shared staff data requires server migrations
6–8 and the matching deployed API reporting businessSchemaVersion 10; this build
does not deploy the backend or retry retained transactions.

### Linux cross-build fallback

When a Windows builder is unavailable, Tauri supports an MSVC x64 NSIS
cross-build using `cargo-xwin`, LLVM/Clang, LLD, NSIS and the Windows Rust target.
See the [official cross-build instructions](https://v2.tauri.app/distribute/windows-installer/#build-windows-apps-on-linux-and-macos).
Use Rust 1.98.0, Tauri CLI 2.12.1 and `cargo-xwin` 0.23.1 for this build:

```sh
rustup target add x86_64-pc-windows-msvc
npm run desktop:windows:check
npm exec --yes --package=@tauri-apps/cli@2.12.1 -- tauri build --ci \
  --config src-tauri/tauri.windows.conf.json --runner cargo-xwin \
  --target x86_64-pc-windows-msvc \
  --features desktop,custom-protocol,packaged-auth --bundles nsis -- --locked
```

Run from `Client/` after installing the prerequisites. The first build downloads
and extracts Microsoft's SDK and the embedded WebView2 installer. Keep build
tools and caches in ignored directories. The output is the same NSIS directory
listed above. A cross-build still requires installation and runtime acceptance
on Windows.

## Account, offline access and remaining milestones

`armstrong@gmail.com` is now registered as Administrator **ArmStrong** for
**ArmStrong Fitness**. The new private SQL function for account-authorized
computer enrollment is applied to production and the existing test project.
Deploy the updated API with `AUTOMATIC_DEVICE_ENROLLMENT=true`; ordinary computer
login then prepares its OS credential automatically and verifies the existing
Administrator/gym. See [server setup](../../server/docs/AUTOMATIC_COMPUTERS.md).
The first active computer can edit; later computers receive read-only access.
No device was fabricated by the SQL registration.

Verified online sign-in stores a separate seven-day OS-vault authorization grant.
Continue offline can unlock existing data after restart; signing out removes that
grant. No password/bearer/refresh token is saved to SQLite or that grant. The open
app rotates refresh tokens in native memory and re-verifies identity/enrollment
before renewing. Offline restart still needs online sign-in to start server sync.
Native gym scheduling retries every 30 seconds and on reconnect, respecting
durable backoff and server receipts. Logout cancels queued work and a durable
session nonce prevents late replies from committing after logout.

Protocol 2 covers attendance, finance, inventory, expenses, memberships, profile
and audit as atomic business transactions. The workflow verifies real native
envelopes against the server contract before installer creation. The isolated
Supabase Auth/PostgreSQL suite passes, and production migrations 1–4 are applied.
Deploy the updated source before native live acceptance. Pending/conflicting
transactions are retained. Same-computer online backup recovery is implemented;
live recovery acceptance, general conflict review and large/legacy bootstrap
remain open. See [restore recovery](RESTORE_RECOVERY.md).

### Signed automatic updates

Windows has a Tauri updater in Settings → Application updates. Opening the page
checks the latest GitHub Release; updates are signature-verified, and staff
choose when to install and restart. The updater replaces application files only;
gym records remain in the separate app-data SQLite database.

The missing public release previously caused `Could not fetch a valid release
JSON from the remote`. On 2026-10-09, public version 0.1.6 was published with the
matching signed installer, `.sig`, `latest.json`, `SHA256SUMS` and verification
records. The configured endpoint returns HTTP 200 and valid version-0.1.6 JSON.
Updater-enabled older installations can open **Settings → Application
updates → Check for updates → Install 0.1.6 and restart**. The feed embeds the
signature for that exact installer and uses the existing application public key.
Later updates need a higher version and a newly built, signed package.
See [Tauri's static update manifest](https://v2.tauri.app/plugin/updater/#static-json-file).

For `business_revision_conflict`, sign in online as the original Administrator
and open **Settings → Server synchronization → Review retained transaction**.
If the native review offers **Recover using server profile**, check the
confirmation and select it. This path creates a validated backup, retains the
original refused transaction and audits, and downloads complete verified shared
history. Keep the displayed backup path and sign in online again after recovery.
Edited profiles or other business data remain guarded for separate review.
See [the recovery limits and steps](RESTORE_RECOVERY.md#retained-installation-profile-recovery-016).
Version 0.1.5's unused-computer download fix remains included, and its published
tag and assets are unchanged.

The signing keypair is backed up outside the repository at
`/home/prinzz/.tauri/armstrong-fitness/updater.key` and `updater.key.pub`.
The backup directory has mode 700 and both files have mode 600. The saved files
match the originals, and the public key matches `src-tauri/tauri.conf.json`.
This persistent backup survives clearing `/tmp`; do not commit or share the
private key. Keep this key for future releases. It has no password, so
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` can remain unset.

For future releases (the signing secret is already configured and `v0.1.6` is
already published):

1. Open [repository Actions secrets](https://github.com/InshafRazmi/ArmStrong-Fitness/settings/secrets/actions)
   and select **New repository secret**. Name it `TAURI_SIGNING_PRIVATE_KEY`,
   paste the complete contents of the backed-up private key into **Secret**, and
   select **Add secret**. Use the key contents, not its path or the `.pub` file.
   Alternatively, with GitHub CLI installed and signed in, run the command below.
2. Commit and push the prepared source changes. Before each release, select an
   unused semantic version and keep `Client/package.json`,
   `Client/src-tauri/Cargo.toml` and `Client/src-tauri/tauri.conf.json` consistent;
   refresh their lock files. The current version is `0.1.6`.
3. Push the matching unused `v<version>` tag after committing the source. Keep
   existing release tags unchanged. The Windows workflow also accepts an
   existing `release_tag` for rebuilding with a corrected workflow; its tests
   still come from that selected source tag.
4. Wait for **Actions → Windows installer** to pass. Download the setup `.exe`
   from the resulting [GitHub Release](https://github.com/InshafRazmi/ArmStrong-Fitness/releases)
   and install it once on each Windows computer. Future releases with higher
   versions can be installed from **Settings → Application updates**.

```sh
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo InshafRazmi/ArmStrong-Fitness < /home/prinzz/.tauri/armstrong-fitness/updater.key
```

The [GitHub CLI command](https://cli.github.com/manual/gh_secret_set) reads the
private key directly from the file without printing it. The Windows workflow
builds the signed NSIS updater package and publishes `latest.json` to the release.
Pull requests continue using unsigned acceptance installers and do not need
signing secrets. Existing installs only receive updates after they have installed
a build that includes the updater and its public key.

The build is not the final all-features release.

Windows acceptance must cover fresh installation, real Credential Manager
readback, online login, offline saves/restart, reconnect without duplicates,
logout/revocation, upgrades preserving data, NFC/manual attendance and printing.
The initial artifact is for acceptance, not a completed production release.

## Exposed server credentials

Inspection found a configured `server/.env` committed in the public repository,
including a database password, Administrator probe password and privileged Auth
key. The tracked file is now removed on main and root ignore rules protect local
copies. Removing a file does not remove its history or revoke exposed credentials.

Reset the exposed database password in the Supabase project, reset the
Administrator account password and revoke its active sessions, and revoke/replace
the exposed privileged Auth key. Update private administrative settings afterward.
The separately prepared `armstrong_api` password was not in that tracked file;
do not replace the restricted Render login with the owner connection. Database
and privileged Auth keys never belong in the desktop configuration or build job.
See [Supabase API key rotation](https://supabase.com/docs/guides/getting-started/api-keys)
and [user password updates](https://supabase.com/docs/reference/javascript/auth-updateuser).
