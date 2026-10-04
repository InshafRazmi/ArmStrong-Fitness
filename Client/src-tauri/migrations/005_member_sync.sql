-- Original outbox rows remain immutable history. Only separate confirmed receipts
-- remove an operation from the pending view; no DELETE trigger is weakened.
CREATE TABLE member_deliveries (
 operation_id TEXT PRIMARY KEY REFERENCES outbox(id),
 request_json TEXT NOT NULL CHECK(json_valid(request_json)),
 state TEXT NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','acknowledged','conflict')),
 response_json TEXT CHECK(response_json IS NULL OR json_valid(response_json)),
 attempts INTEGER NOT NULL DEFAULT 1 CHECK(attempts>0),
 last_error TEXT, acknowledged_at TEXT
) STRICT;
CREATE TRIGGER member_delivery_request_immutable BEFORE UPDATE ON member_deliveries
WHEN NEW.operation_id<>OLD.operation_id OR NEW.request_json<>OLD.request_json
 OR (OLD.state='acknowledged' AND (NEW.state<>OLD.state OR NEW.response_json IS NOT OLD.response_json))
BEGIN SELECT RAISE(ABORT,'Member delivery request/receipt is immutable'); END;
CREATE TABLE member_remote_heads (
 member_id TEXT PRIMARY KEY, revision INTEGER NOT NULL CHECK(revision>0),
 snapshot_json TEXT NOT NULL CHECK(json_valid(snapshot_json))
) STRICT;
CREATE TABLE member_sync_cursor (id INTEGER PRIMARY KEY CHECK(id=1), sequence INTEGER NOT NULL CHECK(sequence>=0)) STRICT;
INSERT INTO member_sync_cursor VALUES(1,0);
CREATE TABLE member_sync_conflicts (
 id TEXT PRIMARY KEY, member_id TEXT NOT NULL, operation_id TEXT,
 reason TEXT NOT NULL, remote_json TEXT NOT NULL CHECK(json_valid(remote_json)),
 created_at TEXT NOT NULL
) STRICT;
CREATE INDEX member_conflict_entity ON member_sync_conflicts(member_id);
PRAGMA user_version=5;
