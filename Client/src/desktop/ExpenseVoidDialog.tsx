import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { Expense } from '../types/domain'
import { money } from '../utils/format'
import { errorText } from './DesktopGymProvider'

export function ExpenseVoidDialog({ expense, onClose }: { expense: Expense; onClose: () => void }) {
  const { desktop } = useGym()
  const [requestId] = useState(() => crypto.randomUUID())
  const [reason, setReason] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const authorization = desktop!.snapshot!.removalAuthorization
  const canVoid = authorization.allowed && expense.status === 'Recorded'
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy || !canVoid) return
    setBusy(true); setError('')
    try { await desktop!.voidExpense({ requestId, expenseId: expense.id, reason }); onClose() }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <Modal title="Void / reverse expense" onClose={() => { if (!busy) onClose() }}>
    <p>{expense.title} · {money(expense.amount)} · {expense.method}</p>
    <p className="form-note">Exclude this expense from effective totals. Its original amount, date and recorded user remain in history. Saves your reason and staff identity in an immutable void and audit record. This does not execute a bank refund.</p>
    {!authorization.allowed && <p role="note" className="foundation-warning">{authorization.reason}</p>}
    <form className="modal-form" onSubmit={event => void submit(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Required void reason</span><input required maxLength={254} value={reason} onChange={event => setReason(event.target.value)}/></label>
      <label><span>Confirmation</span><input required type="checkbox" aria-label="I confirm voiding this expense"/>I confirm voiding {expense.title}.</label>
      <button className="primary" disabled={!canVoid}>{busy ? 'Saving…' : 'Confirm expense void'}</button>
    </fieldset>{error && <div role="alert" className="login-error">{error}</div>}</form>
  </Modal>
}
