-- Durable row journal and immutable transaction envelopes; preserve all v6 data.
CREATE TABLE business_dirty (
 table_name TEXT NOT NULL, record_id TEXT NOT NULL, before_json TEXT,
 after_json TEXT NOT NULL CHECK(json_valid(after_json)), PRIMARY KEY(table_name,record_id)
) STRICT;
CREATE TABLE business_batches (
 id TEXT PRIMARY KEY, ordinal INTEGER NOT NULL UNIQUE,
 request_json TEXT NOT NULL CHECK(json_valid(request_json)),
 state TEXT NOT NULL CHECK(state IN ('pending','confirmed','conflict')),
 response_json TEXT, last_error TEXT,
 CHECK(response_json IS NULL OR json_valid(response_json))
) STRICT;
CREATE TABLE business_batch_operations (
 batch_id TEXT NOT NULL REFERENCES business_batches(id),
 operation_id TEXT NOT NULL UNIQUE REFERENCES outbox(id), PRIMARY KEY(batch_id,operation_id)
) STRICT;
CREATE TABLE business_cursor (id INTEGER PRIMARY KEY CHECK(id=1), sequence INTEGER NOT NULL CHECK(sequence>=0)) STRICT;
INSERT INTO business_cursor VALUES(1,0);
CREATE TRIGGER business_request_immutable BEFORE UPDATE ON business_batches
WHEN NEW.id<>OLD.id OR NEW.ordinal<>OLD.ordinal OR NEW.request_json<>OLD.request_json
 OR (OLD.state='confirmed' AND (NEW.state<>OLD.state OR NEW.response_json IS NOT OLD.response_json))
BEGIN SELECT RAISE(ABORT,'Business request and confirmed receipt are immutable'); END;
CREATE TRIGGER business_batch_no_delete BEFORE DELETE ON business_batches
BEGIN SELECT RAISE(ABORT,'Business delivery history cannot be deleted'); END;
CREATE TRIGGER business_operation_no_update BEFORE UPDATE ON business_batch_operations
BEGIN SELECT RAISE(ABORT,'Business operation mapping is immutable'); END;
CREATE TRIGGER business_operation_no_delete BEFORE DELETE ON business_batch_operations
BEGIN SELECT RAISE(ABORT,'Business operation mapping is immutable'); END;
DROP TRIGGER member_card_insert;
DROP TRIGGER member_card_update;
CREATE TRIGGER member_card_insert AFTER INSERT ON members WHEN NEW.nfc_id IS NOT NULL
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO nfc_cards VALUES(lower(hex(randomblob(16))),NEW.id,NEW.nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL); END;
CREATE TRIGGER member_card_update AFTER UPDATE OF nfc_id ON members WHEN OLD.nfc_id IS NOT NEW.nfc_id
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 UPDATE nfc_cards SET revoked_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE member_id=NEW.id AND revoked_at IS NULL;
 INSERT INTO nfc_cards SELECT lower(hex(randomblob(16))),NEW.id,NEW.nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL WHERE NEW.nfc_id IS NOT NULL;
