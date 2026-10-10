export const LIVE_REFRESH_INTERVAL_MS = 5_000

// A slow request owns its slot until it finishes. Missed ticks never queue up.
export function startPolling(task: () => Promise<void>, intervalMs = LIVE_REFRESH_INTERVAL_MS, timers = globalThis) {
  let stopped = false
  let running = false
  const tick = async () => {
    if (stopped || running) return
    running = true
    try { await task() } catch { /* The task reports errors through native/app state. */ }
    finally { running = false }
  }
  const first = timers.setTimeout(() => { void tick() }, 0)
  const interval = timers.setInterval(() => { void tick() }, intervalMs)
  return () => {
    stopped = true
    timers.clearTimeout(first)
    timers.clearInterval(interval)
  }
}
