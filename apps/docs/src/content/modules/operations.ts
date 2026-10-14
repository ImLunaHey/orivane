import { moduleDoc, setting as s } from '../module-helpers'

export const operationsModules = [
  moduleDoc({
    id: 'tickets', title: 'Tickets', category: 'Operations',
    summary: 'Private support channels, configurable panels, staff ownership, notes, transcripts, and analytics.',
    description: 'Publish one or more ticket panels with distinct routing and intake policy. Staff can claim, transfer, prioritize, tag, close, reopen, and annotate cases while Orivane records response and resolution timing.',
    highlights: ['Multiple deployable ticket panels', 'Claims, transfers, priorities, tags, and internal notes', 'Transcript retention, inactivity closure, and service analytics'],
    commands: ['/ticket', '/close'],
    settingGroups: [
      { title: 'Legacy defaults', description: 'Server-level fallback used when a panel does not override a value.', settings: [
        s('enabled', 'Enabled', 'Register ticket commands and panel buttons.', 'toggle', 'Off'), s('category_id', 'Ticket category', 'Category for new ticket channels.', 'channel'),
        s('support_role_ids', 'Support roles', 'Roles granted access to tickets.', 'role'), s('transcript_channel_id', 'Transcript channel', 'Optional delivery destination for closed transcripts.', 'channel'),
        s('welcome_message', 'Welcome message', 'Opening message in a new ticket.', 'textarea'), s('channel_name', 'Channel name template', 'Name template for created channels.', 'text'),
      ] },
      { title: 'Ticket panels', description: 'Each managed panel has its own message, routing, intake, and lifecycle policy.', settings: [
        s('panels[].enabled', 'Panel enabled', 'Allow opening tickets from this panel.', 'toggle'), s('panels[].name', 'Internal name', 'Dashboard label.'),
        s('panels[].title', 'Panel title', 'Discord message heading.'), s('panels[].description', 'Panel description', 'Discord message instructions.', 'textarea'),
        s('panels[].channel_id', 'Panel channel', 'Where the panel is deployed.', 'channel'), s('panels[].category_id', 'Ticket category', 'Where private channels are created.', 'channel'),
        s('panels[].support_role_ids', 'Support roles', 'Roles added to created channels.', 'role'), s('panels[].button_label', 'Button label', 'Member-facing open button.'),
        s('panels[].welcome_message', 'Ticket welcome', 'First message in a created ticket.', 'textarea'), s('panels[].channel_name', 'Channel template', 'Generated channel name.'),
        s('panels[].intake_enabled', 'Intake modal', 'Ask for a subject and opening note.', 'toggle'), s('panels[].intake_label', 'Intake label', 'Question displayed in the modal.'),
        s('panels[].max_open_per_user', 'Maximum open tickets', 'Per-member open-ticket limit.', 'number'), s('panels[].auto_close_hours', 'Auto-close inactivity', 'Hours of inactivity before automatic closure.', 'number'),
        s('panels[].transcript_retention_days', 'Transcript retention', 'Days before retained transcript content is purged.', 'number'), s('panels[].default_priority', 'Default priority', 'Low, normal, high, or urgent.', 'select'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'forms', title: 'Forms', category: 'Operations',
    summary: 'Reusable native Discord modals with structured submissions and a staff review queue.',
    description: 'Create multiple forms with up to five short or paragraph questions. Restrict access by role, optionally anonymize submitters, and review every response on a dedicated dashboard page.',
    highlights: ['Native Discord modal experience', 'Dedicated form list, editor, and submission pages', 'Approve, reject, or close with reviewer notes'],
    commands: ['/form'], demo: 'forms',
    settingGroups: [
      { title: 'Module', description: 'Shared form policy.', settings: [s('enabled', 'Enabled', 'Register the form command.', 'toggle', 'Off'), s('retention_days', 'Submission retention', 'Days to retain form submissions.', 'number', '365')] },
      { title: 'Forms', description: 'Repeatable modal definitions.', settings: [
        s('forms[].enabled', 'Form enabled', 'Expose the form through /form.', 'toggle'), s('forms[].id', 'Form ID', 'Stable command value.'),
        s('forms[].name', 'Form name', 'Member and dashboard label.'), s('forms[].description', 'Description', 'Explains the form purpose.', 'textarea'),
        s('forms[].destination_channel_id', 'Destination channel', 'Staff channel receiving submissions.', 'channel'), s('forms[].allowed_role_ids', 'Allowed roles', 'Optional roles allowed to submit.', 'role'),
        s('forms[].anonymous', 'Anonymous', 'Hide submitter identity from submission views.', 'toggle'),
      ] },
      { title: 'Questions', description: 'Up to five Discord modal fields per form.', settings: [
        s('forms[].questions[].id', 'Question ID', 'Stable answer key.'), s('forms[].questions[].label', 'Question label', 'Prompt shown above the input.'),
        s('forms[].questions[].style', 'Input style', 'Short line or paragraph.', 'select'), s('forms[].questions[].required', 'Required', 'Require a non-empty answer.', 'toggle'),
        s('forms[].questions[].min_length', 'Minimum length', 'Minimum answer characters.', 'number'), s('forms[].questions[].max_length', 'Maximum length', 'Maximum answer characters.', 'number'),
        s('forms[].questions[].placeholder', 'Placeholder', 'Example or guidance inside the field.'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'scheduled_messages', title: 'Scheduled messages', category: 'Automation',
    summary: 'Durable one-time, interval, and weekday announcements with optional managed embeds.',
    description: 'Schedule server-owned announcements that survive restarts. A distributed scheduler lease and durable database claims prevent duplicate delivery across gateway workers.',
    highlights: ['Once, repeating interval, and weekday schedules', 'Timezone offsets, start and end bounds', 'Optional reusable managed-embed delivery'],
    commands: [],
    settingGroups: [
      { title: 'Module', description: 'Scheduler switch.', settings: [s('enabled', 'Enabled', 'Claim and deliver due schedules.', 'toggle', 'Off')] },
      { title: 'Schedules', description: 'Repeatable announcement definitions.', settings: [
        s('schedules[].enabled', 'Schedule enabled', 'Pause without deleting.', 'toggle'), s('schedules[].id', 'Schedule ID', 'Stable audit identifier.'),
        s('schedules[].channel_id', 'Destination channel', 'Where the message is posted.', 'channel'), s('schedules[].message', 'Message', 'Text content with supported variables.', 'textarea'),
        s('schedules[].kind', 'Schedule kind', 'Once, interval, or weekdays.', 'select'), s('schedules[].weekdays', 'Weekdays', 'Days used by weekday schedules.', 'list'),
        s('schedules[].timezone_offset_minutes', 'Timezone offset', 'Offset applied to weekday wall time.', 'number'), s('schedules[].embed_template_id', 'Embed template', 'Optional managed embed sent with the post.', 'select'),
        s('schedules[].starts_at_unix', 'Starts at', 'First eligible delivery time.', 'timestamp'), s('schedules[].interval_minutes', 'Interval', 'Minutes between interval deliveries.', 'number'),
        s('schedules[].ends_at_unix', 'Ends at', 'Optional final eligibility time.', 'timestamp'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'reminders', title: 'Reminders', category: 'Automation',
    summary: 'Durable personal reminders delivered to a channel or direct message.',
    description: 'Members schedule reminders with natural minute delays, list their active entries, and cancel them. Failed deliveries retain attempts and retry state for operational review.',
    highlights: ['Channel and optional DM delivery', 'Per-member limits and bounded delays', 'Durable status, retry, and cancellation records'],
    commands: ['/remind', '/reminders', '/reminder-cancel'],
    settingGroups: [{ title: 'Reminder policy', description: 'Delivery and member limits.', settings: [
      s('enabled', 'Enabled', 'Register reminder commands and process deliveries.', 'toggle', 'Off'), s('allow_direct_messages', 'Allow direct messages', 'Let members choose private delivery.', 'toggle', 'On'),
      s('max_active_per_user', 'Maximum active reminders', 'Per-member outstanding reminder limit.', 'number', '10'), s('minimum_delay_minutes', 'Minimum delay', 'Shortest schedulable delay.', 'number', '1 minute'),
      s('maximum_delay_days', 'Maximum delay', 'Furthest schedulable delivery.', 'number', '365 days'),
    ] }],
  }),
  moduleDoc({
    id: 'auto_delete', title: 'Auto-delete', category: 'Automation',
    summary: 'Delete eligible channel messages after durable per-rule delays.',
    description: 'Create multiple channel policies. Each matching message becomes a durable job, so delayed deletion survives restarts and records failed attempts without retaining message content.',
    highlights: ['Independent delay per channel rule', 'Bot-message and role exclusions', 'Durable deletion queue and operational history'],
    commands: [],
    settingGroups: [
      { title: 'Module', description: 'Processing switch.', settings: [s('enabled', 'Enabled', 'Schedule and claim deletion jobs.', 'toggle', 'Off')] },
      { title: 'Rules', description: 'Repeatable channel deletion policies.', settings: [
        s('rules[].enabled', 'Rule enabled', 'Pause one rule.', 'toggle'), s('rules[].id', 'Rule ID', 'Stable audit identifier.'), s('rules[].channel_id', 'Channel', 'Messages observed by this rule.', 'channel'),
        s('rules[].delay_seconds', 'Deletion delay', 'Seconds before deletion.', 'number'), s('rules[].ignore_bot_messages', 'Ignore bots', 'Do not schedule bot-authored messages.', 'toggle'),
        s('rules[].excluded_role_ids', 'Excluded roles', 'Do not schedule messages from these roles.', 'role'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'managed_embeds', title: 'Managed embeds', category: 'Operations',
    summary: 'Design reusable rich messages, publish them, and synchronize existing Discord posts.',
    description: 'Maintain an embed template library with a Discord-style preview. Send templates from the dashboard or /embed-send, then synchronize prior deliveries after editing.',
    highlights: ['Message content plus full Discord embed structure', 'Reusable variable-aware templates', 'Delivery history and post synchronization'],
    commands: ['/embed-send'],
    settingGroups: [
      { title: 'Module', description: 'Template delivery switch.', settings: [s('enabled', 'Enabled', 'Allow dashboard and slash-command delivery.', 'toggle', 'Off')] },
      { title: 'Template', description: 'Reusable message and rich embed fields.', settings: [
        s('templates[].name', 'Template name', 'Dashboard and slash-command label.'), s('templates[].content', 'Message content', 'Optional text above the embed.', 'textarea'),
        s('templates[].title', 'Title', 'Embed heading.'), s('templates[].description', 'Description', 'Main rich-text body.', 'textarea'), s('templates[].url', 'Title URL', 'Optional HTTPS title link.'),
        s('templates[].color', 'Accent color', 'Embed side-rail color.', 'color'), s('templates[].author_name', 'Author name', 'Optional heading above the title.'),
        s('templates[].author_url', 'Author URL', 'Optional author link.'), s('templates[].author_icon_url', 'Author icon', 'Square author image URL.'),
        s('templates[].footer_text', 'Footer text', 'Text beneath the embed.'), s('templates[].footer_icon_url', 'Footer icon', 'Small footer image URL.'),
        s('templates[].thumbnail_url', 'Thumbnail', 'Small upper-right image URL.'), s('templates[].image_url', 'Large image', 'Media shown below fields.'),
        s('templates[].timestamp', 'Timestamp', 'Show the Discord delivery time.', 'toggle'),
        s('templates[].fields[].name', 'Field name', 'Heading for one of up to 25 fields.'), s('templates[].fields[].value', 'Field value', 'Field body.', 'textarea'), s('templates[].fields[].inline', 'Inline field', 'Allow neighboring fields on one row.', 'toggle'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'temporary_voice', title: 'Temporary voice', category: 'Community',
    summary: 'Create owner-controlled voice channels from a lobby and remove them when empty.',
    description: 'Joining the configured lobby creates a private managed room and moves the member into it. Owners can rename, limit, lock, unlock, transfer, and remove participants.',
    highlights: ['Join-to-create lobby', 'Owner control commands', 'Durable ownership and delayed empty-room cleanup'],
    commands: ['/voice-rename', '/voice-limit', '/voice-lock', '/voice-unlock', '/voice-transfer', '/voice-kick'],
    settingGroups: [{ title: 'Temporary rooms', description: 'Creation and cleanup policy.', settings: [
      s('enabled', 'Enabled', 'Observe the lobby and register owner controls.', 'toggle', 'Off'), s('lobby_channel_id', 'Lobby channel', 'Joining this voice channel creates a room.', 'channel'),
      s('category_id', 'Room category', 'Category receiving temporary rooms.', 'channel'), s('default_name', 'Default name', 'New channel name template.', 'text'),
      s('user_limit', 'Default user limit', 'Zero means unlimited.', 'number', '0'), s('delete_delay_seconds', 'Empty cleanup delay', 'Seconds an empty room remains before deletion.', 'number', '30'),
      s('owner_controls', 'Owner controls', 'Allow the creator to manage their room.', 'toggle', 'On'),
    ] }],
  }),
  moduleDoc({
    id: 'counters', title: 'Counters and goals', category: 'Operations',
    summary: 'Keep server metrics and progress goals visible in Discord channel names.',
    description: 'Periodically render member, online, boost, or manual goal values into voice-channel names. Each update records its rendered name and outcome.',
    highlights: ['Member, online, boost, and manual goal metrics', '{count}, {goal}, and {remaining} templates', 'Per-counter enable switch and run history'],
    commands: [], demo: 'counters',
    settingGroups: [
      { title: 'Module', description: 'Counter scheduler switch.', settings: [s('enabled', 'Enabled', 'Refresh enabled counters.', 'toggle', 'Off')] },
      { title: 'Counters', description: 'Repeatable channel-name metrics.', settings: [
        s('counters[].id', 'Counter ID', 'Stable run-history identifier.'), s('counters[].enabled', 'Counter enabled', 'Pause one counter.', 'toggle'),
        s('counters[].channel_id', 'Counter channel', 'Voice channel whose name is updated.', 'channel'), s('counters[].label', 'Channel template', 'Supports {count}, {goal}, and {remaining}.'),
        s('counters[].kind', 'Metric', 'Members, online, boosts, or manual goal.', 'select'), s('counters[].value', 'Manual value', 'Current value for manual goals.', 'number'),
        s('counters[].goal', 'Goal', 'Optional target used by goal variables.', 'number'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'voice_text_linking', title: 'Voice and text linking', category: 'Community',
    summary: 'Pair voice rooms with text channels and announce join or leave activity.',
    description: 'Define explicit voice-to-text pairs. Orivane posts content-safe member activity into the linked text channel and retains operational event records.',
    highlights: ['Multiple explicit channel pairs', 'Independent join and leave announcements', 'Attributed activity history'],
    commands: [],
    settingGroups: [
      { title: 'Module', description: 'Voice-state processing switch.', settings: [s('enabled', 'Enabled', 'Observe configured voice rooms.', 'toggle', 'Off')] },
      { title: 'Links', description: 'Repeatable voice and text channel pairs.', settings: [
        s('links[].voice_channel_id', 'Voice channel', 'Voice room to observe.', 'channel'), s('links[].text_channel_id', 'Text channel', 'Announcement destination.', 'channel'),
        s('links[].announce_joins', 'Announce joins', 'Post when a member enters.', 'toggle', 'On'), s('links[].announce_leaves', 'Announce leaves', 'Post when a member exits.', 'toggle', 'Off'),
      ] },
    ],
  }),
]
