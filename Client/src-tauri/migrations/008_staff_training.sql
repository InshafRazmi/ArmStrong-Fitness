-- Add trainer profiles and immutable monthly billing/payroll links without changing existing rows.
CREATE TABLE trainers (
 id TEXT PRIMARY KEY, name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
 phone TEXT NOT NULL CHECK(length(trim(phone)) BETWEEN 1 AND 40),
 nic TEXT NOT NULL UNIQUE CHECK(length(nic) BETWEEN 1 AND 24 AND nic=upper(nic) AND nic NOT GLOB '*[^A-Z0-9]*'),
 salary_minor INTEGER NOT NULL CHECK(salary_minor BETWEEN 0 AND 100000000000),
 training_fee_minor INTEGER NOT NULL CHECK(training_fee_minor BETWEEN 0 AND 100000000000),
 active INTEGER NOT NULL CHECK(active IN (0,1)), version INTEGER NOT NULL CHECK(version>=1)
) STRICT;
CREATE TABLE member_trainers (
 id TEXT PRIMARY KEY REFERENCES members(id), trainer_id TEXT REFERENCES trainers(id),
 version INTEGER NOT NULL CHECK(version>=1)
) STRICT;
CREATE INDEX member_trainer_staff ON member_trainers(trainer_id);
CREATE TABLE training_charges (
 id TEXT PRIMARY KEY, invoice_id TEXT NOT NULL UNIQUE REFERENCES invoices(id),
 member_id TEXT NOT NULL REFERENCES members(id), trainer_id TEXT NOT NULL REFERENCES trainers(id),
 trainer_name TEXT NOT NULL, fee_minor INTEGER NOT NULL CHECK(fee_minor BETWEEN 1 AND 100000000000),
 starts_on TEXT NOT NULL, ends_on TEXT NOT NULL CHECK(ends_on>=starts_on)
) STRICT;
CREATE INDEX training_charge_member_month ON training_charges(member_id,starts_on,ends_on);
CREATE INDEX training_charge_trainer ON training_charges(trainer_id);
CREATE TABLE staff_payouts (
 id TEXT PRIMARY KEY, trainer_id TEXT NOT NULL REFERENCES trainers(id),
 expense_id TEXT NOT NULL UNIQUE REFERENCES expenses(id), salary_month TEXT NOT NULL,
 salary_minor INTEGER NOT NULL CHECK(salary_minor BETWEEN 0 AND 100000000000),
 training_minor INTEGER NOT NULL CHECK(training_minor BETWEEN 0 AND 100000000000),
 CHECK(salary_minor+training_minor BETWEEN 1 AND 100000000000)
) STRICT;
CREATE INDEX staff_payout_salary_month ON staff_payouts(trainer_id,salary_month);
CREATE TABLE staff_payout_items (
 id TEXT PRIMARY KEY, payout_id TEXT NOT NULL REFERENCES staff_payouts(id),
 allocation_id TEXT NOT NULL REFERENCES payment_allocations(id),
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 100000000000),
 UNIQUE(payout_id,allocation_id)
) STRICT;
CREATE INDEX staff_payout_allocation ON staff_payout_items(allocation_id);
CREATE VIEW active_staff_payouts AS SELECT p.* FROM staff_payouts p
 JOIN expenses e ON e.id=p.expense_id WHERE e.reverses_id IS NULL
 AND NOT EXISTS(SELECT 1 FROM expense_voids v WHERE v.expense_id=e.id)
 AND NOT EXISTS(SELECT 1 FROM expenses r WHERE r.reverses_id=e.id);
CREATE VIEW unpaid_training_allocations AS
 SELECT a.*,c.trainer_id,c.trainer_name,c.member_id FROM effective_payment_allocations a
 JOIN training_charges c ON c.invoice_id=a.invoice_id
 WHERE NOT EXISTS(SELECT 1 FROM staff_payout_items item JOIN active_staff_payouts p ON p.id=item.payout_id WHERE item.allocation_id=a.id);
