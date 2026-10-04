# Restricted API database login

The reported production startup error means the deployed `DATABASE_URL` uses an
administrative or built-in role. Keep that startup check. On October 5, the
connected project was checked and the absent `armstrong_api` role was provisioned
and verified through Supabase. No gym/member/staff/device records were changed.

## Complete the Render settings

1. Open the local ignored `server/.env.render`. Copy its `DATABASE_URL` value into
   **Render → armstrong-member-api → Environment → DATABASE_URL**. This value uses
   `armstrong_api.<project-ref>` and a newly generated 256-bit password, the existing
   Session pooler host/port 5432, `sslmode=verify-full`, and the deployed CA path.
   It is a complete new connection, not a username replacement with the owner password.
2. Under **Secret Files**, use filename `hi3.crt` with the contents of
   local `server/certs/prod-ca-2021.crt`. Its runtime path is
   `/etc/secrets/hi3.crt`, now used by the prepared database URL to match the
   certificate filename in the user's supplied Render setting. The local copy's
   filename does not need to match the deployed secret-file name.
   This copies the readable, unexpired CA from the administrative configuration.
3. Confirm the other runtime variables match the prepared file:
   `NODE_VERSION`, `NODE_ENV`, `HOST`, `SUPABASE_URL`,
   `SUPABASE_PUBLISHABLE_KEY`, and `PUBLIC_API_ORIGIN`.
   The API origin is `https://armstrong-fitness.onrender.com`.
4. Choose **Save and deploy**, then check the startup log and `/health`.
   Keep the current verified build command and `npm start`.

[Render variables and secret files](https://render.com/docs/configure-environment-variables)

The prepared file contains only the seven runtime settings and has permissions
`0600`. It is ignored and excluded from the source upload bundle. Keep local
`server/.env` for administrative registration/probes; it retains the original
owner connection and was not overwritten. If importing a file into Render,
use only `.env.render`, not the administrative `.env`.

## Access actually granted and checked

| Object | Runtime access |
| --- | --- |
| `gyms` | SELECT, row locking, UPDATE of `change_sequence` only |
| `staff`, `devices` | SELECT only |
| `members` | SELECT, INSERT, UPDATE of name/phone/email/card/revision/archive fields |
| `member_operations`, `member_changes` | SELECT and INSERT only |

The role cannot grant itself staff/device permissions, insert gyms, change gym
names or member gym/joined-date fields, delete any of the six tables' rows, or
rewrite operation/change history. It has no superuser, database/role creation,
replication, inherited role memberships, table ownership, schema CREATE, Auth
schema usage, or Storage schema usage. The connection limit is 12, above the API's
pool maximum of 10; statement timeout is 10 seconds and search path is `pg_catalog`.

The existing private-API access model requires `BYPASSRLS` combined with these
restricted object/column grants. Staff/gym/device authorization is enforced by
the API on every transaction. RLS remains enabled on all six tables, with no
Data API policies or new grants to `anon`, `authenticated` or `authenticator`.
No API role is granted permission to create/approve staff/devices or run migrations.
[Postgres roles](https://supabase.com/docs/guides/database/postgres/roles),
[RLS bypass](https://supabase.com/docs/guides/database/postgres/row-level-security#bypassing-row-level-security)

The reviewed administrative setup is in [RUNTIME_DATABASE_ROLE.sql](RUNTIME_DATABASE_ROLE.sql).
It creates the role as NOLOGIN and refuses to replace any existing role. The
password was enabled separately and is absent from that SQL and migration history.
Do not rerun creation against the role that is already provisioned.

[VERIFY_RUNTIME_DATABASE_ROLE.sql](VERIFY_RUNTIME_DATABASE_ROLE.sql) performs
real SELECT/lock and zero-row write checks as the runtime role, plus 16 permission
denials. PostgreSQL initially denied the creator's SET ROLE because its automatic
membership has SET false; the verification temporarily enables SET within its
transaction and rolls it back. The final catalog confirms SET false and INHERIT
false are restored. No test inserts, edits or deletes a business row.

These SQL checks passed. A separate Node/pg connection probe with the new login
and local CA failed DNS (`EAI_AGAIN`) before TLS/password authentication. Render
still needs the prepared settings; actual password login, deployed CA/TLS,
HTTPS startup and health remain unverified. Health also does not establish
Administrator/device enrollment or desktop synchronization.
