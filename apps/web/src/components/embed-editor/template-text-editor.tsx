import { autocompletion, type CompletionContext } from '@codemirror/autocomplete'
import { EditorState } from '@codemirror/state'
import { Decoration, type DecorationSet, EditorView, MatchDecorator, ViewPlugin, type ViewUpdate } from '@codemirror/view'
import CodeMirror from '@uiw/react-codemirror'
import { useId, useMemo } from 'react'

const variables=[{token:'{user}',detail:'Member mention'},{token:'{display_name}',detail:'Display name'},{token:'{username}',detail:'Discord handle'},{token:'{user_id}',detail:'Member ID'},{token:'{server}',detail:'Server name'},{token:'{server_id}',detail:'Server ID'}]
const matcher=new MatchDecorator({regexp:/\{(?:user|display_name|username|user_id|server|server_id)\}/g,decoration:()=>Decoration.mark({class:'cm-template-variable'})})
const highlighting=ViewPlugin.fromClass(class{decorations:DecorationSet;constructor(view:EditorView){this.decorations=matcher.createDeco(view)}update(update:ViewUpdate){this.decorations=matcher.updateDeco(update,this.decorations)}},{decorations:p=>p.decorations})
const completion=(context:CompletionContext)=>{const token=context.matchBefore(/\{[a-z_]*/);if(!token||(token.from===token.to&&!context.explicit))return null;return{from:token.from,options:variables.map(x=>({label:x.token,detail:x.detail,type:'variable',apply:x.token}))}}
const theme=EditorView.theme({'&':{backgroundColor:'#17171a',color:'#e5e5e7',fontSize:'16px'},'.cm-content':{minHeight:'92px',padding:'10px 11px',caretColor:'#c4b5fd',fontFamily:'inherit',lineHeight:'1.5'},'.cm-line':{padding:'0'},'.cm-cursor':{borderLeftColor:'#c4b5fd'},'.cm-selectionBackground, &.cm-focused .cm-selectionBackground':{backgroundColor:'#7657ca55'},'.cm-gutters':{display:'none'},'&.cm-focused':{outline:'2px solid #8b6ed1',outlineOffset:'1px'}},{dark:true})

export function TemplateTextEditor({title,detail,value,maxLength,onChange}:{title:string;detail:string;value:string;maxLength:number;onChange:(value:string)=>void}){
  const id=useId();const extensions=useMemo(()=>[EditorView.lineWrapping,EditorState.changeFilter.of(t=>t.newDoc.length<=maxLength),autocompletion({override:[completion],activateOnTyping:true}),highlighting,theme,EditorView.contentAttributes.of({'aria-labelledby':`${id}-label`,'aria-describedby':`${id}-detail`})],[id,maxLength])
  return <div className="embed-text-editor"><div className="welcome-editor-label"><span id={`${id}-label`}>{title}</span><small id={`${id}-detail`}>{detail} Type <code>{'{'}</code> for variables.</small></div><CodeMirror className="welcome-codemirror" theme="none" value={value} extensions={extensions} basicSetup={{lineNumbers:false,foldGutter:false,highlightActiveLine:false,highlightActiveLineGutter:false,closeBrackets:false,autocompletion:false,bracketMatching:false}} onChange={onChange}/></div>
}
