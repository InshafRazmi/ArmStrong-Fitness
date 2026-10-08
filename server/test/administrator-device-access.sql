-- Real PostgreSQL enrollment regression. All synthetic rows roll back.
BEGIN;
SET LOCAL statement_timeout = '15s';
DO $$
DECLARE
  gym uuid := gen_random_uuid();
  other_gym uuid := gen_random_uuid();
  admin uuid := gen_random_uuid();
  reception uuid := gen_random_uuid();
  writer uuid := gen_random_uuid();
  previous_reader uuid := gen_random_uuid();
  new_device uuid := gen_random_uuid();
  foreign_device uuid := gen_random_uuid();
  proof text := repeat('a',64);
  reply jsonb;
  original_devices jsonb;
BEGIN
  IF to_regclass('armstrong.one_active_writer') IS NOT NULL THEN
    RAISE EXCEPTION 'obsolete single-writer index remains';
  END IF;
  IF has_function_privilege('anon','armstrong.enroll_desktop(uuid,uuid,text)','EXECUTE')
    OR has_function_privilege('authenticated','armstrong.enroll_desktop(uuid,uuid,text)','EXECUTE') THEN
    RAISE EXCEPTION 'public client can execute private enrollment';
  END IF;
  INSERT INTO armstrong.gyms(id,name) VALUES(gym,'Synthetic Administrator access'),(other_gym,'Synthetic other gym');
  INSERT INTO armstrong.staff(gym_id,user_id,display_name,role) VALUES
    (gym,admin,'Synthetic Administrator','Administrator'),(gym,reception,'Synthetic Reception','Reception');
  INSERT INTO armstrong.devices(gym_id,id,secret_sha256,can_write) VALUES
    (gym,writer,proof,true),(gym,previous_reader,proof,false),(other_gym,foreign_device,proof,true);

  reply := armstrong.enroll_desktop(admin,previous_reader,proof);
  IF reply->'device'->>'canWrite' IS DISTINCT FROM 'true' OR reply->'device'->>'id' IS DISTINCT FROM previous_reader::text THEN
    RAISE EXCEPTION 'existing Administrator device stayed read-only';
  END IF;
  reply := armstrong.enroll_desktop(admin,new_device,proof);
  IF reply->'device'->>'canWrite' IS DISTINCT FROM 'true' THEN RAISE EXCEPTION 'second computer cannot edit'; END IF;
  IF (SELECT count(*) FROM armstrong.devices WHERE gym_id=gym AND active AND can_write) <> 3 THEN
    RAISE EXCEPTION 'multiple Administrator computers were not preserved';
  END IF;
  SELECT jsonb_agg(to_jsonb(d) ORDER BY d.id) INTO original_devices FROM armstrong.devices d WHERE d.gym_id=gym;
  PERFORM armstrong.enroll_desktop(admin,new_device,proof);
  IF original_devices <> (SELECT jsonb_agg(to_jsonb(d) ORDER BY d.id) FROM armstrong.devices d WHERE d.gym_id=gym) THEN
    RAISE EXCEPTION 'exact retry changed device identity or proof';
  END IF;
  BEGIN
    PERFORM armstrong.enroll_desktop(admin,new_device,repeat('b',64));
    RAISE EXCEPTION 'changed device credential accepted';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  BEGIN
    PERFORM armstrong.enroll_desktop(reception,gen_random_uuid(),proof);
    RAISE EXCEPTION 'Reception gained Administrator enrollment';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  BEGIN
    PERFORM armstrong.enroll_desktop(gen_random_uuid(),gen_random_uuid(),proof);
    RAISE EXCEPTION 'unregistered account enrolled a computer';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  BEGIN
    PERFORM armstrong.enroll_desktop(admin,foreign_device,proof);
    RAISE EXCEPTION 'another gym device was adopted';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  UPDATE armstrong.devices SET active=false WHERE gym_id=gym AND id=previous_reader;
  BEGIN
    PERFORM armstrong.enroll_desktop(admin,previous_reader,proof);
    RAISE EXCEPTION 'revoked device was revived';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  UPDATE armstrong.staff SET active=false WHERE gym_id=gym AND user_id=admin;
  BEGIN
    PERFORM armstrong.enroll_desktop(admin,new_device,proof);
    RAISE EXCEPTION 'inactive Administrator retained enrollment';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  UPDATE armstrong.staff SET active=true,role='Reception' WHERE gym_id=gym AND user_id=admin;
  BEGIN
    PERFORM armstrong.enroll_desktop(admin,new_device,proof);
    RAISE EXCEPTION 'demoted account retained Administrator enrollment';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
  UPDATE armstrong.staff SET role='Administrator' WHERE gym_id=gym AND user_id=admin;
  PERFORM set_config('request.jwt.claim.sub',reception::text,true);
  BEGIN
    PERFORM armstrong.enroll_desktop(admin,new_device,proof);
    RAISE EXCEPTION 'different JWT subject adopted Administrator permissions';
  EXCEPTION WHEN insufficient_privilege THEN NULL;
  END;
END $$;
ROLLBACK;
SELECT true AS multiple_admin_devices, true AS retained_credentials,
  true AS revoked_and_inactive_denied, true AS reception_and_other_gym_denied,
  true AS private_function_permissions, true AS synthetic_rows_rolled_back;
