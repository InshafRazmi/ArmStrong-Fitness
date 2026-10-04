ALTER TABLE members ADD COLUMN archived_at TEXT;
ALTER TABLE members ADD COLUMN archived_by_user_id TEXT REFERENCES users(id);
CREATE INDEX member_archive_name ON members(archived_at,name);
CREATE TRIGGER member_archive_pair_insert BEFORE INSERT ON members WHEN
 (NEW.archived_at IS NULL)<>(NEW.archived_by_user_id IS NULL)
BEGIN SELECT RAISE(ABORT,'Archive timestamp and staff actor must be set together'); END;
CREATE TRIGGER member_archive_pair_update BEFORE UPDATE ON members WHEN
 (NEW.archived_at IS NULL)<>(NEW.archived_by_user_id IS NULL)
 OR (OLD.archived_at IS NOT NULL AND (NEW.archived_at IS NOT OLD.archived_at OR NEW.archived_by_user_id IS NOT OLD.archived_by_user_id))
BEGIN SELECT RAISE(ABORT,'Archive history cannot be removed or overwritten'); END;
CREATE TRIGGER member_linked_delete BEFORE DELETE ON members WHEN
 EXISTS(SELECT 1 FROM membership_periods WHERE member_id=OLD.id)
 OR EXISTS(SELECT 1 FROM attendance WHERE member_id=OLD.id)
 OR EXISTS(SELECT 1 FROM invoices WHERE member_id=OLD.id)
 OR EXISTS(SELECT 1 FROM payments WHERE member_id=OLD.id)
 OR EXISTS(SELECT 1 FROM nfc_cards WHERE member_id=OLD.id)
BEGIN SELECT RAISE(ABORT,'Linked members must be archived; history cannot be deleted'); END;
CREATE TRIGGER archived_membership_insert BEFORE INSERT ON membership_periods WHEN
 EXISTS(SELECT 1 FROM members WHERE id=NEW.member_id AND archived_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Archived members cannot receive new memberships'); END;
CREATE TRIGGER archived_attendance_insert BEFORE INSERT ON attendance WHEN
 EXISTS(SELECT 1 FROM members WHERE id=NEW.member_id AND archived_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Archived members cannot record attendance'); END;
CREATE TABLE expense_voids (
 expense_id TEXT PRIMARY KEY REFERENCES expenses(id),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 254),
 created_at TEXT NOT NULL,
 actor TEXT NOT NULL CHECK(length(trim(actor))>0),
 actor_user_id TEXT REFERENCES users(id),
 legacy_reversal_id TEXT UNIQUE REFERENCES expenses(id),
 CHECK(actor_user_id IS NOT NULL OR legacy_reversal_id IS NOT NULL)
) STRICT;
INSERT INTO expense_voids
 SELECT reverses_id,'Legacy reversal: original reason unavailable',created_at,actor,NULL,id
 FROM expenses WHERE reverses_id IS NOT NULL;
CREATE INDEX expense_void_actor_time ON expense_voids(actor_user_id,created_at);
CREATE TRIGGER expense_void_parent BEFORE INSERT ON expense_voids WHEN
 NOT EXISTS(SELECT 1 FROM expenses WHERE id=NEW.expense_id AND reverses_id IS NULL)
 OR NEW.legacy_reversal_id IS NOT NULL
 OR EXISTS(SELECT 1 FROM expenses WHERE reverses_id=NEW.expense_id)
BEGIN SELECT RAISE(ABORT,'Only an original, unvoided expense may be voided'); END;
CREATE TRIGGER expenses_new_reversal BEFORE INSERT ON expenses WHEN NEW.reverses_id IS NOT NULL
BEGIN SELECT RAISE(ABORT,'Use the audited expense void workflow'); END;
CREATE TRIGGER expense_voids_no_update BEFORE UPDATE ON expense_voids
BEGIN SELECT RAISE(ABORT,'Expense void history is append-only'); END;
CREATE TRIGGER expense_voids_no_delete BEFORE DELETE ON expense_voids
BEGIN SELECT RAISE(ABORT,'Expense void history is append-only'); END;
CREATE VIEW expense_history AS
 SELECT e.*,
 CASE WHEN e.reverses_id IS NOT NULL THEN 'Reversal' WHEN v.expense_id IS NOT NULL THEN 'Voided' ELSE 'Recorded' END status,
 CASE WHEN e.reverses_id IS NULL AND v.expense_id IS NULL THEN e.amount_minor ELSE 0 END effective_amount_minor,
 v.reason void_reason,v.created_at voided_at,v.actor voided_by,v.actor_user_id voided_by_user_id
 FROM expenses e LEFT JOIN expense_voids v ON v.expense_id=e.id;
PRAGMA user_version=4;
