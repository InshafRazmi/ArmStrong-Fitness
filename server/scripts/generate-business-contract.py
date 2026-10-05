"""Build the reviewed protocol row contract from the canonical SQLite schemas.

No database credentials or existing user database are read. The generated native
and server copies are identical, and schema migrations remain explicit files.
"""
import json
import sqlite3
from pathlib import Path

root = Path(__file__).resolve().parents[2]
conn = sqlite3.connect(":memory:")
for name in ["001_foundation", "002_local_operations", "003_finance", "004_removal"]:
    conn.executescript((root / "Client/src-tauri/migrations" / f"{name}.sql").read_text())

names = ["users", "members", "plans", "gym_settings", "nfc_cards", "membership_periods",
         "attendance", "products", "sales", "sale_items", "stock_movements", "expenses",
         "expense_voids", "invoices", "invoice_details", "payments", "payment_allocations",
         "payment_reversal_details", "allocation_reversals", "payment_receipts", "audit"]
mutable = {"members": ["name", "phone", "email", "nfc_id", "version", "archived_at", "archived_by_user_id"],
           "plans": ["name", "duration_months", "price_minor", "active", "version"],
           "gym_settings": ["name", "location", "phone", "email", "version"],
           "products": ["name", "sku", "cost_minor", "price_minor", "reorder_level", "version"],
           "nfc_cards": ["revoked_at"], "users": ["email", "display_name"]}
tables = []
for name in names:
    columns = [{"name": r[1], "type": "integer" if r[2] == "INTEGER" else "text",
                "nullable": not bool(r[3] or r[5])} for r in conn.execute(f"PRAGMA table_info({name})")]
    key = next(r[1] for r in conn.execute(f"PRAGMA table_info({name})") if r[5])
    references = [{"column": r[3], "table": r[2], "key": r[4]}
                  for r in conn.execute(f"PRAGMA foreign_key_list({name})")]
    tables.append({"name": name, "key": key, "columns": columns,
                   "mutable": mutable.get(name, []), "references": references})
content = json.dumps({"protocolVersion": 2, "tables": tables}, indent=2) + "\n"
for path in [root / "server/src/business-schema.json", root / "Client/src-tauri/business-schema.json"]:
    path.write_text(content)
print(f"Generated identical contracts for {len(tables)} business tables")

sql = """-- Durable row journal and immutable transaction envelopes; preserve all v6 data.
CREATE TABLE business_dirty (
 table_name TEXT NOT NULL, record_id TEXT NOT NULL, before_json TEXT,
 after_json TEXT NOT NULL CHECK(json_valid(after_json)), PRIMARY KEY(table_name,record_id)
) STRICT;
CREATE TABLE business_batches (
 id TEXT PRIMARY KEY, ordinal INTEGER NOT NULL UNIQUE,
 request_json TEXT NOT NULL CHECK(json_valid(request_json)),
 state TEXT NOT NULL CHECK(state IN ('pending','confirmed','conflict')),
 response_json TEXT, last_error TEXT,
 CHECK(response_json IS NULL OR json_valid(response_json))
) STRICT;
CREATE TABLE business_batch_operations (
 batch_id TEXT NOT NULL REFERENCES business_batches(id),
 operation_id TEXT NOT NULL UNIQUE REFERENCES outbox(id), PRIMARY KEY(batch_id,operation_id)
) STRICT;
CREATE TABLE business_cursor (id INTEGER PRIMARY KEY CHECK(id=1), sequence INTEGER NOT NULL CHECK(sequence>=0)) STRICT;
INSERT INTO business_cursor VALUES(1,0);
CREATE TRIGGER business_request_immutable BEFORE UPDATE ON business_batches
WHEN NEW.id<>OLD.id OR NEW.ordinal<>OLD.ordinal OR NEW.request_json<>OLD.request_json
 OR (OLD.state='confirmed' AND (NEW.state<>OLD.state OR NEW.response_json IS NOT OLD.response_json))
BEGIN SELECT RAISE(ABORT,'Business request and confirmed receipt are immutable'); END;
CREATE TRIGGER business_batch_no_delete BEFORE DELETE ON business_batches
BEGIN SELECT RAISE(ABORT,'Business delivery history cannot be deleted'); END;
CREATE TRIGGER business_operation_no_update BEFORE UPDATE ON business_batch_operations
BEGIN SELECT RAISE(ABORT,'Business operation mapping is immutable'); END;
CREATE TRIGGER business_operation_no_delete BEFORE DELETE ON business_batch_operations
BEGIN SELECT RAISE(ABORT,'Business operation mapping is immutable'); END;
DROP TRIGGER member_card_insert;
DROP TRIGGER member_card_update;
CREATE TRIGGER member_card_insert AFTER INSERT ON members WHEN NEW.nfc_id IS NOT NULL
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN INSERT INTO nfc_cards VALUES(lower(hex(randomblob(16))),NEW.id,NEW.nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL); END;
CREATE TRIGGER member_card_update AFTER UPDATE OF nfc_id ON members WHEN OLD.nfc_id IS NOT NEW.nfc_id
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 UPDATE nfc_cards SET revoked_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE member_id=NEW.id AND revoked_at IS NULL;
 INSERT INTO nfc_cards SELECT lower(hex(randomblob(16))),NEW.id,NEW.nfc_id,strftime('%Y-%m-%dT%H:%M:%fZ','now'),NULL WHERE NEW.nfc_id IS NOT NULL;
END;
"""

