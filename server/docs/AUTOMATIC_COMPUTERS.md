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
were retained. The normal owner `npm run migrate` now supports both versions.

The function derives one gym from an existing active Administrator subject. It
never creates a staff role, accepts a client gym/role/write permission, revives
revoked devices, rotates credentials or promotes a retry. The gym lock and unique
writer index keep the first computer writable and subsequent computers read-only.
Public/anon/authenticated EXECUTE is revoked; only the restricted private API
login receives the extra function permission. It retains no direct staff/device
INSERT/UPDATE grant.

Real PostgreSQL acceptance in the test project passed first enrollment, exact
retry, second-computer read-only access, wrong secret, revoked device, unknown
subject, Reception and inactive account denials. Synthetic fixtures were rolled
back. This verifies SQL behavior, not live Supabase password login, Render
deployment or Windows OS credential acceptance.

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

The currently deployed source must be updated before automatic approval can
work. GitHub connector content writes previously returned 403; deployment is
not certified merely because SQL provisioning passed. Shared financial/stock
data download, all-module synchronization and offline multi-writer policies
remain separate release work. No one-writer constraint was removed.
