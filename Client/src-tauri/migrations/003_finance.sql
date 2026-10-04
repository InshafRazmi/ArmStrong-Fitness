-- Keep all v2 financial rows; allow multiple immutable partial allocations per pair.
DROP TRIGGER allocation_limits;
DROP TRIGGER payment_allocations_no_update;
DROP TRIGGER payment_allocations_no_delete;
CREATE TABLE payment_allocations_v3 (
 id TEXT PRIMARY KEY, payment_id TEXT NOT NULL REFERENCES payments(id),
 invoice_id TEXT NOT NULL REFERENCES invoices(id),
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 100000000000)
) STRICT;
INSERT INTO payment_allocations_v3 SELECT * FROM payment_allocations;
DROP TABLE payment_allocations;
ALTER TABLE payment_allocations_v3 RENAME TO payment_allocations;
CREATE INDEX allocation_payment_invoice ON payment_allocations(payment_id,invoice_id);
CREATE INDEX allocation_invoice ON payment_allocations(invoice_id);
CREATE TABLE invoice_details (
 invoice_id TEXT PRIMARY KEY REFERENCES invoices(id), number TEXT NOT NULL UNIQUE,
 description TEXT NOT NULL CHECK(length(trim(description)) BETWEEN 1 AND 254),
 member_name TEXT NOT NULL, actor TEXT NOT NULL, legacy INTEGER NOT NULL CHECK(legacy IN (0,1))
) STRICT;
INSERT INTO invoice_details
SELECT i.id,'AF-I-'||(SELECT value FROM metadata WHERE key='device_id')||'-'||i.id,
 COALESCE((SELECT 'Membership: '||plan_name FROM membership_periods WHERE id=i.membership_period_id),'Legacy invoice'),
 COALESCE((SELECT name FROM members WHERE id=i.member_id),'Unassigned legacy invoice'),
 'legacy-record (actor unavailable)',1 FROM invoices i;
CREATE TABLE payment_reversal_details (
 payment_id TEXT PRIMARY KEY REFERENCES payments(id), reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 254)
) STRICT;
INSERT INTO payment_reversal_details SELECT id,'Legacy reversal: original reason unavailable' FROM payments WHERE reverses_id IS NOT NULL;
CREATE TABLE allocation_reversals (
 id TEXT PRIMARY KEY, allocation_id TEXT NOT NULL UNIQUE REFERENCES payment_allocations(id),
 payment_reversal_id TEXT NOT NULL REFERENCES payments(id), created_at TEXT NOT NULL
) STRICT;
INSERT INTO allocation_reversals
SELECT 'migrated-release-'||a.id,a.id,r.id,r.created_at
FROM payment_allocations a JOIN payments r ON r.reverses_id=a.payment_id;
CREATE INDEX allocation_release_payment ON allocation_reversals(payment_reversal_id);
CREATE TABLE payment_receipts (
 payment_id TEXT PRIMARY KEY REFERENCES payments(id), number TEXT NOT NULL UNIQUE,
 snapshot_json TEXT NOT NULL CHECK(json_valid(snapshot_json)), issued_at TEXT NOT NULL,
 legacy INTEGER NOT NULL CHECK(legacy IN (0,1))
) STRICT;
CREATE VIEW effective_payment_allocations AS
SELECT a.* FROM payment_allocations a JOIN payments p ON p.id=a.payment_id
WHERE p.reverses_id IS NULL
 AND NOT EXISTS(SELECT 1 FROM payments r WHERE r.reverses_id=p.id)
 AND NOT EXISTS(SELECT 1 FROM allocation_reversals r WHERE r.allocation_id=a.id);
CREATE VIEW invoice_balances AS
SELECT i.*,COALESCE((SELECT SUM(amount_minor) FROM effective_payment_allocations WHERE invoice_id=i.id),0) paid_minor,
 i.amount_minor-COALESCE((SELECT SUM(amount_minor) FROM effective_payment_allocations WHERE invoice_id=i.id),0) outstanding_minor
