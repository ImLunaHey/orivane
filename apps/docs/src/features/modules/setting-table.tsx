import type { SettingGroup } from '../../content/types'

export function SettingTable({ group }: { group: SettingGroup }) {
  return <section className="setting-section" id={slug(group.title)}><div className="setting-section-head"><h2>{group.title}</h2><p>{group.description}</p></div><table className="setting-table">
    <caption className="sr-only">{group.title} settings</caption>
    <thead><tr className="setting-row setting-header"><th scope="col">Setting</th><th scope="col">What it controls</th><th scope="col">Input</th></tr></thead>
    <tbody>{group.settings.map((setting) => <tr className="setting-row" key={setting.key}>
      <th scope="row"><strong>{setting.label}</strong><code>{setting.key}</code></th>
      <td>{setting.description}{setting.defaultValue ? <small>Default: {setting.defaultValue}</small> : null}</td>
      <td><span className="type-pill">{setting.kind}</span></td>
    </tr>)}</tbody>
  </table></section>
}

function slug(value: string) { return value.toLowerCase().replaceAll(/[^a-z0-9]+/g, '-') }
