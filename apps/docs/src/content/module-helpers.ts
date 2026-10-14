import type { ModuleDoc, SettingDoc, SettingKind } from './types'

export function setting(
  key: string,
  label: string,
  description: string,
  kind: SettingKind = 'text',
  defaultValue?: string,
): SettingDoc {
  return { key, label, description, kind, defaultValue }
}

export function moduleDoc(doc: ModuleDoc): ModuleDoc {
  return doc
}