CREATE TRIGGER training_charge_invoice BEFORE INSERT ON training_charges WHEN NOT EXISTS(
 SELECT 1 FROM invoices i WHERE i.id=NEW.invoice_id AND i.member_id=NEW.member_id
 AND i.amount_minor=NEW.fee_minor AND i.membership_period_id IS NULL AND i.sale_id IS NULL)
 BEGIN SELECT RAISE(ABORT,'Training charge must match its member invoice'); END;
CREATE TRIGGER training_month_overlap BEFORE INSERT ON training_charges WHEN EXISTS(
 SELECT 1 FROM training_charges c WHERE c.member_id=NEW.member_id AND c.starts_on<=NEW.ends_on AND c.ends_on>=NEW.starts_on)
 BEGIN SELECT RAISE(ABORT,'This member already has a training invoice for these dates'); END;
CREATE TRIGGER staff_payout_expense BEFORE INSERT ON staff_payouts WHEN NOT EXISTS(
 SELECT 1 FROM expenses e WHERE e.id=NEW.expense_id AND e.category='Salary'
 AND e.amount_minor=NEW.salary_minor+NEW.training_minor AND e.reverses_id IS NULL)
 BEGIN SELECT RAISE(ABORT,'Staff payout must match its Salary expense'); END;
CREATE TRIGGER staff_salary_once BEFORE INSERT ON staff_payouts WHEN NEW.salary_minor>0 AND EXISTS(
 SELECT 1 FROM active_staff_payouts p WHERE p.trainer_id=NEW.trainer_id AND p.salary_month=NEW.salary_month AND p.salary_minor>0)
 BEGIN SELECT RAISE(ABORT,'This staff salary month has already been paid'); END;
CREATE TRIGGER staff_payout_item_check BEFORE INSERT ON staff_payout_items WHEN NOT EXISTS(
 SELECT 1 FROM staff_payouts p JOIN unpaid_training_allocations a ON a.trainer_id=p.trainer_id
 WHERE p.id=NEW.payout_id AND a.id=NEW.allocation_id AND a.amount_minor=NEW.amount_minor)
 BEGIN SELECT RAISE(ABORT,'Training earnings changed or were already paid'); END;
CREATE TRIGGER training_paid_reversal BEFORE INSERT ON payments WHEN NEW.reverses_id IS NOT NULL AND EXISTS(
 SELECT 1 FROM payment_allocations a JOIN staff_payout_items item ON item.allocation_id=a.id
 JOIN active_staff_payouts p ON p.id=item.payout_id WHERE a.payment_id=NEW.reverses_id)
 BEGIN SELECT RAISE(ABORT,'Void the linked staff payout expense before reversing this training payment'); END;
