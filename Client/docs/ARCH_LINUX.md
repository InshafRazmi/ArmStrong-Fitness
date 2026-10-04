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
sudo pacman -U ./armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst
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
API; the old deployed endpoint still needs manual computer approval. One active
computer can edit; additional account-authorized computers receive read access.
Member changes use the native HTTPS worker after online sign-in, with bounded
automatic retries and a manual action in Settings. Other modules' shared-data
download and full synchronization are still release work.

After a verified online login, **Continue offline** uses this OS user's vault
grant for up to seven days, including application restart. Signing out removes
that grant. Missing/changed credentials, account mappings, expiry, clock rollback
or a restored backup keep access locked and retain local records. Revocation
cannot be checked while offline; connect/sign in to obtain current permissions.
Network outages retain transactional SQLite records and pending changes. Member
sync retries while the online session is valid. An open app renews its session
using rotating credentials held only in native memory, rechecking the online
identity and server enrollment before extending access. A refusal locks access
and removes offline approval; an outage never extends an unverified session.
After token expiry during an outage, use Continue offline. An offline restart
needs another online sign-in before syncing. Real all-module sync and actual
outage/reconnect acceptance remain unfinished.

The application data directory is `~/.local/share/lk.armstrong.fitness/` under
the standard XDG layout. Package removal does not remove gym data or OS vault
items. Export a backup from Settings and keep a copy off the computer.

Package creation, archive contents and dynamic-library resolution passed on the
build host. GTK cannot initialize in this restricted execution session, so actual
window/login/keyring acceptance must be completed in a normal desktop session.
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
