# Auto-delete

Auto-delete queues matching messages by Discord ID and deletes them after each channel rule's delay. It never stores message content. Rules support bot-message and role exclusions. Claims are safe across workers, retry with exponential backoff, recover after worker exits, and fail terminally after five attempts.

Members with Discord's Manage Messages permission can use `/purge amount [channel]` for immediate cleanup. The dashboard exposes rule editing, labeled queue analytics, and recent content-safe job records. Configuration reads, queue reads, scheduling, deletion, retry, terminal failure, and slash-command outcomes use the audit pipeline.
