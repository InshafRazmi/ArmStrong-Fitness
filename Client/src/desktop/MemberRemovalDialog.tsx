import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { Member } from '../types/domain'
import { errorText } from './DesktopGymProvider'

export function MemberRemovalDialog({ member, onClose }: { member: Member; onClose: () => void }) {
  const { desktop } = useGym()
  const [requestId] = useState(() => crypto.randomUUID())
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [confirmed, setConfirmed] = useState(false)
  const authorization = desktop!.snapshot!.removalAuthorization
  const canRemove = authorization.allowed
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy || !canRemove || !confirmed) return
    setBusy(true); setError('')
    try {
      const input = { requestId, memberId: member.id, version: member.version! }
      await desktop!.deleteMember(input)
      onClose()
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <Modal title="Permanently delete member" onClose={() => { if (!busy) onClose() }}>
    <p>{member.name} · {member.id}</p>
    <p className="form-note">Permanently remove this member and release their NFC card. Past payments, receipts, attendance and memberships remain as historical records. This cannot be undone.</p>
    {!authorization.allowed && <p role="note" className="foundation-warning">{authorization.reason}</p>}
    <form className="modal-form" onSubmit={event => void submit(event)}>
      <fieldset className="foundation-fields" disabled={busy}>
        <label className="confirmation-choice"><input required type="checkbox" checked={confirmed} onChange={event => setConfirmed(event.target.checked)} aria-label="I confirm permanent deletion of this member"/><span><b>Confirmation</b>I confirm permanently deleting {member.name}.</span></label>
        <button className="primary" disabled={!canRemove || !confirmed}>{busy ? 'Saving…' : 'Confirm permanent deletion'}</button>
      </fieldset>{error && <div role="alert" className="login-error">{error}</div>}
    </form>
  </Modal>
}
