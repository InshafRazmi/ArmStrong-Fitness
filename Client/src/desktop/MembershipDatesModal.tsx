import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { Member } from '../types/domain'
import { displayDate, money } from '../utils/format'
import { errorText } from './DesktopGymProvider'

export function MembershipDatesModal({ member, onClose }: { member: Member; onClose: () => void }) {
  const { desktop } = useGym()
  const native = desktop!.snapshot!
  const plans = native.plans.filter(plan => plan.active)
  const [form, setForm] = useState({ memberId: member.id, planId: plans[0]?.id ?? '', startsOn: native.today, endsOn: '' })
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)
  const periods = native.periods.filter(period => period.memberId === member.id)
  async function save(event: React.FormEvent) {
    event.preventDefault()
    if (busy || member.active === false) return
    setBusy(true); setError('')
    try { await desktop!.addPeriod(form); onClose() }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <Modal title={`Membership dates — ${member.name}`} onClose={() => { if (!busy) onClose() }}>
    <form className="modal-form" onSubmit={event => void save(event)}><fieldset className="foundation-fields" disabled={busy || !plans.length || member.active === false}>
      <label><span>Plan</span><select value={form.planId} onChange={event => setForm({ ...form, planId: event.target.value })}>{plans.map(plan => <option key={plan.id} value={plan.id}>{plan.name} · {money(plan.priceMinor / 100)}</option>)}</select></label>
      <label><span>Start date</span><input type="date" required min="1900-01-01" max="2200-12-31" value={form.startsOn} onChange={event => setForm({ ...form, startsOn: event.target.value })}/></label>
      <label><span>Last valid day (inclusive)</span><input type="date" required min={form.startsOn} max="2200-12-31" value={form.endsOn} onChange={event => setForm({ ...form, endsOn: event.target.value })}/></label>
      <p className="form-note">Choose both dates explicitly. Overlaps are rejected. This records membership dates only; it does not collect payment or create a receipt. Corrections are not yet available.</p>
      <button className="primary" type="submit">{busy ? 'Saving…' : 'Save membership dates'}</button>
    </fieldset>{error && <div role="alert" className="login-error">{error}</div>}</form>
    {member.active === false && <p className="form-note">Archived member: membership history is read-only. Existing dates and debts are retained.</p>}
    {!plans.length && <p>Create an active plan in Memberships before adding dates.</p>}
    <div className="desktop-history"><h3>Membership history</h3>{periods.length ? periods.map(period => <div className="foundation-period" key={period.id}><b>{period.planName}</b> <span className={period.status === 'Active' ? 'tag green' : 'tag amber'}>{period.status}</span><small className="foundation-id">{displayDate(period.startsOn)} – {displayDate(period.endsOn)} · {money(period.priceMinor / 100)}</small></div>) : <p>No membership recorded.</p>}</div>
  </Modal>
}
