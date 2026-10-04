# Single Administrator login and device setup

Start with `npm run setup:check` to see all missing local settings together.
`npm run api:check` probes the approved existing HTTPS API without credentials.
After native device preparation, `desktop:config:check` and
`desktop:config:write` review/provision only the three public native fields.
See [setup checks and file preservation](LOCAL_SETUP.md). These tools do not
register an account/device or certify live sync.

The user clarified that **one Administrator account signs in and manages all
controls, including attendance**. Use one Supabase Auth account for that login.
Its internal `armstrong.staff` row stores the verified identity, gym and
Administrator permission/audit relationship; it is not another login account.
No additional staff or Reception account is required for normal operation.
This supersedes the earlier device-only/deferral choice.

The normal-terminal runtime `npm run db:verify` has passed verified TLS,
the version 1 migration checksum, all six required private tables and their
RLS-enabled flags. The user also reports `npm run auth:check` PASS for real
Supabase password sign-in and separate online identity verification, with values
withheld and no application database changes. Gym/role/device registration and
desktop login were explicitly not checked. Approved registration, enrollment and
member API/desktop acceptance remain the next steps. Live desktop sync remains
disabled; these are separate database metadata and Auth results, not an
end-to-end sync pass.

## The Administrator account

Use the Administrator's email/password Auth account in the Supabase project
configured by `server/.env`. If it does not exist yet, create that one account
through the project's Authentication > Users dashboard and confirm its email.
The database login and the Supabase dashboard owner login are separate from
this application account. A user in another project cannot authorize this gym.

