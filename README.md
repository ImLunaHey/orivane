# Discord bot platform

A Rust foundation for a multi-purpose Discord bot in the style of MEE6 or Dyno. It follows a control-plane/data-plane split:

```text
Vite + React dashboard + Discord OAuth2
                 |
          Axum configuration API
              /       \
       PostgreSQL     Redis
              \       /
        Twilight gateway workers
```

- **PostgreSQL** is the durable source of truth for guild settings, moderation cases, connected guilds, and audit history.
- **Redis** provides a five-minute read-through configuration cache and is the natural home for cooldowns, rate limits, and queues.
- **API** exposes authenticated configuration, fleet, users, moderation, audit, analytics, and health endpoints.
- **Gateway** owns Discord connections and dispatches events into isolated modules.

## Run locally

1. Create a Discord application and bot in the Developer Portal. The bot uses application (slash) commands rather than prefix commands. Enable the privileged Server Members intent for welcome and autorole events and Message Content intent for AutoMod, leveling, and message-triggered modules.
2. Copy `.env.example` to `.env`. Add the bot token, a random `BETTER_AUTH_SECRET` of at least 32 bytes, and the Discord OAuth client ID and secret. Add your Discord user ID to `BOT_OWNER_DISCORD_IDS` for bot-wide pages.
3. Start the complete stack (PostgreSQL, Redis, API, gateway, and web UI):

   ```sh
   docker compose up --build -d
   ```

   The dashboard is available at `http://127.0.0.1:5173` and the API at
   `http://127.0.0.1:3010`. The web container proxies `/api` and `/v1` to the
   API over the private Compose network, so no API URL is baked into the UI.

4. For development with hot reload, start only the infrastructure and run each
   process in its own terminal:

   ```sh
   docker compose up -d postgres redis
   set -a && source .env && set +a
   cargo run -p discord-bot-api
   cargo run -p discord-bot-gateway
   cd apps/web && VITE_API_BASE_URL=same-origin pnpm dev
   ```

The `.env` file is passed to the Rust containers at runtime and is excluded
from both Git and Docker build contexts. Never place Discord or OAuth secrets
in `apps/web`, `VITE_*` variables, Dockerfiles, or Compose build arguments.

The API automatically applies migrations. Test the worker with `/ping` in a server containing the bot. The gateway registers the global command when it starts; Discord may take a little time to propagate global command changes.

## Dashboard authentication

The Axum API mounts [Lucid Auth](https://github.com/lucid-softworks/auth) at `/api/auth`. The Vite app uses the Better Auth client to sign in with Discord; the signed session stays in an HttpOnly cookie. No bot token, OAuth access token, internal API key, or actor-ID header is exposed to browser JavaScript.

Discord OAuth requests `identify`, `email`, and `guilds`. Every `/v1/guilds/:guildId/*` request reauthorizes the session against Discord and requires `MANAGE_GUILD` or `ADMINISTRATOR`; bot-wide routes require the Discord-ID owner allowlist. `AUTH_DEVELOPMENT_BYPASS=true` is an explicit local-only escape hatch and must not be enabled in a deployed environment.

Update configuration with `PUT /v1/guilds/:guildId/config` and the same JSON shape returned by `GET`. The guild ID in the URL is authoritative. `VITE_API_BASE_URL=same-origin` uses Vite's local `/api` and `/v1` proxy; an absolute API origin works for separate deployments. The dashboard uses local demo data only when that variable is absent.

## Recommended build order

1. Moderation cases, audit logging, warn/mute/ban commands.
2. Automod rules and Redis-backed spam/mention limits.
3. Leveling with buffered XP writes and Redis cooldowns.
4. Welcome messages, reaction roles, custom commands, and scheduled posts.
5. Horizontal sharding, queue workers, metrics, and deployment manifests.

Every feature implements the shared `BotModule` boundary; persistence belongs in `bot-core`, while Discord protocol handling stays in the gateway app.

Each feature lives in its own crate under `modules/`. The gateway only assembles the registry and dispatches normalized commands and events through `bot-module-api`. Current live crates are `utility`, `welcome`, `autorole`, and `logging`; moderation, automod, leveling, starboard, tickets, and custom commands already have isolated crates and validated server configuration ready for their event handlers.

The full design—including the Vite/React/Tailwind/TanStack dashboard, logging, metrics, tracing, and analytics roadmap—is in [docs/architecture.md](docs/architecture.md). The current dashboard includes bot-wide and guild-scoped navigation, accessible shared charts, TanStack tables, audit detail pages, live dependency health, and explicit demo/live data modes.

Discord-facing code is testable without Discord: production uses the Twilight adapter while tests use a deterministic in-memory fake that records calls and injects failures. See [docs/testing.md](docs/testing.md).
