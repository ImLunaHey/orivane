# Reminders

The reminders module provides personal, slash-command-only scheduling within a server.

- `/remind` schedules a message in the current channel or by direct message.
- `/reminders` lists the invoking member's active reminders.
- `/reminder-cancel` cancels an active reminder owned by the invoking member.
- PostgreSQL claims due work safely across workers. Failed Discord deliveries retry with exponential backoff and become terminal after five attempts. Stale claims are recovered after five minutes if a worker exits during delivery.
- Server administrators configure delay and capacity limits and can inspect or cancel reminders from the dashboard.
- Reminder creation, cancellation, delivery, retries, failures, list views, and detail views use the existing audit pipeline.

Reminder content is required for delivery and is visible to authorized server dashboard administrators. Audit metadata does not copy the reminder message.
