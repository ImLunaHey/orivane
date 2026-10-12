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
| Moderation | Live | Note, warn, timeout, kick, ban, timed ban with durable automatic unban, unban; durable cases and evidence links; reasons; optional DMs; configurable moderator roles; exact-count warning escalation to timeout/kick/ban; bulk purge; channel lock/unlock; slowmode | Native Discord attachment ingestion is optional future depth; evidence URLs are retained today |
| AutoMod | Live | Ordered per-rule actions for blocked terms, mentions, invites, links, caps, lines, distributed duplicate/rapid spam, phishing heuristics and domain reputation lists, attachments/extensions, stickers, emoji and Zalgo; ignored roles/channels; delete/warn/timeout/ban; content-safe responses, logs and analytics | External threat-intelligence provider integrations |
| Welcome | Live | Public welcome, optional DM, randomized welcome/goodbye pools, rich embeds with title/color/image, variables and preview | Generated image cards and distinct ban announcements |
| Autorole | Live | Immediate and durable delayed join roles/removals with leased retry and crash recovery, `/role` self-service roles, exclusive self-service mode | Role-hierarchy preflight diagnostics |
| Reaction roles | Live | Existing-message emoji mappings, Unicode/custom emoji, managed bot messages, synchronized button and dropdown panels, toggle/add-only/remove-only modes, exclusive groups, required/blocked role policy, dashboard list/editor/preview, and audited component actions | Timed role expiry is optional future depth rather than core parity |
| Logging | Live | Category routing for commands, content-safe message edits/deletes, member lifecycle and updates, voice transitions, attributed Discord audit-log actions, durable global/guild audit records and analytics, configurable 30–3650 day guild retention | Cached before/after message diffs and dedicated webhook/integration routing |
| Leveling | Live | Durable message and voice XP; cooldowns; minimum-length and duplicate-content anti-farming; text/voice/role exclusions; `/rank` progress embeds; `/leaderboard`; level announcements; cumulative or highest-only role rewards; permission-gated XP add/remove/set administration | Generated image-card themes and seasonal XP resets |
| Temporary voice | Live | Join-to-create lobby, configurable category/name/limit, duplicate-owner reuse, durable channel inventory, delayed empty cleanup, dashboard force-delete, owner rename/limit/lock/unlock/transfer/kick slash controls, and audited lifecycle actions | Regional bitrate presets are optional future depth |
| Starboard | Live | Rich source embeds with first attachment and jump link, synchronized reaction counts, source edit/delete synchronization, automatic removal below threshold, durable destination mapping, per-channel thresholds, ignored channels, and self-star policy | Multiple independent boards |
| Tickets | Live | Configurable synchronized panels and close buttons, native intake modals, private channels, per-panel limits, claims and staff transfers, priority/tags/internal notes, retained downloadable transcripts, read-only close and reopen, inactivity auto-close, retention cleanup, SLA analytics, dashboard inbox/details, and lifecycle audit events | Multi-step intake forms and external help-desk integrations are future depth |
| Custom commands | Live | Slash responses, variables, channel and role restrictions, per-member fleet-wide cooldowns, multiple deterministic response choices, public/ephemeral response | Dynamic per-server command registration and full embed payloads |
| Utility | Live | `/ping`, `/help`, richer live `/serverinfo`, `/userinfo`, and `/avatar` with account creation and profile details | Channel/role inspection commands are optional future depth |
| Autoresponder | Live | Exact, contains, starts-with, and bounded regex triggers; case handling; channel and role controls; fleet-wide per-member cooldowns; variables; rich embeds; up to five Unicode reactions | Multi-message response sequences |
| Anti-raid | Live | Sliding join-burst detection, account-age and avatar risk signals, alert-only/quarantine/timeout/kick/ban actions, optional all-join containment, staff alerts | Discord CAPTCHA remains native-only; persistent cross-worker windows and invite-pattern analysis are future depth |
| Scheduled messages | Live | One-time, fixed-interval, and selected-weekday schedules; saved local UTC offsets; optional end date; plain messages or reusable managed embeds; fleet-wide PostgreSQL deduplication, retry after delivery failure, delivery history, and audit records | Full cron expressions are intentionally omitted in favor of validated schedule modes |
| Forms | Live | Native Discord modals, five configurable fields, role access, anonymous mode, Discord delivery, durable submission inbox and answer detail pages, attributed approve/reject/close workflows for every form, formula-safe CSV export, and durable configurable retention purge | Branching questions cannot be expressed inside one native Discord modal; multi-step form sessions are optional future depth |
| Suggestions | Live | `/suggest`, anonymous mode, durable per-user cooldowns, persistent up/down votes, live Discord vote totals, staff-role workflow, five statuses, notes, synchronized Discord post state, list/detail dashboard and vote/status analytics | Threads and configurable vote emojis |
| Giveaways | Live | Slash and dashboard creation, button entry, automatic durable draws, unique weighted entries, role/account-age eligibility, multiple winners, manual ending, rerolls, entrants and analytics | Rich embeds, external membership requirements and recurring giveaways |
| Reminders | Live | `/remind`, `/reminders`, and `/reminder-cancel`; channel or DM delivery, per-member limits, durable claiming, exponential retries, terminal failure state, dashboard directory/details and analytics | Natural-language dates, repeating reminders and cross-server personal inbox |
| Auto-delete | Live | Per-channel delayed deletion rules, bot/role exclusions, durable content-free queue, retries and crash recovery, queue analytics | Attachment/type filters and calendar-based purge windows |
| Managed embeds | Live | Reusable rich embed templates, Discord-style preview, variable autocomplete, fields/media/author/footer/timestamp controls, `/embed-send`, dashboard sends, synchronized edits, durable delivery history and analytics | Attachments, webhook identity overrides and scheduled template delivery |
| Economy and games | Live | Durable per-server balances and immutable transaction ledger; `/balance`, `/daily`, `/work`, taxed `/pay`, `/economy-leaderboard`; configurable role shop with purchase refunds; bounded `/coinflip` and `/slots`; permission-gated add/remove/set administration; dashboard policy, shop, account and ledger views | Inventory collectibles and seasonal resets are optional future depth |
| Verification and onboarding | Live | Join-time account-age and avatar policy, optional pending role, personal button verification, verified-role assignment, configurable messages, durable attempt history, attributed member links, and audited outcomes | CAPTCHA and external identity providers are intentionally deferred |
| Moderation appeals | Live | Native `/appeal` modal, durable attributed submissions, configurable staff routing, reviewer-role authorization, searchable dashboard inbox, dedicated review details, retained decisions/notes, and audited review actions | Public appeals from users who are no longer in the server still require a dedicated OAuth intake route |
| Invite tracking | Live | Join attribution from Discord invite-use deltas, cold-start snapshot priming, durable attribution history and leaderboard, inviter totals, configurable logging, role milestones, and graceful degradation when invite permissions are unavailable | Vanity URL attribution needs additional Discord metadata |
| Polls | Live | Timed two-to-five-option polls, anonymous policy, replacement voting, durable tallies, dedicated history/result pages, manager-role early close, automatic expiry, and synchronized public result messages with disabled vote controls | Ranked-choice and multi-select ballots are optional future depth |
| Counters and goals | Live | Scheduled member, online, boost, and manual-goal voice-channel names with count, goal, and remaining variables plus durable success/failure run history | Historical goal charts and automatic goal sources |
| AFK | Live | Durable expiring away status, `/afk`, automatic return-on-message, mention notices, configurable expiry, dashboard directory, and audited administrative clearing | Keyword highlights are a separate privacy-sensitive capability |
| Voice/text linking | Live | Reusable channel-pair editor, durable attributed join/leave history, and audited announcements for linked voice rooms | Temporary private text-channel lifecycle |

## Popular-bot areas not yet implemented

These are not represented as fake or configuration-only cards in the dashboard.

| Capability family | Seen in | Orivane status |
| --- | --- | --- |
| Keyword highlights | Dyno | Not built; requires explicit privacy and notification controls |

Social-feed integrations, anti-nuke/server-recovery systems, and music are permanently out of scope for Orivane and are not part of the parity backlog.

## Definition of done for future modules

Every added module must have its own crate and include:

1. Versioned, backwards-compatible configuration with server-side validation.
2. Real Discord gateway or slash-command behavior; slash commands only, never prefix commands.
3. Durable audit events for success and failure without retaining message content unnecessarily.
4. Dashboard enable/disable and configuration UI using Discord resource selectors where applicable.
5. Mock-Discord Rust tests and React interaction tests that preserve the configured coverage thresholds.
