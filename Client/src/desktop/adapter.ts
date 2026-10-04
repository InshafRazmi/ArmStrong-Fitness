import type { GymData, Member } from '../types/domain'
import type { Snapshot } from './api'
export const emptyDesktopData = (): GymData => ({
  members: [], plans: [], attendance: [], payments: [], products: [], sales: [], expenses: [], audit: [], queue: [],
})
export function desktopData(snapshot: Snapshot): GymData {
  return {
    ...emptyDesktopData(),
    plans: snapshot.plans.map(plan => ({
      id: plan.id, version: plan.version, name: plan.name, durationMonths: plan.durationMonths,
      price: plan.priceMinor / 100, activeMembers: plan.activeMembers, status: plan.active ? 'Active' : 'Inactive',
    })),
    members: desktopMembers(snapshot),
    attendance: snapshot.attendance.map(row => ({
      id: row.id, memberId: row.memberId, name: row.name, date: row.businessOn,
      time: new Intl.DateTimeFormat('en-GB', { timeZone: 'Asia/Colombo', hour: '2-digit', minute: '2-digit' }).format(new Date(row.occurredAt)),
      type: row.type, source: row.source, syncState: 'pending',
    })),
    payments: snapshot.payments.map(row => ({ id: row.id, memberId: row.memberId, memberName: row.memberName, date: row.businessOn, method: row.method, amount: row.netAmountMinor / 100, status: row.status, syncState: 'pending' })),
    expenses: snapshot.expenses.map(row => ({ id: row.id, title: row.title, category: row.category, date: row.businessOn, method: row.method, amount: row.amountMinor / 100, recordedBy: row.actor, status: row.status, effectiveAmount: row.effectiveAmountMinor / 100, voidReason: row.voidReason, voidedAt: row.voidedAt, voidedBy: row.voidedBy, syncState: 'pending' })),
    products: snapshot.products.map(row => ({ id: row.id, version: row.version, name: row.name, sku: row.sku, stock: row.stock, reorderLevel: row.reorderLevel, cost: row.costMinor / 100, price: row.priceMinor / 100 })),
    sales: snapshot.sales.map(row => ({ id: row.id, date: row.businessOn, method: row.method, total: row.totalMinor / 100, syncState: 'pending', items: row.items.map(item => ({ productId: item.productId, name: item.name, quantity: item.quantity, price: item.priceMinor / 100 })) })),
    audit: snapshot.audit,
  }
}
export function desktopMembers(snapshot: Snapshot, includeArchived = false): Member[] {
  return snapshot.members.filter(member => includeArchived || member.active).map(member => {
      const periods = snapshot.periods.filter(period => period.memberId === member.id)
      const current = periods.find(period => period.status === 'Active' || period.status === 'Expiring')
      const upcoming = [...periods].filter(period => period.status === 'Scheduled').sort((a, b) => a.startsOn.localeCompare(b.startsOn))[0]
      const latest = [...periods].sort((a, b) => b.endsOn.localeCompare(a.endsOn))[0]
      const period = current ?? upcoming ?? latest
      return {
        active: member.active, archivedAt: member.archivedAt, canDelete: member.canDelete,
        id: member.id, version: member.version, name: member.name, phone: member.phone, email: member.email,
        nfcId: member.nfcId, joinedAt: member.joinedOn,
        initials: member.name.split(/\s+/).slice(0, 2).map(part => part[0]).join('').toUpperCase(),
        plan: period?.planName ?? 'No membership', expiry: period?.endsOn ?? '', status: member.active ? period?.status ?? 'No membership' : 'Archived',
      } satisfies Member
    })
}
