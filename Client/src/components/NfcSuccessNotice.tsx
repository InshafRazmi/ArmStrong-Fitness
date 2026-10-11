import type { NfcSuccess } from '../utils/nfcFeedback'

export function NfcSuccessNotice({ success }: { success: NfcSuccess }) {
  return <div className="nfc-success-notice" role="status" aria-live="polite" key={success.id}>
    <svg className="nfc-success-check" viewBox="0 0 64 64" aria-hidden="true"><circle cx="32" cy="32" r="28"/><path d="m19 32 9 9 17-19"/></svg>
    <div><strong>{success.name}</strong><span>{success.entity} · {success.type} recorded</span></div>
  </div>
}
