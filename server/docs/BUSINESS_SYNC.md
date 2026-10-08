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
sales/items/stock movements, expenses/voids and audit. User rows are historical
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
enrollments without changing credentials. The migration runner includes versions
1–5. The existing restricted API
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
