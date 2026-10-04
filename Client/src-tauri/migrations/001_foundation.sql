CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE plans (
 id TEXT PRIMARY KEY, name TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK(length(trim(name)) BETWEEN 1 AND 80),
 duration_months INTEGER NOT NULL CHECK(duration_months BETWEEN 1 AND 60),
 price_minor INTEGER NOT NULL CHECK(price_minor BETWEEN 0 AND 100000000000),
 active INTEGER NOT NULL CHECK(active IN (0,1)), version INTEGER NOT NULL CHECK(version > 0)
);
CREATE TABLE members (
 id TEXT PRIMARY KEY, name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
 phone TEXT NOT NULL CHECK(length(trim(phone)) BETWEEN 1 AND 40), email TEXT NOT NULL,
 nfc_id TEXT UNIQUE COLLATE NOCASE CHECK(nfc_id IS NULL OR length(nfc_id) BETWEEN 1 AND 128),
 joined_on TEXT NOT NULL, version INTEGER NOT NULL CHECK(version > 0)
);
CREATE TABLE membership_periods (
 id TEXT PRIMARY KEY, member_id TEXT NOT NULL REFERENCES members(id), plan_id TEXT NOT NULL REFERENCES plans(id),
 plan_name TEXT NOT NULL, price_minor INTEGER NOT NULL CHECK(price_minor >= 0),
 starts_on TEXT NOT NULL, ends_on TEXT NOT NULL CHECK(ends_on >= starts_on),
 created_at TEXT NOT NULL
);
CREATE INDEX membership_member_dates ON membership_periods(member_id, starts_on, ends_on);
CREATE TRIGGER membership_no_overlap BEFORE INSERT ON membership_periods
WHEN EXISTS(SELECT 1 FROM membership_periods WHERE member_id=NEW.member_id AND starts_on<=NEW.ends_on AND ends_on>=NEW.starts_on)
BEGIN SELECT RAISE(ABORT, 'Membership dates overlap an existing period'); END;
CREATE TABLE audit (
 id TEXT PRIMARY KEY, actor TEXT NOT NULL, device_id TEXT NOT NULL, action TEXT NOT NULL,
 entity_id TEXT NOT NULL, before_json TEXT, after_json TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE outbox (
 id TEXT PRIMARY KEY, device_id TEXT NOT NULL, entity TEXT NOT NULL, entity_id TEXT NOT NULL,
 action TEXT NOT NULL, expected_version INTEGER, payload_json TEXT NOT NULL,
 schema_version INTEGER NOT NULL, created_at TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0
);
PRAGMA user_version = 1;
