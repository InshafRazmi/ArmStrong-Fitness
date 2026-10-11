import { useMemo, useState } from 'react'
import { Icon } from '../components/ui/Icon'
import { useGym } from '../context/GymContext'
import type { Member } from '../types/domain'

export function Topbar({ title, onSelect, onLogout }: { title: string; onSelect: (member: Member) => void; onLogout: () => void }) {
  const { data, online, syncing, syncNow, mode, desktop } = useGym()
  const [query, setQuery] = useState('')
  const [open, setOpen] = useState(false)
  const [activeIndex, setActiveIndex] = useState(0)
  const results = useMemo(() => {
    const value = query.trim().toLowerCase()
    return value ? data.members.filter(member => [member.name, member.id, member.phone, member.plan].some(field => field.toLowerCase().includes(value))).slice(0, 6) : []
  }, [query, data.members])
  const choose = (member: Member) => { onSelect(member); setQuery(''); setOpen(false) }
  const nativeSync = desktop?.snapshot?.businessSync ?? desktop?.snapshot?.memberSync
  const status = online ? 'ONLINE' : mode === 'desktop' && !desktop?.authStatus?.configured ? 'LOCAL' : 'OFFLINE'
  const detail = desktop
    ? `${desktop.snapshot?.pending ?? 0} PENDING · ${nativeSync?.available ? 'GYM SYNC' : desktop.authStatus?.authenticated ? 'SIGN IN TO SYNC' : 'NO SERVER'}`
    : syncing ? 'SYNCING' : data.queue.length ? `${data.queue.length} PENDING` : 'SYNCED'

  return <header className="topbar">
    <div><span className="eyebrow">ARMSTRONG FITNESS</span><h1>{title}</h1></div>
    <div className="top-actions">
      <div className="global-search">
        <label className="search"><Icon name="search" size={17}/><input value={query} onChange={event => { setQuery(event.target.value); setOpen(true); setActiveIndex(0) }} onFocus={() => setOpen(true)} onKeyDown={event => {
          if (!results.length) return
          if (event.key === 'ArrowDown') { event.preventDefault(); setActiveIndex(index => (index + 1) % results.length) }
          if (event.key === 'ArrowUp') { event.preventDefault(); setActiveIndex(index => (index - 1 + results.length) % results.length) }
          if (event.key === 'Enter') { event.preventDefault(); choose(results[activeIndex]) }
          if (event.key === 'Escape') setOpen(false)
        }} placeholder="Search members by name, ID or phone..."/></label>
        {open && query.trim() && <div className="search-results"><div className="search-result-head"><span>Members</span><small>{results.length} results</small></div>{results.map((member, index) => <button key={member.id} className={index === activeIndex ? 'selected' : ''} onMouseDown={event => { event.preventDefault(); choose(member) }}><span className="avatar tiny">{member.initials}</span><span className="result-main"><b>{member.name}</b><small>{member.id} · {member.phone}</small></span><span className="result-plan">{member.plan}</span></button>)}{!results.length && <div className="empty-search"><b>No members found</b></div>}</div>}
      </div>
      <button className="sync" aria-disabled={syncing} onClick={() => { if (!syncing) void syncNow() }} title={desktop ? 'Synchronize gym records; unconfirmed changes stay queued.' : undefined}>
        <span className="status-dot" style={{ background: online ? 'var(--green)' : 'var(--red)' }}/>{status}<span className="divider"/>{detail}
      </button>
      <button className="profile" onClick={onLogout} disabled={mode === 'desktop' && !desktop?.authStatus?.authenticated} title={mode === 'desktop' && !desktop?.authStatus?.authenticated ? 'Sign in to use this computer' : 'Sign out'}>
        <span className="avatar small">{mode === 'desktop' ? (desktop?.authStatus?.userName?.[0] ?? 'L') : 'P'}</span>
        <span><b>{mode === 'desktop' ? (desktop?.authStatus?.userName ?? 'Local operator') : 'Prinzz'}</b><small>{mode === 'desktop' ? (desktop?.authStatus?.authenticated ? 'Administrator · Sign out' : 'Unauthenticated') : 'Administrator · Sign out'}</small></span>
      </button>
    </div>
  </header>
}
