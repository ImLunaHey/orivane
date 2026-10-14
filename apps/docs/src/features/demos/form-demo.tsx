import { CheckCircle2 } from 'lucide-react'
import { useState } from 'react'

export function FormDemo() {
  const [name, setName] = useState('Staff application')
  const [motivation, setMotivation] = useState('I want to help keep discussions welcoming and organized.')
  const [submitted, setSubmitted] = useState(false)
  return <div className="demo-grid"><form className="modal-preview" onSubmit={(event) => { event.preventDefault(); setSubmitted(true) }}><strong>{name || 'Untitled form'}</strong><label><span>Discord experience</span><input required placeholder="Tell us about your experience" /></label><label><span>Why do you want to join?</span><textarea required value={motivation} onChange={(event) => setMotivation(event.target.value)} /></label><button type="submit">Submit</button></form><div className="demo-controls"><label><span>Form name</span><input value={name} onChange={(event) => { setName(event.target.value); setSubmitted(false) }} /></label><p className="demo-help">Discord modals support up to five short or paragraph inputs. This preview stays entirely in your browser.</p>{submitted ? <div className="success-note"><CheckCircle2 />Submission preview completed. Production stores answers in the form’s dedicated review queue.</div> : null}</div></div>
}
