import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { applyChanges, parseBatch, stateFrom, digest, validateState } from '../src/business-protocol.ts';
const memberId = randomUUID(), planId = randomUUID();
const member = { id: memberId, name: 'Member', phone: '0771234567', email: '', nfc_id: null, joined_on: '2026-10-05', version: 1, archived_at: null, archived_by_user_id: null };
const plan = { id: planId, name: 'Monthly', duration_months: 1, price_minor: 600_000, active: 1, version: 1 };
function batch(changes: unknown[]) { return { protocolVersion: 2, operationId: randomUUID(), deviceId: randomUUID(), actorSubject: null, operationIds: [], changes }; }
const change = (table: string, after: any, before: any = null) => ({ table, id: String(after.id ?? after.payment_id ?? after.invoice_id ?? after.expense_id), before, after });
test('business row contracts are identical in native and server packages', { skip: !existsSync(new URL('../../Client/src-tauri/business-schema.json', import.meta.url)) && 'Standalone API source; native contract checked by desktop CI' }, () => {
  assert.equal(readFileSync(new URL('../src/business-schema.json', import.meta.url), 'utf8'), readFileSync(new URL('../../Client/src-tauri/business-schema.json', import.meta.url), 'utf8'));
});
test('business protocol accepts exact typed rows and stable canonical hashes; identities cannot grant roles', () => {
  const input = batch([change('members', member), change('plans', plan)]);
  assert.equal(parseBatch(input).changes.length, 2);
  assert.equal(digest(input), digest(Object.fromEntries(Object.entries(input).reverse())));
  const user = { id: randomUUID(), subject: randomUUID(), email: 'history@example.invalid', display_name: 'Historical actor', active: 1, version: 1 };
  assert.throws(() => parseBatch(batch([change('users', user)])), /identity_reference_only/);
  assert.throws(() => parseBatch({ ...input, actorRole: 'Administrator' }));
});
test('business protocol refuses fractional money, duplicate rows, immutable changes and unsafe sizes', () => {
  assert.throws(() => parseBatch(batch([change('unknown_future_table', member)])), /unsupported_business_table/);
  for (const patch of [{ price_minor: 1.5 }, { price_minor: 2 ** 53 }, { price_minor: -1 }, { duration_months: 0 }, { active: 2 }]) assert.throws(() => parseBatch(batch([change('plans', { ...plan, ...patch })])));
  assert.throws(() => parseBatch(batch([change('members', member), change('members', member)])));
  assert.throws(() => parseBatch(batch([change('members', { ...member, joined_on: '2026-10-06', version: 2 }, member)])), /immutable_business_history/);
  assert.throws(() => parseBatch(batch([change('plans', { ...plan, version: 3 }, plan)])), /invalid_business_version/);
});
test('business state scopes dependencies, detects overlap and refuses stale master writes', () => {
  const state = stateFrom([]);
  applyChanges(state, parseBatch(batch([change('members', member), change('plans', plan)])).changes);
  const p = { id: randomUUID(), member_id: memberId, plan_id: planId, plan_name: 'Monthly', price_minor: 600_000, starts_on: '2026-10-05', ends_on: '2026-11-04', created_at: '2026-10-05T00:00:00Z' };
  applyChanges(state, parseBatch(batch([change('membership_periods', p)])).changes);
  assert.throws(() => applyChanges(state, parseBatch(batch([change('membership_periods', { ...p, id: randomUUID(), starts_on: '2026-11-04' })])).changes), /membership_overlap/);
  assert.throws(() => applyChanges(state, parseBatch(batch([change('plans', { ...plan, name: 'Edited', version: 2 }, { ...plan, name: 'Different' })])).changes), /business_revision_conflict/);
  const orphan = stateFrom([]);
  assert.throws(() => applyChanges(orphan, parseBatch(batch([change('membership_periods', p)])).changes), /business_reference_missing/);
});
test('business state requires sale items and exactly matching stock, preserves invoice/payment ownership and caps', () => {
  const product = { id: randomUUID(), name: 'Bottle', sku: 'BOTTLE', cost_minor: 50, price_minor: 100, reorder_level: 1, version: 1 };
  const sale = { id: randomUUID(), total_minor: 100, method: 'Cash', business_on: '2026-10-05', created_at: '2026-10-05T00:00:00Z', actor: 'Test actor', reverses_id: null };
  const item = { id: randomUUID(), sale_id: sale.id, product_id: product.id, name: product.name, sku: product.sku, quantity: 1, price_minor: 100, cost_minor: 50 };
  const opening = { id: randomUUID(), product_id: product.id, sale_item_id: null, delta: 1, kind: 'Opening', reason: 'Opening', created_at: sale.created_at, actor: sale.actor };
  const movement = { ...opening, id: randomUUID(), sale_item_id: item.id, delta: -1, kind: 'Sale', reason: 'Retail sale' };
  const entries = [change('products', product), change('stock_movements', opening), change('sales', sale), change('sale_items', item), change('stock_movements', movement)];
  const state = stateFrom([]); applyChanges(state, parseBatch(batch(entries)).changes); validateState(state);
  assert.throws(() => applyChanges(stateFrom([]), parseBatch(batch(entries.slice(0, -1))).changes), /sale_stock_conflict/);
  assert.throws(() => applyChanges(stateFrom([]), parseBatch(batch(entries.filter(c => c.after.id !== opening.id))).changes), /stock_insufficient/);
});

test('admission settings accept bounded fees, advance versions and refuse stale changes', () => {
  const state = stateFrom([]);
  const original = { id: 1, amount_minor: 150000, version: 1 };
  applyChanges(state, parseBatch(batch([change('admission_settings', original)])).changes);
  const updated = { ...original, amount_minor: 250000, version: 2 };
  applyChanges(state, parseBatch(batch([change('admission_settings', updated, original)])).changes);
  assert.throws(() => applyChanges(state, parseBatch(batch([change('admission_settings', updated, original)])).changes), /business_revision_conflict/);
  for (const row of [{ ...original, id: 2 }, { ...original, amount_minor: -1 }, { ...original, amount_minor: 1.5 }, { ...original, amount_minor: 100000000001 }]) {
    assert.throws(() => parseBatch(batch([change('admission_settings', row)])));
  }
});
