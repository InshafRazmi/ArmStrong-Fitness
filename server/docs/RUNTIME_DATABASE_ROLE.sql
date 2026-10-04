-- Administrative setup for the private Node API, not an application migration.
-- The role is initially NOLOGIN: verify the grants before provisioning a password.
-- Run once as the existing migration owner. Never put a password in this file.
-- Fail rather than overwrite an existing role or change its credentials/grants.
DO $runtime_access$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'armstrong_api') THEN
    RAISE EXCEPTION 'Runtime role already exists; inspect it before changing access';
  END IF;
  IF (SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
      WHERE n.nspname = 'armstrong' AND c.relkind = 'r' AND c.relrowsecurity
      AND c.relname IN ('gyms', 'staff', 'devices', 'members', 'member_operations', 'member_changes')) <> 6 THEN
    RAISE EXCEPTION 'Six private RLS-enabled API tables are required';
  END IF;

  -- The API verifies identity and device authorization before every transaction.
  -- Its private tables have no Data API policies. BYPASSRLS is combined with only
  -- these table/column grants; no ownership, role memberships or public API grants.
  CREATE ROLE armstrong_api NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
    NOINHERIT NOREPLICATION BYPASSRLS CONNECTION LIMIT 12;
  GRANT CONNECT ON DATABASE postgres TO armstrong_api;
  GRANT USAGE ON SCHEMA armstrong TO armstrong_api;
  GRANT SELECT ON armstrong.gyms, armstrong.staff, armstrong.devices,
    armstrong.members, armstrong.member_operations, armstrong.member_changes TO armstrong_api;
  GRANT UPDATE (change_sequence) ON armstrong.gyms TO armstrong_api;
  GRANT INSERT ON armstrong.members TO armstrong_api;
  GRANT UPDATE (name, phone, email, nfc_id, revision, archived_at, archived_by_user_id)
    ON armstrong.members TO armstrong_api;
  GRANT INSERT ON armstrong.member_operations, armstrong.member_changes TO armstrong_api;
  ALTER ROLE armstrong_api SET statement_timeout = '10s';
  ALTER ROLE armstrong_api SET search_path = pg_catalog;
END
$runtime_access$;
