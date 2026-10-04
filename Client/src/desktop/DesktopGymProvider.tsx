import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react'
import { GymContext, type GymContextValue } from '../context/GymContext'
import type { ToastMessage } from '../types/domain'
import * as api from './api'
import { desktopData, emptyDesktopData } from './adapter'
export const errorText = (error: unknown) => error instanceof Error ? error.message : String(error)
export function DesktopGymProvider({ children }: { children: ReactNode }) {
  const [native, setNative] = useState<api.Snapshot | null>(null)
  const [authStatus, setAuthStatus] = useState<api.DesktopAuthStatus | null>(null)
  const [error, setError] = useState('')
  const [toasts, setToasts] = useState<ToastMessage[]>([])
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
  useEffect(() => {
    void refresh().catch(() => {})
    const timer = setInterval(() => { void refresh().catch(() => {}) }, 60_000)
    return () => { clearInterval(timer); generation.current++ }
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
    mode: 'desktop', data: native ? desktopData(native) : emptyDesktopData(), online: false, syncing: false, toasts, notify,
    desktop: {
      authStatus,
      login: async (email, password) => { await api.desktopLogin(email, password); await refresh() },
      logout: async () => {
        generation.current++
        setNative(null)
        setAuthStatus(old => old ? {...old, authenticated: false, userName: null, role: null, canWrite: false, expiresAt: null} : old)
        await api.desktopLogout()
        await refresh()
      },
      snapshot: native, error, refresh,
      previewMemberConflict: api.previewMemberConflict,
      resolveMemberConflict: async input => {
        const result = await api.resolveMemberConflict(input)
        notify(result.retryOperationId ? 'Local member version retained. A fresh retry is pending server confirmation.' : 'Recorded server member version applied in SQLite. Review history retained.')
        await afterCommit()
      },
      archiveMember: input => commit(() => api.archiveMember(input), 'Member archived in SQLite. History retained.'),
      deleteMember: input => commit(() => api.deleteMember(input), 'Unlinked member deleted in SQLite. Audit retained.'),
      voidExpense: input => commit(() => api.voidExpense(input), 'Expense void committed in SQLite. Original retained.'),
      createInvoice: input => commit(() => api.createInvoice(input), 'Invoice saved in SQLite.'),
      allocatePayment: input => commit(() => api.allocatePayment(input), 'Payment allocation committed in SQLite.'),
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
    addMember: member => commit(() => api.saveMember({ name: member.name, phone: member.phone, email: member.email, nfcId: member.nfcId })),
    updateMember: member => commit(() => api.saveMember({ id: member.id, version: member.version, name: member.name, phone: member.phone, email: member.email, nfcId: member.nfcId })),
    updatePlan: plan => commit(() => api.savePlan({ id: plan.id, version: plan.version, name: plan.name, durationMonths: plan.durationMonths, priceMinor: api.minorUnits(plan.price), active: plan.status === 'Active' })),
    recordAttendance: (memberOrCard, source, operationId) => commit(() => api.recordAttendance({ requestId: request(operationId), memberOrCard, source })),
    addPayment: payment => commit(() => api.recordPayment({ requestId: request(payment.requestId), memberId: payment.memberId, amountMinor: api.minorUnits(payment.amount), method: payment.method }), 'Payment amount recorded in SQLite. No invoice allocation or membership renewal was made.'),
    addExpense: expense => commit(() => api.recordExpense({ requestId: request(expense.requestId), title: expense.title, category: expense.category, amountMinor: api.minorUnits(expense.amount), method: expense.method })),
    adjustStock: (productId, amount, operationId) => commit(() => api.adjustStock({ requestId: request(operationId), productId, amount })),
    completeSale: (productId, quantity, method, operationId) => commit(() => api.completeSale({ requestId: request(operationId), productId, quantity, method }), 'Sale and stock movement committed in SQLite.'),
    restore: () => { notify('Browser demo JSON cannot replace SQLite. Use native Backup & restore.', 'error') },
    syncNow: async () => { notify('No authenticated server is configured. Pending SQLite operations are retained.', 'info') },
  }
  return <GymContext.Provider value={value}>{children}</GymContext.Provider>
}
