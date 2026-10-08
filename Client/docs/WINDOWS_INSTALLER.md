# Windows desktop delivery

Windows is the required release platform. Linux is optional for development and
later distribution. The target user flow is install, open, sign in with the one
Administrator account and work with the same gym data. New computers need
internet for first sign-in and initial data download. Network outages must retain
local changes for real synchronization after reconnect.

## Current packaging milestone

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
business/authorization and interface checks, then uploads only the setup
executable as `ArmStrong-Fitness-Windows-x64` for seven days. It requests only
repository read permission, uses pinned action commits, and does not publish a
release or deploy the API. Pull requests run it; once merged, it also supports
manual **Run workflow**. The first actual Windows run and artifact are required
before claiming Windows build acceptance.

The connected GitHub integration currently rejects content writes with
`403 Resource not accessible by integration`; no branch/PR/run has been created.
Use `armstrong-desktop-changes.zip` from a writable checkout, or give that
connection contents/workflow write access. The bundle's patch preserves the
latest inspected main at `ef1faa0a729db3c74aa21c8c4dd5d6a498bbc9df` and contains
no server env file or old secret values. Earlier packaging/onboarding changes
and removal of tracked `server/.env` are already committed on that baseline.

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
`Client/dist-windows/ArmStrong-Fitness_0.1.0_x64-setup.exe`, rebuilt on
2026-10-08 (222,072,198 bytes / 211.78 MiB), with `SHA256SUMS` and
`BUILD-INFO.txt`. Its SHA-256 is
`3aee104fdcf710c0895dde57736709fc38920b3f66fe046ae8d8c1fb654b4e19`.
This unsigned Linux cross-build includes Staff/monthly training, combined
collection, salary payouts, Administrator editing access, the Arm logo, compact
login/dashboard, staff NFC/manual attendance, male/female member counts, permanent
operational removal and retained-transaction review/retry, plus the earlier membership,
sign-in, Windows HTTPS and startup fixes. Its x64 native app, static runtime
imports, installer integrity, exact embedded app/WebView2 payloads, Staff
migration 9, recovery IPC and exclusion of UI smoke hooks are verified. Shared Staff data
requires server migrations 6–7 and the matching updated API; this build does not
deploy them. The Windows Actions workflow also runs a credential-free
native HTTPS probe against the public Auth and gym API health endpoints before
packaging. This check depends on those services being reachable from the runner;
it does not authenticate an account or enroll a device. Windows installation and
runtime acceptance remain open.

The current source has since added native schema 10, permanent staff deletion
from active/inactive lists and a larger NFC card panel. The existing Windows
installer above does not include those subsequent changes. Rebuild from the
current source and deploy the API supporting migrations 6–8 before shared use.

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
Automatic updates use manual package upgrades.
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
