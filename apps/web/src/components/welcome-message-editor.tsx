import { autocompletion, type CompletionContext } from '@codemirror/autocomplete'
import { EditorState } from '@codemirror/state'
import { Decoration, type DecorationSet, EditorView, MatchDecorator, ViewPlugin, type ViewUpdate } from '@codemirror/view'
import CodeMirror, { type ReactCodeMirrorRef } from '@uiw/react-codemirror'
import { useId, useMemo, useRef } from 'react'

type WelcomeMessageEditorProps = {
  value: string
  serverId: string
  serverName: string
  onChange: (value: string) => void
}

const variables = [
  { token: '{user}', description: 'Member mention' },
  { token: '{display_name}', description: 'Server display name' },
  { token: '{username}', description: 'Discord username' },
  { token: '{user_id}', description: 'Member ID' },
  { token: '{server}', description: 'Server name' },
  { token: '{server_id}', description: 'Server ID' },
] as const

const previewValues: Record<string, string> = {
  '{user}': '@imlunahey',
  '{display_name}': 'luna',
  '{username}': 'imlunahey',
  '{user_id}': '784365843810222080',
}

const templateMatcher = new MatchDecorator({
  regexp: /\{(?:user|display_name|username|user_id|server|server_id)\}/g,
  decoration: () => Decoration.mark({ class: 'cm-template-variable' }),
})

const templateHighlighting = ViewPlugin.fromClass(class {
  decorations: DecorationSet

  constructor(view: EditorView) {
    this.decorations = templateMatcher.createDeco(view)
  }

  update(update: ViewUpdate) {
    this.decorations = templateMatcher.updateDeco(update, this.decorations)
  }
}, { decorations: (plugin) => plugin.decorations })

const templateCompletion = (context: CompletionContext) => {
  const token = context.matchBefore(/\{[a-z_]*/)
  if (!token || (token.from === token.to && !context.explicit)) return null
  return {
    from: token.from,
    options: variables.map((variable) => ({
      label: variable.token,
      detail: variable.description,
      type: 'variable',
      apply: variable.token,
    })),
  }
}

const editorTheme = EditorView.theme({
  '&': { backgroundColor: '#17171a', color: '#e5e5e7', fontSize: '16px' },
  '.cm-content': { minHeight: '112px', padding: '10px 11px', caretColor: '#c4b5fd', fontFamily: 'inherit', lineHeight: '1.5' },
  '.cm-line': { padding: '0' },
  '.cm-cursor': { borderLeftColor: '#c4b5fd' },
  '.cm-selectionBackground, &.cm-focused .cm-selectionBackground': { backgroundColor: '#7657ca55' },
  '.cm-gutters': { display: 'none' },
  '&.cm-focused': { outline: 'none' },
}, { dark: true })

const editorExtensions = [
  EditorView.lineWrapping,
  EditorState.changeFilter.of((transaction) => transaction.newDoc.length <= 1800),
  autocompletion({ override: [templateCompletion], activateOnTyping: true }),
  templateHighlighting,
  editorTheme,
]

export function WelcomeMessageEditor({ value, serverId, serverName, onChange }: WelcomeMessageEditorProps) {
  const baseId = useId()
  const editorRef = useRef<ReactCodeMirrorRef>(null)
  const extensions = useMemo(() => [
    ...editorExtensions,
    EditorView.contentAttributes.of({
      'aria-labelledby': `${baseId}-label`,
      'aria-describedby': `${baseId}-description`,
    }),
  ], [baseId])

  const insertVariable = (token: string) => {
    const view = editorRef.current?.view
    if (!view) return
    const selection = view.state.selection.main
    view.dispatch({
      changes: { from: selection.from, to: selection.to, insert: token },
      selection: { anchor: selection.from + token.length },
    })
    view.focus()
  }

  return <div className="welcome-editor wide">
    <div className="welcome-editor-label"><span id={`${baseId}-label`}>Welcome message</span><small id={`${baseId}-description`}>Write the message Orivane sends when somebody joins. Type <code>{'{'}</code> for variables.</small></div>
    <CodeMirror
      ref={editorRef}
      className="welcome-codemirror"
      theme="none"
      value={value}
      extensions={extensions}
      basicSetup={{
        lineNumbers: false,
        foldGutter: false,
        highlightActiveLine: false,
        highlightActiveLineGutter: false,
        closeBrackets: false,
        autocompletion: false,
        bracketMatching: false,
      }}
      placeholder="Welcome {user} to {server}!"
      onChange={onChange}
    />
    <div className="welcome-variable-list" aria-label="Welcome message variables">
      {variables.map((variable) => <button key={variable.token} type="button" onClick={() => insertVariable(variable.token)} title={`Insert ${variable.description}`}><code>{variable.token}</code><span>{variable.description}</span></button>)}
    </div>
    <div className="discord-message-preview">
      <div className="discord-preview-heading"><span>Discord preview</span><small>Sample member data</small></div>
      <div className="discord-preview-canvas">
        <img src="/orivane-avatar.png" alt="" />
        <div className="discord-preview-message">
          <div className="discord-preview-author"><strong>Orivane</strong><span>APP</span><time>Today at 12:00</time></div>
          <p>{renderPreview(value || 'Your welcome message will appear here.', serverId, serverName)}</p>
        </div>
      </div>
    </div>
  </div>
}

function renderPreview(template: string, serverId: string, serverName: string) {
  const values: Record<string, string> = { ...previewValues, '{server}': serverName, '{server_id}': serverId }
  const tokens = template.split(/(\{(?:user|display_name|username|user_id|server|server_id)\})/g)
  return tokens.map((part, index) => {
    const rendered = values[part]
    if (!rendered) return part
    return part === '{user}' ? <span className="discord-preview-mention" key={index}>{rendered}</span> : <span key={index}>{rendered}</span>
  })
}
