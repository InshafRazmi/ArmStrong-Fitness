# Account-authorized computers

The owner selected `armstrong@gmail.com`, gym **ArmStrong Fitness**, display name
**ArmStrong**. The confirmed, unbanned account has been linked to that gym in the
production private staff registry. No computer, device credential, fake writer
or local account session was created by that registration.

The CLI-created migration
`supabase/migrations/20261004213002_desktop_onboarding.sql` adds the private
`armstrong.enroll_desktop` function. It is applied to both the existing test
project and production, with application ledger version 2/checksum recorded in
production. Existing version 1, business records, other gyms and registrations
were retained. The normal owner `npm run migrate` now supports versions 1–5,
including `20261008015730_administrator_device_access.sql`.

The function derives one gym from an existing active Administrator subject. It
never creates a staff role, accepts a client gym/role/write permission, revives
revoked devices or rotates credentials. Migration 5 removes the old single-writer
index. Every valid enrolled computer used by an active Administrator receives
editing access, including previously read-only computers on their next online
sign-in. Existing identity and credential hashes stay unchanged. The API also
derives Administrator editing access from the active private staff record; other
roles retain their device and operation restrictions. The gym transaction lock,
revision checks and immutable history still protect overlapping writes.
Public/anon/authenticated EXECUTE is revoked; only the restricted private API
login receives the extra function permission. It retains no direct staff/device
INSERT/UPDATE grant.

Real PostgreSQL regression in the test project covers multiple editing computers,
upgrading an existing read-only computer, exact retries, unchanged credentials,
wrong secrets, revoked devices, unknown subjects, Reception, inactive/demoted
accounts and cross-gym/JWT-subject denials. Synthetic fixtures roll back. This
verifies SQL behavior; desktop login and OS credential acceptance remain separate.

Deploy the updated server source to Render, preserving the restricted runtime
login and verified TLS CA. Add the following environment variable:

```text
AUTOMATIC_DEVICE_ENROLLMENT=true
```

API startup checks that its runtime login can execute the provisioned function.
The feature defaults to false until explicitly configured. `/v1/enrollment`
still verifies the bearer identity online before the function call. Native
desktop login now prepares its OS credential automatically and sends its own
device proof; ordinary users do not edit server env files per computer.

Migration 5 is applied to production and works with the existing automatic
enrollment endpoint: fresh online sign-in grants Administrator editing access.
The additional role-based API request checks remain local until source is
published/deployed. Production verification preserves all existing records;
real enrollment probes run inside a rolled-back transaction and preserve device
credentials. Shared financial/stock download, offline/reconnect acceptance and
general conflict recovery remain separate release gates.
