import { Icon } from '../components/ui/Icon'
import { useGym } from '../context/GymContext'
import { money } from '../utils/format'
import { colomboToday, membershipDaysRemaining } from '../utils/membership'
import type { Page } from '../types/domain'

function Kpi({label,value,note,icon}:{label:string;value:string;note:string;icon:string}) {
  return <div className="card kpi"><div className="icon-box"><Icon name={icon}/></div><div><span>{label}</span><strong>{value}</strong><small>{note}</small></div></div>
}
export function DashboardPage({navigate}:{navigate:(p:Page)=>void}) {
  const {data,desktop} = useGym()
  const today = desktop?.snapshot?.today ?? colomboToday()
  const income = desktop?.snapshot
    ? (desktop.snapshot.payments.filter(row=>row.businessOn===today).reduce((sum,row)=>sum+row.netAmountMinor,0)
      +desktop.snapshot.sales.filter(row=>row.businessOn===today).reduce((sum,row)=>sum+row.totalMinor,0))/100
    : data.payments.filter(row=>row.date===today).reduce((sum,row)=>sum+row.amount,0)
  const days = Array.from({length:28},(_,index)=>{
    const day = new Date(today+'T12:00:00Z')
    day.setUTCDate(day.getUTCDate()-27+index)
    return day.toISOString().slice(0,10)
  })
  const counts = days.map(day=>data.attendance.filter(row=>row.type==='Check-in'&&row.date===day).length)
  const bars = desktop ? counts.map(count=>count/Math.max(1,...counts)*116) : [28,41,31,46,52,64,43,68,74,55,70,82,63,88,94,72,84,100,76,91,106,88,79,98,112,92,116,83]
  return <><div className="kpi-grid">
    <Kpi label="Total Members" value={String(data.members.length)} note="Stored locally" icon="users"/>
    <Kpi label="Today’s Attendance" value={String(data.attendance.filter(row=>row.date===(desktop?today:'2026-09-01')&&(!desktop||row.type==='Check-in')).length)} note={desktop?'Check-ins · Asia/Colombo':'NFC + manual'} icon="signal"/>
    <Kpi label="Expiring Soon" value={String(data.members.filter(row=>row.status==='Expiring').length)} note="Action required" icon="card"/>
    <Kpi label="Today’s Income" value={money(income)} note={desktop?'Payments + retail sales':'Payments recorded'} icon="money"/>
  </div><div className="dashboard-grid">
    <section className="card chart-card"><div className="card-head"><div><h2>Attendance trend</h2><p>{desktop?'Daily check-ins · last 28 business days':'Daily check-ins this month'}</p></div></div><div className="bar-chart">{bars.map((height,index)=><i key={index} title={desktop?days[index]+': '+counts[index]+' check-ins':undefined} style={{height}}/>)}</div><div className="axis">{desktop?<>{[0,7,14,21,27].map(index=><span key={index}>{days[index].slice(5)}</span>)}</>:<><span>Sep 1</span><span>Sep 8</span><span>Sep 15</span><span>Sep 22</span><span>Sep 30</span></>}</div></section>
    <section className="card nfc-card"><div className="card-head"><div><h2>{desktop?'Local storage':'Offline sync'}</h2><p>Local data protection</p></div><span className="live">{desktop?'SQLITE':'LIVE'}</span></div><div className="success-row"><div className="success-icon">✓</div><div><b>LOCAL DATABASE READY</b><span>{desktop?.snapshot?.pending??data.queue.length} changes waiting for server</span></div></div><div className="member-highlight"><span className="avatar">DB</span><div><h3>{desktop?'Local operations persisted':'Safe offline operation'}</h3><p>{desktop?(desktop.snapshot?.businessSync?(desktop.snapshot.businessSync.available?'Gym records synchronize with your verified account.':'Sign in online to synchronize gym records.'):(desktop.snapshot?.memberSync?.available?'Member sync connected; other modules remain local.':'Sign in online to synchronize members.')):'Auto-sync on reconnection'}</p></div></div><button className="primary wide" onClick={()=>navigate('Settings')}>Open sync settings</button></section>
    <section className="card list-card"><div className="card-head"><div><h2>Recent attendance</h2><p>Latest activity</p></div><button className="link" onClick={()=>navigate('NFC Attendance')}>View all</button></div>{data.attendance.slice(0,5).map(row=><div className="list-row" key={row.id}><span className="avatar tiny">{row.name.split(' ').map(part=>part[0]).join('').slice(0,2)}</span><div><b>{row.name}</b><small>{row.memberId} · {row.source}</small></div><time>{row.time}</time><span className={row.type==='Check-in'?'tag green':'tag red'}>{row.type}</span></div>)}</section>
    <section className="card quick-card"><div className="card-head"><div><h2>Quick actions</h2><p>Reception tasks</p></div></div><div className="quick-grid">{[['Members','plus','Add member'],['Payments','money','Receive payment'],['Sales & Inventory','bag','New sale'],['NFC Attendance','signal','Check-in']].map(row=><button key={row[0]} onClick={()=>navigate(row[0] as Page)}><Icon name={row[1]}/><span>{row[2]}</span></button>)}</div></section>
    <section className="card list-card expiry-card"><div className="card-head"><div><h2>Memberships expiring</h2><p>Follow-up list</p></div></div>{data.members.filter(row=>row.status==='Expiring').map(row=><div className="list-row" key={row.id}><span className="avatar tiny">{row.initials}</span><div><b>{row.name}</b><small>{row.plan}</small></div><span className="days">{membershipDaysRemaining(row.expiry,today,row.membershipStartsOn) ?? 0} days left</span></div>)}</section>
  </div></>
}
