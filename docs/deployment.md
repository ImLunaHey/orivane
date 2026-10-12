# Deployment and rollback

Orivane publishes immutable API, gateway, and web images for every `v*` Git tag. Each image is tagged with the semantic version and source commit. Production deployments must pin an exact semantic-version or SHA tag; never deploy a floating local build.

## Pre-deployment checklist

1. CI, security checks, migrations, and the container smoke build are green.
2. A PostgreSQL backup has completed and passed `pg_restore --list` validation.
3. The release notes identify schema changes and Discord command changes.
4. `AUTH_DEVELOPMENT_BYPASS` is absent or `false`.
5. The API readiness endpoint is used for traffic admission; the liveness endpoint is used only for restarts.

## Rolling deployment

1. Deploy the API first. Migrations are backwards-compatible and applied under a database advisory lock.
2. Wait for API readiness.
3. Deploy gateway workers one at a time. Allow each process to drain before termination so interactions and claimed jobs complete.
4. Deploy the web image last.
5. Verify health, shard sessions, interaction error rate, Discord REST rate limits, and job backlog.

## Rollback

1. Stop rollout immediately and record the release SHA in the global audit timeline.
2. Redeploy the previous exact API, gateway, and web image tags in reverse order: web, gateway, API.
3. Do not automatically reverse database migrations. All migrations must be additive or otherwise compatible with the preceding release.
4. If data restoration is necessary, stop every writer, restore into a new database, validate row counts and migrations, then switch the connection string. Never restore over a live database.
5. Re-run health and a private-guild slash-command smoke test before restoring normal traffic.

## Required recovery exercises

- Quarterly PostgreSQL point-in-time restore into an isolated database.
- Monthly Redis loss simulation; durable jobs must recover from PostgreSQL.
- Quarterly single-worker and single-shard failure drill.
- Before major releases, restore the previous release against the new schema to prove rollback compatibility.

