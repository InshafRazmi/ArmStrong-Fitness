-- Real PostgreSQL permission checks, executed as the migration owner.
-- Every write has WHERE false; no business row is inserted, edited or deleted.
-- The creator's temporary SET ROLE permission is rolled back with the test.
BEGIN;
GRANT armstrong_api TO postgres WITH SET TRUE;
SET LOCAL ROLE armstrong_api;

SELECT current_user AS tested_role,
  (SELECT count(*) FROM armstrong.gyms) AS gyms,
  (SELECT count(*) FROM armstrong.staff) AS staff,
  (SELECT count(*) FROM armstrong.devices) AS devices,
  (SELECT count(*) FROM armstrong.members) AS members,
  (SELECT count(*) FROM armstrong.member_operations) AS operations,
  (SELECT count(*) FROM armstrong.member_changes) AS changes;
SELECT id FROM armstrong.gyms WHERE false FOR UPDATE;
UPDATE armstrong.gyms SET change_sequence=change_sequence WHERE false;
INSERT INTO armstrong.members SELECT * FROM armstrong.members WHERE false;
UPDATE armstrong.members SET name=name, phone=phone, email=email, nfc_id=nfc_id,
  revision=revision, archived_at=archived_at, archived_by_user_id=archived_by_user_id
  WHERE false;
INSERT INTO armstrong.member_operations SELECT * FROM armstrong.member_operations WHERE false;
INSERT INTO armstrong.member_changes SELECT * FROM armstrong.member_changes WHERE false;

DO $verify_denials$
DECLARE statement text;
BEGIN
  FOREACH statement IN ARRAY ARRAY[
    'UPDATE armstrong.staff SET active=active WHERE false',
    'INSERT INTO armstrong.staff SELECT * FROM armstrong.staff WHERE false',
    'UPDATE armstrong.devices SET can_write=can_write WHERE false',
    'INSERT INTO armstrong.devices SELECT * FROM armstrong.devices WHERE false',
    'INSERT INTO armstrong.gyms SELECT * FROM armstrong.gyms WHERE false',
    'UPDATE armstrong.gyms SET name=name WHERE false',
    'UPDATE armstrong.members SET gym_id=gym_id WHERE false',
    'UPDATE armstrong.members SET joined_on=joined_on WHERE false',
    'UPDATE armstrong.member_operations SET receipt=receipt WHERE false',
    'UPDATE armstrong.member_changes SET snapshot=snapshot WHERE false',
    'DELETE FROM armstrong.gyms WHERE false',
    'DELETE FROM armstrong.staff WHERE false',
    'DELETE FROM armstrong.devices WHERE false',
    'DELETE FROM armstrong.members WHERE false',
    'DELETE FROM armstrong.member_operations WHERE false',
    'DELETE FROM armstrong.member_changes WHERE false'
  ] LOOP
    BEGIN
      EXECUTE statement;
      RAISE EXCEPTION 'Runtime access verification found an unexpected write permission';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
  END LOOP;
END
$verify_denials$;
RESET ROLE;

SELECT 'PASS' AS runtime_access_verification,
  rolcanlogin AS login_enabled, rolsuper AS superuser, rolcreatedb AS create_database,
  rolcreaterole AS create_role, rolreplication AS replication,
  rolbypassrls AS private_api_rls_bypass, rolinherit AS inherit,
  rolconnlimit AS connection_limit,
  (SELECT count(*) FROM pg_auth_members WHERE member=pg_roles.oid) AS inherited_role_memberships,
  (SELECT count(*) FROM pg_class WHERE relowner=pg_roles.oid) AS owned_relations,
  has_schema_privilege('armstrong_api','armstrong','CREATE') AS create_private_schema_objects,
  has_schema_privilege('armstrong_api','public','CREATE') AS create_public_schema_objects,
  has_schema_privilege('armstrong_api','auth','USAGE') AS auth_schema_usage,
  has_schema_privilege('armstrong_api','storage','USAGE') AS storage_schema_usage,
  (SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
   WHERE n.nspname='armstrong' AND c.relkind='r' AND c.relrowsecurity) AS rls_tables
FROM pg_roles WHERE rolname='armstrong_api';
ROLLBACK;
