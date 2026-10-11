import type { NfcAttendanceOutcome } from '../desktop/api'

export interface NfcSuccess { id: string; name: string; entity: 'Staff' | 'Member'; type: 'Check-in' | 'Check-out' }

export function nfcSuccess(result: NfcAttendanceOutcome): NfcSuccess | null {
  if (result.duplicate || !result.id || !result.name || !result.type) return null
  return { id: result.id, name: result.name, entity: result.entity, type: result.type }
}

// Unlock audio in the scan's keyboard/button gesture, before waiting for SQLite.
// Audio failure must never turn a saved attendance record into a failed scan.
export function createNfcChime(factory = () => new AudioContext()) {
  let context: AudioContext | null = null
  return {
    prepare() {
      try {
        context ??= factory()
        void context.resume().catch(() => {})
      } catch { /* The visual confirmation remains available without audio. */ }
    },
    play() {
      if (!context || context.state !== 'running') return
      try {
        const start = context.currentTime
        for (const [frequency, offset] of [[660, 0], [880, 0.09]]) {
          const tone = context.createOscillator()
          const gain = context.createGain()
          tone.type = 'sine'
          tone.frequency.value = frequency
          gain.gain.setValueAtTime(0, start + offset)
          gain.gain.linearRampToValueAtTime(0.12, start + offset + 0.012)
          gain.gain.exponentialRampToValueAtTime(0.001, start + offset + 0.16)
          tone.connect(gain); gain.connect(context.destination)
          tone.onended = () => { tone.disconnect(); gain.disconnect() }
          tone.start(start + offset); tone.stop(start + offset + 0.18)
        }
      } catch { /* A missing audio device does not affect attendance. */ }
    },
    dispose() {
      void context?.close().catch(() => {})
      context = null
    },
  }
}
