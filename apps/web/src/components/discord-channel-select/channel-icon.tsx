import { Folder, Hash, Volume2 } from 'lucide-react'

export function ChannelIcon({ kind }: { kind: string }) {
  return kind === 'GuildCategory' ? <Folder size={17} /> : kind === 'GuildVoice' || kind === 'GuildStageVoice' ? <Volume2 size={17} /> : <Hash size={17} />
}
