import { Link } from '@tanstack/react-router'
import { CheckCircle2, CircleDot } from 'lucide-react'
import { InfoPanel } from '../../components/info-panel'
import { PageHeader } from '../../components/page-header'

export function GettingStartedPage() {
  const testSteps = ['Enable and configure the module.', 'Save changes at the bottom of the page.', 'Run its slash command or trigger its event.', 'Check the server audit log if the result is unexpected.']
  return <div className="content-page prose-page">
    <PageHeader eyebrow="Server owner guide" title="Getting started" description="Add Orivane to a Discord server you manage, choose its permissions, configure your first module, and confirm the result." />
    <section className="notice"><CircleDot /><div><strong>Orivane is hosted for you and uses slash commands only.</strong><p>You do not need to deploy the bot, create a Discord application, manage a bot token, enable gateway intents, or configure a command prefix.</p></div></section>
    <section><h2>1. Sign in with Discord</h2><p>Open the Orivane dashboard and choose <strong>Sign in with Discord</strong>. Discord tells Orivane which servers you can manage; your password is never shared with Orivane.</p></section>
    <section><h2>2. Add Orivane to your server</h2><p>Choose <strong>Add server</strong> in the dashboard, select a server where you have Manage Server or Administrator, and approve the requested Discord permissions. You can review the permissions before accepting.</p><div className="two-column-list"><InfoPanel title="Can’t see a server?">Your Discord account needs Manage Server or Administrator in that server.</InfoPanel><InfoPanel title="Orivane already installed?">Select the server from the dashboard instead of adding the bot again.</InfoPanel></div></section>
    <section><h2>3. Check the bot role</h2><p>Discord’s role order controls what Orivane can manage. In <strong>Server Settings → Roles</strong>, place the Orivane role above any member or reward roles it needs to assign, and above members it needs to moderate.</p><div className="check-grid"><span><CheckCircle2 />Orivane can view the selected channels</span><span><CheckCircle2 />Orivane can send messages and embeds</span><span><CheckCircle2 />Orivane is above roles it assigns</span><span><CheckCircle2 />Orivane has the permissions your modules need</span></div></section>
    <section><h2>4. Configure a module</h2><p>Select your server, open <strong>Modules</strong>, and choose a feature. Every module is disabled until you enable it. Channel and role fields use Discord-aware selectors, so you do not need to copy raw IDs.</p></section>
    <section><h2>5. Save and test</h2><p>Save controls sit at the bottom of each editor. Test the workflow in a private channel, then open the server audit log to see who changed the configuration and whether the Discord action succeeded.</p><ol className="plain-steps">{testSteps.map((step, index) => <li key={step}><span>{index + 1}</span><p>{step}</p></li>)}</ol></section>
    <section className="next-links"><Link to="/modules/$moduleId" params={{ moduleId: 'automod' }}><small>Common first module</small><strong>Configure Automod →</strong></Link><Link to="/reference/audit"><small>Understand every change</small><strong>Audit and analytics →</strong></Link></section>
  </div>
}
