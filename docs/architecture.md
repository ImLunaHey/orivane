# Platform architecture

## System shape

```text
Vite + React dashboard
  TanStack Router / Query / Table / Form
  Discord OAuth2 + secure server-side session
                       |
                Axum control API
                  /          \
        PostgreSQL            Redis
  config, cases, XP      cache, limits, streams
                  \          /
             Twilight gateway workers
                       |
               analytics event stream
                       |
               analytics consumer
                  /          \
          ClickHouse       OpenTelemetry Collector
        product events      logs / metrics / traces
                  \          /
                     Grafana
```

Trusted AI agents reach the same control API through the `discord-bot-mcp`
adapter. The adapter exposes an explicit `/v1` route allowlist over MCP rather
than direct database access, uses a separate owner-level bearer credential,
and preserves the API's validation, authorization, rate limiting, and audit
boundaries.

The gateway is the data plane: it stays connected to Discord and handles events. The API is the control plane: it owns configuration changes and dashboard reads. They share domain types and persistence code, but deploy independently.

PostgreSQL remains the source of truth for operational data. Redis is disposable acceleration and delivery infrastructure, not the only copy of important data. ClickHouse is intentionally separate: high-volume analytics queries must not compete with moderation commands or configuration updates.

## Observability from day one

There are two related but distinct streams:

1. **Operational telemetry** answers “is the bot healthy?” Structured logs, metrics, and traces go through OpenTelemetry to Grafana/Loki/Tempo/Prometheus (or compatible hosted backends).
2. **Product analytics** answers “how is the bot being used?” Immutable, versioned events go through a Redis Stream to a batching consumer and then ClickHouse.

Every HTTP request, Discord event, command execution, scheduled job, and queue job gets a correlation ID and a tracing span. Standard fields are:

| Field | Purpose |
| --- | --- |
| `trace_id`, `request_id` | Follow one operation across API, queue, and worker |
| `guild_id`, `shard_id` | Locate tenant and gateway ownership |
| `module`, `event`, `event_version` | Stable event classification |
| `command`, `outcome`, `error_code` | Usage and failure analysis |
| `latency_ms`, `queue_delay_ms` | Performance and saturation |
| `deployment`, `version` | Regression comparison |

IDs may be recorded, but message content, access tokens, authorization headers, invite codes, and unredacted user input must never be emitted to telemetry. Content-based automod records the matched rule ID and action, not the message. User IDs should be pseudonymized in long-lived analytics unless a moderation case requires the real ID.

### Core metrics

- Gateway availability, reconnects, heartbeat latency, events/sec, and events dropped.
- Command count, success rate, permission denials, Discord REST errors, and p50/p95/p99 latency by command.
- API rate, errors, latency, active sessions, and configuration update count.
- PostgreSQL pool use/query latency; Redis latency/hit ratio; queue depth/age/retries/dead letters.
- Module outcomes: automod matches/actions, moderation actions, XP awards/cooldown skips, welcome delivery failures.
- Business totals: active guilds, new/removed guilds, daily/weekly active guilds, enabled modules, commands per active guild.

Cardinality must be controlled: guild IDs belong in logs/traces and ClickHouse events, not as Prometheus label values. Metrics use bounded labels such as command name, module, outcome, and shard.

### Analytics event envelope

All producers use one versioned envelope:

```json
{
  "event_id": "uuid-v7",
  "event": "command.completed",
  "event_version": 1,
  "occurred_at": "2026-10-11T20:48:00Z",
  "guild_id": "123456789",
  "shard_id": 0,
  "module": "moderation",
  "actor_id_hash": "...",
  "properties": {
    "command": "ban",
    "outcome": "success",
    "latency_ms": 84
  }
}
```

Delivery is at least once, so `event_id` is the deduplication key. Producers write asynchronously to a bounded channel and Redis Stream; analytics failure must not block a Discord response. The consumer batches inserts, retries with backoff, and moves poison events to a dead-letter stream.

Suggested retention:

- Debug logs: 7–14 days.
- Normal operational logs and traces: 30 days.
- Raw product events: 90 days.
- Hourly/daily ClickHouse rollups: 13–25 months.
- Moderation audit records: configurable per guild, stored in PostgreSQL rather than analytics.

## Dashboard

The website will be a client-side TypeScript application built with Vite, React, and Tailwind CSS. Next.js is explicitly out of scope. Prefer TanStack packages where they fit:

- TanStack Router for typed routes and search parameters.
- TanStack Query for server state, caching, mutations, and optimistic configuration updates.
- TanStack Table for audit logs, cases, server directories, and global user directories.
- TanStack Form for typed module configuration with shared JSON-schema validation where practical.
- TanStack Charts through a shared chart component that requires labelled axes and an accessible rich tooltip.

