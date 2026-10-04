CREATE TABLE roles (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE COLLATE NOCASE) STRICT;
CREATE TABLE users (
 id TEXT PRIMARY KEY, subject TEXT NOT NULL UNIQUE, email TEXT NOT NULL UNIQUE COLLATE NOCASE,
 display_name TEXT NOT NULL, active INTEGER NOT NULL CHECK(active IN (0,1)), version INTEGER NOT NULL CHECK(version>0)
) STRICT;
CREATE TABLE user_roles (user_id TEXT NOT NULL REFERENCES users(id), role_id TEXT NOT NULL REFERENCES roles(id), PRIMARY KEY(user_id,role_id)) STRICT;
CREATE TABLE gym_settings (
 id INTEGER PRIMARY KEY CHECK(id=1), name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
 location TEXT NOT NULL CHECK(length(trim(location)) BETWEEN 1 AND 254), phone TEXT NOT NULL CHECK(length(phone)<=40),
 email TEXT NOT NULL CHECK(length(email)<=254), version INTEGER NOT NULL CHECK(version>0)
) STRICT;
INSERT INTO gym_settings VALUES(1,'Armstrong Fitness','Matale, Sri Lanka','','',1);
CREATE TABLE nfc_cards (
 id TEXT PRIMARY KEY, member_id TEXT NOT NULL REFERENCES members(id),
 uid TEXT NOT NULL COLLATE NOCASE CHECK(length(uid) BETWEEN 1 AND 128), assigned_at TEXT NOT NULL,
 revoked_at TEXT CHECK(revoked_at IS NULL OR revoked_at>=assigned_at)
) STRICT;
CREATE UNIQUE INDEX nfc_active_uid ON nfc_cards(uid) WHERE revoked_at IS NULL;
CREATE UNIQUE INDEX nfc_active_member ON nfc_cards(member_id) WHERE revoked_at IS NULL;
CREATE INDEX nfc_member_history ON nfc_cards(member_id,assigned_at);
INSERT INTO nfc_cards SELECT 'migrated-'||id,id,nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL FROM members WHERE nfc_id IS NOT NULL;
CREATE TRIGGER member_card_insert AFTER INSERT ON members WHEN NEW.nfc_id IS NOT NULL
BEGIN INSERT INTO nfc_cards VALUES(lower(hex(randomblob(16))),NEW.id,NEW.nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL); END;
CREATE TRIGGER member_card_update AFTER UPDATE OF nfc_id ON members WHEN OLD.nfc_id IS NOT NEW.nfc_id
BEGIN
 UPDATE nfc_cards SET revoked_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE member_id=NEW.id AND revoked_at IS NULL;
 INSERT INTO nfc_cards SELECT lower(hex(randomblob(16))),NEW.id,NEW.nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL WHERE NEW.nfc_id IS NOT NULL;
END;
CREATE TABLE local_operations (
 request_id TEXT PRIMARY KEY, command TEXT NOT NULL, request_json TEXT NOT NULL CHECK(json_valid(request_json)),
 result_json TEXT NOT NULL CHECK(json_valid(result_json)), created_at TEXT NOT NULL
) STRICT;
CREATE TABLE attendance (
 id TEXT PRIMARY KEY, member_id TEXT NOT NULL REFERENCES members(id), card_id TEXT REFERENCES nfc_cards(id),
 member_name TEXT NOT NULL, card_uid TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('Check-in','Check-out')),
 source TEXT NOT NULL CHECK(source IN ('NFC','Manual')), business_on TEXT NOT NULL, occurred_at TEXT NOT NULL,
 voids_id TEXT UNIQUE REFERENCES attendance(id), CHECK((source='Manual' AND card_id IS NULL) OR (source='NFC' AND card_id IS NOT NULL))
) STRICT;
CREATE INDEX attendance_member_day ON attendance(member_id,business_on,occurred_at DESC);
CREATE INDEX attendance_day ON attendance(business_on);
CREATE TRIGGER attendance_card_owner BEFORE INSERT ON attendance WHEN NEW.source='NFC' AND NOT EXISTS(
 SELECT 1 FROM nfc_cards WHERE id=NEW.card_id AND member_id=NEW.member_id AND uid=NEW.card_uid)
