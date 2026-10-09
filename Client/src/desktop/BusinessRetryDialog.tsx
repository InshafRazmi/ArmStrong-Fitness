import { useState } from 'react'
import { Modal } from '../components/ui/Modal'
import type { BusinessRetryPreview } from './api'

export function BusinessRetryDialog({preview, busy, error, onClose, onRetry, onRecover}: {
  preview: BusinessRetryPreview; busy: boolean; error: string;
  onClose: () => void; onRetry: () => void; onRecover: () => void;
}) {
  const [confirmed, setConfirmed] = useState(false)
  const recovery = preview.initialProfileRecovery
  const revision = preview.reason.includes('business_revision_conflict')
  return <Modal title="Review retained transaction" onClose={() => {if (!busy) onClose()}}>
    <p>{preview.reason}</p>
    <p className="form-note">{revision
      ? 'The server has a different record version. Retry sends the same saved request and can be refused again. Review the recovery option below.'
      : 'Retry sends the original transaction for another server check after the cause of refusal has been resolved.'}</p>
    <div className="card table-card business-review-rows"><table>
      <thead><tr><th>Action</th><th>Records</th><th>Name / ID</th></tr></thead>
      <tbody>{preview.changes.map(c => <tr key={`${c.table}:${c.id}`}><td>{c.action}</td><td>{c.table.replaceAll('_', ' ')}</td><td>{c.name || c.id}</td></tr>)}</tbody>
    </table></div>
    {revision && (recovery?.allowed ? <>
      <p>This computer has the unchanged installation profile. Recovery creates a backup, retains the original transaction and audit entries, and downloads the current server profile. You will need to sign in online again.</p>
      <label><input type="checkbox" checked={confirmed} disabled={busy} onChange={event => setConfirmed(event.target.checked)}/> I confirm recovery using the server gym profile.</label>
    </> : <p className="form-note">{recovery?.reason || 'These changes require separate reconciliation. Export a backup before reviewing them.'}</p>)}
    {error && <p role="alert" className="login-error">{error}</p>}
    <div className="settings-actions">
      {recovery?.allowed && <button className="primary" disabled={busy || !confirmed} onClick={() => {if (confirmed && !busy) onRecover()}}>{busy ? 'Recovering server profile…' : 'Recover using server profile'}</button>}
      <button className="secondary" disabled={busy} onClick={onRetry}>Retry original transaction</button>
      <button className="secondary" disabled={busy} onClick={onClose}>Cancel</button>
    </div>
  </Modal>
}
