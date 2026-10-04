import { useState } from 'react'
import { Modal, PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { MembershipPlan } from '../types/domain'
import { minorUnits } from '../desktop/api'
import { money } from '../utils/format'

export function MembershipsPage() {
  const { data, updatePlan, desktop } = useGym()
  const [editing, setEditing] = useState<MembershipPlan | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  function open(plan: MembershipPlan) { setError(''); setEditing({ ...plan }) }
  async function save(event: React.FormEvent) {
    event.preventDefault()
    if (!editing || busy) return
    setBusy(true); setError('')
    try {
      if (desktop && !editing.id) await desktop.savePlan({ name: editing.name, durationMonths: editing.durationMonths, priceMinor: minorUnits(editing.price), active: editing.status === 'Active' })
      else await updatePlan(editing)
      setEditing(null)
    } catch (error) { setError(error instanceof Error ? error.message : String(error)) }
    finally { setBusy(false) }
  }
  return <><PageHeader title="Memberships" subtitle="Packages, renewal pricing and member totals" action={desktop ? 'Add plan' : undefined} onAction={() => open({ id: '', name: '', durationMonths: 1, price: 0, activeMembers: 0, status: 'Active' })}/>
    <div className="card table-card"><table><thead><tr><th>Package</th><th>Duration</th><th>Price</th><th>Active members</th><th>Status</th><th></th></tr></thead><tbody>{data.plans.map(plan => <tr key={plan.id}><td><b>{plan.name}</b></td><td>{plan.durationMonths} month{plan.durationMonths > 1 ? 's' : ''}</td><td>{money(plan.price)}</td><td>{plan.activeMembers}</td><td><span className={plan.status === 'Active' ? 'tag green' : 'tag amber'}>{plan.status}</span></td><td><button className="secondary compact" onClick={() => open(plan)}>Edit</button></td></tr>)}</tbody></table>{!data.plans.length && <p className="foundation-empty">No plans recorded.</p>}</div>
    {editing && <Modal title={editing.id ? 'Edit membership package' : 'Add membership package'} onClose={() => { if (!busy) setEditing(null) }}><form className="modal-form" onSubmit={event => void save(event)}><fieldset className="foundation-fields" disabled={busy}>
      <label><span>Package name</span><input required maxLength={40} value={editing.name} onChange={event => setEditing({ ...editing, name: event.target.value })}/></label>
      <label><span>Duration in months</span><input required type="number" min="1" max="60" value={editing.durationMonths} onChange={event => setEditing({ ...editing, durationMonths: Number(event.target.value) })}/></label>
      <label><span>Price (Rs.)</span><input required type="number" min="0" step={desktop ? '0.01' : '100'} value={editing.price} onChange={event => setEditing({ ...editing, price: Number(event.target.value) })}/></label>
      <label><span>Status</span><select value={editing.status} onChange={event => setEditing({ ...editing, status: event.target.value as MembershipPlan['status'] })}><option>Active</option><option>Inactive</option></select></label>
      <label><span>Active members</span><input value={editing.activeMembers} disabled title="Calculated from member records"/></label>
      <p className="form-note">{desktop ? 'Counts come from current membership dates. History retains the original plan name and price.' : 'Active member count is calculated from member records and cannot be edited manually.'}</p>
      {error && <div role="alert" className="login-error">{error}</div>}
      <button className="primary" type="submit">{busy ? 'Saving…' : 'Save package changes'}</button>
    </fieldset></form></Modal>}
  </>
}