BEGIN SELECT RAISE(ABORT,'Attendance card must belong to the recorded member'); END;
CREATE TABLE payments (
 id TEXT PRIMARY KEY, member_id TEXT NOT NULL REFERENCES members(id), member_name TEXT NOT NULL,
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 100000000000), method TEXT NOT NULL CHECK(method IN ('Cash','Card','Transfer')),
 business_on TEXT NOT NULL, created_at TEXT NOT NULL, actor TEXT NOT NULL, reverses_id TEXT UNIQUE REFERENCES payments(id)
) STRICT;
CREATE INDEX payment_member_day ON payments(member_id,business_on);
CREATE INDEX payment_day ON payments(business_on);
CREATE TABLE products (
 id TEXT PRIMARY KEY, name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120), sku TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(sku)) BETWEEN 1 AND 80),
 cost_minor INTEGER NOT NULL CHECK(cost_minor BETWEEN 0 AND 100000000000), price_minor INTEGER NOT NULL CHECK(price_minor BETWEEN 0 AND 100000000000),
 reorder_level INTEGER NOT NULL CHECK(reorder_level BETWEEN 0 AND 1000000), version INTEGER NOT NULL CHECK(version>0)
) STRICT;
CREATE TABLE sales (
 id TEXT PRIMARY KEY, total_minor INTEGER NOT NULL CHECK(total_minor BETWEEN 1 AND 100000000000),
 method TEXT NOT NULL CHECK(method IN ('Cash','Card','Transfer')), business_on TEXT NOT NULL, created_at TEXT NOT NULL,
 actor TEXT NOT NULL, reverses_id TEXT UNIQUE REFERENCES sales(id)
) STRICT;
CREATE INDEX sale_day ON sales(business_on);
CREATE TABLE sale_items (
 id TEXT PRIMARY KEY, sale_id TEXT NOT NULL REFERENCES sales(id), product_id TEXT NOT NULL REFERENCES products(id),
 name TEXT NOT NULL, sku TEXT NOT NULL, quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 1000000),
 price_minor INTEGER NOT NULL CHECK(price_minor BETWEEN 0 AND 100000000000), cost_minor INTEGER NOT NULL CHECK(cost_minor BETWEEN 0 AND 100000000000),
 UNIQUE(sale_id,product_id)
) STRICT;
CREATE INDEX sale_item_product ON sale_items(product_id);
CREATE TABLE stock_movements (
 id TEXT PRIMARY KEY, product_id TEXT NOT NULL REFERENCES products(id), sale_item_id TEXT UNIQUE REFERENCES sale_items(id),
 delta INTEGER NOT NULL CHECK(delta BETWEEN -1000000 AND 1000000 AND delta<>0), kind TEXT NOT NULL CHECK(kind IN ('Opening','Adjustment','Sale')),
 reason TEXT NOT NULL CHECK(length(trim(reason))>0), created_at TEXT NOT NULL, actor TEXT NOT NULL,
 CHECK((kind='Sale' AND delta<0 AND sale_item_id IS NOT NULL) OR (kind='Opening' AND delta>0 AND sale_item_id IS NULL) OR (kind='Adjustment' AND sale_item_id IS NULL))
) STRICT;
CREATE INDEX stock_product_time ON stock_movements(product_id,created_at);
CREATE TRIGGER stock_nonnegative BEFORE INSERT ON stock_movements
WHEN COALESCE((SELECT SUM(delta) FROM stock_movements WHERE product_id=NEW.product_id),0)+NEW.delta<0
BEGIN SELECT RAISE(ABORT,'Not enough stock'); END;
CREATE TRIGGER stock_sale_link BEFORE INSERT ON stock_movements WHEN NEW.kind='Sale' AND NOT EXISTS(
 SELECT 1 FROM sale_items WHERE id=NEW.sale_item_id AND product_id=NEW.product_id AND quantity=-NEW.delta)
BEGIN SELECT RAISE(ABORT,'Stock movement must match sale item'); END;
CREATE TABLE expenses (
 id TEXT PRIMARY KEY, title TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 254),
 category TEXT NOT NULL CHECK(category IN ('Operations','Utilities','Maintenance','Salary','Other')),
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 100000000000), method TEXT NOT NULL CHECK(method IN ('Cash','Card','Bank')),
 business_on TEXT NOT NULL, created_at TEXT NOT NULL, actor TEXT NOT NULL, reverses_id TEXT UNIQUE REFERENCES expenses(id)
) STRICT;
CREATE INDEX expense_day_category ON expenses(business_on,category);
CREATE TABLE invoices (
 id TEXT PRIMARY KEY, member_id TEXT REFERENCES members(id), membership_period_id TEXT UNIQUE REFERENCES membership_periods(id),
 sale_id TEXT UNIQUE REFERENCES sales(id), amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 100000000000),
 issued_on TEXT NOT NULL, created_at TEXT NOT NULL,
 CHECK(membership_period_id IS NULL OR (member_id IS NOT NULL AND sale_id IS NULL))
) STRICT;
CREATE INDEX invoice_member ON invoices(member_id);
CREATE TABLE payment_allocations (
 id TEXT PRIMARY KEY, payment_id TEXT NOT NULL REFERENCES payments(id), invoice_id TEXT NOT NULL REFERENCES invoices(id),
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 100000000000), UNIQUE(payment_id,invoice_id)
) STRICT;
CREATE INDEX allocation_invoice ON payment_allocations(invoice_id);
CREATE TRIGGER allocation_limits BEFORE INSERT ON payment_allocations WHEN
 NEW.amount_minor+COALESCE((SELECT SUM(amount_minor) FROM payment_allocations WHERE payment_id=NEW.payment_id),0)>(SELECT amount_minor FROM payments WHERE id=NEW.payment_id)
 OR NEW.amount_minor+COALESCE((SELECT SUM(amount_minor) FROM payment_allocations WHERE invoice_id=NEW.invoice_id),0)>(SELECT amount_minor FROM invoices WHERE id=NEW.invoice_id)
 OR NOT EXISTS(SELECT 1 FROM payments p JOIN invoices i ON p.member_id=i.member_id WHERE p.id=NEW.payment_id AND i.id=NEW.invoice_id AND p.reverses_id IS NULL)
