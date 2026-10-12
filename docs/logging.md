# Discord event logging

Orivane records Discord gateway activity in the durable audit trail and can mirror enabled categories to Discord channels as timestamped rich embeds.

## Categories

- Commands and module actions
- Message edits, deletions, and bulk deletions
- Member joins, departures, profile changes, role snapshots, and timeout state
- Moderation and other attributed Discord audit-log actions
- Role administration
- Channel, thread, and permission-overwrite administration
- Voice channel joins, moves, and disconnects

Each event family can be disabled or routed to its own channel. A default channel is required while the module is enabled. Message and voice activity can be excluded by source channel.

## Privacy

Message content is never copied into an event log or durable audit record. Message events retain Discord IDs, content length, attachment count, and lifecycle state only.

## Discord permissions

The bot needs `View Audit Log` to receive attributed administrative events through Discord's `GUILD_AUDIT_LOG_ENTRY_CREATE` gateway event. Without it, direct member, message, and voice events still work, but role, channel, kick, ban, and other administrative attribution may be unavailable.
