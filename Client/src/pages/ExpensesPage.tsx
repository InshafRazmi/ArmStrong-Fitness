import { useState } from 'react'
import { Modal, PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import { displayDate, money } from '../utils/format'
import type { Expense } from '../types/domain'
import { ExpenseVoidDialog } from '../desktop/ExpenseVoidDialog'
import { errorText } from '../desktop/DesktopGymProvider'
export function ExpensesPage() {
  const { data, addExpense, desktop } = useGym()
  const [voiding, setVoiding] = useState<Expense | null>(null)
  const [open, setOpen] = useState(false)
  const [form, setForm] = useState({ title: '', category: 'Operations', amount: 0, method: 'Cash' as 'Cash' | 'Card' | 'Bank' })
  const [requestId, setRequest] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy) return
    setBusy(true); setError('')
    try { await addExpense({ ...form, requestId, date: desktop?.snapshot?.today ?? new Date().toISOString().slice(0, 10), recordedBy: desktop ? '' : 'Prinzz' }); setOpen(false) }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <><PageHeader title="Expenses" subtitle="Record and review operational spending" action="Add expense" onAction={() => { setForm({ title: '', category: 'Operations', amount: 0, method: 'Cash' }); setRequest((desktop ? crypto.randomUUID() : '')); setError(''); setOpen(true) }}/>
    {desktop && <><p className="form-note">Effective expenses: {money(desktop.snapshot!.expenses.reduce((sum, row) => sum + row.effectiveAmountMinor, 0) / 100)}. Voided amounts remain in history and are excluded from totals.</p><p className="form-note">{desktop.snapshot!.removalAuthorization.allowed ? 'Voiding requires the authenticated Administrator.' : desktop.snapshot!.removalAuthorization.reason}</p></>}
    <div className="card table-card"><table><thead><tr><th>Expense</th><th>Category</th><th>Date</th><th>Method</th><th>Amount</th><th>Recorded by</th><th>Sync</th>{desktop && <><th>Status / void history</th><th>Action</th></>}</tr></thead><tbody>{data.expenses.map(row => <tr key={row.id}><td>{row.title}</td><td>{row.category}</td><td>{displayDate(row.date)}</td><td>{row.method}</td><td>{money(row.amount)}</td><td>{row.recordedBy}</td><td>{row.syncState}</td>{desktop && <><td><span className={row.status === 'Recorded' ? 'tag green' : 'tag red'}>{row.status}</span>{row.voidReason && <small className="foundation-id">{row.voidReason}<br/>{row.voidedBy}<br/>{row.voidedAt}</small>}</td><td>{row.status === 'Recorded' && <button className="secondary compact" onClick={() => setVoiding(row)}>Void / Reverse</button>}</td></>}</tr>)}</tbody></table>{!data.expenses.length && <p className="foundation-empty">No expenses recorded.</p>}</div>
    {voiding && <ExpenseVoidDialog expense={voiding} onClose={() => setVoiding(null)}/>}
    {open && <Modal title="Add operational expense" onClose={() => { if (!busy) setOpen(false) }}><form className="modal-form" onSubmit={event => void submit(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Description</span><input required maxLength={254} value={form.title} onChange={event => setForm({ ...form, title: event.target.value })}/></label>
      <label><span>Category</span><select value={form.category} onChange={event => setForm({ ...form, category: event.target.value })}><option>Operations</option><option>Utilities</option><option>Maintenance</option><option>Salary</option><option>Other</option></select></label>
      <label><span>Amount</span><input type="number" min={desktop ? '0.01' : '1'} step={desktop ? '0.01' : '1'} required value={form.amount || ''} onChange={event => setForm({ ...form, amount: Number(event.target.value) })}/></label>
      <label><span>Method</span><select value={form.method} onChange={event => setForm({ ...form, method: event.target.value as typeof form.method })}><option>Cash</option><option>Card</option><option>Bank</option></select></label>
      {error && <div role="alert" className="login-error">{error}</div>}
      <button className="primary">{busy ? 'Saving…' : 'Save expense'}</button>
    </fieldset></form></Modal>}
  </>
}
