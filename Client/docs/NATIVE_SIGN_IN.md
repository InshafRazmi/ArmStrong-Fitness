# Native Administrator sign-in

Windows is the required installed-app platform; Linux is optional. Windows builds
using `packaged-auth` include the approved three public connection settings, so
users do not need to create `desktop-auth.json`. Existing conflicting settings
are preserved and access stays locked. The configuration-file instructions below
describe the separate local development path. See
[Windows delivery and unfinished account/offline gates](WINDOWS_INSTALLER.md).
The approved Administrator is `armstrong@gmail.com`, display name **ArmStrong**,
gym **ArmStrong Fitness**. Account-based computer approval SQL is provisioned;
deploy the updated API and enable `AUTOMATIC_DEVICE_ENROLLMENT=true` before using
it. See [automatic computer setup](../../server/docs/AUTOMATIC_COMPUTERS.md).

This is implemented desktop wiring with local tests, **not live acceptance**.
The HTTPS API, approved registration, usable OS credential storage and a GUI
session are still required. Native member sync is wired to the HTTPS worker and
scheduler; other modules remain local. The separate browser
login is a demo and cannot grant a native session.

## Automatic preparation and the older-server fallback

Normal online login prepares this computer's OS credential and asks the API to
approve it against the existing verified Administrator/gym. One active computer
can edit; additional approved computers are read-only. Exact retries keep their
identity/hash/permissions, and revoked computers cannot register themselves again.
No caller-supplied role, gym or writer flag is accepted. The following manual
workflow remains for older deployed API versions and Administrator diagnostics.

In the desktop, open Settings → Server synchronization → **Prepare this computer**.
The same action is available under Computer registration on the native login page.
It reads the database's existing device UUID, generates 32 OS-random bytes once,
stores their 64-character hex encoding in OS credential storage, and verifies
readback before saving its SHA-256 hash in SQLite. Only the UUID, SQLite path and
hash are shown. It grants no permissions and creates no server registration.

Windows uses Credential Manager for the current Windows user, with local-machine
persistence. Linux development uses installed `/usr/bin/secret-tool` and the
Secret Service's persistent default collection. Unlock the OS credential store
when prompted. Missing/locked storage fails; there is no plaintext file fallback.
Missing or changed credentials for a prepared/bound database require administrator
reconciliation. Do not delete the hash or recreate a credential to bypass this.

Give the displayed SQLite path and hash to the server administrator. They belong
in the ignored runtime `server/.env` as `REGISTRATION_SQLITE_PATH` and
`REGISTRATION_DEVICE_SECRET_SHA256`. The registration command now refuses an
unprepared database or a hash differing from native readback. Follow
[the registration review/apply instructions](../../server/docs/STAFF_DEVICE_SETUP.md)
using the one existing Administrator account and approved gym. No second account,
secret in chat, test-schema reset or public deployment is needed.

## Provision native server settings

Copy the shape of [desktop-auth.example.json](../desktop-auth.example.json) into
`desktop-auth.json` beside the actual `armstrong.sqlite3` file. The preparation
screen shows that database path. Provision this file privately in your editor,
then restart the desktop. Do not use frontend env variables or copy server `.env`.

With approved settings and both actual device-preparation entries in the server's
runtime `.env`, `desktop:config:check` / `desktop:config:write` can provision
these three public fields beside the verified existing database. Review is
read-only; the separate write retains matching files and refuses conflicting
settings, duplicate fields or unsafe file targets.
See [the local setup guide](../../server/docs/LOCAL_SETUP.md).

- `authOrigin`: this project's canonical HTTPS Supabase origin, without a trailing
  slash, path, query or credentials.
- `apiOrigin`: the existing approved HTTPS Fastify API origin, distinct from Auth.
- `publishableKey`: the project's public publishable or legacy anon key. Never a
  secret/service-role key, SQL URL, password or device secret.

Settings are read by native code only, capped at 16 KiB, with unknown fields and
invalid keys/origins rejected. Invalid configuration locks access. Once configured
or bound, deleting configuration does not return the database to unrestricted
local-test mode. Existing unconfigured test databases retain the explicitly
labelled test workflow. Do not activate production configuration until the API
and registration are ready.

## Sign-in and session behavior

Enter the approved Administrator's email/password. Native code verifies password
Auth, separately verifies the identity online, and asks the API to verify the
actual device credential. Only then does it reconcile local roles, bind the
API/gym/device and record sign-in atomically. Caller IDs, decoded JWT claims and
browser demo state grant no access. A Reception account cannot operate this
single-Administrator desktop.

Every configured business IPC checks the current native account, expiry, active
Administrator role and device writer permission. Read-only approved computers
can browse/export while writes are refused. New records, audit entries and saved
receipt actor labels use the verified local actor. Existing history is retained.

Account and rotating refresh tokens remain in native process memory, outside
SQLite/backups/webview/OS offline grants. The open app checks for renewal every
30 seconds, starts two minutes before expiry, and re-verifies the online identity
and device enrollment before extending access. Denials lock access and remove
offline approval; outages do not extend a session. Restart requires online sign-in
to resume synchronization because refresh tokens are not persisted.

After verified login, a separate grant in the current OS user's credential store
permits Continue offline for seven days, including restart. It contains scoped
identity/permission/expiry data, not a password or server token. It checks the
existing device credential, database marker, API/Auth/gym scope, active local
Administrator, expiry and clock rollback. It never renews itself while offline.
Signing out invalidates the SQLite marker before clearing the OS item and cancels
queued sign-in/sync/renewal. A durable nonce stops late replies from committing.
Offline revocation cannot be checked until reconnect. An authenticated restore
locks the session; server reconciliation remains required before reuse.

## Native HTTPS implementation and checks

Requests invoke the OS curl executable directly: `/usr/bin/curl` on Linux and
the OS System32 `curl.exe` on Windows. No shell or user PATH lookup is used.
Credentials are supplied through stdin configuration, with curlrc/environment
overrides disabled. TLS uses system certificate/hostname verification and TLS
1.2 or later. Redirects/proxy credentials are disabled; request/response sizes and
both curl/native process deadlines are bounded. Errors discard provider bodies
and stderr. Missing/unsupported OS tools fail without granting a session.

References: [curl options](https://curl.se/docs/manpage.html),
[Windows curl](https://learn.microsoft.com/en-us/windows/curl/) and
[Windows credential API](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew).

Run the core/UI checks from `Client/`:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --offline --locked
npm run build
npm run test:ui
```

Two environment-dependent probes are deliberately excluded from the ordinary
suite; run them individually in a working normal terminal:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked installed_os_vault_read_only_probe -- --ignored --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --locked real_installed_curl_refuses_plaintext_as_tls_and_honors_exact_origin -- --ignored --nocapture
```

The first reads only a random synthetic credential target and never creates a
secret. Here it observed unavailable/locked storage with a redacted error. The
second needs a loopback listener; here binding failed `Operation not permitted`.
Parser/config/mocked login tests do not prove TLS acceptance, actual device
storage, GUI login/logout/restart or Windows behavior. Complete real synthetic
Auth/API/desktop acceptance in the usable environment before release.
