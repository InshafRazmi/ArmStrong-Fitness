import { useGym } from '../context/GymContext'
import type { Page } from '../types/domain'
const notes: Partial<Record<Page, string>> = {
  'NFC Attendance': 'Attendance records are stored in SQLite. Admission/grace/freeze rules and Windows reader acceptance are not implemented.',
  Payments: 'Invoices, allocations, balances, explicit-date renewals and full payment reversals use SQLite. Automatic renewal policy is undecided. Receipt preview uses saved data; Windows print-dialog and printer acceptance remain unverified.',
  'Sales & Inventory': 'Products, sales and stock movements are stored in SQLite. Sale corrections/returns are not implemented.',
  Expenses: 'Expenses are stored in SQLite. Original expenses and reasoned voids are append-only. Removal requires an authenticated Administrator; staff login remains unimplemented.',
  Reports: 'Summaries and CSV exports read local SQLite records. Reports have no date filter yet; exports include all stored records.',
}
export function DesktopScreenNotice({ page }: { page: Page }) {
  const { mode } = useGym()
  return mode === 'desktop' && notes[page] ? <p role="note" className="foundation-warning">{notes[page]}</p> : null
}
