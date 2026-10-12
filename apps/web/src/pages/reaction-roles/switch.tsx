import { Toggle } from '../../components/ui'

export function Switch({title,detail,checked,onChange}:{title:string;detail:string;checked:boolean;onChange:(value:boolean)=>void}){return <div className="module-toggle-field"><span><strong>{title}</strong><small>{detail}</small></span><Toggle label={title} checked={checked} onChange={onChange}/></div>}
