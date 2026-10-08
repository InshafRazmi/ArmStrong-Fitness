import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { Trainer } from './api'
import { errorText } from './DesktopGymProvider'

export function StaffRemovalDialog({ staff, onClose }: { staff: Trainer; onClose: () => void }) {
  const { desktop } = useGym()
  const [requestId] = useState(() => crypto.randomUUID())
  const [confirmed, setConfirmed] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const authorization = desktop!.snapshot!.removalAuthorization
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy || !confirmed || !authorization.allowed) return
    setBusy(true); setError('')
    try {
      await desktop!.deleteStaff({ requestId, staffId: staff.id, version: staff.version })
      onClose()
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <Modal title="Delete staff" onClose={() => { if (!busy) onClose() }}>
    <p>{staff.name}</p>
    <p className="form-note">Permanently remove this profile from active and inactive staff lists, revoke its NFC card and clear current member assignments. Salary payments, training invoices, attendance and unpaid earnings remain. Use Show deleted staff to review retained history and settle remaining earnings. This profile cannot be reactivated.</p>
    <p className="form-note">{staff.assignedMembers} current member assignment(s) will be cleared. Existing invoices keep their recorded trainer and amounts.</p>
    {!authorization.allowed && <p role="note" className="foundation-warning">{authorization.reason}</p>}
    <form className="modal-form" onSubmit={event => void submit(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label className="confirmation-choice"><input required type="checkbox" checked={confirmed} onChange={event => setConfirmed(event.target.checked)} aria-label="I confirm deleting this staff profile"/><span><b>Confirmation</b>I confirm permanently deleting {staff.name}.</span></label>
      <button className="primary" disabled={!authorization.allowed || !confirmed}>{busy ? 'Saving…' : 'Confirm staff deletion'}</button>
    </fieldset>{error && <div role="alert" className="login-error">{error}</div>}</form>
  </Modal>
}
