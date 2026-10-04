# Render API hosting preparation

The installation name is **ArmStrong Fitness** and the existing account's
Administrator display name is **ArmStrong**. Those names and a stable new gym
UUID are prepared in ignored local `server/.env`. They are configuration only;
no server gym/staff/device record has been inserted. Existing `Test gym` and
`Unrelated gym` records remain intact.

Render can host this Node/Fastify API; Supabase remains the database/Auth service
and the Tauri application runs on the gym computer. Render supplies an HTTPS
`onrender.com` origin after service creation. That actual origin becomes
`PUBLIC_API_ORIGIN` and the native `apiOrigin`.
Sources: [Node web service setup](https://render.com/docs/deploy-node-express-app),
[managed TLS](https://render.com/docs/tls).

## Instance choice

Use Free for short experiments. Render documents that free services spin down
after 15 minutes without requests and can take about a minute to wake. Native
HTTPS requests have a 15-second request timeout (18-second outer process limit);
backend probes have ten-second deadlines. A sleeping free service can therefore
make sign-in/enrollment fail. Choose a paid web-service instance for normal daily
operation. Current small web-service compute starts at $7/month; workspace,
bandwidth and Supabase costs are separate as applicable. This recommendation is
not a service creation or spending approval.
Sources: [free limitations](https://render.com/docs/free),
[current pricing](https://render.com/pricing).

The existing `render.yaml` still selects Free as a testing template. Select the
appropriate paid compute plan before a production deployment. Automatic deploys
are off, but creating a Blueprint still triggers its first deployment.

## Complete before service creation

1. Provision and verify the restricted API database login/grants. The local owner
   connection is for registration/migration/probes; production startup refuses it.
   The planned `armstrong_api` login was absent in the last real catalog check.
   Keep role/password preparation out of Git and chat. Requirements are in
   [README](../README.md) and [local setup](LOCAL_SETUP.md).
2. Register the one verified Administrator and actual prepared desktop device
   through the reviewed registration commands. Preserve unrelated registrations.
   See [single-account setup](STAFF_DEVICE_SETUP.md).
3. Provision the configured Supabase database CA on Render as a secret file.
   Render makes a file named `prod-ca-2021.crt` available at
   `/etc/secrets/prod-ca-2021.crt`; use the actual required CA contents. Set the
   deployed database URL's `sslrootcert` to that path (URL-encoded as needed) and
   retain `sslmode=verify-full`, Session pooler port 5432 and the correct project
   username/host. A developer-workstation CA path cannot be used remotely.
   [Render secret files](https://render.com/docs/configure-environment-variables#secret-files)
4. Have the project source in the Git repository connected to Render. Include
   `server/vendor/types-pg` and its license/provenance because the locked dev
   dependency uses it. Keep populated env, credentials and SQLite files local.

## Service form, after prerequisites

In Render choose **New → Web Service** and connect the project repository.
Use these prepared settings:

| Setting | Value |
| --- | --- |
| Runtime | Node |
| Root directory | `server` |
| Build command | `npm ci --include=dev --ignore-scripts --no-audit --no-fund && npm run typecheck && npm test` |
| Start command | `npm start` |
| Health check | `/health` |
| Node version | `NODE_VERSION=24` |
| Environment | `NODE_ENV=production`, `HOST=0.0.0.0` |
| Automatic deploy | Off |

Add only runtime `DATABASE_URL` (restricted login and deployed CA path),
`SUPABASE_URL`, `SUPABASE_PUBLISHABLE_KEY` and the actual `PUBLIC_API_ORIGIN`
through Render's environment settings. Registration/probe account credentials
and workstation paths stay in the local administrative setup.
Sources: [Node version](https://render.com/docs/node-version),
[environment settings](https://render.com/docs/configure-environment-variables).

Alternatively select `server/render.yaml` as the Blueprint path after reviewing
the chosen compute plan, environment values and certificate provisioning.
Creating the service publishes an endpoint; the user's previous no-public-
deployment instruction remains until they explicitly authorize this change.

After an authorized deployment, save the actual HTTPS origin in local `.env`,
run `npm run api:check`, finish native public configuration and restart/sign in.
API process health alone does not complete real enrollment or member-sync
acceptance. The background scheduler, other-module sync and Windows/hardware
release work remain separate unfinished milestones.
