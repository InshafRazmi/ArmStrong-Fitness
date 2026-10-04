import { useEffect, useState } from 'react'
import { Icon } from '../components/ui/Icon'
import { PageHeader } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import type { ReportSummary } from '../desktop/api'
import { money } from '../utils/format'

export function ReportsPage() {
  const { data, notify, desktop } = useGym()
  const [fromOn, setFromOn] = useState('')
  const [toOn, setToOn] = useState('')
  const [summary, setSummary] = useState<ReportSummary | null>(null)
  const [busy, setBusy] = useState(false)
  const [loading, setLoading] = useState(!!desktop)
  const [error, setError] = useState('')
  const [file, setFile] = useState('')
  const query = desktop?.reportSummary
  const snapshot = desktop?.snapshot
  useEffect(() => {
    if (!query || !snapshot) return
    let active = true
    setSummary(null); setError(''); setFile('')
    if (fromOn && toOn && fromOn > toOn) {
      setError('Start date must be on or before the end date.'); setLoading(false)
      return
    }
    setLoading(true)
    void query({ fromOn: fromOn || null, toOn: toOn || null }).then(result => {
      if (active) setSummary(result)
    }).catch(error => {
      if (active) setError(error instanceof Error ? error.message : String(error))
    }).finally(() => { if (active) setLoading(false) })
    return () => { active = false }
  }, [query, snapshot, fromOn, toOn])
  const ready = !desktop || (!!summary && (summary.fromOn ?? '') === fromOn && (summary.toOn ?? '') === toOn)
  const demoIncome = data.payments.reduce((sum, row) => sum + row.amount, 0) + data.sales.reduce((sum, row) => sum + row.total, 0)
  const demoExpenses = data.expenses.reduce((sum, row) => sum + row.amount, 0)
  const income = desktop ? summary ? summary.incomeMinor / 100 : null : demoIncome
  const expenses = desktop ? summary ? summary.expenseMinor / 100 : null : demoExpenses
  const displayMoney = (amount: number | null) => amount === null ? '—' : money(amount)
  const reports = [
    ['Attendance report', (desktop ? summary?.attendanceCount ?? '—' : data.attendance.length) + ' attendance records', 'signal'],
    ['Membership report', (desktop ? summary?.membershipPeriodCount ?? '—' : data.members.length) + (desktop ? ' membership periods' : ' registered members'), 'card'],
    ['Income report', displayMoney(income) + ' total received', 'money'],
    ['Inventory report', (desktop ? summary?.lowStockCount ?? '—' : data.products.filter(row => row.stock <= row.reorderLevel).length) + ' low-stock products · current stock', 'bag'],
    ['Expense report', displayMoney(expenses) + ' total expenses', 'receipt'],
    ['Audit report', (desktop ? summary?.auditCount ?? '—' : data.audit.length) + ' recorded changes', 'settings'],
  ]
  async function exportReport(kind: string) {
    if (!desktop) { notify(kind + ' export will use the current filtered data', 'info'); return }
    if (busy || loading || !ready) return
    setBusy(true); setError(''); setFile('')
    try {
      const result = await desktop.exportReport(kind, { fromOn: fromOn || null, toOn: toOn || null })
      setFile(result.path)
      notify(kind + ' exported: ' + result.rows + ' rows', 'success')
    } catch (error) { setError(error instanceof Error ? error.message : String(error)) }
    finally { setBusy(false) }
  }
  return <>
    <PageHeader title="Reports" subtitle="Operational, financial and audit summaries" />
    {desktop && <>
      <fieldset className="foundation-fields" disabled={busy}>
        <div className="form-grid">
          <label><span>From date</span><input type="date" min="1900-01-01" max="2200-12-31" value={fromOn} onChange={event => setFromOn(event.target.value)} /></label>
          <label><span>To date</span><input type="date" min="1900-01-01" max="2200-12-31" value={toOn} onChange={event => setToOn(event.target.value)} /></label>
        </div>
        <div className="settings-actions">
          <button className="secondary compact" disabled={!snapshot} onClick={() => { setFromOn(snapshot!.today); setToOn(snapshot!.today) }}>Today</button>
          <button className="secondary compact" onClick={() => { setFromOn(''); setToOn('') }}>All dates</button>
        </div>
      </fieldset>
      <p className="form-note">CSV exports and totals use the selected inclusive dates in Asia/Colombo. Blank dates include all history. Memberships include periods overlapping the range; their status is shown as of today. Inventory shows current stock. Income includes payments less reversals posted within the range, plus sales; invoices and allocations do not add cash. Expense totals exclude voided records; CSV retains original amounts and void history.</p>
      {loading && <p role="status">Loading report totals…</p>}
    </>}
    {error && <div role="alert" className="login-error">{error}</div>}
    {file && <p role="status" className="storage-file-result">Export saved to {file}</p>}
    <div className="report-grid">{reports.map(row => <button className="card report-card" key={row[0]} disabled={busy || loading || !ready} onClick={() => void exportReport(row[0])}><span className="icon-box"><Icon name={row[2]} /></span><span><b>{row[0]}</b><small>{row[1]}</small></span><i>→</i></button>)}</div>
    <section className="card report-summary"><h2>Business summary</h2><div className="summary-grid">
      <div><small>Total income</small><strong>{displayMoney(income)}</strong></div>
      <div><small>Total expenses</small><strong>{displayMoney(expenses)}</strong></div>
      <div><small>Net balance</small><strong>{displayMoney(desktop ? summary ? summary.netMinor / 100 : null : demoIncome - demoExpenses)}</strong></div>
      <div><small>Current inventory value</small><strong>{displayMoney(desktop ? summary ? summary.inventoryValueMinor / 100 : null : data.products.reduce((sum, row) => sum + row.cost * row.stock, 0))}</strong></div>
    </div></section>
  </>
}
