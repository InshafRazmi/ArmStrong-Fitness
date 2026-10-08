-- Active Administrator accounts may edit from any valid enrolled computer.
-- Existing records, identities and credential hashes are retained. The API
-- still verifies the Auth subject before invoking this private function.
DROP INDEX IF EXISTS armstrong.one_active_writer;

CREATE OR REPLACE FUNCTION armstrong.enroll_desktop(p_user uuid, p_device uuid, p_secret_sha256 text)
RETURNS jsonb LANGUAGE plpgsql SECURITY DEFINER SET search_path = '' AS $$
DECLARE
  v_gym uuid;
  v_staff armstrong.staff%ROWTYPE;
  v_device armstrong.devices%ROWTYPE;
  v_count integer;
BEGIN
  IF p_user IS NULL OR p_device IS NULL OR p_secret_sha256 IS NULL OR p_secret_sha256 !~ '^[a-f0-9]{64}$' THEN
    RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='registration_not_authorized';
  END IF;
  -- auth.uid() is normally NULL on the server's private SQL connection. If an
  -- owner deliberately calls with JWT settings, they must match the subject.
  IF auth.uid() IS NOT NULL AND auth.uid() <> p_user THEN
    RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='registration_not_authorized';
  END IF;
  SELECT count(*), min(s.gym_id::text)::uuid INTO v_count,v_gym
    FROM armstrong.staff s WHERE s.user_id=p_user AND s.active AND s.role='Administrator';
  IF v_count <> 1 THEN
    RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='staff_not_authorized';
  END IF;
  -- Lock order matches the protected writes and staff/device revoke triggers.
  PERFORM g.id FROM armstrong.gyms g WHERE g.id=v_gym FOR UPDATE;
  SELECT * INTO v_staff FROM armstrong.staff s
    WHERE s.gym_id=v_gym AND s.user_id=p_user AND s.active AND s.role='Administrator';
  IF NOT FOUND THEN RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='staff_not_authorized'; END IF;
  -- Serialize the same public device identity even across different gyms.
  PERFORM pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtextextended(p_device::text,714339807));
  IF EXISTS(SELECT 1 FROM armstrong.devices d WHERE d.id=p_device AND d.gym_id<>v_gym) THEN
    RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='device_not_authorized';
  END IF;
  SELECT * INTO v_device FROM armstrong.devices d WHERE d.gym_id=v_gym AND d.id=p_device;
  IF FOUND THEN
    -- Keep possession proof and revocation intact. An active Administrator
    -- can upgrade an earlier read-only enrollment without replacing its secret.
    IF NOT v_device.active OR v_device.secret_sha256 <> p_secret_sha256 THEN
      RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='device_not_authorized';
    END IF;
    IF NOT v_device.can_write THEN
      UPDATE armstrong.devices SET can_write=true WHERE gym_id=v_gym AND id=p_device
        RETURNING * INTO v_device;
    END IF;
  ELSE
    INSERT INTO armstrong.devices(gym_id,id,secret_sha256,active,can_write)
      VALUES(v_gym,p_device,p_secret_sha256,true,true) RETURNING * INTO v_device;
  END IF;
  RETURN pg_catalog.jsonb_build_object('protocolVersion',1,
    'gym',pg_catalog.jsonb_build_object('id',v_gym,'name',(SELECT g.name FROM armstrong.gyms g WHERE g.id=v_gym)),
    'staff',pg_catalog.jsonb_build_object('id',v_staff.user_id,'name',v_staff.display_name,'role',v_staff.role),
    'device',pg_catalog.jsonb_build_object('id',v_device.id,'canWrite',v_device.can_write));
END $$;
REVOKE ALL ON FUNCTION armstrong.enroll_desktop(uuid,uuid,text) FROM PUBLIC,anon,authenticated;
-- Runtime remains unable to INSERT/UPDATE staff or arbitrary device grants.
-- Owner-only test databases need no runtime role created by this migration.
DO $$ BEGIN
 IF EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='armstrong_api') THEN
   GRANT EXECUTE ON FUNCTION armstrong.enroll_desktop(uuid,uuid,text) TO armstrong_api;
 END IF;
END $$;
