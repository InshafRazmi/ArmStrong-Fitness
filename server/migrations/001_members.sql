-- Apply with a migration owner, then use a separate least-privilege runtime login.
CREATE SCHEMA armstrong;
REVOKE ALL ON SCHEMA armstrong FROM PUBLIC;
CREATE TABLE armstrong.gyms (
 id uuid PRIMARY KEY, name text NOT NULL, change_sequence bigint NOT NULL DEFAULT 0
 CHECK(change_sequence BETWEEN 0 AND 9007199254740991)
);
CREATE TABLE armstrong.staff (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id), user_id uuid NOT NULL,
 display_name text NOT NULL CHECK(length(btrim(display_name)) BETWEEN 1 AND 120),
 role text NOT NULL CHECK(role IN ('Administrator','Reception')), active boolean NOT NULL DEFAULT true,
 PRIMARY KEY(gym_id,user_id)
);
CREATE TABLE armstrong.devices (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id), id uuid NOT NULL,
 secret_sha256 text NOT NULL CHECK(secret_sha256 ~ '^[a-f0-9]{64}$'),
 active boolean NOT NULL DEFAULT true, can_write boolean NOT NULL DEFAULT false,
 PRIMARY KEY(gym_id,id)
);
-- Deliberately one authorized writer until the owner approves multi-writer topology.
CREATE UNIQUE INDEX one_active_writer ON armstrong.devices(gym_id) WHERE active AND can_write;
-- Administrative revocation uses the same gym lock as protected API writes.
-- Runtime needs only SELECT on staff/devices, never privilege to self-grant roles.
CREATE FUNCTION armstrong.lock_identity_gym() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='DELETE' THEN
   PERFORM id FROM armstrong.gyms WHERE id=OLD.gym_id FOR UPDATE;
   RETURN OLD;
 END IF;
 IF TG_OP='UPDATE' AND NEW.gym_id<>OLD.gym_id THEN
   RAISE EXCEPTION 'Identity gym mapping is immutable';
 END IF;
 PERFORM id FROM armstrong.gyms WHERE id=NEW.gym_id FOR UPDATE;
 RETURN NEW;
END $$;
CREATE TRIGGER staff_permission_lock BEFORE INSERT OR UPDATE OR DELETE ON armstrong.staff
 FOR EACH ROW EXECUTE FUNCTION armstrong.lock_identity_gym();
CREATE TRIGGER device_permission_lock BEFORE INSERT OR UPDATE OR DELETE ON armstrong.devices
 FOR EACH ROW EXECUTE FUNCTION armstrong.lock_identity_gym();
CREATE TABLE armstrong.members (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id), id uuid NOT NULL,
 name text NOT NULL CHECK(length(btrim(name)) BETWEEN 1 AND 120),
 phone text NOT NULL CHECK(length(btrim(phone)) BETWEEN 1 AND 40),
 email text NOT NULL CHECK(octet_length(email)<=254),
 nfc_id text CHECK(nfc_id IS NULL OR (nfc_id ~ '^[!-~]{1,128}$' AND nfc_id=upper(nfc_id))),
 joined_on date NOT NULL CHECK(joined_on BETWEEN '1900-01-01' AND '2200-12-31'),
 revision bigint NOT NULL CHECK(revision BETWEEN 1 AND 9007199254740991),
 archived_at timestamptz, archived_by_user_id uuid,
 PRIMARY KEY(gym_id,id), UNIQUE(gym_id,nfc_id),
 FOREIGN KEY(gym_id,archived_by_user_id) REFERENCES armstrong.staff(gym_id,user_id),
 CHECK((archived_at IS NULL)=(archived_by_user_id IS NULL))
);
CREATE TABLE armstrong.member_operations (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id), id uuid NOT NULL,
 device_id uuid NOT NULL, actor_user_id uuid NOT NULL,
 request jsonb NOT NULL, receipt jsonb NOT NULL, created_at timestamptz NOT NULL DEFAULT now(),
 PRIMARY KEY(gym_id,id),
 FOREIGN KEY(gym_id,device_id) REFERENCES armstrong.devices(gym_id,id),
 FOREIGN KEY(gym_id,actor_user_id) REFERENCES armstrong.staff(gym_id,user_id)
);
CREATE TABLE armstrong.member_changes (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id), sequence bigint NOT NULL,
 operation_id uuid NOT NULL, member_id uuid NOT NULL, snapshot jsonb NOT NULL,
 PRIMARY KEY(gym_id,sequence), UNIQUE(gym_id,operation_id),
 FOREIGN KEY(gym_id,operation_id) REFERENCES armstrong.member_operations(gym_id,id),
 FOREIGN KEY(gym_id,member_id) REFERENCES armstrong.members(gym_id,id)
);
CREATE FUNCTION armstrong.immutable() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'Member sync history is append-only'; END $$;
CREATE TRIGGER immutable_operations BEFORE UPDATE OR DELETE ON armstrong.member_operations
 FOR EACH ROW EXECUTE FUNCTION armstrong.immutable();
CREATE TRIGGER immutable_changes BEFORE UPDATE OR DELETE ON armstrong.member_changes
 FOR EACH ROW EXECUTE FUNCTION armstrong.immutable();
CREATE FUNCTION armstrong.preserve_archive() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF OLD.archived_at IS NOT NULL THEN RAISE EXCEPTION 'Archived member history is immutable'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER preserve_archive BEFORE UPDATE ON armstrong.members
 FOR EACH ROW EXECUTE FUNCTION armstrong.preserve_archive();
-- Defense in depth: no public/Data API policies. Runtime SQL login must own these
-- tables or have BYPASSRLS plus only the explicit grants in README (never anon).
ALTER TABLE armstrong.gyms ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.staff ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.members ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.member_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.member_changes ENABLE ROW LEVEL SECURITY;
REVOKE ALL ON ALL TABLES IN SCHEMA armstrong FROM PUBLIC;
