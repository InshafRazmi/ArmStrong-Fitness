import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react'
import { GymContext, type GymContextValue } from '../context/GymContext'
import type { ToastMessage } from '../types/domain'
import * as api from './api'
import { desktopData, emptyDesktopData } from './adapter'
import { startPolling } from './polling'
export const errorText = (error: unknown) => error instanceof Error ? error.message : String(error)
export function DesktopGymProvider({ children }: { children: ReactNode }) {
  const [native, setNative] = useState<api.Snapshot | null>(null)
  const [authStatus, setAuthStatus] = useState<api.DesktopAuthStatus | null>(null)
  const [error, setError] = useState('')
  const [toasts, setToasts] = useState<ToastMessage[]>([])
  const [syncing, setSyncing] = useState(false)
  const syncRunning = useRef(false)
  const renewing = useRef(false)
  const generation = useRef(0)
  const notify = useCallback((message: string, tone: ToastMessage['tone'] = 'success') => {
    const id = crypto.randomUUID()
    setToasts(old => [...old, { id, message, tone }])
    setTimeout(() => setToasts(old => old.filter(item => item.id !== id)), 5000)
  }, [])
  const refresh = useCallback(async () => {
    const request = ++generation.current
    try {
      const auth = await api.desktopAuthStatus()
      if (request !== generation.current) return
      setAuthStatus(auth)
      if (auth.requiresLogin && !auth.authenticated) { setNative(null); setError(''); return }
      const next = await api.snapshot()
      if (request === generation.current) { setNative(next); setError('') }
    } catch (error) {
      if (request === generation.current) setError(errorText(error))
      throw error
    }
  }, [])
  const synchronize = useCallback(async (manual = false) => {
    if (syncRunning.current || renewing.current) { if (manual) notify('Account verification or synchronization is already running.','info'); return }
    syncRunning.current = true; setSyncing(true)
    try {
      await api.desktopRenewSession()
      const outcome = await api.synchronizeMembers()
      if (manual) notify(outcome.reason, outcome.state === 'complete' ? 'success' : outcome.state === 'failed' || outcome.state === 'blocked' ? 'error' : 'info')
      await refresh()
    } catch (error) {
      if (manual) notify(errorText(error), 'error')
      await refresh().catch(() => {})
    } finally { syncRunning.current = false; setSyncing(false) }
  }, [refresh, notify])
  useEffect(() => {
    if (!authStatus?.configured) return
    const renew = async () => {
      if (renewing.current || syncRunning.current) return
      renewing.current = true
      try { await api.desktopRenewSession(); await refresh() } catch { /* Existing access/data remain governed by native expiry. */ }
      finally { renewing.current = false }
    }
    const timer=setInterval(() => { void renew() },30_000)
    const reconnect=() => { void renew() }
    window.addEventListener('online',reconnect)
    return () => { clearInterval(timer); window.removeEventListener('online',reconnect) }
  }, [authStatus?.configured,refresh])
  const memberSyncAvailable = Boolean((native?.businessSync ?? native?.memberSync)?.available && authStatus?.authenticated && !authStatus.offline)
  useEffect(() => {
    if (!memberSyncAvailable) return
    const wake = () => { void synchronize() }
    const stopPolling = startPolling(async () => { await synchronize() })
    window.addEventListener('online', wake)
    return () => { stopPolling(); window.removeEventListener('online', wake) }
  }, [memberSyncAvailable, synchronize])
  useEffect(() => {
    const stopPolling = startPolling(async () => {
      if (!syncRunning.current && !renewing.current) await refresh()
    })
    return () => { stopPolling(); generation.current++ }
  }, [refresh])
  useEffect(() => {
    if (!authStatus?.authenticated || !authStatus.expiresAt) return
    const delay = Math.max(0, Date.parse(authStatus.expiresAt) - Date.now())
    if (!Number.isFinite(delay)) return
    const timer = setTimeout(() => { void refresh().catch(() => {}) }, Math.min(delay, 2_147_483_647))
    return () => clearTimeout(timer)
  }, [authStatus, refresh])
  async function afterCommit() {
    try { await refresh() } catch (error) {
      setError(`Saved, but the list could not refresh: ${errorText(error)}. Use Refresh; do not resubmit.`)
    }
  }
  async function commit(write: () => Promise<api.WriteOutcome | void>, message = 'Saved in SQLite. Server synchronization remains pending.') {
    const result = await write()
    notify(result?.duplicate ? 'Duplicate scan ignored; the earlier attendance event is retained.' : message, result?.duplicate ? 'info' : 'success')
    await afterCommit()
  }
  const request = (id?: string) => {
    if (!id) throw new Error('Missing operation ID. Reopen the form before saving.')
    return id
  }
  const value: GymContextValue = {
    mode: 'desktop', data: native ? desktopData(native) : emptyDesktopData(), online: Boolean(memberSyncAvailable && (native?.businessSync ?? native?.memberSync)?.lastSuccessOn && !(native?.businessSync ?? native?.memberSync)?.lastError && Date.now() - Date.parse((native?.businessSync ?? native?.memberSync)!.lastSuccessOn!) < 90_000), syncing, toasts, notify,
    desktop: {
      authStatus,
      login: async (email, password) => { const status = await api.desktopLogin(email, password); await refresh(); if (status.reason) notify(status.reason, 'info') },
      unlockOffline: async () => { await api.desktopUnlockOffline(); await refresh() },
      logout: async () => {
        generation.current++
        setNative(null)
        setAuthStatus(old => old ? {...old, authenticated: false, userName: null, role: null, canWrite: false, expiresAt: null, offlineUntil: null, offline: false} : old)
        await api.desktopLogout()
        await refresh()
      },
      snapshot: native, error, refresh,
      previewBusinessRetry: api.previewBusinessRetry,
      retryBusinessTransaction: input => commit(() => api.retryBusinessTransaction(input), 'Original transaction queued for another server check.'),
      recoverInitialGymProfile: async input => {
        if (syncRunning.current || renewing.current) throw new Error('Account verification or synchronization is running. Wait, then try recovery again.')
        syncRunning.current = true; setSyncing(true)
        try {
          const result = await api.recoverInitialGymProfile(input)
          notify(`Server profile recovered. Sign in online again. Original transaction backup: ${result.recoveryPath}`, 'info')
          await afterCommit()
        } finally { syncRunning.current = false; setSyncing(false) }
      },
      previewMemberConflict: api.previewMemberConflict,
      resolveMemberConflict: async input => {
        const result = await api.resolveMemberConflict(input)
        notify(result.retryOperationId ? 'Local member version retained. A fresh retry is pending server confirmation.' : 'Recorded server member version applied in SQLite. Review history retained.')
        await afterCommit()
      },
      archiveMember: input => commit(() => api.archiveMember(input), 'Member archived in SQLite. History retained.'),
      deleteMember: input => commit(() => api.deleteMember(input), 'Member permanently removed. Past payments and attendance retained.'),
      voidExpense: input => commit(() => api.voidExpense(input), 'Expense void committed in SQLite. Original retained.'),
      createInvoice: input => commit(() => api.createInvoice(input), 'Invoice saved in SQLite.'),
      allocatePayment: input => commit(() => api.allocatePayment(input), 'Payment allocation committed in SQLite.'),
      saveTrainer: input => commit(() => api.saveTrainer(input), 'Staff details saved.'),
      deleteStaff: input => commit(() => api.deleteStaff(input), 'Staff deleted. Salary and training history retained.'),
      recordStaffAttendance: input => commit(() => api.recordStaffAttendance(input), 'Staff attendance saved.'),
      recordNfcAttendance: async input => {
        const result = await api.recordNfcAttendance(input)
        notify(result.duplicate ? 'Duplicate scan ignored.' : `${result.entity} attendance saved.`, result.duplicate ? 'info' : 'success')
        await afterCommit()
        return result
      },
      createTrainingCharge: input => commit(() => api.createTrainingCharge(input), 'Monthly training invoice saved.'),
      receiveCombinedPayment: input => commit(() => api.receiveCombinedPayment(input), 'Payment and receipt saved.'),
      payStaff: input => commit(() => api.payStaff(input), 'Staff payment recorded as a Salary expense.'),
      receivePayment: input => commit(() => api.receivePayment(input), 'Payment and receipt saved in SQLite.'),
      renewMembership: input => commit(() => api.renewMembership(input), 'Membership dates and invoice committed in SQLite.'),
      reversePayment: input => commit(() => api.reversePayment(input), 'Payment reversal and allocation releases committed in SQLite.'),
      paymentReceipt: api.paymentReceipt,
      savePlan: input => commit(() => api.savePlan(input)),
      addPeriod: input => commit(() => api.addPeriod(input)),
      saveProfile: input => commit(() => api.saveProfile(input), 'Gym profile saved in SQLite.'),
      saveProduct: input => commit(() => api.saveProduct(input)),
      exportBackup: api.exportBackup, previewRestore: api.previewRestore, exportReport: api.exportReport, reportSummary: api.reportSummary,
      restoreBackup: async token => {
        const result = await api.restoreBackup(token)
        notify('SQLite backup restored. Recovery copy retained; server reconciliation is required.')
        await afterCommit()
        return result
      },
    },
    addMember: async member => {
      const result = await api.registerMemberWithTrainer({gender: member.gender ?? null, trainerId: member.trainerId ?? null, trainerVersion: member.trainerVersion ?? null, member: { expectedAdmissionMinor: member.expectedAdmissionMinor, requestId: member.requestId, name: member.name, phone: member.phone, email: member.email, nfcId: member.nfcId, planId: member.planId, planVersion: member.planVersion, startsOn: member.startsOn } })
      notify('Member and joining charges saved. Unpaid amounts are due to pay.')
      await afterCommit()
      return result.id
    },
    updateMember: member => commit(() => api.saveMemberWithTrainer({gender: member.gender ?? null, genderVersion: member.genderVersion ?? null, trainerId: member.trainerId ?? null, trainerVersion: member.trainerVersion ?? null, assignmentVersion: member.assignmentVersion ?? null, member: { id: member.id, version: member.version, name: member.name, phone: member.phone, email: member.email, nfcId: member.nfcId } })),
    updatePlan: plan => commit(() => api.savePlan({ id: plan.id, version: plan.version, name: plan.name, durationMonths: plan.durationMonths, priceMinor: api.minorUnits(plan.price), active: plan.status === 'Active' })),
    recordAttendance: (memberOrCard, source, operationId) => commit(() => api.recordAttendance({ requestId: request(operationId), memberOrCard, source })),
    addPayment: payment => commit(() => api.recordPayment({ requestId: request(payment.requestId), memberId: payment.memberId, amountMinor: api.minorUnits(payment.amount), method: payment.method }), 'Payment amount recorded in SQLite. No invoice allocation or membership renewal was made.'),
    addExpense: expense => commit(() => api.recordExpense({ requestId: request(expense.requestId), title: expense.title, category: expense.category, amountMinor: api.minorUnits(expense.amount), method: expense.method })),
    adjustStock: (productId, amount, operationId) => commit(() => api.adjustStock({ requestId: request(operationId), productId, amount })),
    completeSale: (productId, quantity, method, operationId) => commit(() => api.completeSale({ requestId: request(operationId), productId, quantity, method }), 'Sale and stock movement committed in SQLite.'),
    restore: () => { notify('Browser demo JSON cannot replace SQLite. Use native Backup & restore.', 'error') },
    syncNow: async () => { await synchronize(true) },
  }
  return <GymContext.Provider value={value}>{children}</GymContext.Provider>
}
