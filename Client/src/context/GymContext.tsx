import { createContext, useContext } from 'react'
import type { Expense, GymData, Member, MembershipPlan, Payment, ToastMessage } from '../types/domain'
import type { MemberRemovalInput, ExpenseVoidInput, AllocationInput, InvoiceInput, ReceivePaymentInput, RenewalInput, ReversalInput, ReceiptDocument, BackupEnvelope, FileResult, PeriodInput, PlanInput, ProductInput, Profile, ReportRange, ReportSummary, RestorePreview, RestoreResult, Snapshot } from '../desktop/api'
export type NewMember = Pick<Member, 'name' | 'phone' | 'email' | 'plan' | 'expiry' | 'nfcId'>
type WriteResult = void | Promise<void>
export interface DesktopData {
  previewMemberConflict: (conflictId: string) => Promise<import('../desktop/api').MemberConflictPreview>
  resolveMemberConflict: (input: import('../desktop/api').MemberConflictInput) => Promise<void>
  authStatus?: import('../desktop/api').DesktopAuthStatus | null
  login: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  unlockOffline: () => Promise<void>
  archiveMember: (input: MemberRemovalInput) => Promise<void>
  deleteMember: (input: MemberRemovalInput) => Promise<void>
  voidExpense: (input: ExpenseVoidInput) => Promise<void>
  createInvoice: (input: InvoiceInput) => Promise<void>
  allocatePayment: (input: AllocationInput) => Promise<void>
  receivePayment: (input: ReceivePaymentInput) => Promise<void>
  renewMembership: (input: RenewalInput) => Promise<void>
  reversePayment: (input: ReversalInput) => Promise<void>
  paymentReceipt: (paymentId: string) => Promise<ReceiptDocument>
  snapshot: Snapshot | null
  error: string
  refresh: () => Promise<void>
  savePlan: (input: PlanInput) => Promise<void>
  addPeriod: (input: PeriodInput) => Promise<void>
  saveProfile: (input: Profile) => Promise<void>
  saveProduct: (input: ProductInput) => Promise<void>
  exportBackup: () => Promise<FileResult>
  previewRestore: (input: BackupEnvelope) => Promise<RestorePreview>
  restoreBackup: (token: string) => Promise<RestoreResult>
  exportReport: (kind: string, range?: ReportRange) => Promise<FileResult>
  reportSummary: (range: ReportRange) => Promise<ReportSummary>
}
export interface GymContextValue {
  mode: 'browser' | 'desktop'
  desktop?: DesktopData
  data: GymData
  online: boolean
  syncing: boolean
  toasts: ToastMessage[]
  addMember: (value: NewMember) => WriteResult
  updateMember: (value: Member) => WriteResult
  updatePlan: (value: MembershipPlan) => WriteResult
  recordAttendance: (memberId: string, source: 'NFC' | 'Manual', requestId?: string) => WriteResult
  addPayment: (value: Omit<Payment, 'id' | 'syncState'>) => WriteResult
  addExpense: (value: Omit<Expense, 'id' | 'syncState'>) => WriteResult
  adjustStock: (id: string, amount: number, requestId?: string) => WriteResult
  completeSale: (productId: string, quantity: number, method: 'Cash' | 'Card' | 'Transfer', requestId?: string) => WriteResult
  syncNow: () => Promise<void>
  restore: (value: GymData) => void
  notify: (message: string, tone?: ToastMessage['tone']) => void
}
export const GymContext = createContext<GymContextValue | null>(null)
export function useGym() {
  const value = useContext(GymContext)
  if (!value) throw new Error('useGym must be used inside a gym provider')
  return value
}
