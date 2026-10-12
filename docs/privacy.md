# Privacy and data handling

Orivane minimizes retained Discord data. Audit and moderation records use Discord snowflakes, timestamps, outcomes, and safe metadata. Message edit and deletion logs retain lengths, attachment counts, and IDs—not message content. Metrics never use guild IDs, user IDs, usernames, command text, or other high-cardinality personal labels.

## Personal data export

An authenticated user can request `GET /v1/me/privacy-export`. The export is scoped to the Discord account linked to the active Lucid Auth session and includes the stored identity aggregate plus per-guild command, moderation, XP, and installation context. The request itself creates a global audit event without copying the exported data into the audit log.

Exports must be delivered only over HTTPS and must not be cached by an intermediary. Operators should treat the JSON as personal data.

## Retention and deletion

Server administrators configure durable guild audit retention from 30 to 3,650 days on the Logging page. Form submissions and ticket transcripts have their own module retention settings. Expired rows are removed by the fleet scheduler.

Account deletion is intentionally not an unauthenticated or one-click API operation because user snowflakes participate in moderation evidence, financial ledgers, and security audit history. A deletion request must be verified against the linked Discord session, exported first when requested, then processed under the operator's documented legal policy. Mutable profile fields can be anonymized; immutable security and financial events should be pseudonymized or retained only where there is a documented lawful requirement.

Secrets, OAuth tokens, session cookies, raw message content, and Discord access tokens must never enter audit metadata, logs, fixtures, analytics labels, or support tickets.
