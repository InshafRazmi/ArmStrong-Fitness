# Member conflict review

In desktop Settings → Server synchronization, an enrolled Administrator with a
current writer grant can open **Review conflict**. Rust checks that session and
the saved HTTPS/gym/device binding again when saving. Restored databases remain
blocked until server reconciliation.

The review shows the current local record and the highest validated server
revision retained in the member's conflicts/remote head. It covers all unresolved
conflicts and unconfirmed local member edits for that member. It does not fetch
the latest server version; another server change can reject a later retry.

- **Use recorded server version:** apply the reviewed server member in SQLite
  and explicitly discard the covered rejected/unsent member edits.
- **Keep current local version:** retain the current active member details,
  supersede the covered rejected/unsent edits, and queue one new update with a
  new operation UUID against the recorded remote revision.

Both choices require a review reason and confirmation. A native SHA-256
fingerprint covers the member, scope/actor, conflicts, remote head and original
queued payloads/requests. A changed review fails without writing; close the
dialog and reopen the current review. The request UUID makes an identical
commit retry idempotent. No caller can supply a server snapshot or actor identity.

SQLite migration 006 introduces append-only resolutions and links to covered
conflicts/operations. Original outbox records, frozen requests, rejected delivery
states and conflicts remain intact. Resolved operations leave the pending queue
through those links, never through a fabricated acknowledgement. Late receipts
for retired operations are refused. Only a matching native server receipt
confirms the new retry. The saved review, reason, choice, before/after member and
actor are retained in audit. Business changes, links, retry and local receipt
commit in one IMMEDIATE transaction.

Membership periods, attendance, payments, invoice balances and receipt snapshots
retain their history. Applying a new current card uses existing NFC assignment
history triggers. Server archive actors map to existing verified subjects or an
inactive identity reference without roles or a session.

Some cases still require server reconciliation:

- A frozen request is still pending without a confirmed outcome. Retrying must
  determine whether it already succeeded before its local intent can be discarded.
- A server rejection lacks a complete valid member snapshot, identifies another
  member, or conflicts with another snapshot at the same revision.
- Joined dates differ, a hard deletion is pending, or operations belong to a
  different device.
- Applying the server card would take another local member's card, archive actor
  identity mapping conflicts, or an archived local member would be reactivated.
- Keeping either archived version would replace archive intent with an update.
  Keep-local retry is currently limited to active local/server members.

Validated backups include resolutions and links; older backups migrate in
isolation. Restore preserves those records, writes a recovery copy and continues
to require reconciliation. Runtime review permission is excluded from the restore
storage fingerprint; durable review/history remains included in snapshots/audit.

Production synchronization remains disabled pending real HTTPS/API/device
acceptance. SQLite/session/transport mocks validate the local behavior; they do
not establish live Auth, OS credential persistence, GUI or Windows acceptance.
