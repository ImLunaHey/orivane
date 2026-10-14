import { PageHeader } from '../../components/page-header'
import { modules } from '../../content/modules'

export function CommandReferencePage() {
  const withCommands = modules.filter((module) => module.commands.length)
  const count = new Set(withCommands.flatMap((module) => module.commands)).size
  return <div className="content-page">
    <PageHeader eyebrow="Reference" title="Slash-command catalog" description={`${count} documented application commands grouped by the Rust module that owns their behavior. Orivane does not register or process prefix commands.`} />
    <div className="command-reference">{withCommands.map((module) => <section key={module.id}><div><span className="category-pill">{module.category}</span><h2>{module.title}</h2><p>{module.summary}</p></div><div className="command-list large">{module.commands.map((command) => <code key={command}>{command}</code>)}</div></section>)}</div>
  </div>
}
