-- Add profile details, staff attendance and permanent removal markers. Existing
-- membership, attendance, payment and receipt rows retain their identities.
CREATE TABLE member_profiles (
 id TEXT PRIMARY KEY REFERENCES members(id), gender TEXT NOT NULL CHECK(gender IN ('Male','Female')),
 version INTEGER NOT NULL CHECK(version>=1)
) STRICT;
CREATE TABLE member_deletions (
 id TEXT PRIMARY KEY REFERENCES members(id), deleted_at TEXT NOT NULL,
 actor_user_id TEXT NOT NULL REFERENCES users(id)
) STRICT;
CREATE TABLE staff_nfc_cards (
 id TEXT PRIMARY KEY, trainer_id TEXT NOT NULL REFERENCES trainers(id),
 uid TEXT NOT NULL CHECK(length(uid) BETWEEN 1 AND 128 AND uid=upper(uid)),
 assigned_at TEXT NOT NULL, revoked_at TEXT
) STRICT;
CREATE UNIQUE INDEX staff_nfc_active_uid ON staff_nfc_cards(uid) WHERE revoked_at IS NULL;
CREATE UNIQUE INDEX staff_nfc_active_trainer ON staff_nfc_cards(trainer_id) WHERE revoked_at IS NULL;
CREATE INDEX staff_nfc_history ON staff_nfc_cards(trainer_id,assigned_at);
CREATE TABLE staff_attendance (
 id TEXT PRIMARY KEY, trainer_id TEXT NOT NULL REFERENCES trainers(id),
 card_id TEXT REFERENCES staff_nfc_cards(id), staff_name TEXT NOT NULL,
 card_uid TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('Check-in','Check-out')),
 source TEXT NOT NULL CHECK(source IN ('Manual','NFC')), business_on TEXT NOT NULL,
 occurred_at TEXT NOT NULL, CHECK((source='Manual' AND card_id IS NULL AND card_uid='') OR (source='NFC' AND card_id IS NOT NULL AND card_uid<>''))
) STRICT;
CREATE INDEX staff_attendance_day ON staff_attendance(business_on,occurred_at);
CREATE INDEX staff_attendance_person_day ON staff_attendance(trainer_id,business_on,occurred_at);
CREATE TRIGGER member_profiles_version BEFORE UPDATE ON member_profiles
 WHEN NEW.id<>OLD.id OR NEW.version<>OLD.version+1
 BEGIN SELECT RAISE(ABORT,'Member gender changed; refresh before saving'); END;
CREATE TRIGGER staff_card_member_unique BEFORE INSERT ON staff_nfc_cards WHEN NEW.revoked_at IS NULL
 AND EXISTS(SELECT 1 FROM nfc_cards WHERE uid=NEW.uid AND revoked_at IS NULL)
 BEGIN SELECT RAISE(ABORT,'This NFC card is already assigned to a member'); END;
CREATE TRIGGER member_card_staff_unique BEFORE INSERT ON nfc_cards WHEN NEW.revoked_at IS NULL
 AND EXISTS(SELECT 1 FROM staff_nfc_cards WHERE uid=NEW.uid AND revoked_at IS NULL)
 BEGIN SELECT RAISE(ABORT,'This NFC card is already assigned to staff'); END;
CREATE TRIGGER staff_card_history BEFORE UPDATE ON staff_nfc_cards
 WHEN OLD.revoked_at IS NOT NULL OR NEW.id<>OLD.id OR NEW.trainer_id<>OLD.trainer_id
 OR NEW.uid<>OLD.uid OR NEW.assigned_at<>OLD.assigned_at OR NEW.revoked_at IS NULL
 BEGIN SELECT RAISE(ABORT,'Only revoking an active staff card is allowed'); END;
CREATE TRIGGER staff_attendance_card BEFORE INSERT ON staff_attendance WHEN NEW.source='NFC'
 AND NOT EXISTS(SELECT 1 FROM staff_nfc_cards WHERE id=NEW.card_id AND trainer_id=NEW.trainer_id AND uid=NEW.card_uid)
 BEGIN SELECT RAISE(ABORT,'Staff attendance card does not match'); END;
CREATE TRIGGER deleted_member_archive AFTER INSERT ON member_deletions
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN
  UPDATE nfc_cards SET revoked_at=NEW.deleted_at WHERE member_id=NEW.id AND revoked_at IS NULL;
 END;
