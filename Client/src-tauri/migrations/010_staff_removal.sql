-- Retain staff identity for payroll/training history while removing operational profiles.
CREATE TABLE staff_deletions (
 id TEXT PRIMARY KEY REFERENCES trainers(id), deleted_at TEXT NOT NULL,
 actor_user_id TEXT NOT NULL REFERENCES users(id)
) STRICT;
CREATE TRIGGER staff_deletions_no_update BEFORE UPDATE ON staff_deletions
 BEGIN SELECT RAISE(ABORT,'Staff removal history is immutable'); END;
CREATE TRIGGER staff_deletions_no_delete BEFORE DELETE ON staff_deletions
 BEGIN SELECT RAISE(ABORT,'Staff removal history cannot be deleted'); END;
CREATE TRIGGER staff_removal_inactive BEFORE INSERT ON staff_deletions
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND (EXISTS(SELECT 1 FROM trainers WHERE id=NEW.id AND active=1)
 OR EXISTS(SELECT 1 FROM staff_nfc_cards WHERE trainer_id=NEW.id AND revoked_at IS NULL)
 OR EXISTS(SELECT 1 FROM member_trainers WHERE trainer_id=NEW.id))
 BEGIN SELECT RAISE(ABORT,'Remove active staff assignments and card before deletion'); END;
CREATE TRIGGER deleted_staff_edit BEFORE UPDATE ON trainers
 WHEN EXISTS(SELECT 1 FROM staff_deletions WHERE id=OLD.id)
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN SELECT RAISE(ABORT,'Deleted staff cannot be edited or reactivated'); END;
CREATE TRIGGER business_staff_deletions_insert AFTER INSERT ON staff_deletions
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN INSERT INTO business_dirty VALUES('staff_deletions',NEW.id,NULL,
 json_object('id',NEW.id,'deleted_at',NEW.deleted_at,'actor_user_id',NEW.actor_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
PRAGMA user_version=10;
