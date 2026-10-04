import type { ReceiptDocument } from './api'
import { displayDate, money } from '../utils/format'
import './finance.css'

// Only saved fields appear here. Reprints never regenerate a receipt number or write a payment.
export function ReceiptView({ document }: { document: ReceiptDocument }) {
  const saved = document.snapshot
  const payment = saved.payment
  const reversal = Boolean(payment.reversesId)
  return <article className="finance-receipt">
    <h3>{saved.gym.name || 'Armstrong Fitness'}</h3>
    <p>{saved.gym.location}<br/>{saved.gym.phone}<br/>{saved.gym.email}</p>
    <h4>{reversal ? 'Payment reversal' : 'Payment receipt'}</h4>
    <p className="finance-number">{document.number}</p>
    {document.currentStatus === 'Reversed' && <p className="receipt-status">REVERSED — original record retained</p>}
    {document.reversalReceiptNumber && <p className="finance-number">Reversal receipt: {document.reversalReceiptNumber}</p>}
    {saved.legacy && <p>Legacy received-amount record. This document was created during migration; original receipt issuance is unknown.</p>}
    <dl><dt>Member</dt><dd>{payment.memberName}</dd><dt>Member ID</dt><dd>{payment.memberId}</dd>
      <dt>Business date</dt><dd>{displayDate(payment.businessOn)} · Asia/Colombo</dd>
      <dt>Payment method</dt><dd>{payment.method}</dd><dt>{reversal ? 'Reversed amount' : 'Received amount'}</dt><dd>{money(payment.amountMinor / 100)}</dd>
      <dt>Recorded by</dt><dd>{payment.actor}</dd></dl>
    {saved.originalReceiptNumber && <p className="finance-number">Original receipt: {saved.originalReceiptNumber}</p>}
    {saved.reason && <p>Reason: {saved.reason}</p>}
    <h4>{reversal ? 'Allocation releases at reversal' : 'Allocations at issue'}</h4>
    {saved.allocations.length ? saved.allocations.map((line, index) => <div className="receipt-allocation" key={index}>
      <p className="finance-number">{line.invoiceNumber}</p><p>{line.description}</p>
      <p>{line.released ? 'Released' : 'Applied'}: {money(line.amountMinor / 100)}</p>
      <p>Invoice outstanding at issue: {money(line.outstandingMinor / 100)}</p>
    </div>) : <p>No invoice allocations at issue.</p>}
    {!reversal && <p>Unallocated credit at issue: {money(saved.unallocatedAtIssueMinor / 100)}</p>}
    <p className="receipt-footnote">Allocation figures are the saved issue-time snapshot. Current balances are shown in Payments.</p>
    {reversal && <p className="receipt-footnote">Accounting reversal only. This document does not confirm a bank or card refund.</p>}
    <p className="receipt-footnote">Document saved: {saved.issuedAt}<br/>Payment recorded: {payment.createdAt}</p>
  </article>
}
