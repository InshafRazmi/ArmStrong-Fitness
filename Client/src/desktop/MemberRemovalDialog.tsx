import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { Member } from '../types/domain'
import { errorText } from './DesktopGymProvider'

export function MemberRemovalDialog({ member, kind, onClose }: { member: Member; kind: 'archive' | 'delete'; onClose: () => void }) {
  const { desktop } = useGym()
  const [requestId] = useState(() => crypto.randomUUID())
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const authorization = desktop!.snapshot!.removalAuthorization
  const canRemove = authorization.allowed && (kind !== 'delete' || member.canDelete)
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy || !canRemove) return
    setBusy(true); setError('')
    try {
      const input = { requestId, memberId: member.id, version: member.version! }
      if (kind === 'archive') await desktop!.archiveMember(input)
      else await desktop!.deleteMember(input)
      onClose()
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <Modal title={kind === 'archive' ? 'Archive / deactivate member' : 'Permanently delete unlinked member'} onClose={() => { if (!busy) onClose() }}>
    <p>{member.name} · {member.id}</p>
    <p className="form-note">{kind === 'archive'
      ? 'Hide this member from active lists and prevent new attendance and memberships. Existing dates, NFC assignment, invoices, payments and all history remain in SQLite. This does not cancel debt or free the card.'
      : 'Delete this member only if there are no membership, attendance, invoice, payment or NFC history links. This cannot be undone from this screen. Audit and pending operation history remain.'}</p>
    {!authorization.allowed && <p role="note" className="foundation-warning">{authorization.reason}</p>}
    <form className="modal-form" onSubmit={event => void submit(event)}>
      <fieldset className="foundation-fields" disabled={busy}><label><span>Confirmation</span><input required type="checkbox" aria-label={kind === 'archive' ? 'I confirm archiving this member' : 'I confirm permanent deletion of this member'}/>I confirm this action for {member.name}.</label>
        <button className="primary" disabled={!canRemove}>{busy ? 'Saving…' : kind === 'archive' ? 'Confirm archive' : 'Confirm permanent deletion'}</button>
      </fieldset>{error && <div role="alert" className="login-error">{error}</div>}
    </form>
  </Modal>
}
