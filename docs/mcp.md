# MCP agent access

Orivane includes a Rust Model Context Protocol server that gives trusted AI
agents access to the same validated and audited `/v1` operations used by the
dashboard. It does not connect directly to PostgreSQL, expose Lucid Auth
routes, or provide arbitrary HTTP requests.

The server implements the current Streamable HTTP transport and local stdio
transport using the official Rust MCP SDK.

## Security model

`ORIVANE_MCP_TOKEN` is a dedicated owner-level service credential shared by
the API and MCP process. Generate a unique random value containing at least 32
characters. Never reuse `BETTER_AUTH_SECRET`, the Discord bot token, or an OAuth
secret, and never commit the value.

The API accepts this bearer token only as the synthetic owner actor
`orivane-mcp`. Normal dashboard sessions continue to use Discord OAuth and
Lucid Auth. Every MCP mutation therefore passes through the same request
validation and audit-writing handler as its dashboard equivalent.

Additional boundaries are:

- a static method-and-path allowlist for supported `/v1` operations;
- separate read and mutation tools so clients can require approval for writes;
- no redirects when the MCP service contacts the API;
- a 2 MiB response limit;
- constant-time bearer-token comparisons;
- HTTP `Origin` validation through `MCP_ALLOWED_ORIGINS` or `APP_ORIGIN`;
- loopback-only Compose port publishing, with remote access through the
  existing authenticated tailnet boundary.

Treat the token as full bot-owner access. Rotate it immediately if it is
exposed, then restart both the API and MCP containers.

## Tools

| Tool | Purpose |
| --- | --- |
| `api_endpoints` | Lists every allowlisted method, path template, description, and mutation flag. |
| `api_get` | Calls an allowlisted read endpoint with URL-encoded query parameters. |
| `api_mutate` | Calls an allowlisted `POST`, `PUT`, `PATCH`, or `DELETE` endpoint with an optional JSON body. |

The result includes the HTTP status, request ID, content type, and parsed body.
Non-success API responses become MCP tool errors while retaining the API error
body for the agent.

The MCP server also publishes these read-only resources:

- `orivane://api/endpoints`
- `orivane://fleet/health`
- `orivane://fleet/servers`

## Local stdio

Build the binary, inject the two required values through the agent host's
secret/environment configuration, and launch it as a child process:

```sh
cargo build --release -p discord-bot-mcp
MCP_TRANSPORT=stdio \
ORIVANE_API_BASE_URL=http://127.0.0.1:3010/ \
ORIVANE_MCP_TOKEN="$ORIVANE_MCP_TOKEN" \
target/release/discord-bot-mcp
```

A generic client entry looks like this; use the client's secret mechanism
instead of writing the token directly into a tracked file:

```json
{
  "mcpServers": {
    "orivane": {
      "command": "/absolute/path/to/target/release/discord-bot-mcp",
      "env": {
        "MCP_TRANSPORT": "stdio",
        "ORIVANE_API_BASE_URL": "http://127.0.0.1:3010/",
        "ORIVANE_MCP_TOKEN": "<inject-secret-here>"
      }
    }
  }
}
```

## Streamable HTTP

Compose starts the MCP process on the private network and publishes its health
port only on loopback. Nginx proxies the protocol endpoint at `/mcp`, so the
deployed endpoint is:

```text
https://your-orivane-host.example/mcp
```

Clients must send:

```text
Authorization: Bearer <ORIVANE_MCP_TOKEN>
```

The health endpoint is intentionally separate and returns no operational data:

```sh
curl --fail http://127.0.0.1:3012/health
```

Set `MCP_ALLOWED_ORIGINS` to a comma-separated list when browser-hosted MCP
clients use additional trusted origins. Requests without an `Origin` header
still require the bearer token.

## Agent guidance

Agents should call `api_endpoints` before constructing a path, prefer
`api_get`, and ask for explicit user confirmation before `api_mutate`. A read
followed by a mutation should use the current returned object as the basis for
the update rather than inventing omitted configuration fields.