Primary areas are Overview, Modules, Moderation, Automod, Levels, Audit Log, Analytics, and Developer/Health. An owner-only operations area shows shard state, error groups, queue lag, cache health, deploy markers, and links from analytics points to correlated traces.

Discord OAuth2 authenticates users. A backend-for-frontend session exchanges and stores OAuth tokens server-side in an encrypted session; browser JavaScript never receives the bot token or internal API token. Every guild route verifies the current user still has `MANAGE_GUILD`. Owner routes require a separate allowlist and step-up authentication.

## Audit trail

Audit events are durable PostgreSQL records, separate from high-volume product analytics. The shared event shape records scope, guild, actor, action, target, outcome, source, request ID, reason, metadata, and optional before/after state. The API records authenticated reads, rejected and successful configuration writes, while gateway dispatch records every slash-command outcome. New mutation paths must use the same `record_audit_event` boundary before they are considered complete.

The dashboard exposes `/audit` for the bot-wide owner view and `/servers/:serverId/audit` for a server-scoped view. Both use the same table, filters, event inspector, and CSV export. Secrets, tokens, authorization headers, and message bodies must never be placed in audit metadata or state snapshots.

## Service boundaries

### API

- OAuth callback/session endpoints and authorization checks.
- Guild configuration reads/writes with validation and audit history.
- Moderation case and dashboard query endpoints.
- Aggregated analytics reads; it never exposes arbitrary ClickHouse SQL.
- Rate limits by session, user, guild, and route class.

### MCP

- Local stdio and authenticated Streamable HTTP transports.
- Discoverable read-only resources plus separate read and mutation tools.
- Static allowlist covering customer-facing, server-scoped `/v1` operations;
  fleet, auth, metrics, and raw database access are never exposed.
- MCP mutations use the normal API handlers and are audited as the Discord
  user who created the server-scoped token.
- Expiring, revocable, hashed tokens, origin validation, response-size limits,
  and no redirects when contacting the API.

### Documentation site

- A separate client-only Vite, React, Tailwind, and TanStack Router application
  lives in `apps/docs`; it has no session, database, or runtime API dependency.
- Its catalog covers every serialized `GuildConfig` module field and is tested
  against the Rust source so new configuration cannot silently remain
  undocumented.
- Interactive examples are deterministic browser-local models. They are
  labelled as demos and never call Discord or save configuration.
- The production web proxy mounts the static documentation image at `/docs/`;
  releases also publish it independently as `orivane-docs`.

### Gateway workers

- Discord shard lifecycle and event normalization.
- Fast configuration reads through Redis.
- Module dispatch and Discord REST actions.
- Non-blocking telemetry and analytics emission.
- No dashboard-specific rendering or session logic.

### Background workers

- Analytics stream consumption and ClickHouse batches.
- Scheduled posts, temporary moderation expiry, XP aggregation, exports, and deletion requests.
- Retry policies, idempotency keys, and dead-letter handling per job type.

## Module contract

Modules receive normalized events plus an immutable request context containing configuration, Discord clients, trace context, and service interfaces. They return a typed outcome. The dispatcher owns timeouts, panic isolation, metrics, and analytics emission so every module gets the same operational behavior.

Every module is a separate workspace crate under `modules/`. Shared event and context types live in `bot-module-api`; the gateway must not contain feature implementations. A module crate owns its handlers, command definitions, and deterministic tests, while `bot-core` owns the serialized guild configuration contract and persistence.

Discord itself sits behind the bot-owned `DiscordApi` contract. Modules never call Twilight directly. The production adapter translates this contract to Discord HTTP calls, while the test adapter records every operation and injects deterministic failures. Adapter contract tests and a gateway fixture simulator cover the protocol boundary separately; see [testing.md](testing.md).

The bot exposes commands exclusively as Discord application (slash) commands; prefix commands are not supported. The Message Content intent is enabled only if an automod feature genuinely requires message inspection, never for command handling. The first production modules should be moderation, audit logging, automod, leveling, welcome messages, reaction roles, and custom commands.

## Delivery phases

1. Foundation: configuration API, gateway lifecycle, structured JSON logs, trace propagation, metrics endpoint, migrations, and CI.
2. Moderation: application commands, permissions, cases, audit log, and timed actions.
3. Analytics backbone: event SDK, Redis Stream, consumer, ClickHouse schema, Grafana dashboards, alerts, and privacy controls.
4. Dashboard: OAuth/session backend, Vite application, module settings, moderation views, and owner analytics.
5. Growth modules: automod, leveling, welcome, roles, custom commands, and scheduling.
6. Scale: recommended shard discovery, multi-worker coordination, regional deployment, load tests, SLOs, and disaster recovery exercises.

