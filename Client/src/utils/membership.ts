const DAY_MS = 86_400_000

function calendarDate(value: string): Date | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || value < '1900-01-01' || value > '2200-12-31') return null
  const date = new Date(`${value}T00:00:00Z`)
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value ? date : null
}

export function colomboToday(now = new Date()): string {
  const parts = new Intl.DateTimeFormat('en-GB', { timeZone: 'Asia/Colombo', year: 'numeric', month: '2-digit', day: '2-digit' }).formatToParts(now)
  const part = (name: string) => parts.find(value => value.type === name)!.value
  return `${part('year')}-${part('month')}-${part('day')}`
}

// The end is inclusive: Oct 5 + one month ends Nov 4. If the anniversary
// doesn't exist (Jan 31 -> February), the last day of that month is valid.
export function membershipEndDate(startsOn: string, durationMonths: number): string {
  const start = calendarDate(startsOn)
  if (!start || !Number.isInteger(durationMonths) || durationMonths < 1 || durationMonths > 60) return ''
  const target = new Date(Date.UTC(start.getUTCFullYear(), start.getUTCMonth() + durationMonths, 1))
  const lastDay = new Date(Date.UTC(target.getUTCFullYear(), target.getUTCMonth() + 1, 0)).getUTCDate()
  target.setUTCDate(Math.min(start.getUTCDate() - 1, lastDay))
  const result = target.toISOString().slice(0, 10)
  return calendarDate(result) ? result : ''
}

export function membershipDaysRemaining(endsOn: string, today: string, startsOn?: string): number | null {
  const end = calendarDate(endsOn)
  const day = calendarDate(today)
  const start = startsOn ? calendarDate(startsOn) : null
  if (!end || !day || (startsOn && !start) || (start && end < start)) return null
  // A scheduled package has its full duration left, without counting the wait.
  const from = start && start > day ? start : day
  return Math.max(0, Math.round((end.getTime() - from.getTime()) / DAY_MS) + 1)
}
