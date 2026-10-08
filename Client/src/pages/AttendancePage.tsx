import { useRef, useState } from 'react'
import { PageHeader } from '../components/ui/Modal'
import { Icon } from '../components/ui/Icon'
import { useGym } from '../context/GymContext'
import { errorText } from '../desktop/DesktopGymProvider'
import { colomboToday } from '../utils/membership'
import { attendanceTotals } from '../utils/attendance'

export function AttendancePage() {
  const { data, recordAttendance, desktop } = useGym()
  const [tab, setTab] = useState<'Members' | 'Staff'>('Members')
  const [card, setCard] = useState('')
  const [manual, setManual] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [requestId, setRequest] = useState(() => desktop ? crypto.randomUUID() : '')
  const input = useRef<HTMLInputElement>(null)
  const today = desktop?.snapshot?.today ?? colomboToday()
  const staff = desktop?.snapshot?.trainers.filter(row => row.active) ?? []
  const staffActivity = (desktop?.snapshot?.staffAttendance ?? []).map(row => ({
    ...row, memberId: row.staffId, date: row.businessOn,
    time: new Intl.DateTimeFormat('en-GB', {timeZone:'Asia/Colombo',hour:'2-digit',minute:'2-digit'}).format(new Date(row.occurredAt)),
    gender: null,
  }))
  const rows = tab === 'Staff' ? staffActivity : data.attendance
  const activity = rows.filter(row => row.date === today)
  const totals = attendanceTotals(rows.map(row => ({...row,personId:row.memberId})), today)
  async function record(value: string, source: 'NFC' | 'Manual') {
    if (busy) return
    setBusy(true); setError('')
    try {
      if (desktop && source === 'NFC') {
        const result = await desktop.recordNfcAttendance({requestId,memberOrCard:value,source})
        setTab(result.entity === 'Staff' ? 'Staff' : 'Members')
      } else if (desktop && tab === 'Staff') {
        await desktop.recordStaffAttendance({requestId,staffOrCard:value,source})
      } else {
        await recordAttendance(desktop ? value : value.trim(), source, requestId)
      }
      setCard(''); setManual(''); setRequest(desktop ? crypto.randomUUID() : '')
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false); requestAnimationFrame(() => input.current?.focus()) }
  }
  return <><PageHeader title="NFC Attendance" subtitle="Member and staff check-ins and check-outs · NFC or manual"/>
    {desktop && <div className="attendance-tabs" role="group" aria-label="Attendance category">{(['Members','Staff'] as const).map(name => <button key={name} className={tab === name ? 'primary' : 'secondary'} disabled={busy} aria-pressed={tab === name} onClick={() => {setTab(name);setManual('');setError('')}}>{name}</button>)}</div>}
    <div className="attendance-layout attendance-workspace"><section className="card scanner">
      <span className="reader-eyebrow">NFC ATTENDANCE</span><h2>Tap your card</h2><p>One tap for members and staff</p>
      <div className="reader-stage" aria-hidden="true"><div className="reader-device"><div className="reader-chip"/><Icon name="signal" size={54}/><span>NFC</span><small>ARMSTRONG FITNESS</small></div></div>
      <p className="reader-instruction">Hold your card near the reader to check in or out.</p>
      <form className="scan-form" onSubmit={event => {event.preventDefault();void record(card,'NFC')}}>
        <input ref={input} autoFocus aria-label="NFC card" disabled={busy} maxLength={128} value={card} onChange={event => {setCard(event.target.value);setRequest(desktop ? crypto.randomUUID() : '')}} placeholder="Scan or type NFC ID"/>
        <button className="primary" disabled={busy || !card.trim()}>{busy ? 'Saving…' : 'Record scan'}</button>
      </form>
      <div className="manual-attendance"><span>Or select a {tab === 'Staff' ? 'staff member' : 'member'} manually</span><select aria-label={`Manual ${tab === 'Staff' ? 'staff' : 'member'} check-in`} disabled={busy} value={manual} onChange={event => {setManual(event.target.value);if(event.target.value)void record(event.target.value,'Manual')}}>
        <option value="" disabled>Manual {tab === 'Staff' ? 'staff' : 'member'} check-in</option>{(tab === 'Staff' ? staff : data.members).map(person => <option key={person.id} value={person.id}>{person.name} — {person.id}</option>)}
      </select></div>
      {error && <div role="alert" className="login-error">{error}</div>}
    </section><section className="card activity-panel"><div className="card-head"><div><h2>Today’s {tab === 'Staff' ? 'staff' : 'member'} activity</h2><p>{totals.total} people attended · {activity.length} events</p></div></div>
      {tab === 'Members' && <div className="attendance-counts"><span>Male <b>{totals.male}</b></span><span>Female <b>{totals.female}</b></span>{totals.unspecified > 0 && <span>Unspecified <b>{totals.unspecified}</b></span>}</div>}
      <div className="attendance-activity">{activity.map(row => <div className="list-row" key={row.id}><span className="avatar tiny">{row.name.split(' ').map(part => part[0]).join('').slice(0,2)}</span><div><b>{row.name}</b><small>{row.memberId} · {row.source}</small></div><time>{row.time}</time><span className={row.type === 'Check-in' ? 'tag green' : 'tag red'}>{row.type}</span></div>)}</div>
      {!activity.length && <p className="dashboard-empty">No {tab === 'Staff' ? 'staff' : 'member'} attendance recorded today.</p>}
    </section></div>
  </>
}
