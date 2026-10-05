# Armstrong Fitness Gym Management

## Windows installer

Windows desktop installation is required; Linux support is optional. The Windows
builder and Actions workflow include the approved public API/Auth settings and
an offline WebView2 installer. Fresh packaged builds start locked at account
login without requiring a settings file on each computer. Follow
[Windows build/download and remaining acceptance work](docs/WINDOWS_INSTALLER.md).
Account-based computer enrollment, bounded offline restart and all-module
reconnect sync are implemented. Real native SQLite envelopes pass the server
contract, and isolated Supabase Auth/PostgreSQL integration passes. Production
migrations 1–4 and Administrator registration are applied. Publish/deploy the
updated API, then complete Windows/network/hardware and reconciliation gates
before labeling this a final release.

## Arch package

The native x86_64 package is in `dist-linux/` with `SHA256SUMS`. See
[Arch installation and verification limits](docs/ARCH_LINUX.md). It embeds the
frontend and public server settings; destination computers do not need Node or
Rust. Build it on Arch with `npm run desktop:arch:build`.

## Local foundation test build

The native Tauri build supports SQLite-backed members/cards, explicit membership dates, attendance, invoices/payment allocations and saved receipts, sales/stock, expenses, audit, validated backups and date-filtered CSV reports. Packaged builds require verified Administrator sign-in and OS credential storage. All-module synchronization uses exact server receipts and ordered atomic download. The ordinary unconfigured development build is explicitly for local test records. See [STATUS.md](STATUS.md) and [PLAN.md](PLAN.md) for measured checks and release gates.

```bash
npm run desktop:run    # build embedded frontend and launch Tauri
npm run test:core      # SQLite business/integrity tests
npm run test:desktop   # real webview forms, then process-restart verification
```

Rust and Tauri OS prerequisites are required. The desktop test needs a usable graphical session and uses an isolated temporary database. Linux forms and process-restart checks pass; Windows has not yet been verified. No Tauri npm CLI is required; these scripts invoke Cargo directly. Always use `desktop:run` after the smoke test to rebuild without test hooks. For constrained Linux builds use `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2`; add `CARGO_NET_OFFLINE=true` when using the existing dependency cache.

Reports accept inclusive From/To dates and a Today shortcut using Asia/Colombo.
Blank dates include all history. Payments/reversals count on their own posting
dates; voided expenses retain CSV history and contribute zero effective expense.
Memberships include overlapping periods with status as of today. Inventory is a
current snapshot. Summaries are calculated natively in integer LKR minor units;
invalid bounds/unsafe totals fail instead of silently rounding.

## Existing browser prototype

The sections below describe the original browser demonstration, not production desktop coverage. `npm run dev` opens this separately labeled prototype; its localStorage and seeded data are not imported into SQLite.

Modular React + TypeScript frontend for the Armstrong Fitness offline-first Windows application.

## Run

```bash
npm install
npm run dev
```

Production validation: `npm run build`.

## Architecture

- `src/components/ui` — reusable icons, modal and page heading components
- `src/context` — shared gym state, actions, local-first writes and sync orchestration
- `src/data` — initial demonstration data
- `src/layout` — sidebar, top search/sync bar and application shell
- `src/pages` — dashboard and feature screens
- `src/services` — browser storage, backup/restore and server-sync adapter
- `src/types` — domain models shared by every feature
- `src/utils` — ID, date, initials and currency helpers

Data flows from a page into the `GymContext`, which saves it locally immediately, adds an audit entry and queues a server operation. When the browser detects internet connectivity, the sync service processes the queue. The service is currently a simulated adapter and is the single location to replace with the real Armstrong Fitness API.

## Implemented frontend workflows

- Dashboard summaries, search and quick navigation
- Member registration, editing and NFC card linking
- NFC/manual attendance with check-in/check-out detection
- Membership package overview
- Payments and local receipt-ready records
- Retail sales, stock deduction and stock adjustment
- Expense entry
- Business/report summaries and audit counts
- Local JSON backup and restore
- Online/offline status, persistent mutation queue and automatic sync simulation
- Settings sections for roles, NFC, printing, backup, sync and application updates

Tauri/SQLite and the separate Fastify/PostgreSQL API are implemented in this workspace. Native OS credential storage, HTTPS and Administrator login/logout are wired; real authenticated GUI/API/Windows acceptance is still required. The browser simulation remains demo-only and sync remains disabled. Follow [native sign-in setup](docs/NATIVE_SIGN_IN.md) for device preparation, approved registration and native configuration.
