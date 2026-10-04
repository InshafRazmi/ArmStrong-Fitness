# Native Administrator sign-in

This is implemented desktop wiring with local tests, **not live acceptance**.
The HTTPS API, approved registration, usable OS credential storage and a GUI
session are still required. Member sync remains disabled. The separate browser
login is a demo and cannot grant a native session.

## Prepare and register this computer

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

Account tokens remain in native process memory, outside SQLite/backups/webview.
The app retains no password or refresh token. Sign out locks local access and
cancels pending sign-in results; it does not revoke other Supabase sessions.
Restart and token expiry require another online sign-in. An offline restart
grant/PIN policy and automatic token refresh are not implemented. An authenticated
restore retains the auth requirement, checks its actor mapping and locks the
session; server reconciliation remains necessary afterward.

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
Auth/API/desktop acceptance in the usable environment before enabling sync.
