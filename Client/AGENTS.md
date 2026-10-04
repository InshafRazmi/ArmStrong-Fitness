# Armstrong working rules

- Build a Windows Tauri desktop gym system for Matale; develop on Linux. Preserve the existing black/amber React interface in `Client/`.
- Scope: members, memberships, NFC/manual attendance, payments/receipts, sales/inventory, expenses, reports, users/roles, audit and backups. Exclude workout plans and body-progress tracking.
- Read `STATUS.md`, `PLAN.md` and relevant source before changing behavior. Inspect targeted files; avoid repeated full-repository reads and unrelated refactors.
- Mock data, browser-only persistence, simulated sync, and success toasts are not completed production features. Report them explicitly.
- Use transactional SQLite locally and a real authenticated backend. Keep business validation and authorization outside the UI; never embed server secrets in the frontend or Git.
- Protect money, inventory, membership dates, unique cards, audit history and sync idempotency with meaningful tests. Use integer LKR minor units and Asia/Colombo business dates.
- Preserve user data: explicit migrations, validated backups, recovery before replacement, and no silent demo fallback on corruption. Never silently overwrite financial conflicts.
- Baseline check: `cd Client && npm run build`. Add relevant business/integration tests during implementation; a frontend build does not validate Windows, hardware or synchronization.
- Update `STATUS.md` after each milestone with changes, checks, limitations and next steps. Keep user updates short and scope risks honest.
- First milestone is inspection/planning only. Resolve deployment topology before implementing synchronization. Three days is a constrained core-release target, not a promise of the entire feature set.
