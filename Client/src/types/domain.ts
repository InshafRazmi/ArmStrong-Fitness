export type Page = 'Dashboard' | 'Members' | 'NFC Attendance' | 'Memberships' | 'Payments' | 'Sales & Inventory' | 'Expenses' | 'Reports' | 'Settings'
export type MemberStatus = 'Active' | 'Expiring' | 'Expired' | 'Frozen' | 'Scheduled' | 'No membership'|'Archived'
export type SyncState = 'synced' | 'pending' | 'failed'

export interface Member { active?:boolean; archivedAt?:string|null; canDelete?:boolean; id:string; version?:number; name:string; phone:string; email:string; plan:string; expiry:string; membershipStartsOn?:string; status:MemberStatus; initials:string; nfcId:string; joinedAt:string }
export interface Attendance { id:string; memberId:string; name:string; time:string; date:string; type:'Check-in'|'Check-out'; source:'NFC'|'Manual'; syncState:SyncState }
export interface MembershipPlan { id:string; version?:number; name:string; durationMonths:number; price:number; activeMembers:number; status:'Active'|'Inactive' }
export interface Payment { id:string; requestId?:string; memberId:string; memberName:string; date:string; method:'Cash'|'Card'|'Transfer'; amount:number; status:'Paid'|'Partial'|'Recorded'|'Partly allocated'|'Allocated'|'Reversed'|'Reversal'; syncState:SyncState }
export interface Product { id:string; version?:number; name:string; sku:string; stock:number; reorderLevel:number; cost:number; price:number }
export interface Sale { id:string; date:string; items:{productId:string; name:string; quantity:number; price:number}[]; total:number; method:'Cash'|'Card'|'Transfer'; syncState:SyncState }
export interface Expense { status?:'Recorded'|'Voided'|'Reversal'; effectiveAmount?:number; voidReason?:string|null; voidedAt?:string|null; voidedBy?:string|null; id:string; requestId?:string; title:string; category:string; date:string; method:'Cash'|'Card'|'Bank'; amount:number; recordedBy:string; syncState:SyncState }
export interface AuditEntry { id:string; action:string; entity:string; entityId:string; user:string; timestamp:string }
export interface SyncOperation { id:string; entity:string; action:'create'|'update'|'delete'; payload:unknown; createdAt:string; attempts:number }
export interface GymData { members:Member[]; attendance:Attendance[]; plans:MembershipPlan[]; payments:Payment[]; products:Product[]; sales:Sale[]; expenses:Expense[]; audit:AuditEntry[]; queue:SyncOperation[] }

export interface ToastMessage { id:string; message:string; tone:'success'|'error'|'info' }
