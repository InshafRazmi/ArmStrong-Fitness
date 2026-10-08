import { Icon } from '../components/ui/Icon'
import { useGym } from '../context/GymContext'
import { money } from '../utils/format'
import { colomboToday, membershipDaysRemaining } from '../utils/membership'
import type { Page } from '../types/domain'
import { attendanceTotals } from '../utils/attendance'

function Kpi({label,value,note,icon}:{label:string;value:string;note:string;icon:string}) {
  return <div className="card kpi"><div className="icon-box"><Icon name={icon}/></div><div><span>{label}</span><strong>{value}</strong><small>{note}</small></div></div>
}
export function DashboardPage({navigate}:{navigate:(p:Page)=>void}) {
  const {data,desktop} = useGym()
  const today = desktop?.snapshot?.today ?? colomboToday()
  const recentAttendance = data.attendance.filter(row => row.date === today && !row.voidsId).slice(0, 4)
  const memberCounts = attendanceTotals(data.attendance.map(row=>({...row,personId:row.memberId})),today)
  const staffRows = (desktop?.snapshot?.staffAttendance ?? []).filter(row => row.businessOn === today)
  const staffCount = attendanceTotals(staffRows.map(row=>({...row,personId:row.staffId,date:row.businessOn})),today).total
  const expiringMembers = data.members.filter(row => row.status === 'Expiring').sort((a, b) => a.expiry.localeCompare(b.expiry))
  const income = desktop?.snapshot
    ? (desktop.snapshot.payments.filter(row=>row.businessOn===today).reduce((sum,row)=>sum+row.netAmountMinor,0)
      +desktop.snapshot.sales.filter(row=>row.businessOn===today).reduce((sum,row)=>sum+row.totalMinor,0))/100
    : data.payments.filter(row=>row.date===today).reduce((sum,row)=>sum+row.amount,0)
  const days = Array.from({length:28},(_,index)=>{
    const day = new Date(today+'T12:00:00Z')
    day.setUTCDate(day.getUTCDate()-27+index)
    return day.toISOString().slice(0,10)
  })
  const counts = days.map(day=>attendanceTotals(data.attendance.map(row=>({...row,personId:row.memberId})),day).total)
  const bars = desktop ? counts.map(count=>count/Math.max(1,...counts)*116) : [28,41,31,46,52,64,43,68,74,55,70,82,63,88,94,72,84,100,76,91,106,88,79,98,112,92,116,83]
  return <div className="dashboard-page"><div className="kpi-grid">
    <Kpi label="Total Members" value={String(data.members.length)} note="Stored locally" icon="users"/>
    <div className="card kpi attendance-kpi"><div className="icon-box"><Icon name="signal"/></div><div><span>Today’s Attendance</span><strong>{memberCounts.total}</strong><div className="kpi-attendance-counts"><span>Male <b>{memberCounts.male}</b></span><span>Female <b>{memberCounts.female}</b></span></div>{memberCounts.unspecified > 0 && <small>Unspecified {memberCounts.unspecified}</small>}</div></div>
    <Kpi label="Expiring Soon" value={String(expiringMembers.length)} note="Action required" icon="card"/>
    <Kpi label="Today’s Income" value={money(income)} note={desktop?'Payments + retail sales':'Payments recorded'} icon="money"/>
  </div><div className="dashboard-grid">
    <section className="card chart-card"><div className="card-head"><div><h2>Attendance trend</h2><p>Members per day · last 28 days</p></div></div><div className="bar-chart">{bars.map((height,index)=><i key={index} title={desktop?days[index]+': '+counts[index]+' members':undefined} style={{height:`${height / 116 * 100}%`}}/>)}</div><div className="axis">{desktop?<>{[0,7,14,21,27].map(index=><span key={index}>{days[index].slice(5)}</span>)}</>:<><span>Sep 1</span><span>Sep 8</span><span>Sep 15</span><span>Sep 22</span><span>Sep 30</span></>}</div></section>
    <section className="card nfc-card"><div className="card-head"><div><h2>{desktop?'Local storage':'Offline sync'}</h2><p>Local data protection</p></div><span className="live">{desktop?'SQLITE':'LIVE'}</span></div><div className="success-row"><div className="success-icon">✓</div><div><b>LOCAL DATABASE READY</b><span>{desktop?.snapshot?.pending??data.queue.length} changes waiting for server</span></div></div><div className="member-highlight"><span className="avatar">DB</span><div><h3>{desktop?'Local operations persisted':'Safe offline operation'}</h3><p>{desktop?(desktop.snapshot?.businessSync?(desktop.snapshot.businessSync.available?'Gym records synchronize with your verified account.':'Sign in online to synchronize gym records.'):(desktop.snapshot?.memberSync?.available?'Member sync connected; other modules remain local.':'Sign in online to synchronize members.')):'Auto-sync on reconnection'}</p></div></div><button className="primary wide" onClick={()=>navigate('Settings')}>Open sync settings</button></section>
    <section className="card list-card attendance-card"><div className="card-head"><div><h2>Member attendance</h2><p>{memberCounts.total} members today</p></div><button className="link" onClick={()=>navigate('NFC Attendance')}>View all</button></div><div className="attendance-counts"><span>Male <b>{memberCounts.male}</b></span><span>Female <b>{memberCounts.female}</b></span>{memberCounts.unspecified>0 && <span>Unspecified <b>{memberCounts.unspecified}</b></span>}</div>{recentAttendance.map(row=><div className="list-row" key={row.id}><div><b title={row.name}>{row.name}</b><small>{row.source} · {row.gender ?? 'Unspecified'}</small></div><time>{row.time}</time><span className={row.type==='Check-in'?'tag green':'tag red'}>{row.type}</span></div>)}{!recentAttendance.length && <p className="dashboard-empty">No attendance recorded yet.</p>}</section>
    <section className="card list-card staff-attendance-card"><div className="card-head"><div><h2>Staff attendance</h2><p>{staffCount} staff today</p></div><button className="link" onClick={()=>navigate('NFC Attendance')}>View all</button></div>{staffRows.slice(0,4).map(row=><div className="list-row" key={row.id}><div><b title={row.name}>{row.name}</b><small>{row.source}</small></div><time>{new Date(row.occurredAt).toLocaleTimeString('en-GB',{timeZone:'Asia/Colombo',hour:'2-digit',minute:'2-digit'})}</time><span className={row.type==='Check-in'?'tag green':'tag red'}>{row.type}</span></div>)}{!staffRows.length && <p className="dashboard-empty">No staff attendance recorded yet.</p>}</section>
    <section className="card quick-card"><div className="card-head"><div><h2>Quick actions</h2><p>Reception tasks</p></div></div><div className="quick-grid">{[['Members','plus','Add member'],['Payments','money','Receive payment'],['Sales & Inventory','bag','New sale'],['NFC Attendance','signal','Check-in']].map(row=><button key={row[0]} onClick={()=>navigate(row[0] as Page)}><Icon name={row[1]}/><span>{row[2]}</span></button>)}</div></section>
    <section className="card list-card expiry-card"><div className="card-head"><div><h2>Memberships expiring</h2><p>{expiringMembers.length} needing follow-up</p></div><button className="link" onClick={()=>navigate('Members')}>View all</button></div>{expiringMembers.slice(0, 4).map(row=><div className="list-row" key={row.id}><span className="avatar tiny">{row.initials}</span><div><b title={row.name}>{row.name}</b><small title={row.plan}>{row.plan}</small></div><span className="days">{membershipDaysRemaining(row.expiry,today,row.membershipStartsOn) ?? 0} days left</span></div>)}{!expiringMembers.length && <p className="dashboard-empty">No memberships expiring soon.</p>}</section>
  </div></div>
}
