-- Rebuild only the profile table, preserving every identity and historical link.
-- Store::open disables foreign keys before its migration transaction and checks
-- all references before committing. Copying precedes rebuilding the triggers.
CREATE TEMP TABLE staff_profiles_migration AS SELECT * FROM trainers;
DROP TABLE trainers;
CREATE TABLE trainers (
 id TEXT PRIMARY KEY, name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
 phone TEXT NOT NULL CHECK(length(trim(phone)) BETWEEN 1 AND 40),
 nic TEXT NOT NULL CHECK(length(nic) BETWEEN 1 AND 24 AND nic=upper(nic) AND nic NOT GLOB '*[^A-Z0-9]*'),
 salary_minor INTEGER NOT NULL CHECK(salary_minor BETWEEN 0 AND 100000000000),
 training_fee_minor INTEGER NOT NULL CHECK(training_fee_minor BETWEEN 0 AND 100000000000),
 active INTEGER NOT NULL CHECK(active IN (0,1)), version INTEGER NOT NULL CHECK(version>=1)
) STRICT;
INSERT INTO trainers SELECT * FROM staff_profiles_migration;
DROP TABLE staff_profiles_migration;
CREATE INDEX trainer_nic_lookup ON trainers(nic);
CREATE TRIGGER trainers_nic_insert BEFORE INSERT ON trainers
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND EXISTS(SELECT 1 FROM trainers t WHERE t.nic=NEW.nic AND t.id<>NEW.id
 AND NOT EXISTS(SELECT 1 FROM staff_deletions d WHERE d.id=t.id))
 BEGIN SELECT RAISE(ABORT,'A staff member with this NIC number already exists'); END;
CREATE TRIGGER trainers_nic_update BEFORE UPDATE ON trainers
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND EXISTS(SELECT 1 FROM trainers t WHERE t.nic=NEW.nic AND t.id<>NEW.id
 AND NOT EXISTS(SELECT 1 FROM staff_deletions d WHERE d.id=t.id))
 BEGIN SELECT RAISE(ABORT,'A staff member with this NIC number already exists'); END;
CREATE TRIGGER trainers_version BEFORE UPDATE ON trainers WHEN NEW.id<>OLD.id OR NEW.version<>OLD.version+1
 BEGIN SELECT RAISE(ABORT,'Staff record changed; version must advance by one'); END;
CREATE TRIGGER trainers_no_delete BEFORE DELETE ON trainers
 BEGIN SELECT RAISE(ABORT,'Staff and training history cannot be deleted'); END;
CREATE TRIGGER deleted_staff_edit BEFORE UPDATE ON trainers
 WHEN EXISTS(SELECT 1 FROM staff_deletions WHERE id=OLD.id)
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN SELECT RAISE(ABORT,'Deleted staff cannot be edited or reactivated'); END;
CREATE TRIGGER business_trainers_insert AFTER INSERT ON trainers
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN INSERT INTO business_dirty VALUES('trainers',NEW.id,NULL,json_object('id',NEW.id,'name',NEW.name,'phone',NEW.phone,'nic',NEW.nic,'salary_minor',NEW.salary_minor,'training_fee_minor',NEW.training_fee_minor,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_trainers_update AFTER UPDATE ON trainers
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN INSERT INTO business_dirty VALUES('trainers',NEW.id,json_object('id',OLD.id,'name',OLD.name,'phone',OLD.phone,'nic',OLD.nic,'salary_minor',OLD.salary_minor,'training_fee_minor',OLD.training_fee_minor,'active',OLD.active,'version',OLD.version),json_object('id',NEW.id,'name',NEW.name,'phone',NEW.phone,'nic',NEW.nic,'salary_minor',NEW.salary_minor,'training_fee_minor',NEW.training_fee_minor,'active',NEW.active,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
PRAGMA user_version=12;
