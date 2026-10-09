# Online recovery after restoring a backup

Restore first validates the backup schema, checksum, dates and foreign keys. Its
preview compares current records with the backup. Replacement writes a validated
recovery copy of the current database before changing any tables. Keep that copy:
it contains records and pending transactions created after the selected backup.

A restored cloud-connected database locks access until online reconciliation.
Sign in with the approved account on the same computer. Recovery reads the
existing OS device credential and verifies its saved hash; it cannot create or
replace that credential. Auth and API enrollment are verified online against the
existing server/gym/device scope. Copied backups cannot grant account permissions.

Recovery works on an isolated database copy. It retries the backup's immutable
pending requests using their original IDs, accepts only exact server receipts,
and downloads complete ordered server transactions. Newer master records,
financial history and saved receipts can then be recovered from the cloud.
Server permission checks and financial/stock validation apply to every request.

Only a complete verified run can replace the guarded database. The native login
epoch, account expiry and a fingerprint of all stored tables are checked again
at replacement. A second validated recovery copy preserves the guarded state.
History replacement and guard removal commit atomically; ordinary online
enrollment then grants fresh local access and OS-vault offline authorization.
No password or bearer/refresh/device secret enters either backup.

Outages, lost replies, logout/cancellation, another process changing storage,
expired authorization, read-only writer approval and conflicting local edits
retain the guarded database. Exact retry IDs survive failed attempts. Retry
online; a financial/master conflict requires Administrator review and is never
silently discarded or overwritten. Each attempt has bounded work and a
two-minute worker limit; individual native request deadlines still apply.

This path supports the same enrolled computer with protocol-2 history. It does
not transfer OS credentials between computers, reconcile legacy member-only
gyms, or bootstrap oversized pre-protocol databases. Missing possession proof
or unmatched scope remains a refusal. Cross-device/legacy recovery needs an
explicit migration procedure. The existing backup limit is 64 MiB of SQLite data.

Real temporary SQLite with native Auth/HTTP mocks verifies newer cloud payment
and saved receipt recovery, exact lost-response retry, logout/cancellation,
storage races, wrong scope, read-only approval and retained conflicts. Live
Windows/OS-vault and production native HTTPS recovery acceptance remain open.

## Retained installation profile recovery (0.1.7)

An older Windows installation can retain `business_revision_conflict` for an Add
of the original Armstrong Fitness gym profile and sign-in audits. Version 0.1.7
also handles the normal first-sign-in batch shown as **Add audit**, **Add gym
settings**, **Add users**. Version 0.1.6 incorrectly excluded that user reference
and consequently displayed no recovery checkbox for this case.

Sign in online as the original Administrator. Open **Settings → Server
synchronization → Review retained transaction**. If the native review offers
**Recover using server profile**, check **I confirm recovery using the server
gym profile.** and select that action.
The application saves a validated backup, rechecks the original request with the
server, preserves its frozen bytes and audit records, and downloads complete
verified shared history before replacing the local database. Keep the backup
path displayed by the application. After recovery, sign in online again and let
the remaining recovery audits receive their own server confirmations.

Only a fresh revision refusal permits a replacement of the retained audits and,
when present, the unchanged signed-in account's initial identity reference, with
a stable operation ID. The reference is inactive/version 1 and cannot grant
roles or sign-in authority. The server also refuses a different existing identity;
recovery then leaves the original database intact. A real receipt for the
original request takes precedence. The
superseded request remains refused without a fabricated receipt and is counted
separately from confirmed transactions. Lost replies reuse the same replacement;
failed or cancelled recovery leaves the original database intact.

Edited profiles, members, payments, inventory, other saved business operations,
changed identity or previously downloaded history cannot use this path. The
review displays the reason; retain a backup for separate reconciliation.
Recovery does not authorize general conflict clearing. Actual recovery on the
affected Windows installation remains an acceptance step.
