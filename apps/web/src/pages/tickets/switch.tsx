import { Toggle } from '../../components/ui'

export function Switch({title,checked,onChange}:{title:string;checked:boolean;onChange:(value:boolean)=>void}){return <div className="module-toggle-field"><strong>{title}</strong><Toggle label={title} checked={checked} onChange={onChange}/></div>}
