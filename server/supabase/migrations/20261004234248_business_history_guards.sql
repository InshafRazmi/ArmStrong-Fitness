-- Fix advisor search-path findings without changing the private API access model.
ALTER FUNCTION armstrong.lock_identity_gym() SET search_path=pg_catalog;
ALTER FUNCTION armstrong.immutable() SET search_path=pg_catalog;
ALTER FUNCTION armstrong.preserve_archive() SET search_path=pg_catalog;
CREATE OR REPLACE FUNCTION armstrong.preserve_business_record() RETURNS trigger
LANGUAGE plpgsql SET search_path=pg_catalog AS $$
DECLARE permitted text[];
BEGIN
 IF TG_OP='DELETE' OR NEW.gym_id<>OLD.gym_id OR NEW.table_name<>OLD.table_name OR NEW.record_id<>OLD.record_id THEN
  RAISE EXCEPTION 'Business identity and history cannot be deleted or moved';
 END IF;
 permitted:=CASE OLD.table_name
  WHEN 'members' THEN ARRAY['name','phone','email','nfc_id','version','archived_at','archived_by_user_id']
  WHEN 'plans' THEN ARRAY['name','duration_months','price_minor','active','version']
  WHEN 'gym_settings' THEN ARRAY['name','location','phone','email','version']
  WHEN 'products' THEN ARRAY['name','sku','cost_minor','price_minor','reorder_level','version']
  WHEN 'users' THEN ARRAY['email','display_name']
  WHEN 'nfc_cards' THEN ARRAY['revoked_at'] ELSE ARRAY[]::text[] END;
 IF (NEW.data-permitted)<>(OLD.data-permitted) THEN RAISE EXCEPTION 'Business history fields are immutable'; END IF;
 IF NEW.data<>OLD.data AND OLD.table_name IN ('members','plans','gym_settings','products')
  AND (NEW.data->>'version')::bigint<>(OLD.data->>'version')::bigint+1 THEN
  RAISE EXCEPTION 'Master version must advance by exactly one';
 END IF;
 IF OLD.table_name='members' AND OLD.data->>'archived_at' IS NOT NULL AND NEW.data<>OLD.data THEN
  RAISE EXCEPTION 'Archived member history is immutable';
 END IF;
 IF OLD.table_name='nfc_cards' AND NEW.data<>OLD.data AND
  (OLD.data->>'revoked_at' IS NOT NULL OR NEW.data->>'revoked_at' IS NULL) THEN
  RAISE EXCEPTION 'Only revoking an active card is allowed';
 END IF;
 RETURN NEW;
END $$;
ALTER TABLE armstrong.business_records ADD CONSTRAINT business_record_key_matches
 CHECK((CASE table_name WHEN 'gym_settings' THEN data->>'id'
  WHEN 'expense_voids' THEN data->>'expense_id' WHEN 'invoice_details' THEN data->>'invoice_id'
  WHEN 'payment_reversal_details' THEN data->>'payment_id' WHEN 'payment_receipts' THEN data->>'payment_id'
  ELSE data->>'id' END) IS NOT DISTINCT FROM record_id);
ALTER TABLE armstrong.business_records ADD CONSTRAINT business_identity_reference_only
 CHECK(table_name<>'users' OR (data->>'active'='0' AND data->>'version'='1'));
