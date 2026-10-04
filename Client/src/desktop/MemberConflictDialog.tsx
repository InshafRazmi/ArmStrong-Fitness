import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { MemberConflictInput, MemberConflictPreview, RecordedMember } from './api'
import { errorText } from './DesktopGymProvider'

export function MemberConflictDialog({ preview, onClose }: {preview: MemberConflictPreview; onClose: () => void}) {
  const { desktop } = useGym()
  const [requestId] = useState(() => crypto.randomUUID())
  const [choice, setChoice] = useState<MemberConflictInput['choice']>(preview.useServer.allowed ? 'use_server' : 'keep_local')
  const [reason, setReason] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const permission = choice === 'use_server' ? preview.useServer : preview.keepLocal
  const authorization = desktop?.snapshot?.memberSync?.reviewAuthorization
  const allowed = !!authorization?.allowed && permission.allowed
  const fields = ['name', 'phone', 'email', 'nfcId', 'joinedOn', 'archivedAt'] as const
  const labels = ['Name', 'Phone', 'Email', 'Card', 'Joined date', 'Archived at']
  function field(member: RecordedMember | null, key: typeof fields[number]) { return member ? member[key] || '—' : 'No record' }
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy || !allowed) return
    setBusy(true); setError('')
    try {
      await desktop!.resolveMemberConflict({ requestId, conflictId: preview.conflictId, fingerprint: preview.fingerprint, choice, reason })
      onClose()
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <Modal title="Review member conflict" onClose={() => { if (!busy) onClose() }}>
    <p>{preview.local?.name || preview.remote?.name || preview.memberId}</p>
    <p className="form-note">The server column is the version recorded with the conflict. This review does not contact the server. A fresh retry can encounter another server change.</p>
    <div className="card table-card"><table><thead><tr><th>Field</th><th>Current local version {preview.local?.version ?? '—'}</th><th>Recorded server revision {preview.remote?.revision ?? '—'}</th></tr></thead><tbody>{fields.map((key, index) => <tr key={key}><td>{labels[index]}</td><td>{field(preview.local, key)}</td><td>{field(preview.remote, key)}</td></tr>)}</tbody></table></div>
    <ul>{preview.conflicts.map(conflict => <li key={conflict.id}>{conflict.reason} · {conflict.createdAt}</li>)}</ul>
    <p>{preview.conflicts.length} conflict record(s) and {preview.operations.length} pending member edit(s) are covered by this review. Original edits and conflict records remain in history. Memberships, attendance, payments and receipts retain their history.</p>
    {!authorization?.allowed && <p role="note" className="foundation-warning">{authorization?.reason || 'An enrolled Administrator session is required.'}</p>}
    <form className="modal-form" onSubmit={event => void submit(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><input type="radio" name="resolution" checked={choice === 'use_server'} disabled={!preview.useServer.allowed} onChange={() => setChoice('use_server')}/>Use recorded server version. Discard the reviewed pending member edits.</label>
      {!preview.useServer.allowed && <p className="form-note">{preview.useServer.reason}</p>}
      <label><input type="radio" name="resolution" checked={choice === 'keep_local'} disabled={!preview.keepLocal.allowed} onChange={() => setChoice('keep_local')}/>Keep current local version. Supersede the reviewed edits and queue a fresh retry.</label>
      {!preview.keepLocal.allowed && <p className="form-note">{preview.keepLocal.reason}</p>}
      <label><span>Review reason</span><textarea required maxLength={500} value={reason} onChange={event => setReason(event.target.value)}/></label>
      <label><input required type="checkbox" aria-label="I confirm the reviewed member conflict choice"/>I confirm this choice for this member.</label>
      <button className="primary" disabled={!allowed}>{busy ? 'Saving review…' : 'Confirm member review'}</button>
    </fieldset>{error && <div role="alert" className="login-error">{error}</div>}</form>
  </Modal>
}
