import { useGym } from '../context/GymContext'
import type { Page } from '../types/domain'
const notes: Partial<Record<Page, string>> = {
  'Sales & Inventory': 'Sale corrections and returns are unavailable.',
}
export function DesktopScreenNotice({ page }: { page: Page }) {
  const { mode } = useGym()
  return mode === 'desktop' && notes[page] ? <p role="note" className="foundation-warning">{notes[page]}</p> : null
}
