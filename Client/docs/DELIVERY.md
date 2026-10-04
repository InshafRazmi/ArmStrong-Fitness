# Current acceptance-build delivery

Administrator: `armstrong@gmail.com`, display **ArmStrong**, gym **ArmStrong Fitness**.
The confirmed existing account/gym mapping and onboarding SQL are provisioned
in production. No password is included here; use that account's existing password.

The final all-features release is not finished. The current native app provides
local gym operations, bounded offline access, in-memory session renewal and
member synchronization. Other modules' shared cloud data/sync remain unfinished.
One active computer can edit; additional computers are read-only.

## Apply the prepared source

The GitHub connector rejects writes with `403 Resource not accessible by integration`.
No branch/PR/source push or Windows workflow run was created. Apply
`armstrong-desktop-changes.zip` from a writable checkout of
`InshafRazmi/ArmStrong-Fitness`. Extract the ZIP outside the repository. It contains
an ordinary binary-capable Git patch and allowlisted sources with SHA-256s.

The patch baseline is `d7e659cccf1f9d19fea5fd4f3f4596645d9f43fa`.
Keep any other local work safe and use `git apply --check` before applying it.
Do not force a conflicting patch or overwrite private env data. In the checkout:

```sh
git apply --check /path/to/extracted/changes.patch
git apply /path/to/extracted/changes.patch
git rm --cached --ignore-unmatch server/.env
```

The last command removes only the tracked env entry, preserving the private
local file. Secret deletion lines are deliberately absent from the patch.
Review the source diff, commit it and push through the repository's normal
workflow. Exposed credentials in the public repository/history still need
rotation; removing the file alone does not revoke them.

## Deploy the API

In the existing Render service deploy the updated source with root directory
`server`. Preserve the existing restricted runtime `DATABASE_URL` and secret CA
file `hi3.crt`. Do not replace the runtime login with an owner connection.

```text
Build: npm ci --include=dev --ignore-scripts --no-audit --no-fund && npm run build:verify
Start: npm start
AUTOMATIC_DEVICE_ENROLLMENT=true
```

The production migration is already applied; API startup verifies function
permission. Use Render Environment settings; no populated env file is deployed.
Verify `/health`, then actual desktop login/enrollment. A healthy process alone
does not prove login, shared data or synchronization. See
[automatic computer setup](../../server/docs/AUTOMATIC_COMPUTERS.md).

`server/armstrong-render-source.zip` is the refreshed credential-free server
snapshot if a separate source upload is needed. It includes onboarding migration
version 2 and vendored dependency types; it excludes private env/certificates.

## Install Arch or build Windows

For Arch x86_64, copy the package and SHA256SUMS from `Client/dist-linux/`:

```sh
sha256sum -c SHA256SUMS
sudo pacman -Syu
sudo pacman -U ./armstrong-fitness-0.1.0-1-x86_64.pkg.tar.zst
armstrong-fitness
```

Use an unlocked persistent Secret Service default collection. First login needs
internet and the updated API. After verified login, Continue offline uses the OS
user's cached approval for up to seven days; signing out removes it. An open app
renews online sessions; offline restart requires online sign-in to resume sync.
GTK launch is blocked in the build session, so test the real app in your normal
desktop. See [Arch guide](ARCH_LINUX.md).

For Windows, run **Actions → Windows installer → Run workflow** after the source
and workflow reach the default branch. Download `ArmStrong-Fitness-Windows-x64`
from the successful run and install its setup executable. No Windows executable
has been produced in this Linux-only environment. See [Windows guide](WINDOWS_INSTALLER.md).

Complete real GUI, OS credential, outage/reconnect, Windows upgrade, NFC/printer
and all-module synchronization acceptance before using a final-release label.
