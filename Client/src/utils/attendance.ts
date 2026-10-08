import type { Gender } from '../types/domain'

export interface AttendanceCountRow {
  id: string
  personId: string
  date: string
  type: 'Check-in' | 'Check-out'
  gender?: Gender | null
  voidsId?: string | null
}

export function attendanceTotals(rows: AttendanceCountRow[], today: string) {
  const voided = new Set(rows.flatMap(row => row.voidsId ? [row.voidsId] : []))
  const people = new Map<string, Gender | null>()
  for (const row of rows) {
    if (row.date !== today || row.type !== 'Check-in' || row.voidsId || voided.has(row.id) || people.has(row.personId)) continue
    people.set(row.personId, row.gender ?? null)
  }
  const genders = [...people.values()]
  return {total:people.size,male:genders.filter(g => g === 'Male').length,female:genders.filter(g => g === 'Female').length,unspecified:genders.filter(g => g === null).length}
}