BEGIN SELECT RAISE(ABORT,'Invalid allocation or amount exceeds payment/invoice'); END;
CREATE TRIGGER invoice_period_member BEFORE INSERT ON invoices WHEN NEW.membership_period_id IS NOT NULL AND NOT EXISTS(
 SELECT 1 FROM membership_periods WHERE id=NEW.membership_period_id AND member_id=NEW.member_id)
BEGIN SELECT RAISE(ABORT,'Invoice membership belongs to another member'); END;
ALTER TABLE audit ADD COLUMN entity TEXT NOT NULL DEFAULT 'legacy';
ALTER TABLE audit ADD COLUMN actor_user_id TEXT REFERENCES users(id);
UPDATE audit SET entity=substr(action,instr(action,' ')+1);
CREATE INDEX audit_entity_time ON audit(entity,entity_id,created_at);
CREATE INDEX outbox_time ON outbox(created_at,id);
CREATE INDEX membership_plan_dates ON membership_periods(plan_id,starts_on,ends_on);
CREATE TRIGGER nfc_history_delete BEFORE DELETE ON nfc_cards BEGIN SELECT RAISE(ABORT,'Card history cannot be deleted'); END;
CREATE TRIGGER nfc_history_update BEFORE UPDATE ON nfc_cards WHEN OLD.revoked_at IS NOT NULL OR NEW.id<>OLD.id OR NEW.member_id<>OLD.member_id OR NEW.uid<>OLD.uid OR NEW.assigned_at<>OLD.assigned_at OR NEW.revoked_at IS NULL
BEGIN SELECT RAISE(ABORT,'Only revoking an active card link is allowed'); END;
CREATE TRIGGER outbox_immutable BEFORE UPDATE ON outbox WHEN NEW.id<>OLD.id OR NEW.device_id<>OLD.device_id OR NEW.entity<>OLD.entity OR NEW.entity_id<>OLD.entity_id OR NEW.action<>OLD.action OR NEW.expected_version IS NOT OLD.expected_version OR NEW.payload_json<>OLD.payload_json OR NEW.schema_version<>OLD.schema_version OR NEW.created_at<>OLD.created_at
BEGIN SELECT RAISE(ABORT,'Pending operation payload is immutable'); END;
CREATE TRIGGER outbox_no_local_ack BEFORE DELETE ON outbox BEGIN SELECT RAISE(ABORT,'Authenticated server acknowledgement required'); END;
CREATE TRIGGER plan_minor_insert BEFORE INSERT ON plans WHEN typeof(NEW.price_minor)<>'integer' OR typeof(NEW.duration_months)<>'integer'
BEGIN SELECT RAISE(ABORT,'Price and duration must be integers'); END;
CREATE TRIGGER plan_minor_update BEFORE UPDATE ON plans WHEN typeof(NEW.price_minor)<>'integer' OR typeof(NEW.duration_months)<>'integer'
BEGIN SELECT RAISE(ABORT,'Price and duration must be integers'); END;
CREATE TRIGGER membership_minor_insert BEFORE INSERT ON membership_periods WHEN typeof(NEW.price_minor)<>'integer'
BEGIN SELECT RAISE(ABORT,'Historical price must be integer minor units'); END;
CREATE TRIGGER membership_periods_no_update BEFORE UPDATE ON membership_periods BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER membership_periods_no_delete BEFORE DELETE ON membership_periods BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER attendance_no_update BEFORE UPDATE ON attendance BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER attendance_no_delete BEFORE DELETE ON attendance BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER payments_no_update BEFORE UPDATE ON payments BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER payments_no_delete BEFORE DELETE ON payments BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER sales_no_update BEFORE UPDATE ON sales BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER sales_no_delete BEFORE DELETE ON sales BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER sale_items_no_update BEFORE UPDATE ON sale_items BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER sale_items_no_delete BEFORE DELETE ON sale_items BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER stock_movements_no_update BEFORE UPDATE ON stock_movements BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER stock_movements_no_delete BEFORE DELETE ON stock_movements BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER expenses_no_update BEFORE UPDATE ON expenses BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER expenses_no_delete BEFORE DELETE ON expenses BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER local_operations_no_update BEFORE UPDATE ON local_operations BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER local_operations_no_delete BEFORE DELETE ON local_operations BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER payment_allocations_no_update BEFORE UPDATE ON payment_allocations BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER payment_allocations_no_delete BEFORE DELETE ON payment_allocations BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER invoices_no_update BEFORE UPDATE ON invoices BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
CREATE TRIGGER invoices_no_delete BEFORE DELETE ON invoices BEGIN SELECT RAISE(ABORT,'Historical records are append-only'); END;
PRAGMA user_version = 2;
