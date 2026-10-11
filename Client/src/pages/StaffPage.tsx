import { useMemo, useRef, useState } from 'react'
import { Modal, PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import { displayDate, money } from '../utils/format'
import { errorText } from '../desktop/DesktopGymProvider'
import { minorUnits, type Snapshot, type Trainer } from '../desktop/api'
import { StaffRemovalDialog } from '../desktop/StaffRemovalDialog'

const blank = { name: '', phone: '', nic: '', nfcId: '', salary: '', fee: '', active: true }
export function StaffPage({ navigate }: { navigate: (page: import('../types/domain').Page) => void }) {
  const { desktop } = useGym()
  const native = desktop?.snapshot
  const [search, setSearch] = useState('')
  const [showInactive, setShowInactive] = useState(false)
  const [showDeleted, setShowDeleted] = useState(false)
  const [removing, setRemoving] = useState<Trainer | null>(null)
  const [open, setOpen] = useState(false)
  const [editing, setEditing] = useState<Trainer | null>(null)
  const [form, setForm] = useState(blank)
  const nicOwner = native?.trainers.find(t => !t.deletedAt && t.id !== editing?.id && t.nic === form.nic.trim().toUpperCase())
  const cardInput = useRef<HTMLInputElement>(null)
  const [scanning, setScanning] = useState(false)
  const [requestId, setRequestId] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [assigned, setAssigned] = useState<Trainer | null>(null)
  const [payout, setPayout] = useState<{ trainer: Trainer; base: Snapshot; requestId: string } | null>(null)
  const [salaryMonth, setSalaryMonth] = useState('')
  const [includeSalary, setIncludeSalary] = useState(true)
  const [method, setMethod] = useState<'Cash' | 'Card' | 'Bank'>('Cash')
  const staff = useMemo(() => (native?.trainers ?? []).filter(t => (showDeleted ? !!t.deletedAt : !t.deletedAt && (showInactive ? !t.active : t.active)) && `${t.name} ${t.phone} ${t.nic}`.toLowerCase().includes(search.toLowerCase())), [native, search, showInactive, showDeleted])
  if (!desktop || !native) return <><PageHeader title="Staff" subtitle="Personal trainers and monthly staff payments"/><p className="foundation-empty">Open the installed desktop application to manage staff.</p></>
  function edit(trainer: Trainer | null) {
    setEditing(trainer); setError(''); setRequestId(crypto.randomUUID())
    setScanning(false)
    setForm(trainer ? { name: trainer.name, phone: trainer.phone, nic: trainer.nic, nfcId: trainer.nfcId ?? '', salary: (trainer.salaryMinor / 100).toFixed(2), fee: (trainer.trainingFeeMinor / 100).toFixed(2), active: trainer.active } : blank)
    setOpen(true)
  }
  async function save(event: React.FormEvent) {
    event.preventDefault(); if (busy) return
    setBusy(true); setError('')
    try {
      await desktop!.saveTrainer({ requestId, id: editing?.id, version: editing?.version, name: form.name, phone: form.phone, nic: form.nic, nfcId: form.nfcId, salaryMinor: minorUnits(Number(form.salary)), trainingFeeMinor: minorUnits(Number(form.fee)), active: form.active })
      setOpen(false)
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  function pay(trainer: Trainer) {
    setError(''); setPayout({ trainer, base: native!, requestId: crypto.randomUUID() }); setSalaryMonth(native!.today.slice(0, 7)); setIncludeSalary(trainer.active); setMethod('Cash')
  }
  const salaryPaid = Boolean(payout?.base.staffPayouts.some(p => p.trainerId === payout.trainer.id && p.salaryMonth === salaryMonth && p.active && p.salaryMinor > 0))
  const salary = payout && includeSalary && !salaryPaid ? payout.trainer.salaryMinor : 0
  const training = payout?.trainer.unpaidTrainingMinor ?? 0
  async function savePayout(event: React.FormEvent) {
    event.preventDefault(); if (!payout || busy) return
    setBusy(true); setError('')
    try {
      await desktop!.payStaff({ requestId: payout.requestId, trainerId: payout.trainer.id, trainerVersion: payout.trainer.version, salaryMonth, includeSalary, expectedSalaryMinor: salary, expectedTrainingMinor: training, allocationIds: payout.base.staffTrainingAllocations.filter(a => a.trainerId === payout.trainer.id).map(a => a.id), method })
      setPayout(null)
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <><PageHeader title="Staff" subtitle="Personal trainers, monthly rates and staff payments" action="Add staff" onAction={() => edit(null)}/>
    <div className="staff-summary"><div className="card"><small>Active staff</small><strong>{native.trainers.filter(t => t.active).length}</strong></div><div className="card"><small>Fixed monthly salaries</small><strong>{money(native.trainers.filter(t => t.active).reduce((sum, t) => sum + t.salaryMinor, 0) / 100)}</strong></div><div className="card"><small>Collected training fees awaiting payout</small><strong>{money(native.trainers.reduce((sum, t) => sum + t.unpaidTrainingMinor, 0) / 100)}</strong></div></div>
    <div className="toolbar"><button className="secondary" onClick={() => navigate('NFC Attendance')}>Record staff attendance</button><button className="secondary" onClick={() => {setShowInactive(!showInactive);setShowDeleted(false)}}>{showInactive ? 'Show active staff' : 'Show inactive staff'}</button><button className="secondary" onClick={() => {setShowDeleted(!showDeleted);setShowInactive(false)}}>{showDeleted ? 'Show active staff' : 'Show deleted staff'}</button><input aria-label="Search staff" placeholder="Search name, mobile or NIC…" value={search} onChange={event => setSearch(event.target.value)}/></div>
    <div className="card table-card"><table><thead><tr><th>Staff</th><th>Mobile</th><th>NIC / NFC card</th><th>Monthly salary</th><th>Training / member / month</th><th>Assigned members</th><th>Unpaid training earnings</th><th>Actions</th></tr></thead><tbody>{staff.map(t => <tr key={t.id}><td><b>{t.name}</b><small className="foundation-id">{t.deletedAt ? 'Deleted · history retained' : t.active ? 'Active' : 'Inactive'}</small></td><td>{t.phone}</td><td>••••{t.nic.slice(-4)}<small className="foundation-id">{t.nfcId || "No NFC card"}</small></td><td>{money(t.salaryMinor / 100)}</td><td>{money(t.trainingFeeMinor / 100)}</td><td><button className="link" onClick={() => setAssigned(t)}>{t.assignedMembers} members</button></td><td>{money(t.unpaidTrainingMinor / 100)}</td><td><div className="finance-actions">{!t.deletedAt && <button className="secondary compact" onClick={() => edit(t)}>Edit</button>}<button className="primary compact" onClick={() => pay(t)}>Pay salary</button>{!t.deletedAt && <button className="secondary compact danger-action" onClick={() => setRemoving(t)}>Delete permanently</button>}</div></td></tr>)}</tbody></table>{!staff.length && <p className="foundation-empty">{search ? 'No matching staff.' : showDeleted ? 'No deleted staff.' : showInactive ? 'No inactive staff.' : 'Add your first staff member to assign personal training.'}</p>}</div>
    <h3 className="finance-section">Staff payment history</h3>
    <div className="card table-card"><table><thead><tr><th>Staff</th><th>Salary month</th><th>Salary</th><th>Training fees</th><th>Total paid</th><th>Date / method</th><th>Status</th></tr></thead><tbody>{native.staffPayouts.map(p => <tr key={p.id}><td>{p.trainerName}</td><td>{p.salaryMonth}</td><td>{money(p.salaryMinor / 100)}</td><td>{money(p.trainingMinor / 100)}</td><td>{money(p.amountMinor / 100)}</td><td>{displayDate(p.businessOn)} · {p.method}</td><td><span className={p.active ? 'tag green' : 'tag red'}>{p.active ? 'Paid' : 'Voided'}</span></td></tr>)}</tbody></table>{!native.staffPayouts.length && <p className="foundation-empty">No staff payments recorded.</p>}</div><button className="link staff-expenses-link" onClick={() => navigate('Expenses')}>View salary expenses</button>
    {open && <Modal title={editing ? 'Edit staff' : 'Add staff'} onClose={() => { if (!busy) setOpen(false) }}><form className="modal-form" onSubmit={event => void save(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Full name</span><input required maxLength={120} autoComplete="name" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}/></label>
      <label><span>Mobile number</span><input required type="tel" maxLength={40} autoComplete="tel" placeholder="077 123 4567" value={form.phone} onChange={e => setForm({ ...form, phone: e.target.value })}/></label>
      <label><span>NIC number</span><input required maxLength={24} autoCapitalize="characters" value={form.nic} onChange={e => setForm({ ...form, nic: e.target.value.toUpperCase() })}/></label>
      {nicOwner && <div role="alert" className="foundation-warning">This NIC belongs to {nicOwner.name} ({nicOwner.active ? 'Active' : 'Inactive'}). <button type="button" className="link" onClick={() => edit(nicOwner)}>Edit existing staff</button></div>}
      <label className="nfc-form-field"><span>NFC attendance card</span><div className={`nfc-input-row${scanning ? ' scanning' : ''}`}><input ref={cardInput} maxLength={128} value={form.nfcId} placeholder={scanning ? 'Tap the staff card now…' : 'Optional NFC card ID'} onChange={event => setForm({...form,nfcId:event.target.value.toUpperCase()})} onKeyDown={event => {if(event.key === 'Enter'){event.preventDefault();setScanning(false)}}}/><button type="button" className="secondary scan-button" onClick={() => {setScanning(true);setForm({...form,nfcId:''});requestAnimationFrame(() => cardInput.current?.focus())}}>Scan card</button></div><small className="scan-message">{scanning ? 'Waiting for card; press Enter when captured.' : 'Use a separate card for each member or staff.'}</small></label>
      <label><span>Fixed monthly salary (Rs.)</span><input required type="number" min="0" max="1000000000" step="0.01" value={form.salary} onChange={e => setForm({ ...form, salary: e.target.value })}/></label>
      <label><span>Personal training per member per month (Rs.)</span><input required type="number" min="0" max="1000000000" step="0.01" value={form.fee} onChange={e => setForm({ ...form, fee: e.target.value })}/></label>
      <label><span>Status</span><select value={form.active ? 'active' : 'inactive'} onChange={e => setForm({ ...form, active: e.target.value === 'active' })}><option value="active">Active</option><option value="inactive">Inactive</option></select></label>
      {error && <div role="alert" className="login-error">{error}</div>}<button className="primary">{busy ? 'Saving…' : 'Save staff'}</button>
    </fieldset></form></Modal>}
    {assigned && <Modal title={`Members training with ${assigned.name}`} onClose={() => setAssigned(null)}><div className="staff-member-list">{native.memberTrainers.filter(a => a.trainerId === assigned.id).map(a => native.members.find(m => m.id === a.memberId)).filter(m => m?.active).map(m => <div className="list-row" key={m!.id}><b>{m!.name}</b><span>{m!.phone}</span></div>)}</div>{!assigned.assignedMembers && <p>No active members assigned.</p>}</Modal>}
    {payout && <Modal title={`Pay ${payout.trainer.name}`} onClose={() => { if (!busy) setPayout(null) }}><form className="modal-form" onSubmit={event => void savePayout(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Salary month</span><input required type="month" min="1900-01" max={native.today.slice(0, 7)} value={salaryMonth} onChange={e => setSalaryMonth(e.target.value)}/></label>
      <label className="staff-salary-choice"><input type="checkbox" checked={includeSalary} disabled={salaryPaid} onChange={e => setIncludeSalary(e.target.checked)}/><span>{salaryPaid ? 'Fixed salary already paid for this month' : 'Include fixed monthly salary'}</span></label>
      <div className="staff-payment-breakdown"><div><span>Fixed salary</span><b>{money(salary / 100)}</b></div><div><span>Collected training fees awaiting payout</span><b>{money(training / 100)}</b></div><div className="staff-payment-total"><span>Total staff payment</span><strong>{money((salary + training) / 100)}</strong></div></div>
      <label><span>Payment method</span><select value={method} onChange={e => setMethod(e.target.value as typeof method)}><option>Cash</option><option>Card</option><option>Bank</option></select></label>
      <p className="form-note">Record after paying the staff member.</p>
      {error && <div role="alert" className="login-error">{error}</div>}<button className="primary" disabled={salary + training <= 0}>{busy ? 'Saving…' : 'Record staff payment'}</button>
    </fieldset></form></Modal>}
    {removing && <StaffRemovalDialog staff={removing} onClose={() => setRemoving(null)}/>}
  </>
}
