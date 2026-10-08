# Transactional gym synchronization, protocol 2

An active Administrator can edit from any valid enrolled computer. Computers
download the same gym records. Every local business transaction freezes
its changed rows, historical audit and associated outbox IDs in SQLite before
commit. The server verifies the account, gym, device secret and writer permission
again under the gym lock, then validates the complete proposed business state.
Rows, scoped foreign references, immutable request receipt and ordered change
record commit together. Exact retries return that receipt. Different content
under the same operation ID, stale master data or changed financial history is
refused and remains queued locally.

Protocol 2 covers members/cards, plans/periods, profile, attendance, invoices,
invoice details, payments/allocations/releases/reasons/saved receipts, products,
sales/items/stock movements, expenses/voids, staff/trainer assignments, monthly
training charges, salary/fee payouts, member/staff permanent-removal markers,
member gender, staff cards/attendance and audit. User rows are historical
identity references only: downloaded rows cannot activate users or grant roles.
Passwords, tokens, OS credentials, metadata, roles and local IPC request receipts
are excluded. The existing authenticated enrollment and seven-day offline grant
remain the authority for local access.

The row contract is versioned with both native and server sources. Dates are
Asia/Colombo business dates and money is integer LKR minor units. Financial and
attendance records are append-only; card history allows revocation only. A sale
includes its items and matching stock deductions; a renewal includes its invoice;
a payment includes its original saved receipt and allocations; a reversal includes
its full original amount, reason, receipt and allocation releases.

The worker uses a dedicated SQLite connection, bounded HTTPS requests and
durable retries. No transaction spans network I/O. Replies commit only while
the verified native session nonce still matches. Download and cursor commit
together, and queued writes block incoming replacement. Restored databases
stay locked until online same-computer recovery reconciles an isolated copy and
atomically replaces the guarded data. Existing OS possession proof and fresh
verified enrollment are required; logout, expiry and storage fingerprints fence
replacement. See [restore recovery](../../Client/docs/RESTORE_RECOVERY.md).

Migration 3 adds private RLS tables without public/Data API grants. Migration 4
enforces immutable fields, exact master versions, identity-only users and fixed
trigger search paths. Both are applied to the existing test project and production;
Migration 5 removes the single-writer restriction and upgrades valid Administrator
enrollments without changing credentials. Migration 6 extends the private record
allowlist and immutable/master guards for the five Staff tables and adds a
gym-scoped unique NIC index. Migration 7 adds the four attendance/profile/
removal tables and shared active-card unique indexes; the server/native contracts
remain additive so saved earlier batch bytes are retained. Both 6 and 7 are
applied to the isolated project. Migration 8 adds immutable staff removal and
prevents subsequent edits to removed profiles. All versions 1–8 are now applied
to production with exact checksums and unchanged existing application data
fingerprints. Readiness passes with the actual restricted API login and verified
TLS. The migration runner includes versions 1–8.
Deploy the updated API and upgrade all gym desktops
before creating Staff records. Older clients refuse unknown rows while retaining
their local history and queues. The existing restricted API
role has only SELECT/INSERT and permitted mutable record/reference columns. Existing member
and onboarding history is preserved. A gym using protocol 2 refuses further
protocol 1 member writes so older clients cannot change a separate member copy.

Each group is limited to 1 MiB and 2,000 changed rows; an oversized initial legacy
database is refused atomically until a bounded bootstrap path is implemented.
Existing protocol-1 cloud members also require explicit reconciliation before
protocol-2 use. The approved production gym currently has no such cloud members.
General financial/master conflict review remains guarded without a complete
resolution flow. Same-computer protocol-2 restore recovery is implemented and
tested with real SQLite and native Auth/HTTP mocks; live desktop acceptance remains.
Server validation currently reads the gym's complete state under its lock.

Measured acceptance: ten real native SQLite envelopes match the server contract
and hashes; real isolated Supabase Auth/PostgreSQL tests pass exact retries,
ordered read-only download, stale edits, immutable financial/master guards,
closed-payment refusal, denial and rollback. Production business fixtures were
not created. Actual restricted-runtime TLS/catalog checks also pass. Real Linux
local webview forms and restart pass; production native HTTPS/OS-vault and
Windows/hardware acceptance remain. Multiple Administrator computers may queue
changes offline; conflicting master or financial groups are refused and retained,
never silently overwritten. General conflict resolution and real multi-computer
offline/reconnect acceptance remain release work.

Staff acceptance adds five real native envelopes for trainer creation, member
registration, combined collection and payout. The isolated real Auth/PostgreSQL
test passes exact retries, second-computer download, stale trainer edits,
duplicate-salary rejection, immutable payout guards and unique NIC enforcement;
all fixtures roll back. Native two-database download also preserves reviewed
rates, receipts and payout/refund guards. Salary is paid once per active calendar
month; fee payouts use collected allocations once. Unpaid invoices earn no fee.
See [staff operation](../../Client/docs/STAFF.md).


A blocked business transaction now has an Administrator review-and-retry action
in **Settings → Server synchronization**. The native review lists the affected
records and actual server refusal. Its fingerprint fences stale reviews; only
the first blocked batch and original actor can be retried. The action resets
its delivery state and clears backoff after saving an audit/operation receipt.
The frozen request, operation IDs, historical records and confirmed receipts
are unchanged. The server still validates the original transaction and may
refuse a conflicting edit again. This recovers refusals after a corrected
backend deployment/configuration; it is not a general stale financial/master
merge. No local queue is silently deleted or acknowledged.

The reported `invalid_business_data` refusal was traced read-only to the installed
computer's first blocked staff creation (`trainers` plus `audit`). The current
local API accepts its exact retained request; the published source at workspace
HEAD lacks the Staff row contract. Protocol-2 `/v2/health` alone does not establish
support for those newer tables. Production migrations 6–8 are now applied;
deploy the matching API
before reviewing and retrying the original transaction. Do not clear the queue
or recreate the staff record. Future unsupported tables return the specific
`unsupported_business_table` code. Updated API health adds businessSchemaVersion
10; Render currently still returns the older health body. Native schema 10 adds
an immutable staff removal marker. Deactivation, NFC revocation, assignment
clearing and the marker synchronize together; salary, training and attendance
history remain intact. The server writes revocations first, profile/assignment
changes next and removal markers last so database immutability guards never
block the same transaction's deactivation.

Attendance acceptance uses native SQLite-generated envelopes, real isolated
Supabase Auth/PostgreSQL, exact receipt replays, a second-device download,
cross-category card uniqueness, immutable staff activity, and permanent removal
with retained gender/member/staff attendance. Every synthetic API record rolls
back. Native two-database acceptance separately verifies retained memberships,
payments, saved receipts and attendance after shared permanent removal.

Staff deletion acceptance uses eight exact native SQLite envelopes against real
isolated Supabase Auth/PostgreSQL: removal receipts replay, another device
downloads the marker and retained payroll/attendance, and both API/SQL reject
reactivation or removal-history edits. All synthetic rows roll back. Active and
inactive deletion, failure rollback, final fee payout, backup/restore and native
two-database download are covered separately.
