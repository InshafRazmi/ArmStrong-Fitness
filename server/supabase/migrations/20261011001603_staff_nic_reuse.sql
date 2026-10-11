-- Permit a deleted staff profile's NIC to be used by a new profile, while
-- keeping NICs unique among current staff in each gym.
DROP INDEX armstrong.trainer_nic_per_gym;
CREATE INDEX trainer_nic_lookup ON armstrong.business_records(gym_id,(data->>'nic')) WHERE table_name='trainers';

CREATE OR REPLACE FUNCTION armstrong.staff_nic_unique() RETURNS trigger
LANGUAGE plpgsql SET search_path=pg_catalog AS $$
BEGIN
 PERFORM 1 FROM armstrong.gyms WHERE id=NEW.gym_id FOR UPDATE;
 IF EXISTS(
  SELECT 1 FROM armstrong.business_records t
  WHERE t.gym_id=NEW.gym_id AND t.table_name='trainers'
    AND NOT EXISTS(SELECT 1 FROM armstrong.business_records d
      WHERE d.gym_id=t.gym_id AND d.table_name='staff_deletions' AND d.record_id=t.record_id)
  GROUP BY t.data->>'nic' HAVING count(*)>1
 ) THEN RAISE EXCEPTION USING ERRCODE='23505',MESSAGE='A staff member with this NIC number already exists'; END IF;
 RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION armstrong.staff_nic_unique() FROM PUBLIC,anon,authenticated;

DROP TRIGGER IF EXISTS staff_nic_unique ON armstrong.business_records;
CREATE CONSTRAINT TRIGGER staff_nic_unique AFTER INSERT OR UPDATE ON armstrong.business_records
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
 WHEN (NEW.table_name IN ('trainers','staff_deletions'))
 EXECUTE FUNCTION armstrong.staff_nic_unique();
