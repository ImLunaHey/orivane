import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { findModule, moduleCategories, modules, settingCount } from './modules'

const expectedModules = [
  'afk', 'anti_raid', 'auto_delete', 'automod', 'autoresponder', 'autorole', 'counters',
  'custom_commands', 'economy', 'forms', 'giveaways', 'invite_tracking', 'leveling', 'logging',
  'managed_embeds', 'moderation', 'moderation_appeals', 'polls', 'reaction_roles', 'reminders',
  'scheduled_messages', 'starboard', 'suggestions', 'temporary_voice', 'tickets', 'utility',
  'verification', 'voice_text_linking', 'welcome',
]

describe('module documentation catalog', () => {
  it('covers every live GuildConfig module exactly once', () => {
    expect(modules.map((module) => module.id).sort()).toEqual(expectedModules)
    expect(new Set(modules.map((module) => module.id))).toHaveLength(expectedModules.length)
  })

  it('documents configuration and feature context for every module', () => {
    for (const module of modules) {
      expect(module.summary.length).toBeGreaterThan(30)
      expect(module.description.length).toBeGreaterThan(60)
      expect(module.highlights.length).toBeGreaterThanOrEqual(3)
      expect(settingCount(module)).toBeGreaterThanOrEqual(1)
      expect(module.settingGroups.flatMap((group) => group.settings).some((setting) => setting.key === 'enabled')).toBe(true)
      expect(module.commands.every((command) => command.startsWith('/'))).toBe(true)
    }
  })

  it('provides stable lookup and all navigation categories', () => {
    expect(findModule('automod')?.title).toBe('Automod')
    expect(findModule('not-real')).toBeUndefined()
    expect(new Set(modules.map((module) => module.category))).toEqual(new Set(moduleCategories))
  })

  it('covers every serialized Rust configuration field', () => {
    const source = readFileSync(resolve(process.cwd(), '../../crates/bot-core/src/config.rs'), 'utf8')
    const contracts: Record<string, Array<[string, string]>> = {
      afk: [['AfkConfig', '']], anti_raid: [['AntiRaidConfig', '']], auto_delete: [['AutoDeleteConfig', ''], ['AutoDeleteRule', 'rules[]']],
      automod: [['AutomodConfig', ''], ['AutomodRuleConfig', 'rules[]']], autoresponder: [['AutoresponderConfig', ''], ['AutoresponseConfig', 'responses[]']],
      autorole: [['AutoroleConfig', '']], counters: [['CountersConfig', ''], ['CounterConfig', 'counters[]']],
      custom_commands: [['CustomCommandsConfig', ''], ['CustomCommandConfig', 'commands[]']], economy: [['EconomyConfig', ''], ['EconomyShopItem', 'shop_items[]']],
      forms: [['FormsConfig', ''], ['FormConfig', 'forms[]'], ['FormQuestionConfig', 'forms[].questions[]']], giveaways: [['GiveawaysConfig', '']],
      invite_tracking: [['InviteTrackingConfig', ''], ['InviteRewardConfig', 'rewards[]']], leveling: [['LevelingConfig', ''], ['LevelRewardConfig', 'role_rewards[]']],
      logging: [['LoggingConfig', '']], managed_embeds: [['ManagedEmbedsConfig', '']], moderation: [['ModerationConfig', ''], ['WarningThresholdConfig', 'warning_thresholds[]']],
      moderation_appeals: [['ModerationAppealsConfig', '']], polls: [['PollsConfig', '']], reaction_roles: [['ReactionRolesConfig', ''], ['ReactionRoleBinding', 'bindings[]'], ['ReactionRolePanel', 'panels[]'], ['ReactionRoleOption', 'panels[].options[]']],
      reminders: [['RemindersConfig', '']], scheduled_messages: [['ScheduledMessagesConfig', ''], ['ScheduledMessageConfig', 'schedules[]']],
      starboard: [['StarboardConfig', ''], ['StarboardChannelThreshold', 'channel_thresholds[]']], suggestions: [['SuggestionsConfig', '']],
      temporary_voice: [['TemporaryVoiceConfig', '']], tickets: [['TicketsConfig', '']], utility: [['UtilityConfig', '']], verification: [['VerificationConfig', '']],
      voice_text_linking: [['VoiceTextLinkingConfig', ''], ['VoiceTextLinkConfig', 'links[]']], welcome: [['WelcomeConfig', '']],
    }
    for (const [moduleId, structs] of Object.entries(contracts)) {
      const documented = new Set(findModule(moduleId)?.settingGroups.flatMap((group) => group.settings.map((setting) => setting.key)))
      for (const [structName, prefix] of structs) {
        const body = source.match(new RegExp(`pub struct ${structName} \\{([\\s\\S]*?)\\n\\}`))?.[1]
        expect(body, `${structName} exists`).toBeTruthy()
        for (const match of body?.matchAll(/pub (\w+):/g) ?? []) {
          const key = prefix ? `${prefix}.${match[1]}` : match[1]
          const collectionCovered = [...documented].some((documentedKey) => documentedKey.startsWith(`${key}[].`))
          expect(documented.has(key) || collectionCovered, `${moduleId} documents ${key}`).toBe(true)
        }
      }
    }
  })
})
