# Production reset for version 0.1.10

The owner requested a clean production start. ArmStrong Fitness's cloud members,
staff profiles, attendance, invoices, payments, stock, expenses and audit/sync
history were cleared on 2026-10-11. The Administrator account and gym registration
remain. A protected pre-reset cloud backup is retained outside Git.

Previously enrolled computers were revoked so their old local test data cannot
upload into the empty production gym. Installing an update preserves local
databases, so those computers need a fresh local profile before signing in.
New computers can install 0.1.10 and sign in normally.

## Previously enrolled Windows computer

1. Install version 0.1.10 and close ArmStrong Fitness completely.
2. Open File Explorer, enter `%APPDATA%`, and find `lk.armstrong.fitness`.
3. Rename that folder to `lk.armstrong.fitness.before-production-reset-20261011`.
   If that name already exists, use another unique name. Keep this folder as the
   local backup; it includes the database and any SQLite WAL/SHM sidecars.
4. Open ArmStrong Fitness and sign in online with the existing Administrator
   account. The app creates a fresh database and a new computer registration.
5. Configure the gym profile, admission fee and membership plans before adding
   production members or receiving payments.

Keep the backup private. Do not restore or copy the old database into the new
production profile: it contains the cleared test history and a revoked device
identity. Existing offline grants cannot unlock the fresh profile; first sign-in
requires internet and the deployed schema-12 API.

## Previously enrolled Linux computer

Close the application, then rename
`~/.local/share/lk.armstrong.fitness/` to a unique backup folder alongside it.
Use the configured XDG data directory if it differs from the standard path.
Reopen the app and sign in online, then configure production settings as above.

This is a one-time operation for the explicitly approved production reset.
Future application upgrades preserve the new production profile and records.