END;
CREATE TRIGGER business_users_insert AFTER INSERT ON users
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('users',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'subject',NEW.subject,'email',NEW.email,'display_name',NEW.display_name,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_users_update AFTER UPDATE ON users
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('users',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'subject',OLD.subject,'email',OLD.email,'display_name',OLD.display_name,'active',OLD.active,'version',OLD.version),json_object('id',NEW.id,'subject',NEW.subject,'email',NEW.email,'display_name',NEW.display_name,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'users',CAST(id AS TEXT),NULL,json_object('id',id,'subject',subject,'email',email,'display_name',display_name,'active',active,'version',version) FROM users;
CREATE TRIGGER business_members_insert AFTER INSERT ON members
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('members',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'name',NEW.name,'phone',NEW.phone,'email',NEW.email,'nfc_id',NEW.nfc_id,'joined_on',NEW.joined_on,'version',NEW.version,'archived_at',NEW.archived_at,'archived_by_user_id',NEW.archived_by_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_members_update AFTER UPDATE ON members
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('members',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'name',OLD.name,'phone',OLD.phone,'email',OLD.email,'nfc_id',OLD.nfc_id,'joined_on',OLD.joined_on,'version',OLD.version,'archived_at',OLD.archived_at,'archived_by_user_id',OLD.archived_by_user_id),json_object('id',NEW.id,'name',NEW.name,'phone',NEW.phone,'email',NEW.email,'nfc_id',NEW.nfc_id,'joined_on',NEW.joined_on,'version',NEW.version,'archived_at',NEW.archived_at,'archived_by_user_id',NEW.archived_by_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'members',CAST(id AS TEXT),NULL,json_object('id',id,'name',name,'phone',phone,'email',email,'nfc_id',nfc_id,'joined_on',joined_on,'version',version,'archived_at',archived_at,'archived_by_user_id',archived_by_user_id) FROM members;
CREATE TRIGGER business_plans_insert AFTER INSERT ON plans
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('plans',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'name',NEW.name,'duration_months',NEW.duration_months,'price_minor',NEW.price_minor,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_plans_update AFTER UPDATE ON plans
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('plans',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'name',OLD.name,'duration_months',OLD.duration_months,'price_minor',OLD.price_minor,'active',OLD.active,'version',OLD.version),json_object('id',NEW.id,'name',NEW.name,'duration_months',NEW.duration_months,'price_minor',NEW.price_minor,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'plans',CAST(id AS TEXT),NULL,json_object('id',id,'name',name,'duration_months',duration_months,'price_minor',price_minor,'active',active,'version',version) FROM plans;
CREATE TRIGGER business_gym_settings_insert AFTER INSERT ON gym_settings
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('gym_settings',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'name',NEW.name,'location',NEW.location,'phone',NEW.phone,'email',NEW.email,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_gym_settings_update AFTER UPDATE ON gym_settings
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('gym_settings',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'name',OLD.name,'location',OLD.location,'phone',OLD.phone,'email',OLD.email,'version',OLD.version),json_object('id',NEW.id,'name',NEW.name,'location',NEW.location,'phone',NEW.phone,'email',NEW.email,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'gym_settings',CAST(id AS TEXT),NULL,json_object('id',id,'name',name,'location',location,'phone',phone,'email',email,'version',version) FROM gym_settings;
CREATE TRIGGER business_nfc_cards_insert AFTER INSERT ON nfc_cards
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('nfc_cards',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'member_id',NEW.member_id,'uid',NEW.uid,'assigned_at',NEW.assigned_at,'revoked_at',NEW.revoked_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_nfc_cards_update AFTER UPDATE ON nfc_cards
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('nfc_cards',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'member_id',OLD.member_id,'uid',OLD.uid,'assigned_at',OLD.assigned_at,'revoked_at',OLD.revoked_at),json_object('id',NEW.id,'member_id',NEW.member_id,'uid',NEW.uid,'assigned_at',NEW.assigned_at,'revoked_at',NEW.revoked_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'nfc_cards',CAST(id AS TEXT),NULL,json_object('id',id,'member_id',member_id,'uid',uid,'assigned_at',assigned_at,'revoked_at',revoked_at) FROM nfc_cards;
CREATE TRIGGER business_membership_periods_insert AFTER INSERT ON membership_periods
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('membership_periods',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'member_id',NEW.member_id,'plan_id',NEW.plan_id,'plan_name',NEW.plan_name,'price_minor',NEW.price_minor,'starts_on',NEW.starts_on,'ends_on',NEW.ends_on,'created_at',NEW.created_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_membership_periods_update AFTER UPDATE ON membership_periods
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('membership_periods',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'member_id',OLD.member_id,'plan_id',OLD.plan_id,'plan_name',OLD.plan_name,'price_minor',OLD.price_minor,'starts_on',OLD.starts_on,'ends_on',OLD.ends_on,'created_at',OLD.created_at),json_object('id',NEW.id,'member_id',NEW.member_id,'plan_id',NEW.plan_id,'plan_name',NEW.plan_name,'price_minor',NEW.price_minor,'starts_on',NEW.starts_on,'ends_on',NEW.ends_on,'created_at',NEW.created_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'membership_periods',CAST(id AS TEXT),NULL,json_object('id',id,'member_id',member_id,'plan_id',plan_id,'plan_name',plan_name,'price_minor',price_minor,'starts_on',starts_on,'ends_on',ends_on,'created_at',created_at) FROM membership_periods;
CREATE TRIGGER business_attendance_insert AFTER INSERT ON attendance
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('attendance',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'member_id',NEW.member_id,'card_id',NEW.card_id,'member_name',NEW.member_name,'card_uid',NEW.card_uid,'kind',NEW.kind,'source',NEW.source,'business_on',NEW.business_on,'occurred_at',NEW.occurred_at,'voids_id',NEW.voids_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_attendance_update AFTER UPDATE ON attendance
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('attendance',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'member_id',OLD.member_id,'card_id',OLD.card_id,'member_name',OLD.member_name,'card_uid',OLD.card_uid,'kind',OLD.kind,'source',OLD.source,'business_on',OLD.business_on,'occurred_at',OLD.occurred_at,'voids_id',OLD.voids_id),json_object('id',NEW.id,'member_id',NEW.member_id,'card_id',NEW.card_id,'member_name',NEW.member_name,'card_uid',NEW.card_uid,'kind',NEW.kind,'source',NEW.source,'business_on',NEW.business_on,'occurred_at',NEW.occurred_at,'voids_id',NEW.voids_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'attendance',CAST(id AS TEXT),NULL,json_object('id',id,'member_id',member_id,'card_id',card_id,'member_name',member_name,'card_uid',card_uid,'kind',kind,'source',source,'business_on',business_on,'occurred_at',occurred_at,'voids_id',voids_id) FROM attendance;
CREATE TRIGGER business_products_insert AFTER INSERT ON products
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('products',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'name',NEW.name,'sku',NEW.sku,'cost_minor',NEW.cost_minor,'price_minor',NEW.price_minor,'reorder_level',NEW.reorder_level,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_products_update AFTER UPDATE ON products
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('products',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'name',OLD.name,'sku',OLD.sku,'cost_minor',OLD.cost_minor,'price_minor',OLD.price_minor,'reorder_level',OLD.reorder_level,'version',OLD.version),json_object('id',NEW.id,'name',NEW.name,'sku',NEW.sku,'cost_minor',NEW.cost_minor,'price_minor',NEW.price_minor,'reorder_level',NEW.reorder_level,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'products',CAST(id AS TEXT),NULL,json_object('id',id,'name',name,'sku',sku,'cost_minor',cost_minor,'price_minor',price_minor,'reorder_level',reorder_level,'version',version) FROM products;
CREATE TRIGGER business_sales_insert AFTER INSERT ON sales
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('sales',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'total_minor',NEW.total_minor,'method',NEW.method,'business_on',NEW.business_on,'created_at',NEW.created_at,'actor',NEW.actor,'reverses_id',NEW.reverses_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_sales_update AFTER UPDATE ON sales
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('sales',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'total_minor',OLD.total_minor,'method',OLD.method,'business_on',OLD.business_on,'created_at',OLD.created_at,'actor',OLD.actor,'reverses_id',OLD.reverses_id),json_object('id',NEW.id,'total_minor',NEW.total_minor,'method',NEW.method,'business_on',NEW.business_on,'created_at',NEW.created_at,'actor',NEW.actor,'reverses_id',NEW.reverses_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'sales',CAST(id AS TEXT),NULL,json_object('id',id,'total_minor',total_minor,'method',method,'business_on',business_on,'created_at',created_at,'actor',actor,'reverses_id',reverses_id) FROM sales;
CREATE TRIGGER business_sale_items_insert AFTER INSERT ON sale_items
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('sale_items',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'sale_id',NEW.sale_id,'product_id',NEW.product_id,'name',NEW.name,'sku',NEW.sku,'quantity',NEW.quantity,'price_minor',NEW.price_minor,'cost_minor',NEW.cost_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_sale_items_update AFTER UPDATE ON sale_items
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('sale_items',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'sale_id',OLD.sale_id,'product_id',OLD.product_id,'name',OLD.name,'sku',OLD.sku,'quantity',OLD.quantity,'price_minor',OLD.price_minor,'cost_minor',OLD.cost_minor),json_object('id',NEW.id,'sale_id',NEW.sale_id,'product_id',NEW.product_id,'name',NEW.name,'sku',NEW.sku,'quantity',NEW.quantity,'price_minor',NEW.price_minor,'cost_minor',NEW.cost_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'sale_items',CAST(id AS TEXT),NULL,json_object('id',id,'sale_id',sale_id,'product_id',product_id,'name',name,'sku',sku,'quantity',quantity,'price_minor',price_minor,'cost_minor',cost_minor) FROM sale_items;
CREATE TRIGGER business_stock_movements_insert AFTER INSERT ON stock_movements
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('stock_movements',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'product_id',NEW.product_id,'sale_item_id',NEW.sale_item_id,'delta',NEW.delta,'kind',NEW.kind,'reason',NEW.reason,'created_at',NEW.created_at,'actor',NEW.actor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_stock_movements_update AFTER UPDATE ON stock_movements
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('stock_movements',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'product_id',OLD.product_id,'sale_item_id',OLD.sale_item_id,'delta',OLD.delta,'kind',OLD.kind,'reason',OLD.reason,'created_at',OLD.created_at,'actor',OLD.actor),json_object('id',NEW.id,'product_id',NEW.product_id,'sale_item_id',NEW.sale_item_id,'delta',NEW.delta,'kind',NEW.kind,'reason',NEW.reason,'created_at',NEW.created_at,'actor',NEW.actor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'stock_movements',CAST(id AS TEXT),NULL,json_object('id',id,'product_id',product_id,'sale_item_id',sale_item_id,'delta',delta,'kind',kind,'reason',reason,'created_at',created_at,'actor',actor) FROM stock_movements;
CREATE TRIGGER business_expenses_insert AFTER INSERT ON expenses
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('expenses',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'title',NEW.title,'category',NEW.category,'amount_minor',NEW.amount_minor,'method',NEW.method,'business_on',NEW.business_on,'created_at',NEW.created_at,'actor',NEW.actor,'reverses_id',NEW.reverses_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_expenses_update AFTER UPDATE ON expenses
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('expenses',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'title',OLD.title,'category',OLD.category,'amount_minor',OLD.amount_minor,'method',OLD.method,'business_on',OLD.business_on,'created_at',OLD.created_at,'actor',OLD.actor,'reverses_id',OLD.reverses_id),json_object('id',NEW.id,'title',NEW.title,'category',NEW.category,'amount_minor',NEW.amount_minor,'method',NEW.method,'business_on',NEW.business_on,'created_at',NEW.created_at,'actor',NEW.actor,'reverses_id',NEW.reverses_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'expenses',CAST(id AS TEXT),NULL,json_object('id',id,'title',title,'category',category,'amount_minor',amount_minor,'method',method,'business_on',business_on,'created_at',created_at,'actor',actor,'reverses_id',reverses_id) FROM expenses;
CREATE TRIGGER business_expense_voids_insert AFTER INSERT ON expense_voids
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('expense_voids',CAST(NEW.expense_id AS TEXT),NULL,json_object('expense_id',NEW.expense_id,'reason',NEW.reason,'created_at',NEW.created_at,'actor',NEW.actor,'actor_user_id',NEW.actor_user_id,'legacy_reversal_id',NEW.legacy_reversal_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_expense_voids_update AFTER UPDATE ON expense_voids
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('expense_voids',CAST(NEW.expense_id AS TEXT),json_object('expense_id',OLD.expense_id,'reason',OLD.reason,'created_at',OLD.created_at,'actor',OLD.actor,'actor_user_id',OLD.actor_user_id,'legacy_reversal_id',OLD.legacy_reversal_id),json_object('expense_id',NEW.expense_id,'reason',NEW.reason,'created_at',NEW.created_at,'actor',NEW.actor,'actor_user_id',NEW.actor_user_id,'legacy_reversal_id',NEW.legacy_reversal_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'expense_voids',CAST(expense_id AS TEXT),NULL,json_object('expense_id',expense_id,'reason',reason,'created_at',created_at,'actor',actor,'actor_user_id',actor_user_id,'legacy_reversal_id',legacy_reversal_id) FROM expense_voids;
CREATE TRIGGER business_invoices_insert AFTER INSERT ON invoices
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('invoices',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'member_id',NEW.member_id,'membership_period_id',NEW.membership_period_id,'sale_id',NEW.sale_id,'amount_minor',NEW.amount_minor,'issued_on',NEW.issued_on,'created_at',NEW.created_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_invoices_update AFTER UPDATE ON invoices
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('invoices',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'member_id',OLD.member_id,'membership_period_id',OLD.membership_period_id,'sale_id',OLD.sale_id,'amount_minor',OLD.amount_minor,'issued_on',OLD.issued_on,'created_at',OLD.created_at),json_object('id',NEW.id,'member_id',NEW.member_id,'membership_period_id',NEW.membership_period_id,'sale_id',NEW.sale_id,'amount_minor',NEW.amount_minor,'issued_on',NEW.issued_on,'created_at',NEW.created_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'invoices',CAST(id AS TEXT),NULL,json_object('id',id,'member_id',member_id,'membership_period_id',membership_period_id,'sale_id',sale_id,'amount_minor',amount_minor,'issued_on',issued_on,'created_at',created_at) FROM invoices;
CREATE TRIGGER business_invoice_details_insert AFTER INSERT ON invoice_details
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('invoice_details',CAST(NEW.invoice_id AS TEXT),NULL,json_object('invoice_id',NEW.invoice_id,'number',NEW.number,'description',NEW.description,'member_name',NEW.member_name,'actor',NEW.actor,'legacy',NEW.legacy))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_invoice_details_update AFTER UPDATE ON invoice_details
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('invoice_details',CAST(NEW.invoice_id AS TEXT),json_object('invoice_id',OLD.invoice_id,'number',OLD.number,'description',OLD.description,'member_name',OLD.member_name,'actor',OLD.actor,'legacy',OLD.legacy),json_object('invoice_id',NEW.invoice_id,'number',NEW.number,'description',NEW.description,'member_name',NEW.member_name,'actor',NEW.actor,'legacy',NEW.legacy))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'invoice_details',CAST(invoice_id AS TEXT),NULL,json_object('invoice_id',invoice_id,'number',number,'description',description,'member_name',member_name,'actor',actor,'legacy',legacy) FROM invoice_details;
CREATE TRIGGER business_payments_insert AFTER INSERT ON payments
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payments',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'member_id',NEW.member_id,'member_name',NEW.member_name,'amount_minor',NEW.amount_minor,'method',NEW.method,'business_on',NEW.business_on,'created_at',NEW.created_at,'actor',NEW.actor,'reverses_id',NEW.reverses_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_payments_update AFTER UPDATE ON payments
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payments',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'member_id',OLD.member_id,'member_name',OLD.member_name,'amount_minor',OLD.amount_minor,'method',OLD.method,'business_on',OLD.business_on,'created_at',OLD.created_at,'actor',OLD.actor,'reverses_id',OLD.reverses_id),json_object('id',NEW.id,'member_id',NEW.member_id,'member_name',NEW.member_name,'amount_minor',NEW.amount_minor,'method',NEW.method,'business_on',NEW.business_on,'created_at',NEW.created_at,'actor',NEW.actor,'reverses_id',NEW.reverses_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'payments',CAST(id AS TEXT),NULL,json_object('id',id,'member_id',member_id,'member_name',member_name,'amount_minor',amount_minor,'method',method,'business_on',business_on,'created_at',created_at,'actor',actor,'reverses_id',reverses_id) FROM payments;
CREATE TRIGGER business_payment_allocations_insert AFTER INSERT ON payment_allocations
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payment_allocations',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'payment_id',NEW.payment_id,'invoice_id',NEW.invoice_id,'amount_minor',NEW.amount_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_payment_allocations_update AFTER UPDATE ON payment_allocations
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payment_allocations',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'payment_id',OLD.payment_id,'invoice_id',OLD.invoice_id,'amount_minor',OLD.amount_minor),json_object('id',NEW.id,'payment_id',NEW.payment_id,'invoice_id',NEW.invoice_id,'amount_minor',NEW.amount_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'payment_allocations',CAST(id AS TEXT),NULL,json_object('id',id,'payment_id',payment_id,'invoice_id',invoice_id,'amount_minor',amount_minor) FROM payment_allocations;
CREATE TRIGGER business_payment_reversal_details_insert AFTER INSERT ON payment_reversal_details
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payment_reversal_details',CAST(NEW.payment_id AS TEXT),NULL,json_object('payment_id',NEW.payment_id,'reason',NEW.reason))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_payment_reversal_details_update AFTER UPDATE ON payment_reversal_details
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payment_reversal_details',CAST(NEW.payment_id AS TEXT),json_object('payment_id',OLD.payment_id,'reason',OLD.reason),json_object('payment_id',NEW.payment_id,'reason',NEW.reason))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'payment_reversal_details',CAST(payment_id AS TEXT),NULL,json_object('payment_id',payment_id,'reason',reason) FROM payment_reversal_details;
CREATE TRIGGER business_allocation_reversals_insert AFTER INSERT ON allocation_reversals
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('allocation_reversals',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'allocation_id',NEW.allocation_id,'payment_reversal_id',NEW.payment_reversal_id,'created_at',NEW.created_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_allocation_reversals_update AFTER UPDATE ON allocation_reversals
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('allocation_reversals',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'allocation_id',OLD.allocation_id,'payment_reversal_id',OLD.payment_reversal_id,'created_at',OLD.created_at),json_object('id',NEW.id,'allocation_id',NEW.allocation_id,'payment_reversal_id',NEW.payment_reversal_id,'created_at',NEW.created_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'allocation_reversals',CAST(id AS TEXT),NULL,json_object('id',id,'allocation_id',allocation_id,'payment_reversal_id',payment_reversal_id,'created_at',created_at) FROM allocation_reversals;
CREATE TRIGGER business_payment_receipts_insert AFTER INSERT ON payment_receipts
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payment_receipts',CAST(NEW.payment_id AS TEXT),NULL,json_object('payment_id',NEW.payment_id,'number',NEW.number,'snapshot_json',NEW.snapshot_json,'issued_at',NEW.issued_at,'legacy',NEW.legacy))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_payment_receipts_update AFTER UPDATE ON payment_receipts
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('payment_receipts',CAST(NEW.payment_id AS TEXT),json_object('payment_id',OLD.payment_id,'number',OLD.number,'snapshot_json',OLD.snapshot_json,'issued_at',OLD.issued_at,'legacy',OLD.legacy),json_object('payment_id',NEW.payment_id,'number',NEW.number,'snapshot_json',NEW.snapshot_json,'issued_at',NEW.issued_at,'legacy',NEW.legacy))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'payment_receipts',CAST(payment_id AS TEXT),NULL,json_object('payment_id',payment_id,'number',number,'snapshot_json',snapshot_json,'issued_at',issued_at,'legacy',legacy) FROM payment_receipts;
CREATE TRIGGER business_audit_insert AFTER INSERT ON audit
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('audit',CAST(NEW.id AS TEXT),NULL,json_object('id',NEW.id,'actor',NEW.actor,'device_id',NEW.device_id,'action',NEW.action,'entity_id',NEW.entity_id,'before_json',NEW.before_json,'after_json',NEW.after_json,'created_at',NEW.created_at,'entity',NEW.entity,'actor_user_id',NEW.actor_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
CREATE TRIGGER business_audit_update AFTER UPDATE ON audit
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('audit',CAST(NEW.id AS TEXT),json_object('id',OLD.id,'actor',OLD.actor,'device_id',OLD.device_id,'action',OLD.action,'entity_id',OLD.entity_id,'before_json',OLD.before_json,'after_json',OLD.after_json,'created_at',OLD.created_at,'entity',OLD.entity,'actor_user_id',OLD.actor_user_id),json_object('id',NEW.id,'actor',NEW.actor,'device_id',NEW.device_id,'action',NEW.action,'entity_id',NEW.entity_id,'before_json',NEW.before_json,'after_json',NEW.after_json,'created_at',NEW.created_at,'entity',NEW.entity,'actor_user_id',NEW.actor_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
INSERT INTO business_dirty SELECT 'audit',CAST(id AS TEXT),NULL,json_object('id',id,'actor',actor,'device_id',device_id,'action',action,'entity_id',entity_id,'before_json',before_json,'after_json',after_json,'created_at',created_at,'entity',entity,'actor_user_id',actor_user_id) FROM audit;
-- Verified historical download may contain attendance/periods preceding archive,
-- and legacy reversal rows. Ordinary local commands retain all original guards.
DROP TRIGGER archived_membership_insert;
DROP TRIGGER archived_attendance_insert;
CREATE TRIGGER archived_membership_insert BEFORE INSERT ON membership_periods WHEN
 COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND EXISTS(SELECT 1 FROM members WHERE id=NEW.member_id AND archived_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Archived members cannot receive new memberships'); END;
CREATE TRIGGER archived_attendance_insert BEFORE INSERT ON attendance WHEN
 COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND EXISTS(SELECT 1 FROM members WHERE id=NEW.member_id AND archived_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Archived members cannot record attendance'); END;
DROP TRIGGER expenses_new_reversal;
CREATE TRIGGER expenses_new_reversal BEFORE INSERT ON expenses WHEN NEW.reverses_id IS NOT NULL
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN SELECT RAISE(ABORT,'Use the audited expense void workflow'); END;
DROP TRIGGER expense_void_parent;
CREATE TRIGGER expense_void_parent BEFORE INSERT ON expense_voids WHEN
 NOT EXISTS(SELECT 1 FROM expenses WHERE id=NEW.expense_id AND reverses_id IS NULL)
 OR (COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND (NEW.legacy_reversal_id IS NOT NULL OR EXISTS(SELECT 1 FROM expenses WHERE reverses_id=NEW.expense_id)))
BEGIN SELECT RAISE(ABORT,'Only an original, unvoided expense may be voided'); END;
PRAGMA user_version=7;
