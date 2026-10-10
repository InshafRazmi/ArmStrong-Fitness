CREATE TABLE admission_settings (
 id INTEGER PRIMARY KEY CHECK(id=1), amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 0 AND 100000000000), version INTEGER NOT NULL CHECK(version>=1)
) STRICT;
CREATE TRIGGER business_admission_insert AFTER INSERT ON admission_settings
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN INSERT INTO business_dirty VALUES('admission_settings','1',NULL,json_object('id',NEW.id,'amount_minor',NEW.amount_minor,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
CREATE TRIGGER business_admission_update AFTER UPDATE ON admission_settings
 WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 BEGIN INSERT INTO business_dirty VALUES('admission_settings','1',json_object('id',OLD.id,'amount_minor',OLD.amount_minor,'version',OLD.version),json_object('id',NEW.id,'amount_minor',NEW.amount_minor,'version',NEW.version))
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json; END;
PRAGMA user_version=11;
