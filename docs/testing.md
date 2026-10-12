# Discord testing framework

The bot tests against its own `DiscordApi` contract instead of making network calls. `TwilightDiscord` is the production adapter and `FakeDiscord` is the deterministic test adapter. Every Discord operation used by a module must be added to this contract; direct use of `twilight_http::Client` inside modules is not allowed.

This makes the fake complete for the bot-facing Discord surface without attempting to reproduce unrelated Discord endpoints. As moderation, roles, messages, channels, webhooks, and autocomplete are added, their typed methods and recorded `DiscordCall` variants are added at the same time.

## What the fake supports

- Records command registration and interaction responses in call order.
- Returns recorded calls for exact assertions or drains them between test phases.
- Injects deterministic failures into the next operation, allowing rate-limit, permission, timeout, and server-error paths to be tested.
- Uses normalized bot-owned request and response types, so module tests do not depend on Twilight payload internals.
- Never opens a socket and needs no Discord token, PostgreSQL instance, or Redis instance.

Example:

```rust
let discord = FakeDiscord::new();
let context = BotContext {
    discord: Arc::new(discord.clone()),
};

PingModule
    .on_command(&context, &config, &interaction)
    .await?;

assert_eq!(
    discord.calls(),
    vec![DiscordCall::RespondToCommand {
        interaction_id: interaction.id,
        response: CommandResponse::public("Pong!"),
    }],
);
```

## Testing layers

1. **Module tests** use `FakeDiscord` and in-memory repository fakes. These cover most behavior and run on every edit.
2. **Dispatcher tests** feed normalized interactions through all middleware, permission checks, analytics, and module routing.
3. **Adapter contract tests** run `TwilightDiscord` against a local HTTP fixture server to verify Discord request paths and payload serialization. These protect the production adapter when Twilight or Discord API versions change.
4. **Gateway simulation tests** replay captured, redacted gateway fixtures through the event normalizer. Reconnection, sequence, resume, and heartbeat behavior belongs in a dedicated WebSocket simulator rather than individual module tests.
5. **Sandbox smoke tests** use a dedicated Discord application and private guild only in an explicitly enabled CI job. They confirm assumptions that cannot be reproduced locally and never gate ordinary pull requests.

Captured fixtures must be redacted and checked into `tests/fixtures`. Tokens, message content, usernames, real guild IDs, and real user IDs are forbidden in fixtures. Stable synthetic snowflakes make failures reproducible.

## Required scenarios for every command

- Valid invocation and exact Discord response/action.
- Missing bot permissions and missing caller permissions.
- Discord 429, 403, 404, timeout, and 5xx behavior where relevant.
- Duplicate delivery/idempotency.
- Invalid or deleted channels, roles, users, and messages.
- Analytics outcome and correlation fields without sensitive content.
- Public versus ephemeral error responses.

