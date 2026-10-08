import { ServerConfigurationError } from './configuration-error.ts';

// Catalog/permission checks only, safe for the restricted runtime connection.
// No access to application rows or the migration owner's private ledger.
export async function verifyBusinessReadiness(client: { query(sql: string, values?: unknown[]): Promise<{ rows: Record<string, unknown>[] }> }) {
  const names = ['business_records', 'business_references', 'business_operations', 'business_changes'];
  const rows = (await client.query(`SELECT c.relname AS name, c.relrowsecurity AS rls,
    has_table_privilege(current_user,c.oid,'SELECT') AS can_read,
    has_table_privilege(current_user,c.oid,'INSERT') AS can_insert,
    has_table_privilege(current_user,c.oid,'DELETE') AS can_delete
    FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
    WHERE n.nspname='armstrong' AND c.relkind='r' AND c.relname=ANY($1::text[])`, [names])).rows;
  if (rows.length !== names.length || rows.some(r => !names.includes(String(r.name)) || r.rls !== true || r.can_read !== true || r.can_insert !== true || r.can_delete !== false)) {
    throw new ServerConfigurationError('Business synchronization schema or restricted runtime permissions are missing');
  }
  const result = (await client.query(`SELECT
    has_column_privilege(current_user,'armstrong.business_records','data','UPDATE') AS record_update,
    has_column_privilege(current_user,'armstrong.business_references','target_id','UPDATE') AS reference_update,
    has_column_privilege(current_user,'armstrong.gyms','business_sequence','UPDATE') AS sequence_update,
    EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='armstrong.business_records'::regclass
      AND conname='business_records_table_name_check' AND convalidated
      AND pg_catalog.pg_get_constraintdef(oid) LIKE '%staff_payout_items%') AS staff_contract,
    EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='armstrong.business_records'::regclass
      AND conname='business_records_table_name_check' AND convalidated
      AND pg_catalog.pg_get_constraintdef(oid) LIKE '%staff_attendance%') AS attendance_contract,
    EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='armstrong.business_records'::regclass
      AND conname='business_records_table_name_check' AND convalidated
      AND pg_catalog.pg_get_constraintdef(oid) LIKE '%staff_deletions%') AS staff_removal_contract,
    (SELECT count(*)=2 FROM pg_catalog.pg_indexes WHERE schemaname='armstrong'
      AND indexname IN ('active_attendance_card_per_gym','active_staff_card_per_gym')) AS attendance_card_guards,
    EXISTS(SELECT 1 FROM pg_catalog.pg_indexes WHERE schemaname='armstrong'
      AND indexname='trainer_nic_per_gym') AS staff_nic_guard,
    (SELECT count(*)=2 FROM pg_catalog.pg_constraint WHERE conrelid='armstrong.business_records'::regclass
      AND conname IN ('business_record_key_matches','business_identity_reference_only') AND convalidated) AS history_guards,
    (SELECT count(*)=3 FROM pg_catalog.pg_trigger WHERE NOT tgisinternal AND tgenabled<>'D'
      AND (tgrelid,tgname) IN (('armstrong.business_records'::regclass,'preserve_business_record'),
        ('armstrong.business_operations'::regclass,'immutable_business_operations'),
        ('armstrong.business_changes'::regclass,'immutable_business_changes'))) AS immutable_triggers`)).rows[0];
  if (!result || Object.values(result).some(value => value !== true)) {
    throw new ServerConfigurationError('Business synchronization history guards or runtime column permissions are missing');
  }
}
