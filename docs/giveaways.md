# Giveaways

The Giveaways module runs durable, button-entry prize draws through Discord and the dashboard.

- Staff can create giveaways with `/giveaway-create` or the dashboard, then end or reroll them with slash commands or dedicated detail-page actions.
- Every giveaway snapshots its winner count, end time, required roles, minimum Discord account age, bonus role, and bonus-entry weight.
- Entry is unique per Discord user. Bonus roles increase selection weight without creating duplicate entrant rows.
- A fleet-wide scheduler claims due giveaways transactionally, chooses unique weighted winners, closes entry, and announces the result once.
- Rerolls exclude all current winners and fail safely when no replacement entrant exists.
- PostgreSQL retains giveaways, entrants, results, reroll counts, and delivery state. Dashboard and Discord operations produce content-safe audit records.

Discord behavior lives in `modules/giveaways`. Its command, component, scheduler, persistence, and announcement boundaries are mockable through `bot-test-support`.
