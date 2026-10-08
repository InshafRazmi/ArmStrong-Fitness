# Render API hosting preparation

## Recover the retained Staff transaction

On 2026-10-08, GitHub main is still
`6cb5aef06bdc9a36f53872ff638e9663156c9b6c`; its row contract does not contain
Staff tables. Render `/v2/health` still lacks `businessSchemaVersion`. The
installed schema-10 desktop's refused transaction adds `trainers` and `audit`.
All 22 retained requests pass the current server's row/state validation against
production data in a read-only check. Production migrations 1–8 and restricted
runtime readiness are verified; the running API needs the current source.

1. Review, commit and push the current `server/` source to the branch linked to
   this existing service. Include the updated row manifest, validator, history
   guards, readiness checks and migrations. The verified standalone
   `server/armstrong-render-source.zip` is available for transfer/review. Keep
   populated env, certificates and local databases outside the repository.
2. In the existing Render service, choose **Manual Deploy → Deploy latest
   commit** and wait for a successful live deployment. Preserve the restricted
   database login, `/etc/secrets/hi3.crt` and existing runtime settings. Restart
   service runs the same previously deployed commit.
   [Render deployment options](https://render.com/docs/deploys#manual-deploys)
3. Verify `https://armstrong-fitness.onrender.com/v2/health` includes
   `"businessSchemaVersion":10` alongside `"protocolVersion":2`.
4. In the desktop application, open **Settings → Server synchronization →
   Review retained transaction → Retry original transaction**. This preserves
   the frozen operation and sends it to the corrected API for its real receipt.

The user handles commit/push. The Render plugin is available but unconnected,
and no Render CLI login, API key or deploy hook is configured locally. Live
deployment and retained-request acceptance remain pending that access/source
publication. Local changes and database migrations alone do not replace Render's
running application code.

Render is now the selected provider. Following the free-hosting discussion,
the setup target is **Free in Singapore** for testing. The latest user selection
authorizes this Render setup and supersedes the earlier hosting decision gate.
Finishing live setup still needs account/repository access and restricted database
credentials with the mounted CA. The public health endpoint was independently
reached on 2026-10-05. Protocol-2 source publication/deployment and live native
acceptance are still pending; health alone does not identify the deployed revision.

The installation name is **ArmStrong Fitness** and the existing account's
Administrator display name is **ArmStrong**. Those names and a stable new gym
UUID are prepared in ignored local `server/.env`. The confirmed account
`armstrong@gmail.com` is now registered against that gym as Administrator.
Account-authorized computer approval SQL is provisioned; the API source/feature
flag still needs deployment. Existing unrelated records remain intact. Follow
[automatic computer setup](AUTOMATIC_COMPUTERS.md).

Render hosts this Node/Fastify API; Supabase remains the database/Auth service
and the Tauri application runs on the gym computer. Render supplies an HTTPS
`onrender.com` origin after service creation. That actual origin becomes
`PUBLIC_API_ORIGIN` and the native `apiOrigin`. For the first hosted startup,
the API can use Render's automatic `RENDER_EXTERNAL_URL` while `RENDER=true`.
It accepts only a canonical HTTPS `onrender.com` origin without a non-default
port; explicit `PUBLIC_API_ORIGIN` takes precedence for custom domains. Local
desktop setup still requires the actual origin copied explicitly into `.env`.
Sources: [Node web service setup](https://render.com/docs/deploy-node-express-app),
[managed TLS](https://render.com/docs/tls),
[automatic environment](https://render.com/docs/environment-variables).

## Instance choice

Free is selected for initial testing. Render documents that free services spin down
after 15 minutes without requests and can take about a minute to wake. Native
HTTPS requests have a 15-second request timeout (18-second outer process limit);
backend probes have ten-second deadlines. A sleeping free service can therefore
make the first sign-in/enrollment request fail. Free services can also restart
at any time and share 750 monthly free hours per workspace. Check actual cold
starts before daily use; a paid instance remains an option later.
Source: [free limitations](https://render.com/docs/free).

`render.yaml` selects Free in Singapore and disables automatic deploys. Creating
a Blueprint still triggers its first deployment. The source upload bundle
`server/armstrong-render-source.zip` includes `server/render.yaml` and all code
required for the locked build, with credentials/certificates/native data excluded.
Extract it into a separate folder and upload its `server/` directory to a private
GitHub repository if no project repository exists yet. Use this current snapshot;
the earlier Koyeb zip predates the Render startup changes.

## Complete before service creation

1. Provision and verify the restricted API database login/grants. The local owner
   connection is for registration/migration/probes; production startup refuses it.
   `armstrong_api` is now provisioned on the connected project, with actual
   grants/denials verified. The ignored local `.env.render` contains its new
   connection; use it in Render instead of the owner connection. Keep the password
   private. See [runtime database setup](RUNTIME_DATABASE_SETUP.md).
2. Provision the configured Supabase database CA on Render as a secret file.
   The current prepared connection uses the user's configured filename `hi3.crt`,
   available at `/etc/secrets/hi3.crt`; use the actual required CA contents. Set the
   deployed database URL's `sslrootcert` to that path (URL-encoded as needed) and
   retain `sslmode=verify-full`, Session pooler port 5432 and the correct project
   username/host. The local upload copy is `server/certs/prod-ca-2021.crt`.
   A developer-workstation CA path cannot be used remotely.
   [Render secret files](https://render.com/docs/configure-environment-variables#secret-files)
3. Have the project source in the Git repository connected to Render. Include
   `server/vendor/types-pg` and its license/provenance because the locked dev
   dependency uses it. Keep populated env, credentials and SQLite files local.

Actual Administrator/device registration follows verified API hosting and native
preparation. The API can start before enrollment; no fake registration is needed
for a health check. Preserve unrelated registrations and follow
[single-account setup](STAFF_DEVICE_SETUP.md).

## Service form, after prerequisites

In Render choose **New → Web Service** and connect the project repository.
Use these prepared settings:

| Setting | Value |
| --- | --- |
| Runtime | Node |
| Region | Singapore |
| Instance | Free |
| Root directory | `server` |
| Build command | `npm ci --include=dev --ignore-scripts --no-audit --no-fund && npm run build:verify` |
| Start command | `npm start` |
| Health check | `/health` |
| Node version | `NODE_VERSION=24` |
| Environment | `NODE_ENV=production`, `HOST=0.0.0.0` |
| Computer approval | `AUTOMATIC_DEVICE_ENROLLMENT=true` after deploying migration version 2 |
| Automatic deploy | Off |

Add only runtime `DATABASE_URL` (restricted login and deployed CA path),
`SUPABASE_URL` and `SUPABASE_PUBLISHABLE_KEY` through Render's environment settings.
The default `onrender.com` origin comes from Render automatically. Set
`PUBLIC_API_ORIGIN` explicitly only for a custom API domain. Registration/probe account credentials
and workstation paths stay in the local administrative setup.
Sources: [Node version](https://render.com/docs/node-version),
[environment settings](https://render.com/docs/configure-environment-variables).

Alternatively select `server/render.yaml` as the Blueprint path after reviewing
the chosen compute plan, environment values and certificate provisioning.
Creating the service publishes an endpoint; the user's previous no-public-
deployment gate is superseded by the current request to set up Render. Account
access and the actual runtime database prerequisites still need completion.

## Troubleshoot a successful build with failed startup

The supplied October 4 logs show **66 tests passed**, **Build successful**, then
`API startup failed (details_withheld)` and exit 1. Keep the build/start commands;
the failure is during process startup. `.env not found. Continuing without it.`
is expected with the optional env-file flag: runtime values come from Render's
Environment settings, and a populated `.env` stays local.

Verify `DATABASE_URL`, `SUPABASE_URL` and `SUPABASE_PUBLISHABLE_KEY` are present
and contain real values. On older source, also set `PUBLIC_API_ORIGIN` to the
actual service HTTPS origin without a trailing slash, path or query. The supplied
test count predates the current Render-origin test, which suggests older source;
the count alone does not establish the deployed commit. Setting an explicit
origin also works with current source. Keep `NODE_ENV=production` and
`HOST=0.0.0.0`. Use **Save and deploy** after changing runtime settings.
[Environment settings](https://render.com/docs/configure-environment-variables)

The refreshed `armstrong-render-source.zip` includes safe configuration
diagnostics. Replace the repository's source from this bundle and deploy its
latest commit to receive the fix; changing local files cannot update Render.
Missing/placeholder values now identify the variable name, and missing HTTPS
origin, invalid port or disabled TLS report their fixed configuration messages.
Unexpected errors and provider payloads remain withheld. Database login
restrictions and readable CA/TLS verification are still required; do not copy
the administrative owner connection into the service. If startup still fails,
share the new error and deployed repository/commit without credential values.

## Optional external uptime monitor

After real deployment, an external monitor can request the public `/health`
endpoint every 5–10 minutes without credentials. Based on Render's documented
inbound-traffic idle rule, this should reset its idle timer; the interval is an
inference, not an uptime guarantee. Exactly 15 minutes leaves no delay margin.
Render's internal health checks test process readiness; configuring them does
not remove the Free instance limits. Monitoring still uses running-instance
hours and does not prevent provider restarts. No external monitor was created.
[Free idle behavior](https://render.com/docs/free),
[platform health checks](https://render.com/docs/health-checks)

After an authorized deployment, save the actual HTTPS origin in local `.env`,
run `npm run api:check`, finish native public configuration and restart/sign in.
API process health alone does not complete real enrollment or member-sync
acceptance. The background scheduler, other-module sync and Windows/hardware
release work remain separate unfinished milestones.
