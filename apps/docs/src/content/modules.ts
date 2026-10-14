import type { ModuleCategory, ModuleDoc } from './types'
import { automationModules } from './modules/automation'
import { engagementModules } from './modules/engagement'
import { operationsModules } from './modules/operations'
import { safetyModules } from './modules/safety'

export const modules: ModuleDoc[] = [
  ...safetyModules,
  ...engagementModules,
  ...operationsModules,
  ...automationModules,
].sort((a, b) => a.title.localeCompare(b.title))

export const moduleCategories: ModuleCategory[] = ['Safety', 'Community', 'Engagement', 'Automation', 'Operations']

export function findModule(moduleId: string): ModuleDoc | undefined {
  return modules.find((module) => module.id === moduleId)
}

export function settingCount(module: ModuleDoc): number {
  return module.settingGroups.reduce((total, group) => total + group.settings.length, 0)
}
