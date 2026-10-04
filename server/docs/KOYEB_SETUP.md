# Koyeb setup for ArmStrong Fitness

**Alternative only:** the user subsequently selected Render. Current setup is
in [RENDER_SETUP.md](RENDER_SETUP.md); the Koyeb source zip is an older snapshot.

The selected deployment target is **Koyeb Eco Micro in Singapore**, with one
instance kept running. The Node/Fastify API runs on Koyeb, Supabase continues to
provide PostgreSQL and Auth, and the desktop application runs on the gym computer.
The supplied gym and Administrator names are **ArmStrong Fitness** / **ArmStrong**.
The user's request to set up Koyeb supersedes the earlier hosting decision gate
for this setup. Account access, database runtime access and live verification
are still required; preparation alone does not publish the service.

## 1. Account and source

Sign in at [Koyeb](https://app.koyeb.com/). Check that the account uses the
**Starter (pay-as-you-go)** plan rather than a Pro subscription. Koyeb's pricing
FAQ describes card verification and a possible Pro charge during signup. Account
validation usually takes minutes but can take up to three business days.
[Billing FAQ](https://www.koyeb.com/docs/faqs/pricing)

Connect the GitHub repository containing `server/`. The repository must include
`package.json`, `package-lock.json`, `Procfile`, `tsconfig.json`, `src/`, `scripts/`,
`test/`, and all of `vendor/types-pg/` including its license and provenance. The
local file dependency must be available during dependency installation.

If the project has no Git repository yet, a source-only bundle is prepared as
`server/armstrong-koyeb-source.zip`. Extract it to a separate folder and upload
the resulting `server/` directory to a **private** GitHub repository. Populated
env files, certificates, node_modules, SQLite files and desktop Auth configuration
are excluded. Never upload the populated working `server/` directory wholesale.
This zip is a snapshot; regenerate it after later code changes.

Koyeb also supports uploading a project directory through its CLI, without
GitHub. Use a source-only directory, not the populated working directory, and
review the installed CLI's help for build, environment, certificate, port and
scaling options before deployment. The CLI is not installed/authenticated in
this workspace.
[GitHub deployment](https://www.koyeb.com/docs/build-and-deploy/deploy-with-git),
[directory deployment](https://www.koyeb.com/docs/build-and-deploy/deploy-project-directory)

## 2. Prepare the database connection

The latest read-only catalog check confirms that `armstrong_api` does not exist
in the configured runtime project. The existing local `DATABASE_URL` uses the
administrative owner and must stay in the controlled local setup. API production
startup refuses administrative/built-in role names.

Provision and verify a separate runtime login with the existing API's restricted
grants and RLS requirements, described in [README](../README.md). It must read
identity mappings, perform member operations and allocate change sequences,
without administering staff/devices, deleting history or running migrations.
Keep owner credentials and Administrator passwords out of Koyeb.

Use the project's **Session pooler**, port **5432**, database **postgres**, and
pooler username `armstrong_api.<project-ref>`. URL-encode the password. Use the
actual hostname from Supabase's Connect dialog and preserve verified TLS.
[Supabase roles](https://supabase.com/docs/guides/database/postgres/roles),
[Supabase connections](https://supabase.com/docs/guides/database/connecting-to-postgres)

In Koyeb's service configuration, expand **Environment variables and files**,
open **Files**, and add the actual database CA certificate at:

```text
/etc/armstrong/database-ca.crt
```

Use permissions `0644` for this public CA certificate, with the process user as
owner. Paste only the required PEM certificate, not a private key. Koyeb mounts
this file at runtime; a workstation certificate path is not valid remotely.
[Config files](https://www.koyeb.com/docs/build-and-deploy/config-files)

The Koyeb-only database secret follows this structure; fill it privately:

```text
postgresql://armstrong_api.PROJECT_REF:ENCODED_PASSWORD@ACTUAL_SESSION_POOLER_HOST:5432/postgres?sslmode=verify-full&sslrootcert=%2Fetc%2Farmstrong%2Fdatabase-ca.crt
```

## 3. Service settings

Create a **Web Service** from the GitHub repository and configure all settings
and the certificate before starting its first deployment.

| Setting | Value |
| --- | --- |
| App name | `armstrong-fitness` |
| Service name | `api` |
| Region | Singapore (`sin`) |
| Instance | `eco-micro`, 512 MB RAM, 0.25 vCPU |
| Scaling | Fixed, one instance; scale-to-zero disabled |
| Builder | Buildpack / Node.js |
| Work directory | `server` |
| Build command override | `npm ci --include=dev --ignore-scripts --no-audit --no-fund && npm run build:koyeb` |
| Run command override | `npm start` |
| Exposed port | `3000`, HTTP |
| Public route | `/` to port `3000` |
| Health check | HTTP `GET /health` on port `3000` |
| Health settings | 30-second grace period; 60-second interval; 5-second timeout |
| Automatic Git deployment | Off initially |

`package.json` and the lockfile specify Node `24.x`, which Koyeb supports.
`Procfile` also defines `web: npm start`. Koyeb runs its automated dependency
installation before a custom build command, so the explicit clean install keeps
the final build aligned with this project's locked verification procedure.
`build:koyeb` performs strict typechecking and unit/mock tests; it does not run
migrations, registration or live database/Auth integration tests.
[Node runtime](https://www.koyeb.com/docs/build-and-deploy/build-from-git/nodejs),
[build process](https://www.koyeb.com/docs/build-and-deploy/build-from-git),
[work directory](https://www.koyeb.com/docs/build-and-deploy/monorepo),
[HTTP health checks](https://www.koyeb.com/docs/run-and-scale/health-checks)

Eco Micro's listed compute is about **$2.68/month** for one continuously running
instance, before taxes and other applicable charges. Measure actual memory and
database latency; Eco Small provides 1 GB at about $5.36/month if needed. Free
instances sleep and do not offer Singapore. Fixed scaling keeps a paid instance
running without a keep-alive monitor.
[Instance resources/pricing](https://www.koyeb.com/docs/reference/instances),
[fixed scaling](https://www.koyeb.com/docs/reference/scaling)

## 4. Environment variables

| Variable | Koyeb value |
| --- | --- |
| `NODE_ENV` | `production` |
| `HOST` | `0.0.0.0` |
| `PORT` | `3000` |
| `NPM_CONFIG_PRODUCTION` | `false` (make build-time typechecking dependencies available) |
| `NPM_CONFIG_IGNORE_SCRIPTS` | `true` (disable dependency install lifecycle scripts) |
| `DATABASE_URL` | Secret containing the restricted connection and mounted CA path above |
| `SUPABASE_URL` | Existing runtime project's canonical HTTPS Auth origin |
| `SUPABASE_PUBLISHABLE_KEY` | Existing public publishable/legacy anon key; never service-role/secret key |
| `PUBLIC_API_ORIGIN` | `https://{{ KOYEB_PUBLIC_DOMAIN }}` |

Koyeb interpolates `KOYEB_PUBLIC_DOMAIN` in configured variable values. This
provides the generated canonical HTTPS origin on the first startup; no invented
hostname or trailing slash is needed. Store the database connection as a Koyeb
Secret and reference it in the environment form. Local registration names, native
database paths, device hashes, Auth-check passwords and test env values are not
API runtime requirements and must not be uploaded.
[Environment interpolation](https://www.koyeb.com/docs/build-and-deploy/environment-variables),
[Secrets](https://www.koyeb.com/docs/reference/secrets)

## 5. Deployment acceptance

Deploy, wait for **Healthy**, and copy the actual `https://...koyeb.app` origin.
Koyeb supplies TLS for its domains. Open `<actual-origin>/health`; expect:

```json
{"status":"ok","service":"armstrong-member-api","protocolVersion":1}
```

The API connects to PostgreSQL before listening. A startup failure about a
restricted login or readable CA must be fixed through the correct credentials
or mounted file, rather than weakening TLS or using an owner login. The health
response validates process/protocol only; it does not prove ongoing database
access, runtime grants, Administrator enrollment or synchronization.
[Domains and TLS](https://www.koyeb.com/docs/run-and-scale/domains)

Save only the actual canonical `PUBLIC_API_ORIGIN` in ignored local `.env`,
preserving its administrative connection and existing installation values. From
`server/`, run:

```sh
npm run api:check
npm run setup:check
```

Then complete actual native device preparation and approved Administrator/device
registration, followed by public desktop config review/provisioning and real
sign-in/enrollment. See [local setup](LOCAL_SETUP.md) and
[staff/device setup](STAFF_DEVICE_SETUP.md). API hosting can be verified before
device enrollment; it does not require a fake device or test registration.

Check the deployment on Node 24, restart behavior, memory, restricted SQL
privileges and real TLS/desktop requests before daily operation. Live member
scheduling is still disabled; other-module sync, restore reconciliation and
Windows/NFC/printing/installer acceptance remain separate unfinished work.
