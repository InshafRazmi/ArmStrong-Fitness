import { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { Modal, PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import { displayDate, money } from '../utils/format'
import { errorText } from './DesktopGymProvider'
import { minorUnits, type Snapshot, type ReceiptDocument } from './api'
import { ReceiptView } from './ReceiptView'

type Workflow = 'training' | 'invoice' | 'receive' | 'allocate' | 'renew' | 'reverse'
const titles: Record<Workflow, string> = { training: 'Monthly personal-training invoice', invoice: 'New invoice', receive: 'Receive member payment', allocate: 'Allocate existing credit', renew: 'Renew membership', reverse: 'Reverse received payment' }
const actions: Record<Workflow, string> = { training: 'Save training invoice', invoice: 'Save invoice', receive: 'Save payment', allocate: 'Save allocation', renew: 'Save renewal and invoice', reverse: 'Confirm full reversal' }
const amountText = (minor: number) => (minor / 100).toFixed(2)
function outstandingInvoices(snapshot: Snapshot, memberId: string) {
  return snapshot.invoices.filter(i => i.memberId === memberId && i.outstandingMinor > 0).sort((a,b) => Number(snapshot.trainingCharges.some(c=>c.invoiceId===a.id)) - Number(snapshot.trainingCharges.some(c=>c.invoiceId===b.id)) || a.createdAt.localeCompare(b.createdAt))
}
function nextTrainingStart(snapshot: Snapshot, memberId: string) {
  const lastTraining = snapshot.trainingCharges.filter(c=>c.memberId===memberId).sort((a,b)=>b.endsOn.localeCompare(a.endsOn))[0]
  if (lastTraining) {
    const next = new Date(lastTraining.endsOn + 'T12:00:00Z')
    next.setUTCDate(next.getUTCDate()+1)
    return next.toISOString().slice(0,10)
  }
  const periods = snapshot.periods.filter(p=>p.memberId===memberId).sort((a,b)=>a.startsOn.localeCompare(b.startsOn))
  return periods.find(p=>p.startsOn<=snapshot.today && p.endsOn>=snapshot.today)?.startsOn ?? periods.find(p=>p.startsOn>snapshot.today)?.startsOn ?? snapshot.today
}
export function DesktopPaymentsPage({ receiveFor, onClose }: { receiveFor?: string; onClose?: () => void } = {}) {
  const { desktop } = useGym()
  const native = desktop!.snapshot!
  const [workflow, setWorkflow] = useState<Workflow | null>(null)
  // Capture expected plan version/history at open, so a refresh cannot silently change the agreed renewal.
  const [base, setBase] = useState<Snapshot | null>(null)
  const [form, setForm] = useState({ requestId: '', memberId: '', periodId: '', invoiceId: '', paymentId: '', planId: '', amount: '', method: 'Cash' as 'Cash' | 'Card' | 'Transfer', description: '', startsOn: '', endsOn: '', reason: '' })
  const [invoiceIds, setInvoiceIds] = useState<string[]>([])
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [receipt, setReceipt] = useState<ReceiptDocument | null>(null)
  const [receiptBusy, setReceiptBusy] = useState(false)
  const [receiptError, setReceiptError] = useState('')
  const [receiptOpen, setReceiptOpen] = useState(false)
  function open(kind: Workflow, memberId = native.members.find(member => member.active)?.id ?? '', invoiceId = '', paymentId = '') {
    setBase(native); setWorkflow(kind); setError('')
    const available = native.payments.find(payment => payment.memberId === memberId && payment.unallocatedMinor > 0)
    const due = outstandingInvoices(native, memberId)
    const selected = kind === 'receive' ? (invoiceId ? [invoiceId] : due.map(i=>i.id)) : []
    setInvoiceIds(selected)
    setForm({ requestId: crypto.randomUUID(), memberId, periodId: '', invoiceId, paymentId: paymentId || available?.id || '', planId: native.plans.find(plan => plan.active && plan.priceMinor > 0)?.id ?? '', amount: selected.length ? amountText(due.filter(i=>selected.includes(i.id)).reduce((sum,i)=>sum+i.outstandingMinor,0)) : '', method: 'Cash', description: '', startsOn: kind === 'training' ? nextTrainingStart(native, memberId) : '', endsOn: '', reason: '' })
  }
  useEffect(() => { if (receiveFor && !workflow && native.members.some(member => member.id === receiveFor)) open('receive', receiveFor) }, [receiveFor, native])
  async function showReceipt(paymentId: string) {
    setReceipt(null); setReceiptError(''); setReceiptOpen(true); setReceiptBusy(true)
    try { setReceipt(await desktop!.paymentReceipt(paymentId)) }
    catch (error) { setReceiptError(errorText(error)) }
    finally { setReceiptBusy(false) }
  }
  function print() {
    try { window.print() }
    catch (error) { setReceiptError('Could not open the print dialog: ' + errorText(error)) }
  }
  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (busy || !workflow || !base) return
    setBusy(true); setError('')
    try {
      const common = { requestId: form.requestId, memberId: form.memberId }
      if (workflow === 'invoice') await desktop!.createInvoice({ ...common, membershipPeriodId: form.periodId || null, description: form.description, amountMinor: minorUnits(Number(form.amount)) })
      if (workflow === 'receive') await desktop!.receiveCombinedPayment({ payment: { ...common, invoiceId: null, amountMinor: minorUnits(Number(form.amount)), method: form.method }, invoiceIds })
      if (workflow === 'training') {
        const assignment = base.memberTrainers.find(a=>a.memberId===form.memberId)
        const trainer = base.trainers.find(t=>t.id===assignment?.trainerId && t.active)
        if (!trainer) throw new Error('Assign an active staff member in Members before billing personal training.')
        await desktop!.createTrainingCharge({ ...common, trainerId: trainer.id, trainerVersion: trainer.version, startsOn: form.startsOn })
      }
      if (workflow === 'allocate') await desktop!.allocatePayment({ requestId: form.requestId, paymentId: form.paymentId, invoiceId: form.invoiceId, amountMinor: minorUnits(Number(form.amount)) })
      if (workflow === 'renew') {
        const plan = base.plans.find(plan => plan.id === form.planId)
        if (!plan) throw new Error('Select an active paid plan.')
        const latest = base.periods.filter(period => period.memberId === form.memberId).sort((a, b) => b.endsOn.localeCompare(a.endsOn) || b.id.localeCompare(a.id))[0]
        await desktop!.renewMembership({ ...common, planId: plan.id, planVersion: plan.version, expectedLastPeriodId: latest?.id ?? null, startsOn: form.startsOn, endsOn: form.endsOn })
      }
      if (workflow === 'reverse') await desktop!.reversePayment({ requestId: form.requestId, paymentId: form.paymentId, reason: form.reason })
      setWorkflow(null)
      onClose?.()
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  const capture = base ?? native
  const invoices = outstandingInvoices(capture, form.memberId)
  const payments = capture.payments.filter(payment => payment.memberId === form.memberId && payment.unallocatedMinor > 0)
  const periods = capture.periods.filter(period => period.memberId === form.memberId && period.priceMinor > 0 && !capture.invoices.some(invoice => invoice.membershipPeriodId === period.id))
  const plan = capture.plans.find(plan => plan.id === form.planId)
  const latest = capture.periods.filter(period => period.memberId === form.memberId).sort((a,b) => b.endsOn.localeCompare(a.endsOn) || b.id.localeCompare(a.id))[0]
  const original = capture.payments.find(payment => payment.id === form.paymentId)
  const invoice = invoices.find(invoice => invoice.id === form.invoiceId)
  const assignedTrainer = capture.trainers.find(t=>t.id===capture.memberTrainers.find(a=>a.memberId===form.memberId)?.trainerId)
  const credit = payments.find(payment => payment.id === form.paymentId)?.unallocatedMinor ?? 0
  if (receiveFor && !native.members.some(member => member.id === receiveFor)) return <Modal title="Refresh joining charges" onClose={() => onClose?.()}><p>The member was saved. Refresh the saved records before receiving payment.</p><button className="primary" onClick={() => void desktop!.refresh().catch(() => {})}>Refresh records</button></Modal>
  return <>{!receiveFor && <><PageHeader title="Payments" subtitle="Member balances, payments and receipts" action="Receive payment" disabled={!native.members.some(member => member.active)} onAction={() => open('receive')}/>
    <div className="finance-toolbar"><button className="secondary" disabled={!native.memberTrainers.some(a=>native.trainers.some(t=>t.id===a.trainerId && t.active && t.trainingFeeMinor>0))} onClick={() => open('training', native.memberTrainers.find(a=>native.trainers.some(t=>t.id===a.trainerId && t.active && t.trainingFeeMinor>0))?.memberId)}>Training invoice</button><button className="secondary" disabled={!native.members.length} onClick={() => open('invoice')}>New invoice</button>
      <button className="secondary" disabled={!native.payments.some(payment => payment.unallocatedMinor > 0)} onClick={() => { const payment = native.payments.find(payment => payment.unallocatedMinor > 0)!; open('allocate', payment.memberId, '', payment.id) }}>Allocate credit</button>
      <button className="secondary" disabled={!native.members.some(member => member.active) || !native.plans.some(plan => plan.active && plan.priceMinor > 0)} onClick={() => open('renew')}>Renew membership</button></div>
    {receiptError && !receiptOpen && <div role="alert" className="login-error">{receiptError}</div>}
    <h3 className="finance-section">Member balances</h3>
    <div className="card table-card"><table><thead><tr><th>Member</th><th>Invoice outstanding</th><th>Unallocated credit</th><th>Net balance</th></tr></thead><tbody>{native.financialAccounts.map(account => <tr key={account.memberId}><td>{account.memberName}</td><td>{money(account.outstandingMinor / 100)}</td><td>{money(account.creditMinor / 100)}</td><td className={account.netBalanceMinor < 0 ? 'finance-credit' : ''}>{account.netBalanceMinor < 0 ? 'Credit ' : 'Due '}{money(Math.abs(account.netBalanceMinor) / 100)}</td></tr>)}</tbody></table></div>
    <h3 className="finance-section">Invoices</h3>
    <div className="card table-card"><table><thead><tr><th>Invoice</th><th>Member / description</th><th>Issued</th><th>Amount</th><th>Paid</th><th>Outstanding</th><th>Status</th><th>Action</th></tr></thead><tbody>{native.invoices.map(invoice => <tr key={invoice.id}>
      <td className="finance-number">{invoice.number}</td><td>{invoice.memberName}<small className="foundation-id">{invoice.description}</small></td><td>{displayDate(invoice.issuedOn)}</td><td>{money(invoice.amountMinor / 100)}</td><td>{money(invoice.paidMinor / 100)}</td><td>{money(invoice.outstandingMinor / 100)}</td><td><span className={invoice.status === 'Paid' ? 'tag green' : 'tag amber'}>{invoice.status}</span></td><td>{invoice.memberId && invoice.outstandingMinor > 0 && <button className="primary compact" onClick={() => open('receive', invoice.memberId!, invoice.id)}>Receive</button>}</td>
    </tr>)}</tbody></table>{!native.invoices.length && <p className="foundation-empty">No invoices saved.</p>}</div>
    <h3 className="finance-section">Received payments and reversals</h3>
    <div className="card table-card"><table><thead><tr><th>Receipt</th><th>Member</th><th>Date / method</th><th>Cash effect</th><th>Allocated</th><th>Credit</th><th>Status</th><th>Actions</th></tr></thead><tbody>{native.payments.map(payment => <tr key={payment.id}>
      <td className="finance-number">{payment.receiptNumber}</td><td>{payment.memberName}</td><td>{displayDate(payment.businessOn)}<small className="foundation-id">{payment.method}</small></td><td>{money(payment.netAmountMinor / 100)}</td><td>{money(payment.allocatedMinor / 100)}</td><td>{money(payment.unallocatedMinor / 100)}</td><td><span className={payment.status === 'Reversed' || payment.status === 'Reversal' ? 'tag red' : 'tag green'}>{payment.status}</span>{payment.reversalReason && <small className="foundation-id">{payment.reversalReason}</small>}</td>
      <td><div className="finance-actions"><button className="link" disabled={receiptBusy} onClick={() => void showReceipt(payment.id)}>Receipt</button>{payment.unallocatedMinor > 0 && <button className="link" onClick={() => open('allocate', payment.memberId, '', payment.id)}>Allocate</button>}{!payment.reversesId && payment.status !== 'Reversed' && <button className="link" onClick={() => open('reverse', payment.memberId, '', payment.id)}>Reverse</button>}</div></td>
    </tr>)}</tbody></table>{!native.payments.length && <p className="foundation-empty">No payments recorded.</p>}</div>
    <h3 className="finance-section">Allocation history</h3>
    <div className="card table-card"><table><thead><tr><th>Receipt</th><th>Invoice</th><th>Amount</th><th>Status</th></tr></thead><tbody>{native.allocations.map(allocation => <tr key={allocation.id}><td className="finance-number">{native.payments.find(payment => payment.id === allocation.paymentId)?.receiptNumber ?? allocation.paymentId}</td><td className="finance-number">{native.invoices.find(invoice => invoice.id === allocation.invoiceId)?.number ?? allocation.invoiceId}</td><td>{money(allocation.amountMinor / 100)}</td><td>{allocation.reversedBy ? 'Released by payment reversal' : 'Applied'}</td></tr>)}</tbody></table>{!native.allocations.length && <p className="foundation-empty">No allocations saved.</p>}</div>
    </>}
    {workflow && <Modal title={titles[workflow]} onClose={() => { if (!busy) { setWorkflow(null); onClose?.() } }}><form className="modal-form" onSubmit={event => void submit(event)}><fieldset className="foundation-fields" disabled={busy}>
      {workflow !== 'reverse' && <label className={workflow === 'receive' ? 'payment-member-field' : undefined}><span>Member</span><select value={form.memberId} onChange={event => {
        const memberId = event.target.value
        const due = workflow === 'receive' ? outstandingInvoices(capture, memberId) : []
        setInvoiceIds(due.map(i=>i.id))
        setForm({ ...form, memberId, invoiceId: '', periodId: '', paymentId: capture.payments.find(payment => payment.memberId === memberId && payment.unallocatedMinor > 0)?.id ?? '', amount: due.length ? amountText(due.reduce((sum,i)=>sum+i.outstandingMinor,0)) : '', description: '', startsOn: workflow === 'training' ? nextTrainingStart(capture, memberId) : form.startsOn })
      }}>{capture.members.filter(member => member.active || (workflow !== 'renew' && member.id === form.memberId)).map(member => <option key={member.id} value={member.id}>{member.name}{!member.active ? ' (Archived — financial history)' : ''} — {member.id}</option>)}</select></label>}
      {workflow === 'invoice' && <><label><span>Membership period (optional)</span><select value={form.periodId} onChange={event => { const period = periods.find(period => period.id === event.target.value); setForm({ ...form, periodId: period?.id ?? '', amount: period ? amountText(period.priceMinor) : '', description: period ? 'Membership: ' + period.planName : '' }) }}><option value="">General member invoice</option>{periods.map(period => <option key={period.id} value={period.id}>{period.planName} · {period.startsOn} – {period.endsOn}</option>)}</select></label>
        <label><span>Description</span><input required maxLength={254} value={form.description} onChange={event => setForm({ ...form, description: event.target.value })}/></label></>}
      {workflow === 'allocate' && <label><span>Received payment</span><select required value={form.paymentId} onChange={event => setForm({ ...form, paymentId: event.target.value, amount: '' })}><option value="">Select saved credit</option>{payments.map(payment => <option key={payment.id} value={payment.id}>{payment.receiptNumber} · credit {money(payment.unallocatedMinor / 100)}</option>)}</select></label>}
      {workflow === 'receive' && <div className="invoice-payment-section" role="group" aria-label="Outstanding invoices"><span className="invoice-payment-heading">Outstanding invoices</span><div className="invoice-payment-list">{invoices.map(i => <label className="invoice-payment-choice" key={i.id}><input type="checkbox" checked={invoiceIds.includes(i.id)} onChange={e => {
        const selected = e.target.checked ? [...invoiceIds, i.id] : invoiceIds.filter(id=>id!==i.id)
        const ordered = invoices.filter(invoice=>selected.includes(invoice.id))
        setInvoiceIds(ordered.map(i=>i.id)); setForm({...form, amount: ordered.length ? amountText(ordered.reduce((sum,i)=>sum+i.outstandingMinor,0)) : ''})
      }}/><span>{i.description}</span><b>Outstanding {money(i.outstandingMinor / 100)}</b></label>)}</div>{!invoices.length && <p className="form-note">No dues. Payment is saved as credit.</p>}</div>}
      {workflow === 'allocate' && <label><span>Invoice</span><select required value={form.invoiceId} onChange={event => setForm({ ...form, invoiceId: event.target.value })}><option value="">Select outstanding invoice</option>{invoices.map(invoice => <option key={invoice.id} value={invoice.id}>{invoice.description} · outstanding {money(invoice.outstandingMinor / 100)}</option>)}</select></label>}
      {workflow === 'training' && <><p>{assignedTrainer ? `${assignedTrainer.name} · ${money(assignedTrainer.trainingFeeMinor / 100)} per month` : 'Select a member with an assigned personal trainer.'}</p><label><span>Training month start</span><input required type="date" min="1900-01-01" max="2200-12-31" value={form.startsOn} onChange={e=>setForm({...form, startsOn:e.target.value})}/></label></>}
      {['invoice','receive','allocate'].includes(workflow) && <label><span>Amount (Rs.)</span><input required type="number" min="0.01" step="0.01" max={workflow === 'allocate' ? Math.min(credit, invoice?.outstandingMinor ?? 0) / 100 : 1_000_000_000} readOnly={workflow === 'invoice' && Boolean(form.periodId)} value={form.amount} onChange={event => setForm({ ...form, amount: event.target.value })}/></label>}
      {workflow === 'receive' && <><label><span>Payment method</span><select value={form.method} onChange={event => setForm({ ...form, method: event.target.value as typeof form.method })}><option>Cash</option><option>Card</option><option>Transfer</option></select></label></>}
      {workflow === 'renew' && <><label><span>Plan</span><select required value={form.planId} onChange={event => setForm({ ...form, planId: event.target.value })}>{capture.plans.filter(plan => plan.active && plan.priceMinor > 0).map(plan => <option key={plan.id} value={plan.id}>{plan.name} · {money(plan.priceMinor / 100)}</option>)}</select></label>
        <p>Invoice amount: {money((plan?.priceMinor ?? 0) / 100)}{latest && <><br/>Latest last valid day: {displayDate(latest.endsOn)}</>}</p>
        <label><span>Start date</span><input required type="date" min="1900-01-01" max="2200-12-31" value={form.startsOn} onChange={event => setForm({ ...form, startsOn: event.target.value })}/></label>
        <label><span>Last valid day (inclusive)</span><input required type="date" min={form.startsOn || '1900-01-01'} max="2200-12-31" value={form.endsOn} onChange={event => setForm({ ...form, endsOn: event.target.value })}/></label>
        </>}
      {workflow === 'reverse' && <><p className="finance-number">{original?.receiptNumber}</p><p>{original?.memberName} · {money((original?.amountMinor ?? 0) / 100)} · {original?.method}</p>
        <p className="form-note">Reopens unpaid invoices. Bank or card refunds must be handled separately.</p>
        <label><span>Reversal reason</span><input required maxLength={254} value={form.reason} onChange={event => setForm({ ...form, reason: event.target.value })}/></label></>}
      <button className="primary" type="submit">{busy ? 'Saving…' : actions[workflow]}</button>
    </fieldset>{error && <div role="alert" className="login-error">{error}</div>}</form></Modal>}
    {receiptOpen && <Modal title="Saved receipt preview" onClose={() => { if (!receiptBusy) setReceiptOpen(false) }}>
      {receiptBusy && <p>Reading saved receipt…</p>}{receiptError && <div role="alert" className="login-error">{receiptError}</div>}
      {receipt && <><div className="finance-actions"><button className="primary" onClick={print}>Print / system preview</button></div><ReceiptView document={receipt}/>{createPortal(<div className="finance-print-only"><ReceiptView document={receipt}/></div>, document.body)}</>}
    </Modal>}
  </>
}