The account's identity must be verified through the same project's online
Auth endpoint. Existing `src/auth.ts` uses `/auth/v1/user` over HTTPS with the
publishable key and bearer token. A decoded JWT, email string or user-editable
role claim cannot grant staff access. No administrative Supabase API key is
needed by the runtime token verifier. See Supabase's
[user management](https://supabase.com/docs/guides/auth/managing-user-data) and
[online user verification](https://supabase.com/docs/reference/javascript/auth-getuser).

For the private local account check, add `AUTH_CHECK_EMAIL` and
`AUTH_CHECK_PASSWORD` to ignored `server/.env` in your editor, using that one
account's credentials. These are optional probe settings, not required for API
startup or intended as application runtime password storage. From `server/`:

```sh
npm run auth:check
```

This performs real password sign-in and a separate online `/auth/v1/user` check.
It uses the publishable/legacy anon key and enforces HTTPS, TLS verification and
no redirects. It prints no email, password, token, key or user UUID, and writes
no application records. It does not create a user, grant Administrator access,
register a device or bind the desktop. A PASS verifies Auth only. Remove the
optional probe credentials locally when the check is finished. The future native
login must establish its own securely stored session. See Supabase's
[password sign-in](https://supabase.com/docs/reference/javascript/auth-signinwithpassword).

## Owner-approved registration

Registration is an administrative database operation. Before making it,
identify and approve the gym, staff Auth identity/role and device. The runtime
API only verifies existing registrations; it cannot create them or grant roles.

| Record | Required relationship |
| --- | --- |
| `armstrong.gyms` | A stable gym UUID and name. Use synthetic gym data for acceptance checks. |
| `armstrong.staff` | That gym UUID, the one online-verified Administrator's Auth UUID, display name, Administrator role and active status; no second account. |
| `armstrong.devices` | That gym UUID, the approved device UUID, secret hash, active status and approved writer permission. |

For an actual desktop database, use its existing persisted device UUID. A
backend-only synthetic acceptance check can use its own fixture device UUID;
that result does not bind a desktop database. The current migration permits at
most one active writer per gym.

Inspect existing registrations before provisioning. Preserve their IDs and
permissions; do not silently replace a gym, duplicate the Administrator,
reassign a device or rotate its secret. The desktop UUID is stored in SQLite
`metadata` with key `device_id`, and native sync checks that its bound scope
matches that value. Completing an Auth check does not create these records or
approve a desktop device.

The device secret is 32 random bytes represented as 64 lowercase hex characters.
The existing API hashes **that 64-character string as UTF-8**, not the decoded
32 bytes, with SHA-256. Store only the resulting hash in `secret_sha256`.
The plaintext belongs in protected native credential storage for a real desktop;
that storage is now implemented through native device preparation; actual OS
and online enrollment acceptance remain pending. No secret belongs in frontend
code, committed files, logs, SQL output, command arguments or chat.

Use the administrative commands below to review the exact local settings before
database writes. `db:verify` and `auth:check` create no registrations or secrets.

## Registration commands

`npm run registration:check` and `npm run registration:apply` use only runtime
`server/.env`, the existing pg driver, verified CA/TLS and the same Administrator
Auth probe credentials. They reject inherited overrides of target/registration
settings. The administrative connection must use the project's `postgres` owner;
do not widen the separate API runtime role's permissions to run these commands.
No privileged Auth API key, account creation or additional login is involved.

In your editor, add these optional settings privately to ignored `server/.env`:

| Setting | Required local value |
| --- | --- |
| `REGISTRATION_GYM_ID` | The approved stable gym UUID; reuse an existing gym's ID if already registered. Retain this same value for retries. |
| `REGISTRATION_GYM_NAME` | The approved gym name, 1–120 characters. |
| `REGISTRATION_ADMIN_NAME` | This account's display name, 1–120 characters. |

The Administrator UUID comes from real password sign-in plus online identity
verification, never from a supplied ID, email lookup or editable role claim.
The command also confirms that this verified subject exists and is confirmed
in the runtime database project's `auth.users`. It verifies the existing
migration checksum/tables/RLS; it never runs migrations or repairs schema.

From `server/`, review first:

```sh
npm run registration:check
```

This performs real Auth/SQL checks and reads registration state in a read-only
transaction. Output contains only `create`, `existing` or `not included` states;
no IDs, names, env values or credential hashes. Review the actual approved
settings privately in your editor. A PASS is registration review, not an
enrollment or live-sync pass. On failure, stop and resolve the reported condition.

After that review, the distinct administrative write command is:

```sh
npm run registration:apply
```

Apply rechecks the registration inside a transaction with an advisory lock and
the existing gym's row lock. It inserts missing approved records only; exact
retries preserve matching registrations. Different names, gyms, identities,
roles, revocations, hashes or active writers fail without implicit replacement,
promotion, reactivation or secret rotation. Failures roll back the transaction.
No member data is accessed or written, and no API client can call this tool.
Local SQL-mock tests verify command behavior; actual PostgreSQL race/constraint
verification remains separate integration work.

Gym and Administrator registration can be completed without device approval.
Omit both optional device settings until **Prepare this computer** succeeds in
the actual desktop's OS credential store. Follow [native sign-in setup](../../Client/docs/NATIVE_SIGN_IN.md).
That action exposes only the actual SQLite path and verified hash; it does not
approve the device or enable sync. Then optionally configure both:

| Setting | Required local value |
| --- | --- |
| `REGISTRATION_SQLITE_PATH` | Path to the existing desktop SQLite database, resolved from `server/` for a relative path. It is opened read-only and must already exist. |
| `REGISTRATION_DEVICE_SECRET_SHA256` | Hash of the actual native credential's 64-character hex string, using the UTF-8 hashing convention above; never the plaintext credential. |

The command reads only SQLite metadata, obtains its existing `device_id`, rejects
restored databases awaiting reconciliation and requires any saved scope to match
the approved gym/device and configured HTTPS API origin. It does not migrate,
bind or modify SQLite or generate a substitute device ID. Only the hash is stored
server-side. The command also requires the supplied hash to match the
`native_device_secret_sha256` marker saved after native OS-store readback. Missing
or different markers fail without changing SQLite. These administrative commands
do not prove online possession of the device secret or live desktop acceptance.

Placeholders are documented in `.env.example`. No populated env file should be
printed, committed or copied into Client. Registration live checks must run in
an environment with working Supabase DNS; current Codex dependency access still
fails `EAI_AGAIN`. Existing isolated integration tests continue to use `.env.test`
and `TEST_DATABASE_URL` only, with no runtime fallback.

## API acceptance

Once registration exists, `POST /v1/enrollment` takes the verified staff bearer
token and `{ protocolVersion: 1, deviceId, deviceSecret }`. The server derives
gym access from the active staff/device join and checks the secret and permissions
under the gym lock. Enrollment has no client gym selector or role-grant field.

Verify real Auth success/failures and approved-device enrollment before testing
member operations with synthetic data. The server's PostgreSQL login also needs
reviewed least-privilege runtime grants before deployment; the migration-owner
login is administrative. Native HTTPS/sign-in/credential-storage wiring is now
implemented; real OS/GUI/API acceptance, desktop scope binding and
offline/restart/reconnect/pull/retry/conflict acceptance remain
required before live sync becomes available. Full server typing now passes with
the pinned official pg declaration snapshot; current checks and the remaining
live acceptance gates are recorded in `Client/STATUS.md`.

Administrator permissions will cover the existing application modules, including
attendance. The current online sync protocol remains member-only; this account
decision does not implement attendance synchronization or unlock native saves
without a verified native login/session.
