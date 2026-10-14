# Deployment and rollback

Release Please maintains a release pull request from Conventional Commits after
the main-branch CI workflow passes. Merging that pull request creates the
versioned GitHub release and publishes API, gateway, MCP, and web images to GitHub
Container Registry:

- `ghcr.io/imlunahey/orivane-api`
- `ghcr.io/imlunahey/orivane-gateway`
- `ghcr.io/imlunahey/orivane-mcp`
- `ghcr.io/imlunahey/orivane-web`

Each image receives the full semantic version, major/minor, major, `latest`, and
source-commit tags. Production deployments must pin an exact semantic version
or SHA tag; never deploy the floating `latest` tag. Features create minor
versions, fixes create patch versions, and breaking changes create major
versions once the project reaches `1.0.0`.

## Pre-deployment checklist

1. CI, security checks, migrations, and the container smoke build are green.
2. A PostgreSQL backup has completed and passed `pg_restore --list` validation.
3. The release notes identify schema changes and Discord command changes.
4. `AUTH_DEVELOPMENT_BYPASS` is absent or `false`.
5. The API readiness endpoint is used for traffic admission; the liveness endpoint is used only for restarts.

## Rolling deployment

Set `ORIVANE_VERSION` to an exact released semantic version in `.env`, then use
the production override for every Compose operation:

```sh
docker compose -f docker-compose.yml -f docker-compose.production.yml pull api gateway mcp web
docker compose -f docker-compose.yml -f docker-compose.production.yml up -d
```

The override removes local build definitions and refuses to render without a
configured version. Always supply a full semantic version such as `0.1.0`;
keep the base Compose file for local source builds.

1. Deploy the API first. Migrations are backwards-compatible and applied under a database advisory lock.
2. Wait for API readiness.
3. Deploy gateway workers one at a time. Allow each process to drain before termination so interactions and claimed jobs complete.
4. Deploy the MCP service, verify `/health`, and confirm unauthenticated MCP
   requests are rejected.
5. Deploy the web image last.
6. Verify health, shard sessions, interaction error rate, Discord REST rate limits, and job backlog.

## Gateway sharding and scheduler ownership

Run one gateway process per Discord shard and set both variables explicitly:

- `DISCORD_SHARD_TOTAL` is the total shard count shared by the deployment.
- `DISCORD_SHARD_ID` is the zero-based shard handled by that process and must be lower than the total.

The defaults are `0` of `1`, which is appropriate only for a single gateway worker. Never run two processes with the same shard ID. Every gateway may start the scheduler loop, but a short Redis `SET NX` lease elects exactly one worker for each 15-second tick. Durable jobs still use PostgreSQL claims and leases as a second idempotency boundary. A Redis outage therefore pauses scheduler work instead of risking duplicate delivery; Discord event handling continues and readiness exposes the dependency failure.

Change shard totals only during a coordinated gateway restart. Confirm Discord's recommended shard count, stop the old gateway set, deploy every new shard ID exactly once, then check connected guild totals and gateway event rates.

## Rollback

1. Stop rollout immediately and record the release SHA in the global audit timeline.
2. Redeploy the previous exact API, gateway, MCP, and web image tags in reverse order: web, MCP, gateway, API.
3. Do not automatically reverse database migrations. All migrations must be additive or otherwise compatible with the preceding release.
4. If data restoration is necessary, stop every writer, restore into a new database, validate row counts and migrations, then switch the connection string. Never restore over a live database.
5. Re-run health and a private-guild slash-command smoke test before restoring normal traffic.

## Required recovery exercises

- Quarterly PostgreSQL point-in-time restore into an isolated database.
- Monthly Redis loss simulation; durable jobs must recover from PostgreSQL.
- Quarterly single-worker and single-shard failure drill.
- Before major releases, restore the previous release against the new schema to prove rollback compatibility.

