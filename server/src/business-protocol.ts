import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { ApiError, uuid } from './protocol.ts';

export type Row = Record<string, string | number | null>;
export type Table = { name: string; key: string; columns: { name: string; type: string; nullable: boolean }[]; mutable: string[]; references: { column: string; table: string; key: string }[] };
export const tables = (JSON.parse(readFileSync(new URL('./business-schema.json', import.meta.url), 'utf8')) as { tables: Table[] }).tables;
export const byTable = new Map(tables.map(t => [t.name, t]));
export type Change = { table: string; id: string; before: Row | null; after: Row };
export type Batch = { protocolVersion: 2; operationId: string; deviceId: string; actorSubject: string | null; operationIds: string[]; changes: Change[] };
export const REQUEST_LIMIT = 1024 * 1024;
const MONEY_MAX = 100_000_000_000;
function fail(code = 'invalid_business_data'): never { throw new ApiError(400, code); }
function conflict(code: string): never { throw new ApiError(409, code); }
function exact(value: unknown, keys: string[]): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).length !== keys.length || keys.some(k => !Object.hasOwn(value, k))) fail('invalid_fields');
  return value as Record<string, unknown>;
}
export function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value !== null && typeof value === 'object') return `{${Object.keys(value).sort().map(k => `${JSON.stringify(k)}:${canonical((value as Record<string, unknown>)[k])}`).join(',')}}`;
  return JSON.stringify(value);
}
export function digest(value: unknown): string { return createHash('sha256').update(canonical(value)).digest('hex'); }
export function same(a: unknown, b: unknown): boolean { return canonical(a) === canonical(b); }
function int(v: unknown, min: number, max: number) { if (!Number.isSafeInteger(v) || (v as number) < min || (v as number) > max) fail(); }
function text(v: unknown, max: number, required = true) {
  if (typeof v !== 'string' || [...v].length > max || /\u0000/.test(v) || (required && !v.trim())) fail();
}
function date(v: unknown) {
  if (typeof v !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(v) || v < '1900-01-01' || v > '2200-12-31' || !Number.isFinite(Date.parse(v)) || new Date(v).toISOString().slice(0, 10) !== v) fail();
}
function instant(v: unknown) { if (typeof v !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?(?:Z|[+-]\d{2}:\d{2})$/.test(v) || !Number.isFinite(Date.parse(v))) fail(); }
function one(v: unknown, options: unknown[]) { if (!options.includes(v)) fail(); }
function json(v: unknown): any { try { if (typeof v !== 'string') fail(); return JSON.parse(v as string); } catch { return fail(); } }
export function validateRow(table: Table, value: unknown): Row {
  const r = exact(value, table.columns.map(c => c.name)) as Row;
  for (const c of table.columns) {
    const v = r[c.name];
    if (v === null && c.nullable) continue;
    if (c.type === 'integer') int(v, -Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER);
    else text(v, ['after_json', 'before_json', 'snapshot_json'].includes(c.name) ? REQUEST_LIMIT : 1024, false);
    if (c.name.endsWith('_minor')) int(v, ['payments', 'invoices', 'payment_allocations', 'expenses', 'sales', 'training_charges', 'staff_payout_items'].includes(table.name) ? 1 : 0, MONEY_MAX);
    if (c.name === 'version') int(v, 1, Number.MAX_SAFE_INTEGER);
    if (['active', 'legacy'].includes(c.name)) int(v, 0, 1);
    if (['joined_on', 'starts_on', 'ends_on', 'business_on', 'issued_on'].includes(c.name)) date(v);
    if (['created_at', 'occurred_at', 'assigned_at', 'revoked_at', 'archived_at', 'issued_at', 'deleted_at'].includes(c.name)) instant(v);
    if (c.name.endsWith('_json')) json(v);
  }
  text(String(r[table.key]), 128);
  const requiredLengths: Record<string, number> = { name: 120, phone: 40, location: 254, sku: 80, plan_name: 80, title: 254, description: 254, reason: 254, number: 254, display_name: 120, actor: 254, action: 254, entity: 80, entity_id: 128 };
  for (const [k, max] of Object.entries(requiredLengths)) if (k in r && !(k === 'phone' && table.name === 'gym_settings')) text(r[k], max);
  if (table.name === 'trainers') {
    text(r.nic, 24); if (!/^[A-Z0-9]+$/.test(String(r.nic))) fail();
    const digits=String(r.phone).replace(/[^0-9]/g,'');
    if (!/^[0-9+ ()-]+$/.test(String(r.phone)) || digits.length<7 || digits.length>15) fail();
  }
  if (table.name === 'staff_payouts') { if (!/^\d{4}-\d{2}$/.test(String(r.salary_month))) fail(); date(String(r.salary_month)+'-01'); }
  if (table.name === 'training_charges') text(r.trainer_name,120);
  if (table.name === 'plans') { text(r.name, 80); int(r.duration_months, 1, 60); }
  if ('email' in r) { text(r.email, 254, table.name === 'users'); if (r.email && (!String(r.email).includes('@') || /\s/.test(String(r.email)))) fail(); }
  if (table.name === 'gym_settings' && r.id !== 1) fail();
  if (table.name === 'users' && (r.active !== 0 || r.version !== 1)) fail('identity_reference_only');
  if (table.name === 'members' && (r.nfc_id !== null && !/^[!-~]{1,128}$/.test(String(r.nfc_id)))) fail();
  if (table.name === 'members' && ((r.archived_at === null) !== (r.archived_by_user_id === null))) fail();
  if (['nfc_cards','staff_nfc_cards'].includes(table.name) && (!/^[!-~]{1,128}$/.test(String(r.uid)) || r.uid !== String(r.uid).toUpperCase() || (r.revoked_at !== null && Date.parse(String(r.revoked_at)) < Date.parse(String(r.assigned_at))))) fail();
  if (table.name === 'member_profiles') one(r.gender,['Male','Female']);
  if (['products', 'sale_items'].includes(table.name) && r.sku !== String(r.sku).toUpperCase()) fail();
  if (table.name === 'products') int(r.reorder_level, 0, 1_000_000);
  if (table.name === 'sale_items') int(r.quantity, 1, 1_000_000);
  if (['payments', 'sales'].includes(table.name)) one(r.method, ['Cash', 'Card', 'Transfer']);
  if (table.name === 'expenses') { one(r.method, ['Cash', 'Card', 'Bank']); one(r.category, ['Operations', 'Utilities', 'Maintenance', 'Salary', 'Other']); }
  if (['attendance','staff_attendance'].includes(table.name)) { one(r.kind, ['Check-in', 'Check-out']); one(r.source, ['Manual', 'NFC']); if ((r.source === 'Manual') !== (r.card_id === null) || (r.source === 'Manual' && r.card_uid !== '') || (r.source === 'NFC' && !r.card_uid)) fail(); }
  if (table.name === 'staff_attendance') text(r.staff_name,120);
  if (table.name === 'stock_movements') { int(r.delta, -1_000_000, 1_000_000); if (r.delta === 0) fail(); one(r.kind, ['Opening', 'Adjustment', 'Sale']); if ((r.kind === 'Sale') !== (r.sale_item_id !== null) || (r.kind === 'Sale' && Number(r.delta) >= 0) || (r.kind === 'Opening' && Number(r.delta) <= 0)) fail(); }
  return r;
}
export function parseBatch(value: unknown): Batch {
  const b = exact(value, ['protocolVersion', 'operationId', 'deviceId', 'actorSubject', 'operationIds', 'changes']);
  if (b.protocolVersion !== 2 || !Array.isArray(b.changes) || b.changes.length < 1 || b.changes.length > 2000 || !Array.isArray(b.operationIds) || b.operationIds.length > 2000 || Buffer.byteLength(canonical(b)) > REQUEST_LIMIT) fail('invalid_business_batch');
  const operationIds = b.operationIds.map(uuid);
  if (new Set(operationIds).size !== operationIds.length) fail();
  const keys = new Set<string>();
  const changes = b.changes.map(v => {
    const c = exact(v, ['table', 'id', 'before', 'after']);
    const t = byTable.get(c.table as string);
    if (!t) fail('unsupported_business_table');
    if (typeof c.id !== 'string') fail();
    const after = validateRow(t, c.after);
    const before = c.before === null ? null : validateRow(t, c.before);
    if (String(after[t.key]) !== c.id || (before && String(before[t.key]) !== c.id) || keys.has(`${t.name}:${c.id}`)) fail();
    keys.add(`${t.name}:${c.id}`);
    if (before) {
      if (t.columns.some(col => !t.mutable.includes(col.name) && !same(before[col.name], after[col.name]))) fail('immutable_business_history');
      if ('version' in before && t.name !== 'users' && after.version !== Number(before.version) + 1) fail('invalid_business_version');
      if (['nfc_cards','staff_nfc_cards'].includes(t.name) && (before.revoked_at !== null || after.revoked_at === null)) fail('immutable_card_history');
      if (t.name === 'members' && before.archived_at !== null) fail('immutable_archive');
    }
    return { table: t.name, id: c.id, before, after };
  });
  return { protocolVersion: 2, operationId: uuid(b.operationId), deviceId: uuid(b.deviceId), actorSubject: b.actorSubject === null ? null : uuid(b.actorSubject), operationIds, changes };
}
export type State = Map<string, Map<string, Row>>;
export function stateFrom(rows: { table_name: string; record_id: string; data: Row }[]): State {
  const state: State = new Map(tables.map(t => [t.name, new Map()]));
  for (const r of rows) { const t = byTable.get(r.table_name); if (!t) fail(); state.get(t.name)!.set(r.record_id, validateRow(t, r.data)); }
  return state;
}
function sum(values: number[]): number { const n = values.reduce((a, b) => a + b, 0); if (!Number.isSafeInteger(n)) conflict('business_total_overflow'); return n; }
export function validateState(s: State) {
  const rows = (t: string) => [...s.get(t)!.values()];
  const get = (t: string, id: unknown) => s.get(t)!.get(String(id));
  const need = (t: string, id: unknown) => { const r = get(t, id); if (!r) conflict('business_reference_missing'); return r!; };
  const unique = (values: unknown[]) => { if (new Set(values).size !== values.length) conflict('business_unique_conflict'); };
  for (const t of tables) for (const r of rows(t.name)) for (const ref of t.references) if (r[ref.column] !== null) need(ref.table, r[ref.column]);
  unique(rows('plans').map(r => String(r.name).toLowerCase()));
  unique(rows('products').map(r => String(r.sku).toUpperCase()));
  unique(rows('users').map(r => r.subject)); unique(rows('users').map(r => String(r.email).toLowerCase()));
  unique(rows('members').filter(r => r.nfc_id !== null).map(r => String(r.nfc_id).toUpperCase()));
  const cards = rows('nfc_cards').filter(r => r.revoked_at === null);
  unique(cards.map(r => r.uid)); unique(cards.map(r => r.member_id));
  for (const r of cards) if (need('members', r.member_id).nfc_id !== r.uid) conflict('business_card_conflict');
  for (const r of rows('attendance')) if (r.source === 'NFC') { const c = need('nfc_cards', r.card_id); if (c.member_id !== r.member_id || c.uid !== r.card_uid) conflict('business_card_conflict'); }
  const staffCards=rows('staff_nfc_cards').filter(r=>r.revoked_at===null);
  unique([...cards,...staffCards].map(r=>r.uid)); unique(staffCards.map(r=>r.trainer_id));
  for (const r of rows('staff_attendance')) if (r.source==='NFC') { const c=need('staff_nfc_cards',r.card_id); if(c.trainer_id!==r.trainer_id || c.uid!==r.card_uid) conflict('business_card_conflict'); }
  for (const r of rows('member_deletions')) if(need('members',r.id).archived_at===null || cards.some(c=>c.member_id===r.id)) conflict('deleted_member_still_active');
  for (const r of rows('staff_deletions')) if(need('trainers',r.id).active!==0 || staffCards.some(c=>c.trainer_id===r.id) || rows('member_trainers').some(a=>a.trainer_id===r.id)) conflict('deleted_staff_still_active');
  const periods = rows('membership_periods');
  for (const member of new Set(periods.map(p => p.member_id))) {
    const ordered = periods.filter(p => p.member_id === member).sort((a, b) => String(a.starts_on).localeCompare(String(b.starts_on)));
    for (let i = 0; i < ordered.length; i++) if (String(ordered[i].ends_on) < String(ordered[i].starts_on) || (i && String(ordered[i - 1].ends_on) >= String(ordered[i].starts_on))) conflict('membership_overlap');
  }
  unique(rows('trainers').map(r=>r.nic));
  const payments = rows('payments'), allocations = rows('payment_allocations'), releases = rows('allocation_reversals');
  unique(payments.filter(r => r.reverses_id !== null).map(r => r.reverses_id));
  unique(rows('invoices').filter(r => r.membership_period_id !== null).map(r => r.membership_period_id));
  unique(rows('invoices').filter(r => r.sale_id !== null).map(r => r.sale_id));
  unique(rows('invoice_details').map(r => r.number)); unique(rows('payment_receipts').map(r => r.number));
  unique(releases.map(r => r.allocation_id));
  for (const r of payments) if (r.reverses_id !== null) {
    const p = need('payments', r.reverses_id);
    if (p.reverses_id !== null || ['member_id', 'member_name', 'amount_minor', 'method'].some(k => p[k] !== r[k])) conflict('invalid_payment_reversal');
    need('payment_reversal_details', r.id);
    for (const a of allocations.filter(a => a.payment_id === p.id)) if (!releases.some(x => x.allocation_id === a.id && x.payment_reversal_id === r.id)) conflict('missing_allocation_release');
  }
  for (const r of releases) { const a = need('payment_allocations', r.allocation_id), p = need('payments', r.payment_reversal_id); if (p.reverses_id !== a.payment_id) conflict('invalid_allocation_release'); }
  for (const a of allocations) { const p = need('payments', a.payment_id), i = need('invoices', a.invoice_id); if (p.reverses_id !== null || p.member_id !== i.member_id) conflict('allocation_owner_conflict'); }
  const effective = allocations.filter(a => !payments.some(p => p.reverses_id === a.payment_id) && !releases.some(r => r.allocation_id === a.id));
  for (const p of payments) if (sum(allocations.filter(a => a.payment_id === p.id).map(a => Number(a.amount_minor))) > Number(p.amount_minor)) conflict('payment_overallocated');
  for (const i of rows('invoices')) {
    need('invoice_details', i.id);
    if (sum(effective.filter(a => a.invoice_id === i.id).map(a => Number(a.amount_minor))) > Number(i.amount_minor)) conflict('invoice_overallocated');
    if (i.membership_period_id !== null) { const p = need('membership_periods', i.membership_period_id); if (p.member_id !== i.member_id || i.sale_id !== null || p.price_minor !== i.amount_minor) conflict('invoice_membership_conflict'); }
  }
  for (const p of payments) {
    const receipt = need('payment_receipts', p.id), v = json(receipt.snapshot_json);
    if (!v || typeof v !== 'object' || v.number !== receipt.number || v.payment?.id !== p.id || v.payment?.memberId !== p.member_id || v.payment?.amountMinor !== p.amount_minor || v.payment?.method !== p.method) conflict('receipt_payment_conflict');
  }
  const items = rows('sale_items'), movements = rows('stock_movements');
  unique(items.map(r => `${r.sale_id}:${r.product_id}`));
  unique(movements.filter(r => r.sale_item_id !== null).map(r => r.sale_item_id));
  for (const sale of rows('sales')) {
    if (sale.reverses_id !== null) conflict('unsupported_sale_reversal');
    const lines = items.filter(r => r.sale_id === sale.id);
    if (!lines.length || sum(lines.map(r => Number(r.quantity) * Number(r.price_minor))) !== sale.total_minor) conflict('sale_total_conflict');
    for (const line of lines) if (!movements.some(m => m.sale_item_id === line.id && m.product_id === line.product_id && m.delta === -Number(line.quantity) && m.kind === 'Sale')) conflict('sale_stock_conflict');
  }
  for (const p of rows('products')) if (sum(movements.filter(m => m.product_id === p.id).map(m => Number(m.delta))) < 0) conflict('stock_insufficient');
  for (const m of movements) if (m.sale_item_id !== null) { const i = need('sale_items', m.sale_item_id); if (i.product_id !== m.product_id || m.delta !== -Number(i.quantity)) conflict('sale_stock_conflict'); }
  for (const e of rows('expense_voids')) if (need('expenses', e.expense_id).reverses_id !== null || (e.actor_user_id === null && e.legacy_reversal_id === null)) conflict('invalid_expense_void');
  for (const e of rows('expenses')) if (e.reverses_id !== null) { const original = need('expenses', e.reverses_id); if (original.reverses_id !== null || ['title', 'category', 'amount_minor', 'method'].some(k => original[k] !== e[k])) conflict('invalid_expense_reversal'); }
  const trainingCharges=rows('training_charges');
  unique(trainingCharges.map(r=>r.invoice_id));
  for (const c of trainingCharges) {
    const invoice=need('invoices',c.invoice_id);
    if (invoice.member_id!==c.member_id || invoice.amount_minor!==c.fee_minor || invoice.membership_period_id!==null || invoice.sale_id!==null) conflict('training_invoice_conflict');
    const start=new Date(String(c.starts_on)+'T12:00:00Z');
    const month=start.getUTCMonth()+1, day=start.getUTCDate();
    const anniversary=new Date(Date.UTC(start.getUTCFullYear(),month+1,0,12));
    anniversary.setUTCDate(Math.min(day,anniversary.getUTCDate()));
    if (anniversary.getUTCDate()===day) anniversary.setUTCDate(anniversary.getUTCDate()-1);
    if (anniversary.toISOString().slice(0,10)!==c.ends_on) conflict('training_month_conflict');
  }
  for (const member of new Set(trainingCharges.map(c=>c.member_id))) {
    const ordered=trainingCharges.filter(c=>c.member_id===member).sort((a,b)=>String(a.starts_on).localeCompare(String(b.starts_on)));
    for (let i=1;i<ordered.length;i++) if (String(ordered[i-1].ends_on)>=String(ordered[i].starts_on)) conflict('training_month_overlap');
  }
  const activePayouts=rows('staff_payouts').filter(p=>!rows('expense_voids').some(v=>v.expense_id===p.expense_id) && !rows('expenses').some(e=>e.reverses_id===p.expense_id));
  unique(rows('staff_payouts').map(p=>p.expense_id));
  unique(activePayouts.filter(p=>Number(p.salary_minor)>0).map(p=>`${p.trainer_id}:${p.salary_month}`));
  unique(rows('staff_payout_items').map(i=>`${i.payout_id}:${i.allocation_id}`));
  const paidAllocationIds: unknown[]=[];
  for (const p of rows('staff_payouts')) {
    const expense=need('expenses',p.expense_id);
    const items=rows('staff_payout_items').filter(i=>i.payout_id===p.id);
    if (expense.category!=='Salary' || expense.reverses_id!==null || expense.amount_minor!==sum([Number(p.salary_minor),Number(p.training_minor)]) || Number(p.training_minor)!==sum(items.map(i=>Number(i.amount_minor)))) conflict('staff_payout_total_conflict');
    for (const item of items) {
      const a=need('payment_allocations',item.allocation_id), c=trainingCharges.find(c=>c.invoice_id===a.invoice_id);
      if (!c || c.trainer_id!==p.trainer_id || item.amount_minor!==a.amount_minor) conflict('staff_payout_owner_conflict');
      if (activePayouts.includes(p)) {
        if (!effective.includes(a)) conflict('paid_training_reversal_requires_payout_void');
        paidAllocationIds.push(a.id);
      }
    }
  }
  unique(paidAllocationIds);

}

export function applyChanges(state: State, changes: Change[]) {
  const deletedStaff = new Set(state.get('staff_deletions')!.keys());
  // A saved reversal closes its original payment. Later batches cannot append
  // allocations/releases to disguise additional spending of that payment.
  const reversed = new Set([...state.get('payments')!.values()].filter(p => p.reverses_id !== null).map(p => p.reverses_id));
  for (const c of changes) if (c.table === 'payment_allocations' && !state.get(c.table)!.has(c.id) && reversed.has(c.after.payment_id)) conflict('payment_already_reversed');
  for (const c of changes) {
    const current = state.get(c.table)!.get(c.id) ?? null;
    if (!same(current, c.before)) {
      // A baseline may include an already identical row; no history is rewritten.
      if (c.before === null && same(current, c.after)) continue;
      conflict('business_revision_conflict');
    }
    if (c.table === 'trainers' && current && deletedStaff.has(c.id) && !same(current, c.after)) conflict('deleted_staff_immutable');
    state.get(c.table)!.set(c.id, c.after);
  }
  validateState(state);
}
