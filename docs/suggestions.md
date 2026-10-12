# Suggestions

The Suggestions module collects server feedback through `/suggest`, publishes each proposal to a configured channel, and gives members persistent upvote and downvote buttons.

- Servers can permit anonymous proposals, select staff roles, and configure a per-member cooldown.
- `/suggestion-status` lets authorized staff mark a proposal pending, approved, rejected, implemented, or duplicate and attach an internal note.
- Votes are unique per member and may be changed without inflating totals.
- The dashboard provides status and vote summaries, a searchable TanStack table, and a dedicated review page for each suggestion.
- Reads, configuration changes, votes, submissions, and staff decisions have content-safe audit paths. Suggestion text and staff notes are not copied into audit metadata.

Configuration, suggestions, and votes are stored in PostgreSQL. Cooldowns use Redis so they remain consistent across gateway workers. The Discord behavior lives in its own Rust crate at `modules/suggestions`, and its command, component, and persistence boundaries are mockable through `bot-test-support`.
