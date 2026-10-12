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
| Moderation | Live | Note, warn, timeout, kick, ban, unban; durable cases; reasons; optional DMs; configurable moderator roles; exact-count warning escalation to timeout/kick/ban; bulk purge; channel lock/unlock; slowmode | Timed bans and case evidence attachments |
| AutoMod | Live | Ordered per-rule actions for blocked terms, mentions, invites, links, caps, lines, distributed duplicate/rapid spam, phishing heuristics and domain reputation lists, attachments/extensions, stickers, emoji and Zalgo; ignored roles/channels; delete/warn/timeout/ban; content-safe responses, logs and analytics | External threat-intelligence provider integrations |
| Welcome | Live | Public welcome, optional DM, goodbye, variables and preview | Images, embeds, multiple randomized messages, join/leave/ban announcements |
| Autorole | Live | Delayed join roles, delayed removals, `/role` self-service roles, exclusive self-service mode | Persistent delayed-job recovery and role-hierarchy preflight diagnostics |
| Reaction roles | Live | Existing-message emoji mappings, Unicode/custom emoji, managed bot messages, synchronized button and dropdown panels, toggle/add-only/remove-only modes, exclusive groups, required/blocked role policy, dashboard list/editor/preview, and audited component actions | Timed role expiry is optional future depth rather than core parity |
| Logging | Live | Category routing for commands, content-safe message edits/deletes, member lifecycle and updates, voice transitions, and attributed Discord audit-log actions for moderation/roles/channels; durable global/guild audit records and analytics | Cached before/after diffs, webhook/integration events, and configurable retention |
| Leveling | Live | Durable message and voice XP; cooldowns; minimum-length and duplicate-content anti-farming; text/voice/role exclusions; `/rank` progress embeds; `/leaderboard`; level announcements; cumulative or highest-only role rewards; permission-gated XP add/remove/set administration | Generated image-card themes and seasonal XP resets |
| Temporary voice | Live | Join-to-create lobby, configurable category/name/limit, duplicate-owner reuse, durable channel inventory, delayed empty cleanup, dashboard force-delete, owner rename/limit/lock/unlock/transfer/kick slash controls, and audited lifecycle actions | Regional bitrate presets are optional future depth |
| Starboard | Live | Threshold, emoji, ignored channels, self-star policy, deduplication | Rich source embeds, attachments, edits/deletes, per-channel thresholds |
| Tickets | Live | Configurable synchronized panels and close buttons, native intake modals, private channels, per-panel limits, claims and staff transfers, priority/tags/internal notes, retained downloadable transcripts, read-only close and reopen, inactivity auto-close, retention cleanup, SLA analytics, dashboard inbox/details, and lifecycle audit events | Multi-step intake forms and external help-desk integrations are future depth |
| Custom commands | Live | Slash responses, variables, per-channel restriction, public/ephemeral response | Dynamic command registration, embeds, role permissions, cooldowns, multiple/random responses |
| Utility | Live | `/ping`, `/help`, `/serverinfo` | User/server/avatar info variants, polls and richer diagnostics |
| Autoresponder | Live | Exact, contains and starts-with triggers; case handling; channel and role controls; variables | Reactions as responses, cooldowns, regex and rich embeds |
| Anti-raid | Live | Sliding join-burst detection, account-age and avatar risk signals, alert-only/quarantine/timeout/kick/ban actions, optional all-join containment, staff alerts | Discord CAPTCHA remains native-only; persistent cross-worker windows and invite-pattern analysis are future depth |
| Scheduled messages | Live | Recurring channel posts, local-time editor, optional end date, fleet-wide PostgreSQL deduplication, retry after delivery failure and audit records | Calendar/weekday expressions, rich embeds and one-time schedules are future depth |
| Forms | Live | Native Discord modals, five configurable fields, role access, anonymous mode, Discord delivery, durable submission inbox and answer detail pages | Branching questions, approval workflows, exports and configurable retention |
| Suggestions | Live | `/suggest`, anonymous mode, durable per-user cooldowns, persistent up/down votes, staff-role workflow, five statuses, notes, list/detail dashboard and vote/status analytics | Discord post status updates, threads and configurable vote emojis |
| Giveaways | Live | Slash and dashboard creation, button entry, automatic durable draws, unique weighted entries, role/account-age eligibility, multiple winners, manual ending, rerolls, entrants and analytics | Rich embeds, external membership requirements and recurring giveaways |
| Reminders | Live | `/remind`, `/reminders`, and `/reminder-cancel`; channel or DM delivery, per-member limits, durable claiming, exponential retries, terminal failure state, dashboard directory/details and analytics | Natural-language dates, repeating reminders and cross-server personal inbox |
| Auto-delete | Live | Per-channel delayed deletion rules, bot/role exclusions, durable content-free queue, retries and crash recovery, queue analytics | Attachment/type filters and calendar-based purge windows |
| Managed embeds | Live | Reusable rich embed templates, Discord-style preview, variable autocomplete, fields/media/author/footer/timestamp controls, `/embed-send`, dashboard sends, synchronized edits, durable delivery history and analytics | Attachments, webhook identity overrides and scheduled template delivery |
| Economy and games | Live | Durable per-server balances and immutable transaction ledger; `/balance`, `/daily`, `/work`, taxed `/pay`, `/economy-leaderboard`; configurable role shop with purchase refunds; bounded `/coinflip` and `/slots`; permission-gated add/remove/set administration; dashboard policy, shop, account and ledger views | Inventory collectibles and seasonal resets are optional future depth |
| Verification and onboarding | Live | Join-time account-age and avatar policy, optional pending role, personal button verification, verified-role assignment, configurable messages and audited outcomes | CAPTCHA and external identity providers are intentionally deferred |

## Popular-bot areas not yet implemented

These are not represented as fake or configuration-only cards in the dashboard.

| Capability family | Seen in | Orivane status |
| --- | --- | --- |
| AFK and keyword highlights | Dyno | Not built |

Social-feed integrations, anti-nuke/server-recovery systems, and music are permanently out of scope for Orivane and are not part of the parity backlog.

## Definition of done for future modules

Every added module must have its own crate and include:

1. Versioned, backwards-compatible configuration with server-side validation.
2. Real Discord gateway or slash-command behavior; slash commands only, never prefix commands.
3. Durable audit events for success and failure without retaining message content unnecessarily.
4. Dashboard enable/disable and configuration UI using Discord resource selectors where applicable.
5. Mock-Discord Rust tests and React interaction tests that preserve the configured coverage thresholds.
