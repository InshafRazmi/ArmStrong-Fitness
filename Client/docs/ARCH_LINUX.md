# Arch Linux x86_64 desktop

This is an acceptance build, not a final all-features release. Windows remains
the mandatory release platform. The native app embeds its frontend and the
approved public Render/Supabase connection settings. Node and Rust are not
required on the destination computer.
The packaged frontend excludes browser demo login/data. A missing native bridge
keeps gym records locked; `npm run dev` remains the separate browser prototype.

Update Arch fully before installing the supplied package. Arch rolling
libraries must be kept together; this build used glibc 2.44 and WebKitGTK 4.1.

```sh
sudo pacman -Syu
sudo pacman -U ./armstrong-fitness-0.1.8-1-x86_64.pkg.tar.zst
armstrong-fitness
```

The package declares its GTK/WebKitGTK, curl and libsecret dependencies.
Linux sign-in requires an unlocked persistent Secret Service default collection
(for example GNOME Keyring). Passwords/device secrets stay in native HTTPS/OS
storage; they are not placed in process arguments, the frontend or a packaged
env file. If credential storage is unavailable, unlock/configure it through your
desktop session rather than replacing existing application credentials.

First sign-in needs the approved Administrator account, internet and server-side
account/gym registration. The app prepares its computer credential automatically.
The new account-based computer approval endpoint must be deployed/enabled on the
API; the older endpoint still needs manual computer approval. An active
Administrator can read and write from any valid enrolled computer. Migration 5
upgrades older read-only enrollments on online sign-in without replacing device
credentials. If an open session still reports read-only access, sign out and
sign in online to obtain the current permission.
This update includes Staff, monthly personal-training invoices, combined member
payments and salary-plus-collected-fee payouts. It also includes compact login/
dashboard layouts, staff NFC/manual attendance with its own dashboard panel,
Male/Female member profiles and daily counts, permanent removal from both member
lists while retaining history, and review/retry for retained sync transactions.
Native schema 11 preserves existing records and queued transactions on upgrade.
Version 0.1.8 adds configurable admission fees, automatic membership and admission
invoices at registration, visible dues, Receive payment and Pay salary shortcuts.
The latest update aligns deletion confirmation checkboxes, adds Administrator
permanent staff deletion from active/inactive lists with retained payment history, and displays larger male/female
attendance counts at the top of a dashboard that fills the available height.
The NFC page has a larger amber card, spacious reader panel and larger scan
controls. Deleted staff remain in **Show deleted staff** for history and final
payments; they cannot be edited or reactivated.
Shared records require server migrations 6–9 and the matching schema 11 API. Production
deployment status is recorded in the delivery guide. Upgrade every desktop sharing the gym. See
[staff payments](STAFF.md) and [deployment](DELIVERY.md).
All gym modules use the native HTTPS worker after online sign-in, with bounded
automatic retries and a manual action in Settings. Shared download preserves
atomic financial/stock groups and refuses conflicting local history. Deploy the
updated protocol-2 source before acceptance. Same-computer online backup recovery
is implemented; live recovery acceptance, general conflict review and large/legacy
bootstrap remain release work. See [restore recovery](RESTORE_RECOVERY.md).

After a verified online login, **Continue offline** uses this OS user's vault
grant for up to seven days, including application restart. Signing out removes
that grant. Missing/changed credentials, account mappings, expiry, clock rollback
or a restored backup keep access locked and retain local records. Revocation
cannot be checked while offline; connect/sign in to obtain current permissions.
Network outages retain transactional SQLite records and pending changes. Gym
sync retries while the online session is valid. An open app renews its session
using rotating credentials held only in native memory, rechecking the online
identity and server enrollment before extending access. A refusal locks access
and removes offline approval; an outage never extends an unverified session.
After token expiry during an outage, use Continue offline. An offline restart
needs another online sign-in before syncing. Real native production HTTPS and
actual outage/reconnect acceptance remain unfinished.

The application data directory is `~/.local/share/lk.armstrong.fitness/` under
the standard XDG layout. Package removal does not remove gym data or OS vault
items. Export a backup from Settings and keep a copy off the computer.

Package creation, archive contents and dynamic-library resolution are verified on
the build host. Real Linux webview local forms, staff attendance, gender counts,
finance receipts and process restart passed in an isolated database. Production login/keyring/network acceptance must
still be completed in a normal desktop session.
Verify the package checksum with `sha256sum -c SHA256SUMS` before installation.

Source rebuild:

```sh
cd Client
npm ci
npm run desktop:arch:build
```

The builder uses locked Rust dependencies and the `desktop`, `custom-protocol`
and `packaged-auth` features, plus the Linux window configuration. Its release
build includes no smoke commands. See Tauri's official
[Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux).
