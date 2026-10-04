export interface Plan { id: string; version: number; name: string; durationMonths: number; priceMinor: number; active: boolean; activeMembers: number }
export interface Member { id: string; version: number; name: string; phone: string; email: string; nfcId: string; joinedOn: string; active: boolean; archivedAt: string | null; archivedByUserId: string | null; canDelete: boolean }
export interface MemberSyncConflict { id: string; memberId: string; operationId: string | null; reason: string; remote: { name?: string; phone?: string; email?: string; nfcId?: string | null; archivedAt?: string | null } | null; createdAt: string }
export interface RecordedMember { id: string; name: string; phone: string; email: string; nfcId: string | null; joinedOn: string; archivedAt: string | null; revision?: number; version?: number }
export interface MemberConflictPreview { conflictId: string; memberId: string; fingerprint: string; local: RecordedMember | null; remote: RecordedMember | null; conflicts: {id: string; reason: string; createdAt: string}[]; operations: {id: string; action: string}[]; useServer: {allowed: boolean; reason: string}; keepLocal: {allowed: boolean; reason: string} }
export interface MemberConflictInput { requestId: string; conflictId: string; fingerprint: string; choice: 'use_server' | 'keep_local'; reason: string }
export interface MemberConflictResult { resolutionId: string; memberId: string; choice: MemberConflictInput['choice']; superseded: number; retryOperationId: string | null; serverConfirmed: false }
export interface MemberSyncStatus { resolved?: number; superseded?: number; reviewAuthorization?: {allowed: boolean; reason: string}; available: boolean; reason: string; cursor: number; acknowledged: number; conflicts: MemberSyncConflict[]; retryOn?: string | null; lastError?: string | null; lastSuccessOn?: string | null }
export interface Period { id: string; memberId: string; planId: string; planName: string; priceMinor: number; startsOn: string; endsOn: string; status: 'Scheduled' | 'Active' | 'Expiring' | 'Expired' }
export interface Profile { version: number; name: string; location: string; phone: string; email: string }
export interface NativeAttendance { id: string; memberId: string; name: string; cardId: string | null; cardUid: string; type: 'Check-in' | 'Check-out'; source: 'NFC' | 'Manual'; businessOn: string; occurredAt: string; voidsId: string | null }
export interface NativePayment { id: string; memberId: string; memberName: string; amountMinor: number; method: 'Cash' | 'Card' | 'Transfer'; businessOn: string; createdAt: string; actor: string; reversesId: string | null; netAmountMinor: number; allocatedMinor: number; unallocatedMinor: number; status: 'Recorded' | 'Partly allocated' | 'Allocated' | 'Reversed' | 'Reversal'; receiptNumber: string; reversalReason: string | null }
export interface RemovalAuthorization { allowed: boolean; userId: string | null; user: string | null; reason: string }
export interface MemberRemovalInput { requestId: string; memberId: string; version: number }
export interface ExpenseVoidInput { requestId: string; expenseId: string; reason: string }
export interface NativeExpense { id: string; title: string; category: string; amountMinor: number; method: 'Cash' | 'Card' | 'Bank'; businessOn: string; createdAt: string; actor: string; reversesId: string | null; effectiveAmountMinor: number; status: 'Recorded' | 'Voided' | 'Reversal'; voidReason: string | null; voidedAt: string | null; voidedBy: string | null; voidedByUserId: string | null }
export interface NativeProduct { id: string; version: number; name: string; sku: string; costMinor: number; priceMinor: number; reorderLevel: number; stock: number }
export interface NativeSale { id: string; totalMinor: number; method: 'Cash' | 'Card' | 'Transfer'; businessOn: string; createdAt: string; actor: string; reversesId: string | null; items: { id: string; productId: string; name: string; sku: string; quantity: number; priceMinor: number; costMinor: number }[] }
export interface NativeAudit { id: string; action: string; entity: string; entityId: string; user: string; deviceId: string; beforeJson: string | null; afterJson: string; timestamp: string }
export interface NativeUser { id: string; name: string; email: string; active: number; roles: string[] }
export interface NativeInvoice { id: string; number: string; memberId: string | null; memberName: string; membershipPeriodId: string | null; amountMinor: number; paidMinor: number; outstandingMinor: number; description: string; issuedOn: string; createdAt: string; status: 'Paid' | 'Partial' | 'Unpaid' }
export interface NativeAllocation { id: string; paymentId: string; invoiceId: string; amountMinor: number; reversedBy: string | null }
export interface FinancialAccount { memberId: string; memberName: string; outstandingMinor: number; creditMinor: number; netBalanceMinor: number }
export interface InvoiceInput { requestId: string; memberId: string; membershipPeriodId: string | null; description: string; amountMinor: number }
export interface AllocationInput { requestId: string; paymentId: string; invoiceId: string; amountMinor: number }
export interface ReceivePaymentInput { requestId: string; memberId: string; amountMinor: number; method: NativePayment['method']; invoiceId: string | null }
export interface RenewalInput { requestId: string; memberId: string; planId: string; planVersion: number; expectedLastPeriodId: string | null; startsOn: string; endsOn: string }
export interface ReversalInput { requestId: string; paymentId: string; reason: string }
export interface ReceiptDocument { number: string; currentStatus: NativePayment['status']; reversalReceiptNumber: string | null; snapshot: { formatVersion: number; number: string; payment: Pick<NativePayment, 'id' | 'memberId' | 'memberName' | 'amountMinor' | 'method' | 'businessOn' | 'createdAt' | 'actor' | 'reversesId'>; gym: Omit<Profile, 'version'>; allocations: { invoiceNumber: string; description: string; amountMinor: number; outstandingMinor: number; released: boolean | 0 | 1 }[]; unallocatedAtIssueMinor: number; originalReceiptNumber: string | null; reason: string | null; issuedAt: string; legacy: boolean } }
export interface Snapshot { memberSync?: MemberSyncStatus; removalAuthorization: RemovalAuthorization; invoices: NativeInvoice[]; allocations: NativeAllocation[]; financialAccounts: FinancialAccount[]; plans: Plan[]; members: Member[]; periods: Period[]; pending: number; auditCount: number; today: string; profile: Profile; attendance: NativeAttendance[]; payments: NativePayment[]; products: NativeProduct[]; sales: NativeSale[]; expenses: NativeExpense[]; audit: NativeAudit[]; users: NativeUser[]; restoreRequiresReconciliation: boolean }
export type PlanInput = Omit<Plan, 'id' | 'version' | 'activeMembers'> & { id?: string; version?: number }
export type MemberInput = Pick<Member, 'name' | 'phone' | 'email' | 'nfcId'> & { id?: string; version?: number }
export interface PeriodInput { memberId: string; planId: string; startsOn: string; endsOn: string }
export interface ProductInput { requestId: string; id?: string; version?: number; name: string; sku: string; costMinor: number; priceMinor: number; reorderLevel: number; openingStock: number }
export interface WriteOutcome { id?: string; duplicate?: boolean }
export interface FileResult { path: string; sha256?: string; rows?: number }
export interface DeviceApproval { deviceId: string; sqlitePath: string; secretSha256: string }
export interface DesktopAuthStatus { requiresLogin: boolean; configured: boolean; authenticated: boolean; canWrite: boolean; userName: string | null; role: string | null; expiresAt: string | null; offlineUntil?: string | null; offline?: boolean; reason: string }
export interface ReportRange { fromOn?: string | null; toOn?: string | null }
export interface ReportSummary { fromOn: string | null; toOn: string | null; incomeMinor: number; expenseMinor: number; netMinor: number; inventoryValueMinor: number; attendanceCount: number; membershipPeriodCount: number; auditCount: number; lowStockCount: number }
export interface BackupEnvelope { format: 'armstrong-sqlite-backup'; formatVersion: number; schemaVersion: number; createdAt: string; sha256: string; data: number[] }
export interface BackupSummary { members: number; periods: number; payments: number; sales: number; expenses: number; pending: number; auditCount: number }
export interface RestorePreview { token: string; current: BackupSummary; backup: BackupSummary; schemaVersion: number; sha256: string }
export interface RestoreResult { recoveryPath: string; requiresReconciliation: boolean }
declare global { interface Window { __TAURI__?: { core: { invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> } } } }
export const isDesktop = () => Boolean(window.__TAURI__ || '__TAURI_INTERNALS__' in window)
function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!window.__TAURI__) return Promise.reject(new Error('Open the desktop application to use SQLite storage.'))
  return window.__TAURI__.core.invoke<T>(command, args)
}
export function requireSnapshot(value: Snapshot): Snapshot {
  const arrays = ['members', 'plans', 'periods', 'attendance', 'payments', 'products', 'sales', 'expenses', 'audit', 'users', 'invoices', 'allocations', 'financialAccounts'] as const
  if (!value || !value.removalAuthorization || typeof value.removalAuthorization.allowed !== 'boolean' || arrays.some(key => !Array.isArray(value[key])) || value.members.some(member => typeof member.active !== 'boolean') || !value.profile || !Number.isInteger(value.profile.version) || !Number.isInteger(value.pending)) throw new Error('Desktop storage returned an incompatible snapshot. Rebuild/relaunch the app; no demo data was substituted.')
  return value
}
export const snapshot = async () => requireSnapshot(await invoke<Snapshot>('foundation_snapshot'))
export const prepareNativeDevice = () => invoke<DeviceApproval>('prepare_native_device')
export const desktopAuthStatus = () => invoke<DesktopAuthStatus>('desktop_auth_status')
export const desktopLogin = (email: string, password: string) => invoke<DesktopAuthStatus>('desktop_login', { email, password })
export const desktopRenewSession = () => invoke<DesktopAuthStatus>('desktop_renew_session')
export const desktopLogout = () => invoke<void>('desktop_logout')
export const desktopUnlockOffline = () => invoke<DesktopAuthStatus>('desktop_unlock_offline')
export interface MemberSyncOutcome { state: 'complete' | 'yielded' | 'deferred' | 'failed' | 'blocked'; pushed: number; pages: number; reason: string }
export const synchronizeMembers = () => invoke<MemberSyncOutcome>('synchronize_members')
export const savePlan = (input: PlanInput) => invoke<void>('save_plan', { input })
export const saveMember = (input: MemberInput) => invoke<void>('save_member', { input })
export const addPeriod = (input: PeriodInput) => invoke<void>('add_membership_period', { input })
export const saveProfile = (input: Profile) => invoke<WriteOutcome>('save_gym_profile', { input })
export const saveProduct = (input: ProductInput) => invoke<WriteOutcome>('save_product', { input })
export const recordAttendance = (input: { requestId: string; memberOrCard: string; source: 'NFC' | 'Manual' }) => invoke<WriteOutcome>('record_attendance', { input })
export const recordPayment = (input: { requestId: string; memberId: string; amountMinor: number; method: 'Cash' | 'Card' | 'Transfer' }) => invoke<WriteOutcome>('record_payment', { input })
export const recordExpense = (input: { requestId: string; title: string; category: string; amountMinor: number; method: 'Cash' | 'Card' | 'Bank' }) => invoke<WriteOutcome>('record_expense', { input })
export const adjustStock = (input: { requestId: string; productId: string; amount: number }) => invoke<WriteOutcome>('adjust_stock', { input })
export const completeSale = (input: { requestId: string; productId: string; quantity: number; method: 'Cash' | 'Card' | 'Transfer' }) => invoke<WriteOutcome>('complete_sale', { input })
export const exportBackup = () => invoke<FileResult>('export_backup')
export const previewRestore = (input: BackupEnvelope) => invoke<RestorePreview>('preview_restore', { input })
export const restoreBackup = (token: string) => invoke<RestoreResult>('restore_backup', { token })
export const exportReport = (kind: string, range?: ReportRange) => invoke<FileResult>('export_report', { kind, range })
export const reportSummary = (range: ReportRange) => invoke<ReportSummary>('report_summary', { range })
export function minorUnits(value: number): number {
  const minor = Math.round(value * 100)
  if (!Number.isFinite(value) || !Number.isSafeInteger(minor) || value < 0 || minor > 100_000_000_000 || Math.abs(value * 100 - minor) > 0.00001) throw new Error('Enter a valid LKR amount with at most two decimal places.')
  return minor
}

export const createInvoice = (input: InvoiceInput) => invoke<WriteOutcome>('create_invoice', { input })
export const allocatePayment = (input: AllocationInput) => invoke<WriteOutcome>('allocate_payment', { input })
export const receivePayment = (input: ReceivePaymentInput) => invoke<WriteOutcome>('receive_payment', { input })
export const renewMembership = (input: RenewalInput) => invoke<WriteOutcome>('renew_membership', { input })
export const reversePayment = (input: ReversalInput) => invoke<WriteOutcome>('reverse_payment', { input })
export const paymentReceipt = (paymentId: string) => invoke<ReceiptDocument>('payment_receipt', { paymentId })

export const archiveMember = (input: MemberRemovalInput) => invoke<WriteOutcome>('archive_member', { input })
export const deleteMember = (input: MemberRemovalInput) => invoke<WriteOutcome>('delete_member', { input })
export const voidExpense = (input: ExpenseVoidInput) => invoke<WriteOutcome>('void_expense', { input })

export const previewMemberConflict = (conflictId: string) => invoke<MemberConflictPreview>('preview_member_conflict', { conflictId })
export const resolveMemberConflict = (input: MemberConflictInput) => invoke<MemberConflictResult>('resolve_member_conflict', { input })
