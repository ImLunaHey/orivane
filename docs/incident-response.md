# Incident response runbook

## First five minutes

1. Identify whether the API, web dashboard, Discord gateway, PostgreSQL, Redis, or Discord itself is failing. Check `/health/ready`, `/metrics`, structured logs, and the global audit timeline.
2. Record the release SHA, start time, impact, and incident owner. Never paste tokens, cookies, message content, or member data into the incident record.
3. Stop deployments. If a mutation path is corrupting data, remove API traffic or stop gateway writers before investigating further.
4. Check the durable-job backlog and interaction failure rate. A Redis failure should pause the fleet scheduler; it must not cause duplicate jobs.

## Common failures

### Discord gateway disconnects

- Confirm every `DISCORD_SHARD_ID` is unique and lower than `DISCORD_SHARD_TOTAL`.
- Compare reconnect and event counters by process. Restart one affected shard at a time.
- A close code caused by invalid intents or authentication requires configuration correction; repeated restarts will not repair it.

### Discord REST 403 or missing access

- Open the guild resource diagnostics page and verify the configured channel or role still exists and is visible to Orivane.
- Check role hierarchy and channel overwrites in Discord.
- Do not retry a permanent 403 in a tight loop. Durable jobs use bounded retries and expose terminal failure.

### PostgreSQL unavailable

- Remove API readiness and pause gateway writers if the outage is prolonged.
- Restore service before Redis. PostgreSQL owns configuration, audit history, cases, and durable jobs.
- If restoration is required, follow `docs/deployment.md`; never restore over the live database.

### Redis unavailable

- Expect readiness failure, paused scheduler leases, cooldown failures, and temporary cache misses.
- Restore or replace Redis. Durable PostgreSQL jobs remain authoritative and resume on later scheduler ticks.
- Verify one scheduler outcome per occurrence after recovery.

### Queue backlog

- Determine the job family from backlog metrics and audit failures.
- Fix permissions or dependency errors before increasing workers.
- Scale gateway shards only for Discord event load; scheduler ownership remains one worker per tick and durable claims distribute safe work.

## Recovery validation

1. API and gateway readiness are green for ten minutes.
2. Interaction errors, scheduler failures, and backlog are returning to baseline.
3. Run `/ping` and one reversible module action in the private smoke-test guild.
4. Confirm one audit event and one Discord side effect were produced—never duplicates.
5. Record the root cause, affected data, recovery actions, and follow-up test or alert.
