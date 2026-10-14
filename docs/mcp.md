# MCP access for Discord server owners

Orivane exposes a hosted Model Context Protocol endpoint so an MCP-compatible
assistant can help configure and operate a Discord server. Customers do not
deploy the bot, run this repository, expose a local port, or handle Orivane's
Discord bot token.

## Create a connection

1. Sign in to the Orivane dashboard with Discord.
2. Select a server where you have Manage Server or Administrator.
3. Open **Integrations → AI agent access**.
4. Name the connection, choose an expiry, and create the access token.
5. Copy the token into the MCP client's secret store. It is shown only once.

Configure the client with the hosted endpoint:

```text
URL: https://your-orivane-host.example/mcp
Authorization: Bearer <server access token>
```

## Authorization model

An MCP access token represents the Discord user who created it and exactly one
Discord server. It is not a bot-owner credential.

- The token cannot inspect Orivane fleet health, connected servers, global
  users, command registration, or other operator-only data.
- It cannot read or modify another Discord server, even when the same person
  manages that server.
- Token creation requires a signed-in dashboard session and live Discord
  Manage Server or Administrator permission.
- Tokens expire, are stored only as SHA-256 hashes, and can be revoked from the
  server's Integrations page.
- Mutations use the same validation and server audit path as dashboard changes
  and are attributed to the creating Discord user.

## MCP capabilities

The server publishes `orivane://server/context`, the current server
configuration, and an allowlist of supported server operations.
`server_capabilities` describes those operations, `server_read` inspects
state, and `server_change` changes live server configuration or performs a
supported workflow.

Agents should:

1. read `orivane://server/context` before doing other work;
2. inspect current state before proposing a change;
3. explain the Discord-facing effect and ask for confirmation before a write;
4. return the API request ID so the user can find the action in the audit log.

## Revocation

Open **Integrations → AI agent access** for the server and choose **Revoke**.
Revocation takes effect on the next API request. Revoke a token immediately if
it was pasted into chat, committed to a repository, or stored on a lost device.

## Maintainer-only local transport

The Rust binary also supports stdio for development and automated tests. This
is not part of the customer setup flow. It requires a server-scoped token in
`ORIVANE_USER_TOKEN`; operator automation credentials are not accepted.
