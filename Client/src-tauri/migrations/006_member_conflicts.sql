-- Explicit Administrator review retires rejected/unsent member operations.
-- It never fabricates an acknowledgement or edits the original operation.
CREATE TABLE member_conflict_resolutions (
 id TEXT PRIMARY KEY, member_id TEXT NOT NULL,
 actor_user_id TEXT NOT NULL REFERENCES users(id), actor TEXT NOT NULL,
 choice TEXT NOT NULL CHECK(choice IN ('use_server','keep_local')),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 500),
 review_json TEXT NOT NULL CHECK(json_valid(review_json)), created_at TEXT NOT NULL
) STRICT;
CREATE TABLE member_resolved_conflicts (
 conflict_id TEXT PRIMARY KEY REFERENCES member_sync_conflicts(id),
 resolution_id TEXT NOT NULL REFERENCES member_conflict_resolutions(id)
) STRICT;
CREATE TABLE member_resolved_operations (
 operation_id TEXT PRIMARY KEY REFERENCES outbox(id),
 resolution_id TEXT NOT NULL REFERENCES member_conflict_resolutions(id)
) STRICT;
CREATE VIEW member_active_conflicts AS
 SELECT c.* FROM member_sync_conflicts c
 WHERE NOT EXISTS(SELECT 1 FROM member_resolved_conflicts r WHERE r.conflict_id=c.id);
CREATE TRIGGER resolved_conflict_matches BEFORE INSERT ON member_resolved_conflicts
WHEN NOT EXISTS(SELECT 1 FROM member_sync_conflicts c JOIN member_conflict_resolutions r
 ON r.member_id=c.member_id WHERE c.id=NEW.conflict_id AND r.id=NEW.resolution_id)
BEGIN SELECT RAISE(ABORT,'Resolution must match the conflict member'); END;
CREATE TRIGGER resolved_operation_matches BEFORE INSERT ON member_resolved_operations
WHEN NOT EXISTS(SELECT 1 FROM outbox o JOIN member_conflict_resolutions r
 ON r.member_id=o.entity_id WHERE o.id=NEW.operation_id AND r.id=NEW.resolution_id
 AND o.entity='member' AND o.action IN ('create','update','archive')
 AND NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=o.id AND d.state<>'conflict'))
BEGIN SELECT RAISE(ABORT,'Only rejected or unsent member operations can be resolved'); END;
CREATE TRIGGER member_conflicts_no_update BEFORE UPDATE ON member_sync_conflicts
BEGIN SELECT RAISE(ABORT,'Conflict history is immutable'); END;
CREATE TRIGGER member_conflicts_no_delete BEFORE DELETE ON member_sync_conflicts
BEGIN SELECT RAISE(ABORT,'Conflict history is immutable'); END;
CREATE TRIGGER member_resolutions_no_update BEFORE UPDATE ON member_conflict_resolutions
BEGIN SELECT RAISE(ABORT,'Resolution history is immutable'); END;
CREATE TRIGGER member_resolutions_no_delete BEFORE DELETE ON member_conflict_resolutions
BEGIN SELECT RAISE(ABORT,'Resolution history is immutable'); END;
CREATE TRIGGER resolved_conflicts_no_update BEFORE UPDATE ON member_resolved_conflicts
BEGIN SELECT RAISE(ABORT,'Resolution history is immutable'); END;
CREATE TRIGGER resolved_conflicts_no_delete BEFORE DELETE ON member_resolved_conflicts
BEGIN SELECT RAISE(ABORT,'Resolution history is immutable'); END;
CREATE TRIGGER resolved_operations_no_update BEFORE UPDATE ON member_resolved_operations
BEGIN SELECT RAISE(ABORT,'Resolution history is immutable'); END;
CREATE TRIGGER resolved_operations_no_delete BEFORE DELETE ON member_resolved_operations
BEGIN SELECT RAISE(ABORT,'Resolution history is immutable'); END;
PRAGMA user_version=6;
