-- Protocol 2: gym-scoped current rows, foreign references, immutable transactions.
ALTER TABLE armstrong.gyms ADD COLUMN business_sequence bigint NOT NULL DEFAULT 0
 CHECK(business_sequence BETWEEN 0 AND 9007199254740991);
CREATE TABLE armstrong.business_records (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id),
 table_name text NOT NULL CHECK(table_name IN ('users','members','plans','gym_settings','nfc_cards','membership_periods','attendance','products','sales','sale_items','stock_movements','expenses','expense_voids','invoices','invoice_details','payments','payment_allocations','payment_reversal_details','allocation_reversals','payment_receipts','audit')),
 record_id text NOT NULL CHECK(length(record_id) BETWEEN 1 AND 128),
 data jsonb NOT NULL CHECK(jsonb_typeof(data)='object'),
 PRIMARY KEY(gym_id,table_name,record_id)
);
CREATE TABLE armstrong.business_references (
 gym_id uuid NOT NULL, table_name text NOT NULL, record_id text NOT NULL,
 column_name text NOT NULL, target_table text NOT NULL, target_id text NOT NULL,
 PRIMARY KEY(gym_id,table_name,record_id,column_name),
 FOREIGN KEY(gym_id,table_name,record_id) REFERENCES armstrong.business_records(gym_id,table_name,record_id),
 FOREIGN KEY(gym_id,target_table,target_id) REFERENCES armstrong.business_records(gym_id,table_name,record_id)
);
CREATE INDEX business_reference_target ON armstrong.business_references(gym_id,target_table,target_id);
CREATE TABLE armstrong.business_operations (
 gym_id uuid NOT NULL REFERENCES armstrong.gyms(id), id uuid NOT NULL,
 device_id uuid NOT NULL, actor_user_id uuid NOT NULL,
 request jsonb NOT NULL CHECK(jsonb_typeof(request)='object'),
 receipt jsonb NOT NULL CHECK(jsonb_typeof(receipt)='object'),
 created_at timestamptz NOT NULL DEFAULT now(), PRIMARY KEY(gym_id,id),
 FOREIGN KEY(gym_id,device_id) REFERENCES armstrong.devices(gym_id,id),
 FOREIGN KEY(gym_id,actor_user_id) REFERENCES armstrong.staff(gym_id,user_id)
);
CREATE INDEX business_operation_device ON armstrong.business_operations(gym_id,device_id);
CREATE INDEX business_operation_actor ON armstrong.business_operations(gym_id,actor_user_id);
CREATE TABLE armstrong.business_changes (
 gym_id uuid NOT NULL, sequence bigint NOT NULL CHECK(sequence>0), operation_id uuid NOT NULL,
 PRIMARY KEY(gym_id,sequence), UNIQUE(gym_id,operation_id),
 FOREIGN KEY(gym_id,operation_id) REFERENCES armstrong.business_operations(gym_id,id)
);
CREATE TRIGGER immutable_business_operations BEFORE UPDATE OR DELETE ON armstrong.business_operations
 FOR EACH ROW EXECUTE FUNCTION armstrong.immutable();
CREATE TRIGGER immutable_business_changes BEFORE UPDATE OR DELETE ON armstrong.business_changes
 FOR EACH ROW EXECUTE FUNCTION armstrong.immutable();
CREATE FUNCTION armstrong.preserve_business_record() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='DELETE' OR NEW.gym_id<>OLD.gym_id OR NEW.table_name<>OLD.table_name OR NEW.record_id<>OLD.record_id THEN
  RAISE EXCEPTION 'Business identity and history cannot be deleted or moved';
 END IF;
 IF OLD.table_name NOT IN ('users','members','plans','gym_settings','products','nfc_cards') AND NEW.data<>OLD.data THEN
  RAISE EXCEPTION 'Business history is append-only';
 END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER preserve_business_record BEFORE UPDATE OR DELETE ON armstrong.business_records
 FOR EACH ROW EXECUTE FUNCTION armstrong.preserve_business_record();
ALTER TABLE armstrong.business_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.business_references ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.business_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE armstrong.business_changes ENABLE ROW LEVEL SECURITY;
REVOKE ALL ON armstrong.business_records,armstrong.business_references,armstrong.business_operations,armstrong.business_changes FROM PUBLIC,anon,authenticated;
REVOKE ALL ON FUNCTION armstrong.preserve_business_record() FROM PUBLIC,anon,authenticated;
-- The role may not exist on a fresh isolated test database; never create credentials here.
DO $$ BEGIN
 IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='armstrong_api') THEN
  GRANT SELECT,INSERT ON armstrong.business_records,armstrong.business_references,armstrong.business_operations,armstrong.business_changes TO armstrong_api;
  GRANT UPDATE(data) ON armstrong.business_records TO armstrong_api;
  GRANT UPDATE(target_id) ON armstrong.business_references TO armstrong_api;
  GRANT UPDATE(business_sequence) ON armstrong.gyms TO armstrong_api;
 END IF;
END $$;
