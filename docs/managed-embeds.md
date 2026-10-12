# Managed embeds

Managed embeds are reusable rich-message templates owned by a Discord server. Enable the module, create templates in the dashboard, and send them through the dashboard or `/embed-send template:<id> [channel]`.

Templates support message content, title and link, description, accent color, author, footer, timestamp, thumbnail, image, and up to 25 fields. Text surfaces support `{user}`, `{display_name}`, `{username}`, `{user_id}`, `{server}`, and `{server_id}` variables with editor autocomplete and a Discord-style preview.

Every send and synchronized edit records a content-free delivery row and an audit event. Template deletion is soft: it removes the template from the active library while retaining delivery and audit history. Dashboard delivery requires the module to be enabled; the slash command also requires Manage Messages.
