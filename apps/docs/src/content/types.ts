export type SettingKind = 'toggle' | 'number' | 'text' | 'textarea' | 'select' | 'channel' | 'role' | 'list' | 'color' | 'timestamp'

export type SettingDoc = {
  key: string
  label: string
  description: string
  kind: SettingKind
  defaultValue?: string
}

export type SettingGroup = {
  title: string
  description: string
  settings: SettingDoc[]
}

export type DemoKind = 'welcome' | 'automod' | 'leveling' | 'anti-raid' | 'forms' | 'polls' | 'counters'

export type ModuleCategory = 'Safety' | 'Community' | 'Engagement' | 'Automation' | 'Operations'

export type ModuleDoc = {
  id: string
  title: string
  category: ModuleCategory
  summary: string
  description: string
  highlights: string[]
  commands: string[]
  settingGroups: SettingGroup[]
  demo?: DemoKind
  dashboardOnly?: boolean
}
