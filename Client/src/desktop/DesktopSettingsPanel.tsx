import { useEffect, useRef, useState } from 'react'
import { Modal } from '../components/ui/Modal'
import { useGym } from '../context/GymContext'
import { version } from '../../package.json'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { prepareNativeDevice, type BackupEnvelope, type DeviceApproval, type RestorePreview } from './api'
import { MemberConflictDialog } from './MemberConflictDialog'
import { BusinessRetryDialog } from './BusinessRetryDialog'
import type { BusinessRetryPreview, MemberConflictPreview } from './api'
import { errorText } from './DesktopGymProvider'
import { updateCheckError } from './update-errors'
import { minorUnits } from './api'

function GymProfile() {
  const { desktop } = useGym()
  const [form, setForm] = useState(() => ({ ...desktop!.snapshot!.profile }))
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  async function save(event: React.FormEvent) {
    event.preventDefault()
    if (busy) return
    setBusy(true); setError('')
    try {
      await desktop!.saveProfile({ ...form, admissionMinor: form.admissionMinor ?? 0 })
      setForm({ ...form, version: form.version + 1, admissionVersion: (form.admissionVersion ?? 0) + 1 })
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <><form onSubmit={event => void save(event)}><fieldset className="foundation-fields" disabled={busy}><div className="form-grid">
    <label><span>Gym name</span><input required maxLength={120} value={form.name} onChange={event => setForm({ ...form, name: event.target.value })}/></label>
    <label><span>Location</span><input required maxLength={254} value={form.location} onChange={event => setForm({ ...form, location: event.target.value })}/></label>
    <label><span>Phone</span><input maxLength={40} value={form.phone} onChange={event => setForm({ ...form, phone: event.target.value })}/></label>
    <label><span>Email</span><input type="email" maxLength={254} value={form.email} onChange={event => setForm({ ...form, email: event.target.value })}/></label>
    <label><span>Admission fee (LKR)</span><input required type="number" min="0" max="1000000000" step="0.01" value={(form.admissionMinor ?? 0) / 100} onChange={event => setForm({ ...form, admissionMinor: minorUnits(Number(event.target.value)) })}/></label>
  </div>{error && <div role="alert" className="login-error">{error}</div>}<div className="settings-actions"><button className="primary">{busy ? 'Saving…' : 'Save changes'}</button><button type="button" className="secondary" onClick={() => { setForm({ ...desktop!.snapshot!.profile }); setError('') }}>Reload values</button></div></fieldset></form></>
}
function BackupActions() {
  const { desktop, notify } = useGym()
  const file = useRef<HTMLInputElement>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [notice, setNotice] = useState('')
  const [preview, setPreview] = useState<RestorePreview | null>(null)
  const [fileName, setFileName] = useState('')
  async function exportFile() {
    setBusy(true); setError('')
    try { const result = await desktop!.exportBackup(); setNotice('Validated SQLite backup saved: ' + result.path); notify('Validated SQLite backup file written.') }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  async function select(selected?: File) {
    if (!selected) return
    setBusy(true); setError(''); setPreview(null)
    try {
      if (selected.size > 350 * 1024 * 1024) throw new Error('Backup file is too large.')
      const backup: BackupEnvelope = JSON.parse(await selected.text())
      const result = await desktop!.previewRestore(backup)
      setFileName(selected.name); setPreview(result)
    } catch (error) { setError(errorText(error)) }
    finally { setBusy(false); if (file.current) file.current.value = '' }
  }
  async function restore() {
    if (!preview || busy) return
    setBusy(true); setError('')
    try { const result = await desktop!.restoreBackup(preview.token); setNotice('Restored SQLite. Recovery copy: ' + result.recoveryPath); setPreview(null) }
    catch (error) { setError(errorText(error)); setPreview(null) }
    finally { setBusy(false) }
  }
  const keys = ['members', 'periods', 'payments', 'sales', 'expenses', 'pending', 'auditCount'] as const
  return <><p>Keep a backup copy outside this computer.</p>
    <div className="settings-actions"><button className="primary" disabled={busy} onClick={() => void exportFile()}>Export backup</button><button className="secondary" disabled={busy} onClick={() => file.current?.click()}>Restore backup</button><input hidden ref={file} type="file" accept=".armstrong-backup.json" onChange={event => void select(event.target.files?.[0])}/></div>
    {notice && <p role="status" className="storage-file-result">{notice}</p>}{error && <div role="alert" className="login-error">{error}</div>}
    {preview && <Modal title="Review SQLite replacement" onClose={() => { if (!busy) setPreview(null) }}>
      <p>{fileName}</p><p className="form-note">This replaces current local records and settings. Changes since the backup, including payments and stock, will be retained in a validated recovery copy written before replacement. Pending operations will also be replaced. After restore, sign in online on this computer to reconcile the backup with server history before access resumes.</p>
      <div className="card table-card"><table><thead><tr><th>Records</th><th>Current</th><th>Backup</th></tr></thead><tbody>{keys.map(key => <tr key={key}><td>{key}</td><td>{preview.current[key]}</td><td>{preview.backup[key]}</td></tr>)}</tbody></table></div>
      <div className="settings-actions"><button className="primary" disabled={busy} onClick={() => void restore()}>{busy ? 'Restoring…' : 'Replace with this backup'}</button><button className="secondary" disabled={busy} onClick={() => setPreview(null)}>Cancel</button></div>
    </Modal>}
  </>
}
export function DevicePreparation() {
  const { desktop } = useGym()
  const [busy, setBusy] = useState(false)
  const [approval, setApproval] = useState<DeviceApproval | null>(null)
  const [error, setError] = useState('')
  const restored = desktop?.snapshot?.restoreRequiresReconciliation
  async function prepare() {
    if (busy || restored) return
    setBusy(true); setError(''); setApproval(null)
    try { setApproval(await prepareNativeDevice()) }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <section className="form-card">
    <h3>Approve this computer</h3>
    <p>Computer approval details.</p>
    <button className="secondary" disabled={busy || restored} onClick={() => void prepare()}>{busy ? 'Opening credential storage…' : 'Prepare this computer'}</button>
    {error && <div role="alert" className="login-error">{error}</div>}
    {approval && <><p role="status">Credential verified in OS storage. Sign in online to check account and server approval.</p><details open><summary>Registration details</summary><div className="form-grid">
      <label><span>Device ID</span><input readOnly value={approval.deviceId}/></label>
      <label><span>SQLite path</span><input readOnly value={approval.sqlitePath}/></label>
      <label><span>Device secret SHA-256</span><input readOnly value={approval.secretSha256}/></label>
    </div></details></>}
  </section>
}
function BusinessSyncPanel() {
  const { desktop, syncing, syncNow } = useGym()
  const snapshot = desktop?.snapshot
  const sync = snapshot?.businessSync
  const revisionConflict = sync?.conflicts[0]?.reason.includes('business_revision_conflict') ?? false
  const [review,setReview] = useState<BusinessRetryPreview | null>(null)
  const [busy,setBusy] = useState(false)
  const [error,setError] = useState('')
  async function openReview() {
    const first=sync?.conflicts[0]; if(!first || busy) return
    setBusy(true);setError('')
    try {setReview(await desktop!.previewBusinessRetry(first.id))}
    catch(error){setError(errorText(error))} finally {setBusy(false)}
  }
  async function retry() {
    if(!review || busy) return
    setBusy(true);setError('')
    try {await desktop!.retryBusinessTransaction({requestId:crypto.randomUUID(),batchId:review.batchId,fingerprint:review.fingerprint});setReview(null);await syncNow()}
    catch(error){setError(errorText(error))} finally {setBusy(false)}
  }
  async function recover() {
    if (!review?.initialProfileRecovery?.allowed || busy) return
    setBusy(true); setError('')
    try {
      await desktop!.recoverInitialGymProfile({requestId:crypto.randomUUID(),batchId:review.batchId,fingerprint:review.fingerprint,confirmation:true})
      setReview(null)
    } catch(error) {setError(errorText(error))} finally {setBusy(false)}
  }
  return <>
    <p>{sync?.available ? 'Changes sync automatically when online.' : 'Sign in online to synchronize gym records. Your offline changes are retained.'}</p>
    {sync?.lastError && <p role="status" className="form-note">{sync.lastError}</p>}
    {snapshot?.restoreRequiresReconciliation && <p className="form-note">This restored database requires server reconciliation before synchronization can resume.</p>}
    <div className="summary-grid"><div><small>Storage</small><strong>SQLite</strong></div><div><small>Pending operations</small><strong>{snapshot?.pending ?? 0}</strong></div><div><small>Confirmed transactions</small><strong>{sync?.acknowledged ?? 0}</strong></div><div><small>Transactions to review</small><strong>{sync?.conflicts.length ?? 0}</strong></div></div>
    {!!sync?.conflicts.length && <div role="alert" className="login-error"><p>{revisionConflict ? 'The saved transaction refers to a different server record version. Retry sends the same saved request; the affected records need reconciliation before it can succeed. Review the records and available recovery option. Local changes and history remain saved.' : 'The server refused a retained transaction. Review its reason and the affected records. Retry sends the original request after the cause of refusal has been resolved.'}</p>{sync.conflicts.map(c => <p key={c.id}>{c.reason}</p>)}<button className="secondary" disabled={busy || syncing || !sync.available || !snapshot?.removalAuthorization.allowed} onClick={() => void openReview()}>Review retained transaction</button></div>}
    {error && <p role="alert" className="login-error">{error}</p>}
    {review && <BusinessRetryDialog preview={review} busy={busy} error={error} onClose={() => setReview(null)} onRetry={() => void retry()} onRecover={() => void recover()}/>}
    {!!sync?.recoveredInitialProfiles && <p>{sync.recoveredInitialProfiles} initial profile recovery. The original refused transaction remains retained separately from server confirmations.</p>}
    {sync?.lastSuccessOn && <p>Last server confirmation: {new Date(sync.lastSuccessOn).toLocaleString()}.</p>}
    <button className="primary" disabled={!sync?.available || syncing} onClick={() => void syncNow()}>{syncing ? 'Synchronizing gym records…' : sync?.available ? 'Sync gym records now' : 'Sync unavailable'}</button>
  </>
}
function MemberSyncPanel() {
  const { desktop, syncing, syncNow } = useGym()
  const snapshot = desktop?.snapshot
  const sync = snapshot?.memberSync
  const authenticated = desktop?.authStatus?.authenticated
  const [review, setReview] = useState<MemberConflictPreview | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  async function openReview(conflictId: string) {
    if (busy) return
    setBusy(true); setError('')
    try { setReview(await desktop!.previewMemberConflict(conflictId)) }
    catch (error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  return <>
    <p>{authenticated ? sync?.available ? 'Member changes synchronize with your verified account. Changes made during an outage are retained and retried when the connection returns.' : 'Sign in online to synchronize members. Your offline changes are retained.' : 'No authenticated backend is connected. SQLite operations remain pending until the server confirms them.'}</p>
    <p>{authenticated ? 'Other modules are saved locally and do not yet synchronize.' : 'Member synchronization is being prepared. Staff sign-in and a secure server connection are still required.'}</p>
    {sync?.lastError && <p role="status" className="form-note">{sync.lastError}</p>}
    {snapshot?.restoreRequiresReconciliation && <p className="form-note">This database was restored and requires server reconciliation before synchronization can be enabled.</p>}
    <div className="summary-grid"><div><small>Storage</small><strong>SQLite</strong></div><div><small>Pending operations</small><strong>{snapshot?.pending ?? '—'}</strong></div><div><small>Confirmed member operations</small><strong>{sync?.acknowledged ?? 0}</strong></div><div><small>Member conflicts</small><strong>{sync?.conflicts.length ?? 0}</strong></div></div>
    {!!sync?.conflicts.length && <><p className="form-note">Local member records and history have been retained. An enrolled Administrator can review these recorded server changes.</p><div className="card table-card"><table><thead><tr><th>Member</th><th>Local name / card</th><th>Server name / card</th><th>Review required</th><th>Action</th></tr></thead><tbody>{sync.conflicts.map(conflict => {
      const local = snapshot?.members.find(member => member.id === conflict.memberId)
      return <tr key={conflict.id}><td>{local?.name ?? conflict.remote?.name ?? 'Member no longer present locally'}</td><td>{local ? `${local.name} / ${local.nfcId || 'No card'}` : 'No local record'}</td><td>{conflict.remote?.name ? `${conflict.remote.name} / ${conflict.remote.nfcId || 'No card'}${conflict.remote.archivedAt ? ' · Archived' : ''}` : 'Server rejected this change'}</td><td>{conflict.reason}</td><td><button className="secondary" disabled={busy || !sync.reviewAuthorization?.allowed} onClick={() => void openReview(conflict.id)}>Review conflict</button></td></tr>
    })}</tbody></table></div></>}
    {!!sync?.conflicts.length && !sync.reviewAuthorization?.allowed && <p className="form-note">{sync.reviewAuthorization?.reason || 'Conflict review requires an enrolled Administrator session.'}</p>}
    {!!sync?.resolved && <p>{sync.resolved} saved member review(s) · {sync.superseded ?? 0} discarded or superseded edits. These are separate from confirmed server operations.</p>}
    {error && <div role="alert" className="login-error">{error}</div>}
    {review && <MemberConflictDialog preview={review} onClose={() => setReview(null)}/>}
    {sync?.lastSuccessOn && <p>Last member server confirmation: {new Date(sync.lastSuccessOn).toLocaleString()}.</p>}
    <button className="primary" disabled={!sync?.available || syncing} onClick={() => void syncNow()}>{syncing ? 'Synchronizing members…' : sync?.available ? 'Sync members now' : 'Sync unavailable'}</button>
    <DevicePreparation/>
  </>
}
export function DesktopSettingsPanel({ tab }: { tab: string }) {
  const { desktop, notify } = useGym()
  const [update, setUpdate] = useState<Update | null>(null)
  const [checkingUpdates, setCheckingUpdates] = useState(false)
  const [installingUpdate, setInstallingUpdate] = useState(false)
  const [updateStatus, setUpdateStatus] = useState('')
  const [updateError, setUpdateError] = useState('')
  async function checkForUpdates() {
    if (checkingUpdates || installingUpdate) return
    if (!/Windows/i.test(navigator.userAgent)) {
      setUpdateStatus('Automatic application updates are available in the Windows version.')
      return
    }
    setCheckingUpdates(true)
    setUpdateError('')
    setUpdateStatus('Checking for a signed Windows update…')
    try {
      const available = await check()
      setUpdate(available)
      setUpdateStatus(available ? `Version ${available.version} is ready to install.` : 'You’re up to date.')
    } catch (error) {
      setUpdate(null)
      setUpdateStatus('')
      setUpdateError(updateCheckError(error))
    } finally {
      setCheckingUpdates(false)
    }
  }
  async function installUpdate() {
    if (!update || installingUpdate) return
    setInstallingUpdate(true)
    setUpdateError('')
    setUpdateStatus('Downloading and verifying the signed update…')
    try {
      await update.downloadAndInstall()
      setUpdateStatus('Update installed. Restarting ArmStrong Fitness…')
      await relaunch()
    } catch (error) {
      setUpdateStatus('')
      setUpdateError(errorText(error))
      setInstallingUpdate(false)
    }
  }
  useEffect(() => {
    if (tab === 'Application updates' && !updateStatus && !updateError) void checkForUpdates()
  }, [tab])
  if (tab === 'Gym profile') return <GymProfile/>
  if (tab === 'Backup & restore') return <BackupActions/>
  if (tab === 'Users & roles') return <><p>{desktop?.authStatus?.requiresLogin ? 'First sign-in verifies the Administrator account and computer online. After verification, Continue offline uses this computer’s unlocked OS account for up to seven days. Signing out removes offline access. Permissions are checked whenever you use the app.' : 'Authentication is not configured. This local test operator has unrestricted access. Use test records only. Demo accounts are not enrolled users.'}</p>{desktop?.authStatus?.offlineUntil && <p>Offline access until {new Date(desktop.authStatus.offlineUntil).toLocaleString()}.</p>}{desktop?.snapshot?.users.length ? desktop.snapshot.users.map(user => <div className="setting-line" key={user.id}><div><b>{user.name}</b><small>{user.roles.join(', ') || 'No roles'}</small></div><span>{user.active ? 'Active' : 'Inactive'}</span></div>) : <p>No users have been enrolled.</p>}</>
  if (tab === 'NFC reader') return <><p>Use a USB HID reader that types the card UID and Enter. Card linking and attendance logs are saved locally; no hardware has been detected or verified.</p><div className="setting-line"><div><b>Reader mode</b><small>Keyboard / HID input</small></div><span>Unverified</span></div><button className="primary" onClick={() => notify('Open NFC Attendance, focus the card field and scan. Verify the recorded UID and member; hardware has not been certified.', 'info')}>Test reader</button></>
  if (tab === 'Receipt printing') return <><p>Open a saved payment in Payments to preview or reprint its receipt. Print opens the webview system print dialog; Windows dialog and physical printer acceptance remain unverified. Direct printer selection is not implemented.</p><div className="setting-line"><div><b>Paper size</b><small>80 mm receipt layout; select the printer in the print dialog</small></div><span>Unverified</span></div></>
  if (tab === 'Server synchronization') return desktop?.snapshot?.businessSync ? <BusinessSyncPanel/> : <MemberSyncPanel/>
  return <><p>ArmStrong checks for signed Windows updates when you open this page. Choose when to install; your gym records stay in their separate local database.</p><div className="setting-line"><div><b>Installed version</b><small>{version}</small></div><span>{update ? 'Update available' : updateStatus === 'You’re up to date.' ? 'Current' : 'Windows updater'}</span></div>{updateStatus && <p role="status">{updateStatus}</p>}{updateError && <div role="alert" className="login-error">{updateError}</div>}<div className="settings-actions"><button className="primary" disabled={checkingUpdates || installingUpdate} onClick={() => void checkForUpdates()}>{checkingUpdates ? 'Checking…' : 'Check for updates'}</button>{update && <button className="secondary" disabled={checkingUpdates || installingUpdate} onClick={() => void installUpdate()}>{installingUpdate ? 'Installing…' : `Install ${update.version} and restart`}</button>}</div></>
}