FROM invoices i;
CREATE VIEW payment_balances AS
SELECT p.*,CASE WHEN p.reverses_id IS NULL THEN p.amount_minor ELSE -p.amount_minor END net_amount_minor,
 COALESCE((SELECT SUM(amount_minor) FROM effective_payment_allocations WHERE payment_id=p.id),0) allocated_minor,
 CASE WHEN p.reverses_id IS NULL AND NOT EXISTS(SELECT 1 FROM payments r WHERE r.reverses_id=p.id)
 THEN p.amount_minor-COALESCE((SELECT SUM(amount_minor) FROM effective_payment_allocations WHERE payment_id=p.id),0) ELSE 0 END unallocated_minor,
 CASE WHEN p.reverses_id IS NOT NULL THEN 'Reversal'
 WHEN EXISTS(SELECT 1 FROM payments r WHERE r.reverses_id=p.id) THEN 'Reversed'
 WHEN COALESCE((SELECT SUM(amount_minor) FROM effective_payment_allocations WHERE payment_id=p.id),0)=0 THEN 'Recorded'
 WHEN (SELECT SUM(amount_minor) FROM effective_payment_allocations WHERE payment_id=p.id)=p.amount_minor THEN 'Allocated'
 ELSE 'Partly allocated' END status
FROM payments p;
CREATE TRIGGER allocation_limits BEFORE INSERT ON payment_allocations WHEN
 NOT EXISTS(SELECT 1 FROM payment_balances p JOIN invoices i ON p.member_id=i.member_id
  WHERE p.id=NEW.payment_id AND i.id=NEW.invoice_id AND p.reverses_id IS NULL AND p.status<>'Reversed'
   AND NEW.amount_minor<=p.unallocated_minor
   AND NEW.amount_minor<=(SELECT outstanding_minor FROM invoice_balances WHERE id=i.id))
BEGIN SELECT RAISE(ABORT,'Invalid allocation: member, available credit or invoice outstanding changed'); END;
CREATE TRIGGER payment_full_reversal BEFORE INSERT ON payments WHEN NEW.reverses_id IS NOT NULL AND NOT EXISTS(
 SELECT 1 FROM payments p WHERE p.id=NEW.reverses_id AND p.reverses_id IS NULL
 AND p.member_id=NEW.member_id AND p.member_name=NEW.member_name
 AND p.amount_minor=NEW.amount_minor AND p.method=NEW.method)
BEGIN SELECT RAISE(ABORT,'Reversal must copy one original received payment in full'); END;
CREATE TRIGGER reversal_detail_parent BEFORE INSERT ON payment_reversal_details WHEN NOT EXISTS(
 SELECT 1 FROM payments WHERE id=NEW.payment_id AND reverses_id IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Reason requires a reversing payment'); END;
CREATE TRIGGER allocation_release_parent BEFORE INSERT ON allocation_reversals WHEN NOT EXISTS(
 SELECT 1 FROM payment_allocations a JOIN payments r ON r.reverses_id=a.payment_id
 WHERE a.id=NEW.allocation_id AND r.id=NEW.payment_reversal_id)
BEGIN SELECT RAISE(ABORT,'Allocation release must belong to the original reversed payment'); END;
CREATE TRIGGER receipt_payment_match BEFORE INSERT ON payment_receipts WHEN
 json_extract(NEW.snapshot_json,'$.payment.id') IS NOT NEW.payment_id
 OR NOT EXISTS(SELECT 1 FROM payments p WHERE p.id=NEW.payment_id
 AND p.amount_minor=json_extract(NEW.snapshot_json,'$.payment.amountMinor')
 AND p.method=json_extract(NEW.snapshot_json,'$.payment.method')
 AND p.member_id=json_extract(NEW.snapshot_json,'$.payment.memberId'))
BEGIN SELECT RAISE(ABORT,'Receipt must use saved payment data'); END;
CREATE TRIGGER invoice_details_no_update BEFORE UPDATE ON invoice_details BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER invoice_details_no_delete BEFORE DELETE ON invoice_details BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER payment_reversal_details_no_update BEFORE UPDATE ON payment_reversal_details BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER payment_reversal_details_no_delete BEFORE DELETE ON payment_reversal_details BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER allocation_reversals_no_update BEFORE UPDATE ON allocation_reversals BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER allocation_reversals_no_delete BEFORE DELETE ON allocation_reversals BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER payment_receipts_no_update BEFORE UPDATE ON payment_receipts BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER payment_receipts_no_delete BEFORE DELETE ON payment_receipts BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER payment_allocations_no_update BEFORE UPDATE ON payment_allocations BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
CREATE TRIGGER payment_allocations_no_delete BEFORE DELETE ON payment_allocations BEGIN SELECT RAISE(ABORT,'Financial history is append-only'); END;
PRAGMA user_version=3;