def projection(table, prefix):
    return "json_object(" + ",".join(f"'{c['name']}',{prefix}{c['name']}" for c in table["columns"]) + ")"

for table in tables:
    name, key = table["name"], table["key"]
    for verb in ["INSERT", "UPDATE"]:
        before = "NULL" if verb == "INSERT" else projection(table, "OLD.")
        after = projection(table, "NEW.")
        sql += f"""CREATE TRIGGER business_{name}_{verb.lower()} AFTER {verb} ON {name}
WHEN COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN
 INSERT INTO business_dirty VALUES('{name}',CAST(NEW.{key} AS TEXT),{before},{after})
 ON CONFLICT(table_name,record_id) DO UPDATE SET after_json=excluded.after_json;
END;
"""
    sql += f"INSERT INTO business_dirty SELECT '{name}',CAST({key} AS TEXT),NULL,{projection(table, '')} FROM {name};\n"
sql += """-- Verified historical download may contain attendance/periods preceding archive,
-- and legacy reversal rows. Ordinary local commands retain all original guards.
DROP TRIGGER archived_membership_insert;
DROP TRIGGER archived_attendance_insert;
CREATE TRIGGER archived_membership_insert BEFORE INSERT ON membership_periods WHEN
 COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND EXISTS(SELECT 1 FROM members WHERE id=NEW.member_id AND archived_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Archived members cannot receive new memberships'); END;
CREATE TRIGGER archived_attendance_insert BEFORE INSERT ON attendance WHEN
 COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND EXISTS(SELECT 1 FROM members WHERE id=NEW.member_id AND archived_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT,'Archived members cannot record attendance'); END;
DROP TRIGGER expenses_new_reversal;
CREATE TRIGGER expenses_new_reversal BEFORE INSERT ON expenses WHEN NEW.reverses_id IS NOT NULL
 AND COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
BEGIN SELECT RAISE(ABORT,'Use the audited expense void workflow'); END;
DROP TRIGGER expense_void_parent;
CREATE TRIGGER expense_void_parent BEFORE INSERT ON expense_voids WHEN
 NOT EXISTS(SELECT 1 FROM expenses WHERE id=NEW.expense_id AND reverses_id IS NULL)
 OR (COALESCE((SELECT value FROM metadata WHERE key='business_import'),'0')<>'1'
 AND (NEW.legacy_reversal_id IS NOT NULL OR EXISTS(SELECT 1 FROM expenses WHERE reverses_id=NEW.expense_id)))
BEGIN SELECT RAISE(ABORT,'Only an original, unvoided expense may be voided'); END;
"""
sql += "PRAGMA user_version=7;\n"
(root / "Client/src-tauri/migrations/007_business_sync.sql").write_text(sql)
print("Generated explicit SQLite v7 journal migration")