CREATE TRIGGER trainers_version BEFORE UPDATE ON trainers WHEN NEW.id<>OLD.id OR NEW.version<>OLD.version+1 BEGIN SELECT RAISE(ABORT,'Staff record changed; version must advance by one'); END;
CREATE TRIGGER trainers_no_delete BEFORE DELETE ON trainers BEGIN SELECT RAISE(ABORT,'Staff and training history cannot be deleted'); END;
CREATE TRIGGER business_trainers_insert AFTER INSERT ON trainers
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('trainers',NEW.id,NULL,json_object('id',NEW.id,'name',NEW.name,'phone',NEW.phone,'nic',NEW.nic,'salary_minor',NEW.salary_minor,'training_fee_minor',NEW.training_fee_minor,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_trainers_update AFTER UPDATE ON trainers
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('trainers',NEW.id,json_object('id',OLD.id,'name',OLD.name,'phone',OLD.phone,'nic',OLD.nic,'salary_minor',OLD.salary_minor,'training_fee_minor',OLD.training_fee_minor,'active',OLD.active,'version',OLD.version),json_object('id',NEW.id,'name',NEW.name,'phone',NEW.phone,'nic',NEW.nic,'salary_minor',NEW.salary_minor,'training_fee_minor',NEW.training_fee_minor,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER member_trainers_version BEFORE UPDATE ON member_trainers WHEN NEW.id<>OLD.id OR NEW.version<>OLD.version+1 BEGIN SELECT RAISE(ABORT,'Staff record changed; version must advance by one'); END;
CREATE TRIGGER member_trainers_no_delete BEFORE DELETE ON member_trainers BEGIN SELECT RAISE(ABORT,'Staff and training history cannot be deleted'); END;
CREATE TRIGGER business_member_trainers_insert AFTER INSERT ON member_trainers
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('member_trainers',NEW.id,NULL,json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_member_trainers_update AFTER UPDATE ON member_trainers
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('member_trainers',NEW.id,json_object('id',OLD.id,'trainer_id',OLD.trainer_id,'version',OLD.version),json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER training_charges_no_update BEFORE UPDATE ON training_charges BEGIN SELECT RAISE(ABORT,'Training and payout history is append-only'); END;
CREATE TRIGGER training_charges_no_delete BEFORE DELETE ON training_charges BEGIN SELECT RAISE(ABORT,'Staff and training history cannot be deleted'); END;
CREATE TRIGGER business_training_charges_insert AFTER INSERT ON training_charges
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('training_charges',NEW.id,NULL,json_object('id',NEW.id,'invoice_id',NEW.invoice_id,'member_id',NEW.member_id,'trainer_id',NEW.trainer_id,'trainer_name',NEW.trainer_name,'fee_minor',NEW.fee_minor,'starts_on',NEW.starts_on,'ends_on',NEW.ends_on))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_training_charges_update AFTER UPDATE ON training_charges
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('training_charges',NEW.id,json_object('id',OLD.id,'invoice_id',OLD.invoice_id,'member_id',OLD.member_id,'trainer_id',OLD.trainer_id,'trainer_name',OLD.trainer_name,'fee_minor',OLD.fee_minor,'starts_on',OLD.starts_on,'ends_on',OLD.ends_on),json_object('id',NEW.id,'invoice_id',NEW.invoice_id,'member_id',NEW.member_id,'trainer_id',NEW.trainer_id,'trainer_name',NEW.trainer_name,'fee_minor',NEW.fee_minor,'starts_on',NEW.starts_on,'ends_on',NEW.ends_on))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER staff_payouts_no_update BEFORE UPDATE ON staff_payouts BEGIN SELECT RAISE(ABORT,'Training and payout history is append-only'); END;
CREATE TRIGGER staff_payouts_no_delete BEFORE DELETE ON staff_payouts BEGIN SELECT RAISE(ABORT,'Staff and training history cannot be deleted'); END;
CREATE TRIGGER business_staff_payouts_insert AFTER INSERT ON staff_payouts
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_payouts',NEW.id,NULL,json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'expense_id',NEW.expense_id,'salary_month',NEW.salary_month,'salary_minor',NEW.salary_minor,'training_minor',NEW.training_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_staff_payouts_update AFTER UPDATE ON staff_payouts
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_payouts',NEW.id,json_object('id',OLD.id,'trainer_id',OLD.trainer_id,'expense_id',OLD.expense_id,'salary_month',OLD.salary_month,'salary_minor',OLD.salary_minor,'training_minor',OLD.training_minor),json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'expense_id',NEW.expense_id,'salary_month',NEW.salary_month,'salary_minor',NEW.salary_minor,'training_minor',NEW.training_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER staff_payout_items_no_update BEFORE UPDATE ON staff_payout_items BEGIN SELECT RAISE(ABORT,'Training and payout history is append-only'); END;
CREATE TRIGGER staff_payout_items_no_delete BEFORE DELETE ON staff_payout_items BEGIN SELECT RAISE(ABORT,'Staff and training history cannot be deleted'); END;
CREATE TRIGGER business_staff_payout_items_insert AFTER INSERT ON staff_payout_items
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_payout_items',NEW.id,NULL,json_object('id',NEW.id,'payout_id',NEW.payout_id,'allocation_id',NEW.allocation_id,'amount_minor',NEW.amount_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_staff_payout_items_update AFTER UPDATE ON staff_payout_items
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_payout_items',NEW.id,json_object('id',OLD.id,'payout_id',OLD.payout_id,'allocation_id',OLD.allocation_id,'amount_minor',OLD.amount_minor),json_object('id',NEW.id,'payout_id',NEW.payout_id,'allocation_id',NEW.allocation_id,'amount_minor',NEW.amount_minor))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
PRAGMA user_version=8;
