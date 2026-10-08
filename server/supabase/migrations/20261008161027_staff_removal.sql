-- Keep payroll/attendance references while permanently removing operational staff profiles.
ALTER TABLE armstrong.business_records DROP CONSTRAINT business_records_table_name_check;
ALTER TABLE armstrong.business_records ADD CONSTRAINT business_records_table_name_check
 CHECK(table_name IN ('users','members','plans','gym_settings','nfc_cards','membership_periods','attendance','products','sales','sale_items','stock_movements','expenses','expense_voids','invoices','invoice_details','payments','payment_allocations','payment_reversal_details','allocation_reversals','payment_receipts','audit','trainers','member_trainers','training_charges','staff_payouts','staff_payout_items','member_profiles','member_deletions','staff_nfc_cards','staff_attendance','staff_deletions'));
CREATE OR REPLACE FUNCTION armstrong.preserve_business_record() RETURNS trigger
LANGUAGE plpgsql SET search_path=pg_catalog AS $$
DECLARE permitted text[];
BEGIN
 IF TG_OP='DELETE' OR NEW.gym_id<>OLD.gym_id OR NEW.table_name<>OLD.table_name OR NEW.record_id<>OLD.record_id THEN
  RAISE EXCEPTION 'Business identity and history cannot be deleted or moved';
 END IF;
 IF OLD.table_name='trainers' AND NEW.data<>OLD.data AND EXISTS(
  SELECT 1 FROM armstrong.business_records WHERE gym_id=OLD.gym_id AND table_name='staff_deletions' AND record_id=OLD.record_id
 ) THEN RAISE EXCEPTION 'Deleted staff cannot be edited or reactivated'; END IF;
 permitted:=CASE OLD.table_name
  WHEN 'trainers' THEN ARRAY['name','phone','nic','salary_minor','training_fee_minor','active','version']
  WHEN 'member_trainers' THEN ARRAY['trainer_id','version']
  WHEN 'members' THEN ARRAY['name','phone','email','nfc_id','version','archived_at','archived_by_user_id']
  WHEN 'plans' THEN ARRAY['name','duration_months','price_minor','active','version']
  WHEN 'gym_settings' THEN ARRAY['name','location','phone','email','version']
  WHEN 'products' THEN ARRAY['name','sku','cost_minor','price_minor','reorder_level','version']
  WHEN 'users' THEN ARRAY['email','display_name']
  WHEN 'member_profiles' THEN ARRAY['gender','version']
  WHEN 'staff_nfc_cards' THEN ARRAY['revoked_at']
  WHEN 'nfc_cards' THEN ARRAY['revoked_at'] ELSE ARRAY[]::text[] END;
 IF (NEW.data-permitted)<>(OLD.data-permitted) THEN RAISE EXCEPTION 'Business history fields are immutable'; END IF;
 IF NEW.data<>OLD.data AND OLD.table_name IN ('members','plans','gym_settings','products','trainers','member_trainers','member_profiles')
  AND (NEW.data->>'version')::bigint<>(OLD.data->>'version')::bigint+1 THEN
  RAISE EXCEPTION 'Master version must advance by exactly one';
 END IF;
 IF OLD.table_name='members' AND OLD.data->>'archived_at' IS NOT NULL AND NEW.data<>OLD.data THEN
  RAISE EXCEPTION 'Archived member history is immutable';
 END IF;
 IF OLD.table_name IN ('nfc_cards','staff_nfc_cards') AND NEW.data<>OLD.data AND
  (OLD.data->>'revoked_at' IS NOT NULL OR NEW.data->>'revoked_at' IS NULL) THEN
  RAISE EXCEPTION 'Only revoking an active card is allowed';
 END IF;
 RETURN NEW;
END $$;
