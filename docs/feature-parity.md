# Feature parity ledger

This document distinguishes working Discord behavior from dashboard-only settings. A module is only marked **live** when its event or slash-command path, validation, audit path, and tests are implemented.

Reference feature sets:

- [Dyno modules](https://docs.dyno.gg/en/modules)
- [Dyno AutoMod](https://docs.dyno.gg/modules/automod)
- [Dyno reaction roles](https://docs.dyno.gg/modules/reactionroles)
- [Carl-bot documentation](https://docs.carl.gg/)

## Current Orivane modules

| Area | Status | Implemented now | Remaining depth |
| --- | --- | --- | --- |
| Moderation | Live | Note, warn, timeout, kick, ban, unban; cases; reasons; optional DMs; Discord permission defaults | Warning thresholds, bulk purge, lockdown, slowmode, role-based moderator policy |
| AutoMod | Live | Blocked terms, invites, links, caps, lines, mention limit, ignored roles/channels, delete/warn/timeout/ban, responses and logs | Per-rule objects, duplicate/rapid spam, phishing reputation, attachment/sticker/emoji/Zalgo filters |
| Welcome | Live | Public welcome, optional DM, goodbye, variables and preview | Images, embeds, multiple randomized messages, join/leave/ban announcements |
| Autorole | Live | Delayed join roles, delayed removals, `/role` self-service roles, exclusive self-service mode | Persistent delayed-job recovery and role-hierarchy preflight diagnostics |
| Reaction roles | Live | Existing-message emoji mappings; role add and remove; custom emoji IDs | Bot-created messages, buttons/dropdowns, add-only/remove-only, exclusive groups |
| Logging | Live, narrow | Content-safe command outcomes plus durable global/guild audit records | Discord event categories for edits/deletes, role/channel changes, voice and moderation audit-log attribution |
| Leveling | Live | Durable XP, cooldown, exclusions, `/rank`, `/leaderboard`, announcements | Role rewards, XP administration, cards, voice XP, anti-farming policy |
| Starboard | Live | Threshold, emoji, ignored channels, self-star policy, deduplication | Rich source embeds, attachments, edits/deletes, per-channel thresholds |
| Tickets | Live | Private ticket channels, duplicate protection, support roles, close command, closure log | Panel/buttons, claims, transcripts, intake forms, per-user limits, reopen/transfer |
| Custom commands | Live | Slash responses, variables, per-channel restriction, public/ephemeral response | Dynamic command registration, embeds, role permissions, cooldowns, multiple/random responses |
| Utility | Live | `/ping`, `/help`, `/serverinfo` | User/server/avatar info variants, polls and richer diagnostics |
| Autoresponder | Live | Exact, contains and starts-with triggers; case handling; channel and role controls; variables | Reactions as responses, cooldowns, regex and rich embeds |
| Anti-raid | Live | Sliding join-burst detection, account-age and avatar risk signals, alert-only/quarantine/timeout/kick/ban actions, optional all-join containment, staff alerts | Discord CAPTCHA remains native-only; persistent cross-worker windows and invite-pattern analysis are future depth |
| Scheduled messages | Live | Recurring channel posts, local-time editor, optional end date, fleet-wide PostgreSQL deduplication, retry after delivery failure and audit records | Calendar/weekday expressions, rich embeds and one-time schedules are future depth |

## Popular-bot areas not yet implemented

These are not represented as fake or configuration-only cards in the dashboard.

| Capability family | Seen in | Orivane status |
| --- | --- | --- |
| Suggestions and voting | Carl-bot and similar community bots | Not built |
| Giveaways | Dyno and MEE6-class bots | Not built |
| Reminders and scheduled messages | Dyno and similar bots | Not built |
| Auto-delete and scheduled purge | Dyno | Not built |
| Forms | Dyno | Not built |
| Managed embeds | Dyno and Carl-bot-class bots | Not built |
| Social feeds | Dyno and MEE6-class bots | Not built |
| Economy, points, games and tags | Dyno and MEE6-class bots | Not built |
| AFK and keyword highlights | Dyno | Not built |
| Voice/text linking and temporary voice channels | Dyno and similar bots | Not built |

## Definition of done for future modules

Every added module must have its own crate and include:

1. Versioned, backwards-compatible configuration with server-side validation.
2. Real Discord gateway or slash-command behavior; slash commands only, never prefix commands.
3. Durable audit events for success and failure without retaining message content unnecessarily.
4. Dashboard enable/disable and configuration UI using Discord resource selectors where applicable.
5. Mock-Discord Rust tests and React interaction tests that preserve the configured coverage thresholds.
