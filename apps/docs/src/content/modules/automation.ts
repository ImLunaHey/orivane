import { moduleDoc, setting as s } from '../module-helpers'

export const automationModules = [
  moduleDoc({
    id: 'utility', title: 'Utility commands', category: 'Community',
    summary: 'Fast slash commands for latency, profiles, avatars, help, and server information.',
    description: 'Give members useful server and account lookups without prefix commands. Responses can be public or visible only to the person who invoked them.',
    highlights: ['Discord and gateway latency', 'User profile, account age, and avatar views', 'Discoverable help and server information'],
    commands: ['/ping', '/userinfo', '/avatar', '/help', '/serverinfo'],
    settingGroups: [{ title: 'Responses', description: 'Utility command behavior.', settings: [
      s('enabled', 'Enabled', 'Register and serve utility commands.', 'toggle', 'On'),
      s('ephemeral_responses', 'Ephemeral responses', 'Only the invoking member can see replies.', 'toggle', 'Off'),
    ] }],
  }),
  moduleDoc({
    id: 'custom_commands', title: 'Custom commands', category: 'Automation',
    summary: 'Server-owned slash responses with restrictions, cooldowns, and randomized alternatives.',
    description: 'Define named responses members invoke through /custom. Limit each command to selected channels or roles, choose private delivery, and rotate between multiple responses.',
    highlights: ['Slash-only server commands', 'Channel and role eligibility', 'Cooldowns, variables, and randomized responses'],
    commands: ['/custom'],
    settingGroups: [
      { title: 'Module', description: 'Catalog limits.', settings: [s('enabled', 'Enabled', 'Register the custom command dispatcher.', 'toggle', 'Off'), s('max_commands', 'Maximum commands', 'Maximum definitions this server may store.', 'number', '25')] },
      { title: 'Commands', description: 'Repeatable server-owned command definitions.', settings: [
        s('commands[].enabled', 'Command enabled', 'Pause one command.', 'toggle'), s('commands[].name', 'Name', 'Slash-command selection value.'),
        s('commands[].description', 'Description', 'Help text shown to members.'), s('commands[].response', 'Primary response', 'Message body supporting variables.', 'textarea'),
        s('commands[].additional_responses', 'Additional responses', 'Alternatives selected randomly.', 'list'), s('commands[].ephemeral', 'Private response', 'Only the invoking member sees it.', 'toggle'),
        s('commands[].allowed_channel_ids', 'Allowed channels', 'Optional channel allowlist.', 'channel'), s('commands[].allowed_role_ids', 'Allowed roles', 'Optional role allowlist.', 'role'),
        s('commands[].cooldown_seconds', 'Cooldown', 'Per-member delay between uses.', 'number'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'autoresponder', title: 'Autoresponder', category: 'Automation',
    summary: 'Respond to exact, contains, prefix, or regular-expression message triggers.',
    description: 'Create controlled message automations with channel and role policy, cooldowns, reactions, and optional rich embed styling.',
    highlights: ['Four matching modes including regex', 'Channel allowlists and role exclusions', 'Text, reactions, and rich embed responses'],
    commands: [],
    settingGroups: [
      { title: 'Module', description: 'Message-trigger switch.', settings: [s('enabled', 'Enabled', 'Inspect eligible messages for configured triggers.', 'toggle', 'Off')] },
      { title: 'Responses', description: 'Repeatable trigger and response definitions.', settings: [
        s('responses[].enabled', 'Response enabled', 'Pause one automation.', 'toggle'), s('responses[].trigger', 'Trigger', 'Text or regular expression to match.'),
        s('responses[].response', 'Response', 'Message sent when matched.', 'textarea'), s('responses[].match_mode', 'Match mode', 'Exact, contains, starts with, or regex.', 'select'),
        s('responses[].case_sensitive', 'Case sensitive', 'Preserve letter-case matching.', 'toggle'), s('responses[].allowed_channel_ids', 'Allowed channels', 'Optional source allowlist.', 'channel'),
        s('responses[].ignored_role_ids', 'Ignored roles', 'Members with these roles do not trigger it.', 'role'), s('responses[].cooldown_seconds', 'Cooldown', 'Delay between repeated matches.', 'number'),
        s('responses[].reactions', 'Reactions', 'Emoji Orivane adds to the source message.', 'list'), s('responses[].embed_title', 'Embed title', 'Optional rich-response heading.'),
        s('responses[].embed_color', 'Embed color', 'Rich-response accent.', 'color'),
      ] },
    ],
  }),
  moduleDoc({
    id: 'afk', title: 'AFK statuses', category: 'Community',
    summary: 'Let members set an away reason and notify people who mention them.',
    description: 'Store an expiring server-specific away status. Orivane can clear it as soon as the member speaks and safely tells others when they mention someone who is away.',
    highlights: ['Optional reason through /afk', 'Automatic expiry and clear-on-message', 'Mention notices without mass-mention injection'],
    commands: ['/afk'],
    settingGroups: [{ title: 'Away policy', description: 'Status lifetime and notification behavior.', settings: [
      s('enabled', 'Enabled', 'Register /afk and process message activity.', 'toggle', 'Off'),
      s('maximum_hours', 'Maximum duration', 'Hours before an away status expires.', 'number', '168 hours'),
      s('clear_on_message', 'Clear on message', 'Return members automatically when they speak.', 'toggle', 'On'),
      s('notify_mentions', 'Mention notifications', 'Tell people when a mentioned member is away.', 'toggle', 'On'),
    ] }],
  }),
]