CREATE TRIGGER deleted_member_edit BEFORE UPDATE ON members
 WHEN EXISTS(SELECT 1 FROM member_deletions WHERE id=OLD.id)
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN SELECT RAISE(ABORT,'Deleted members cannot be edited'); END;
CREATE TRIGGER member_profiles_no_delete BEFORE DELETE ON member_profiles BEGIN SELECT RAISE(ABORT,'Attendance and profile history cannot be deleted'); END;
CREATE TRIGGER business_member_profiles_insert AFTER INSERT ON member_profiles
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('member_profiles',NEW.id,NULL,json_object('id',NEW.id,'gender',NEW.gender,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_member_profiles_update AFTER UPDATE ON member_profiles
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('member_profiles',NEW.id,json_object('id',OLD.id,'gender',OLD.gender,'version',OLD.version),json_object('id',NEW.id,'gender',NEW.gender,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER member_deletions_no_update BEFORE UPDATE ON member_deletions BEGIN SELECT RAISE(ABORT,'Attendance and removal history is immutable'); END;
CREATE TRIGGER member_deletions_no_delete BEFORE DELETE ON member_deletions BEGIN SELECT RAISE(ABORT,'Attendance and profile history cannot be deleted'); END;
CREATE TRIGGER business_member_deletions_insert AFTER INSERT ON member_deletions
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('member_deletions',NEW.id,NULL,json_object('id',NEW.id,'deleted_at',NEW.deleted_at,'actor_user_id',NEW.actor_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_member_deletions_update AFTER UPDATE ON member_deletions
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('member_deletions',NEW.id,json_object('id',OLD.id,'deleted_at',OLD.deleted_at,'actor_user_id',OLD.actor_user_id),json_object('id',NEW.id,'deleted_at',NEW.deleted_at,'actor_user_id',NEW.actor_user_id))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER staff_nfc_cards_no_delete BEFORE DELETE ON staff_nfc_cards BEGIN SELECT RAISE(ABORT,'Attendance and profile history cannot be deleted'); END;
CREATE TRIGGER business_staff_nfc_cards_insert AFTER INSERT ON staff_nfc_cards
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_nfc_cards',NEW.id,NULL,json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'uid',NEW.uid,'assigned_at',NEW.assigned_at,'revoked_at',NEW.revoked_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_staff_nfc_cards_update AFTER UPDATE ON staff_nfc_cards
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_nfc_cards',NEW.id,json_object('id',OLD.id,'trainer_id',OLD.trainer_id,'uid',OLD.uid,'assigned_at',OLD.assigned_at,'revoked_at',OLD.revoked_at),json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'uid',NEW.uid,'assigned_at',NEW.assigned_at,'revoked_at',NEW.revoked_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER staff_attendance_no_update BEFORE UPDATE ON staff_attendance BEGIN SELECT RAISE(ABORT,'Attendance and removal history is immutable'); END;
CREATE TRIGGER staff_attendance_no_delete BEFORE DELETE ON staff_attendance BEGIN SELECT RAISE(ABORT,'Attendance and profile history cannot be deleted'); END;
CREATE TRIGGER business_staff_attendance_insert AFTER INSERT ON staff_attendance
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_attendance',NEW.id,NULL,json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'card_id',NEW.card_id,'staff_name',NEW.staff_name,'card_uid',NEW.card_uid,'kind',NEW.kind,'source',NEW.source,'business_on',NEW.business_on,'occurred_at',NEW.occurred_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_staff_attendance_update AFTER UPDATE ON staff_attendance
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO business_dirty VALUES('staff_attendance',NEW.id,json_object('id',OLD.id,'trainer_id',OLD.trainer_id,'card_id',OLD.card_id,'staff_name',OLD.staff_name,'card_uid',OLD.card_uid,'kind',OLD.kind,'source',OLD.source,'business_on',OLD.business_on,'occurred_at',OLD.occurred_at),json_object('id',NEW.id,'trainer_id',NEW.trainer_id,'card_id',NEW.card_id,'staff_name',NEW.staff_name,'card_uid',NEW.card_uid,'kind',NEW.kind,'source',NEW.source,'business_on',NEW.business_on,'occurred_at',NEW.occurred_at))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
PRAGMA user_version=9;
