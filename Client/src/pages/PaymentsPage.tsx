import { useState } from 'react'
import { Modal, PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import { displayDate, money } from '../utils/format'
import { errorText } from '../desktop/DesktopGymProvider'
import { DesktopPaymentsPage } from '../desktop/DesktopPaymentsPage'
export function PaymentsPage() {
  const { desktop } = useGym()
  return desktop ? <DesktopPaymentsPage/> : <BrowserPaymentsPage/>
}
function BrowserPaymentsPage() {
  const { data, addPayment, desktop } = useGym()
  const [open, setOpen] = useState(false)
  const [memberId, setMember] = useState('')
  const [amount, setAmount] = useState(0)
  const [method, setMethod] = useState<'Cash' | 'Card' | 'Transfer'>('Cash')
  const [requestId, setRequest] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  function start() { setMember(data.members[0]?.id ?? ''); setAmount(0); setRequest((desktop ? crypto.randomUUID() : '')); setError(''); setOpen(true) }
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy) return
    const member = data.members.find(row => row.id === memberId)
    if (!member) { setError('Select a member'); return }
    setBusy(true); setError('')
    try {
      await addPayment({ memberId, memberName: member.name, date: desktop?.snapshot?.today ?? new Date().toISOString().slice(0, 10), method, amount, status: desktop ? 'Recorded' : 'Paid', requestId })
      setOpen(false)
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <><PageHeader title="Payments" subtitle={desktop ? 'Received amounts recorded locally; invoice allocation and renewal are separate' : 'Membership payments, balances and receipt records'} action="Receive payment" disabled={!data.members.length} onAction={start}/>
    <div className="card table-card"><table><thead><tr><th>{desktop ? 'Payment' : 'Invoice'}</th><th>Member</th><th>Date</th><th>Method</th><th>Amount</th><th>Status</th><th>Sync</th></tr></thead><tbody>{data.payments.map(row => <tr key={row.id}><td>{row.id}</td><td>{row.memberName}</td><td>{displayDate(row.date)}</td><td>{row.method}</td><td>{money(row.amount)}</td><td><span className="tag green">{row.status}</span></td><td>{row.syncState}</td></tr>)}</tbody></table>{!data.payments.length && <p className="foundation-empty">No payments recorded.</p>}</div>
    {open && <Modal title="Receive membership payment" onClose={() => { if (!busy) setOpen(false) }}><form className="modal-form" onSubmit={event => void submit(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Member</span><select value={memberId} onChange={event => setMember(event.target.value)}>{data.members.map(member => <option value={member.id} key={member.id}>{member.name} — {member.id}</option>)}</select></label>
      <label><span>Amount (Rs.)</span><input type="number" min={desktop ? '0.01' : '1'} step={desktop ? '0.01' : '1'} required value={amount || ''} onChange={event => setAmount(Number(event.target.value))}/></label>
      <label><span>Payment method</span><select value={method} onChange={event => setMethod(event.target.value as typeof method)}><option>Cash</option><option>Card</option><option>Transfer</option></select></label>
      {desktop && <p className="form-note">Records the received amount only. Does not settle an invoice, renew membership or print a receipt.</p>}
      {error && <div role="alert" className="login-error">{error}</div>}
      <button className="primary">{busy ? 'Saving…' : desktop ? 'Save payment' : 'Save and prepare receipt'}</button>
    </fieldset></form></Modal>}
  </>
}
