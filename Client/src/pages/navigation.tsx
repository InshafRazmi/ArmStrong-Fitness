import type { ComponentType } from 'react'
import type { Page } from '../types/domain'
import { AttendancePage } from './AttendancePage'
import { DashboardPage } from './DashboardPage'
import { ExpensesPage } from './ExpensesPage'
import { InventoryPage } from './InventoryPage'
import { MembersPage } from './MembersPage'
import { MembershipsPage } from './MembershipsPage'
import { PaymentsPage } from './PaymentsPage'
import { ReportsPage } from './ReportsPage'
import { SettingsPage } from './SettingsPage'

type Navigate = (page: Page) => void
export const pages: { name: Page; icon: string; Screen: ComponentType<{ navigate: Navigate }> }[] = [
  { name: 'Dashboard', icon: 'grid', Screen: DashboardPage },
  { name: 'Members', icon: 'users', Screen: MembersPage },
  { name: 'NFC Attendance', icon: 'signal', Screen: AttendancePage },
  { name: 'Memberships', icon: 'card', Screen: MembershipsPage },
  { name: 'Payments', icon: 'money', Screen: PaymentsPage },
  { name: 'Sales & Inventory', icon: 'bag', Screen: InventoryPage },
  { name: 'Expenses', icon: 'receipt', Screen: ExpensesPage },
  { name: 'Reports', icon: 'chart', Screen: ReportsPage },
  { name: 'Settings', icon: 'settings', Screen: SettingsPage },
]

export function PageContent({ page, navigate }: { page: Page; navigate: Navigate }) {
  const entry = pages.find(entry => entry.name === page)
  if (!entry) throw new Error(`Unknown navigation page: ${page}`)
  return <entry.Screen navigate={navigate}/>
}
