import { useState } from 'react'
import { PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import { errorText } from '../desktop/DesktopGymProvider'
export function AttendancePage() {
  const { data, recordAttendance, desktop } = useGym()
  const [card, setCard] = useState('')
  const [manual, setManual] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [requestId, setRequest] = useState(() => (desktop ? crypto.randomUUID() : ''))
  const today = desktop?.snapshot?.today ?? new Date().toISOString().slice(0, 10)
  const activity = desktop ? data.attendance.filter(row => row.date === today) : data.attendance
  async function record(value: string, source: 'NFC' | 'Manual') {
    if (busy) return
    setBusy(true); setError('')
    try {
      await recordAttendance(desktop ? value : value.trim(), source, requestId)
      setCard(''); setManual(''); setRequest((desktop ? crypto.randomUUID() : ''))
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <><PageHeader title="NFC Attendance" subtitle="Record member check-ins and check-outs using NFC or manual selection"/>
    <div className="attendance-layout"><section className="card scanner">
      <span className="reader-wave">)))</span><h2>Tap member card</h2><p>Keep this field focused while scanning</p>
      <div className="reader-device"><i/><i/><i/><span>NFC</span></div>
      <form className="scan-form" onSubmit={event => { event.preventDefault(); void record(card, 'NFC') }}>
        <input autoFocus disabled={busy} value={card} onChange={event => { setCard(event.target.value); setRequest((desktop ? crypto.randomUUID() : '')) }} placeholder="Scan or type NFC ID"/>
        <button className="primary" disabled={busy || !card.trim()}>{busy ? 'Saving…' : 'Record scan'}</button>
      </form>
      <select disabled={busy} value={manual} onChange={event => { setManual(event.target.value); if (event.target.value) void record(event.target.value, 'Manual') }}>
        <option value="" disabled>Manual member check-in</option>{data.members.map(member => <option key={member.id} value={member.id}>{member.name} — {member.id}</option>)}
      </select>
      {error && <div role="alert" className="login-error">{error}</div>}
    </section><section className="card activity-panel"><div className="card-head"><div><h2>Today’s activity</h2><p>{activity.length} attendance records</p></div><span className="live">{desktop ? 'LOCAL' : 'LIVE'}</span></div>
      {activity.slice(0, 12).map(row => <div className="list-row" key={row.id}><span className="avatar tiny">{row.name.split(' ').map(part => part[0]).join('').slice(0, 2)}</span><div><b>{row.name}</b><small>{row.memberId} · {row.source} · {row.syncState}</small></div><time>{row.time}</time><span className={row.type === 'Check-in' ? 'tag green' : 'tag red'}>{row.type}</span></div>)}
    </section></div>
  </>
}
