use super::*;
use rusqlite::{DatabaseName, OpenFlags};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupEnvelope {
    pub format: String,
    pub format_version: i64,
    pub schema_version: i64,
    pub created_at: String,
    pub sha256: String,
    pub data: Vec<u8>,
}
struct TemporaryFile(PathBuf);
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(format!("{}-wal", self.0.display()));
        let _ = fs::remove_file(format!("{}-shm", self.0.display()));
        // Isolated native recovery creates only its own disposable backup dir.
        let _ = fs::remove_dir(self.0.with_extension("backups"));
    }
}
pub(super) struct RestoreCandidate {
    token: String,
    file: TemporaryFile,
    state: String,
    checksum: String,
}
// Opaque native recovery result: never supplied by the webview or a backup file.
pub(super) struct NativeRestore {
    file: TemporaryFile,
    original_state: String,
    complete: bool,
    verified_until: Option<DateTime<Utc>>,
}
pub(super) struct InitialProfileRecovery {
    file: Option<TemporaryFile>,
    original_state: String,
    original_request: Value,
    session: super::removal::Session,
    input: super::InitialProfileRecoveryInput,
    bound: Value,
    result: Value,
    complete: bool,
    replayed: bool,
}

struct RecoveryTransport<'a, T> {
    inner: &'a mut T,
    cancelled: &'a dyn Fn() -> Result<()>,
    deadline: std::time::Instant,
}
impl<T> RecoveryTransport<'_, T> {
    fn check(&self) -> std::result::Result<(), super::member_worker::RemoteFailure> {
        if std::time::Instant::now() >= self.deadline || (self.cancelled)().is_err() {
            return Err(super::member_worker::RemoteFailure::Authorization);
        }
        Ok(())
    }
}
impl<T: super::member_worker::BusinessTransport> super::member_worker::MemberTransport
    for RecoveryTransport<'_, T>
{
    fn scope(&self) -> &super::member_worker::SyncScope {
        self.inner.scope()
    }
    fn subject(&self) -> &str {
        self.inner.subject()
    }
    fn push(
        &mut self,
        request: &Value,
    ) -> std::result::Result<super::member_sync::Receipt, super::member_worker::RemoteFailure> {
        self.check()?;
        let result = self.inner.push(request);
        self.check()?;
        result
    }
    fn pull(
        &mut self,
        cursor: i64,
    ) -> std::result::Result<super::member_sync::Page, super::member_worker::RemoteFailure> {
        self.check()?;
        let result = self.inner.pull(cursor);
        self.check()?;
        result
    }
}
impl<T: super::member_worker::BusinessTransport> super::member_worker::BusinessTransport
    for RecoveryTransport<'_, T>
{
    fn push_business(
        &mut self,
        request: &Value,
    ) -> std::result::Result<Value, super::member_worker::RemoteFailure> {
        self.check()?;
        let result = self.inner.push_business(request);
        self.check()?;
        result
    }
    fn pull_business(
        &mut self,
        cursor: i64,
    ) -> std::result::Result<Value, super::member_worker::RemoteFailure> {
        self.check()?;
        let result = self.inner.pull_business(cursor);
        self.check()?;
        result
    }
}

impl InitialProfileRecovery {
    pub(super) fn run(
        &mut self,
        transport: &mut impl super::member_worker::BusinessTransport,
        cancelled: impl Fn() -> Result<()>,
    ) -> Result<()> {
        use super::member_worker::{authorized, BusinessTransport, Limits, RemoteFailure, Run};
        cancelled()?;
        if self.replayed {
            return Ok(());
        }
        let mut isolated = Store::open(&self.file.as_ref().ok_or("Missing recovery snapshot")?.0)?;
        isolated.removal_session = Some(self.session.clone());
        let actor = super::removal::authorized(&isolated.conn, isolated.removal_session.as_ref())?;
        isolated
            .conn
            .execute(
                "INSERT INTO temp.native_actor VALUES(?1,?2)",
                params![actor.user_id, actor.label],
            )
            .map_err(db_error)?;
        authorized(&isolated, transport)?;
        let mut fenced = RecoveryTransport {
            inner: transport,
            cancelled: &cancelled,
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(120),
        };
        // Recheck the exact original request first. A receipt wins over any old
        // refusal; only a fresh revision refusal can supersede the seed.
        match fenced.push_business(&self.original_request) {
            Ok(receipt) => {
                business_sync::validate_receipt(&receipt, &self.original_request, fenced.inner.scope(), Some(fenced.inner.subject()))?;
                isolated.conn.execute("UPDATE business_batches SET state='confirmed',response_json=?2,last_error=NULL WHERE id=?1 AND state='conflict'", params![self.input.batch_id, receipt.to_string()]).map_err(db_error)?;
            }
            Err(RemoteFailure::BusinessConflict { code }) if code == "business_revision_conflict" => {
                let replacement = super::initial_profile_recovery::replacement(&self.original_request, fenced.inner.subject())?;
                self.result["superseded"] = json!(true);
                self.result["replacementBatchId"] = replacement["operationId"].clone();
                let tx = isolated.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
                operations::receipt(&tx, super::initial_profile_recovery::COMMAND, &self.input.request_id, &self.bound, &self.result)?;
                tx.execute("INSERT INTO business_batches(id,ordinal,request_json,state) SELECT ?1,COALESCE(MAX(ordinal),0)+1,?2,'pending' FROM business_batches", params![replacement["operationId"].as_str(), replacement.to_string()]).map_err(db_error)?;
                tx.commit().map_err(db_error)?;
            }
            Err(_) => return Err("Profile recovery could not verify the original server refusal. The original database is unchanged; retry online.".into()),
        }
        isolated.conn.execute("DELETE FROM metadata WHERE key IN ('business_last_error','business_retry_on','business_failures')", []).map_err(db_error)?;
        for _ in 0..25 {
            cancelled()?;
            match isolated.run_business_sync(&mut fenced, Utc::now(), Limits { pushes: 10, pages: 5 })? {
                Run::Complete { .. } => {
                    cancelled()?;
                    if self.result["superseded"] != true {
                        let tx = isolated.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
                        operations::receipt(&tx, super::initial_profile_recovery::COMMAND, &self.input.request_id, &self.bound, &self.result)?;
                        tx.commit().map_err(db_error)?;
                    }
                    isolated.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").map_err(db_error)?;
                    integrity(&isolated.conn)?;
                    self.complete = true;
                    return Ok(());
                }
                Run::Yielded { .. } => continue,
                _ => return Err("Profile recovery did not complete. The original records and transaction remain saved; retry online or review the other conflict.".into()),
            }
        }
        Err("Profile recovery reached its bounded limit. The original database remains unchanged; retry online.".into())
    }

    pub(super) fn commit(self, store: &mut Store) -> Result<Value> {
        if !self.complete {
            return Err("Profile recovery has no complete verified server history".into());
        }
        if self.replayed {
            let mut result = self.result;
            result["duplicate"] = json!(true);
            return Ok(result);
        }
        let check = || -> Result<()> {
            if self.session.expires_at <= Utc::now() {
                return Err("Recovery account session expired; no records were replaced".into());
            }
            super::removal::current_session(&store.conn, &self.session)
        };
        check()?;
        let file = self.file.as_ref().ok_or("Missing recovery snapshot")?;
        verify_file(&file.0, SCHEMA_VERSION)?;
        let isolated = Store::open(&file.0)?;
        let preview = store.preview_restore(isolated.backup_envelope()?)?;
        drop(isolated);
        let original_state = self.original_state;
        let expiry = self.session.expires_at;
        let saved_result = self.result.clone();
        store.restore_checked_with(
            preview["token"].as_str().ok_or("Missing recovery replacement token")?.into(),
            |conn| {
                if expiry <= Utc::now() || storage_fingerprint(conn)? != original_state {
                    return Err("This computer changed during profile recovery; no records were replaced. Review and retry.".into());
                }
                Ok(())
            },
            |conn| {
                conn.execute("DELETE FROM metadata WHERE key IN ('restore_requires_reconciliation','business_initial_download','business_initial_profile_received','business_last_error','business_retry_on','business_failures')", []).map_err(db_error)?;
                conn.execute("INSERT INTO metadata VALUES('business_initial_download_complete','1') ON CONFLICT(key) DO UPDATE SET value='1'", []).map_err(db_error)?;
                record(conn, "recover initial gym profile", &self.input.batch_id, None, Some(self.original_request.to_string()), saved_result)
            },
        )?;
        Ok(self.result)
    }
    pub(super) fn replayed(&self) -> bool {
        self.replayed
    }
}

impl Store {
    pub(super) fn prepare_initial_profile_recovery(
        &mut self,
        input: super::InitialProfileRecoveryInput,
    ) -> Result<InitialProfileRecovery> {
        let bound = super::initial_profile_recovery::bound(
            &self.conn,
            self.removal_session.as_ref(),
            &input,
        )?;
        let session = self
            .removal_session
            .as_ref()
            .ok_or("Verified Administrator sign-in is required")?
            .clone();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let replay = operations::replay(
            &tx,
            super::initial_profile_recovery::COMMAND,
            &input.request_id,
            &bound,
        )?;
        drop(tx);
        if let Some(result) = replay {
            return Ok(InitialProfileRecovery {
                file: None,
                original_state: String::new(),
                original_request: Value::Null,
                session,
                input,
                bound,
                result,
                complete: true,
                replayed: true,
            });
        }
        let envelope = self.backup_envelope()?;
        let file = TemporaryFile(
            self.files_dir()?
                .join(format!("initial-profile-{}.sqlite3", id())),
        );
        write_new(&file.0, &envelope.data)?;
        let isolated = Store::open(&file.0)?;
        let preview =
            super::business_review::preview_on(&isolated.conn, Some(&session), &input.batch_id)?;
        if preview["fingerprint"] != input.fingerprint {
            return Err("The blocked transaction changed. Review it again before recovery.".into());
        }
        if preview["initialProfileRecovery"]["allowed"] != true {
            return Err(preview["initialProfileRecovery"]["reason"]
                .as_str()
                .unwrap_or("This transaction needs separate reconciliation")
                .into());
        }
        let encoded: String = isolated
            .conn
            .query_row(
                "SELECT request_json FROM business_batches WHERE id=?1",
                [&input.batch_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let original_request: Value =
            serde_json::from_str(&encoded).map_err(|_| "Invalid retained transaction")?;
        let original_state = storage_fingerprint(&isolated.conn)?;
        let backup = self.files_dir()?.join(format!(
            "armstrong-profile-recovery-{}.armstrong-backup.json",
            id()
        ));
        write_new(
            &backup,
            &serde_json::to_vec(&envelope).map_err(|e| e.to_string())?,
        )?;
        let result = json!({"batchId":input.batch_id,"recoveredInitialProfile":true,"superseded":false,"originalRequestSha256":business_sync::hash(&original_request)?,"recoveryPath":backup,"requiresLogin":true});
        Ok(InitialProfileRecovery {
            file: Some(file),
            original_state,
            original_request,
            session,
            input,
            bound,
            result,
            complete: false,
            replayed: false,
        })
    }
}
fn storage_fingerprint(conn: &Connection) -> Result<String> {
    let mut digest = Sha256::new();
    for (table, sql) in schema(conn)? {
        digest.update(sql.as_bytes());
        if !sql.starts_with("CREATE TABLE") {
            continue;
        }
        let mut columns = conn
            .prepare(&format!("PRAGMA table_info({})", identifier(&table)))
            .map_err(db_error)?;
        let columns = columns
            .query_map([], |r| r.get::<_, String>(1))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        let names = columns
            .iter()
            .map(|c| identifier(c))
            .collect::<Vec<_>>()
            .join(",");
        let mut rows = conn
            .prepare(&format!(
                "SELECT json_array({names}) FROM {} ORDER BY {names}",
                identifier(&table)
            ))
            .map_err(db_error)?;
        for row in rows
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?
        {
            let row = row.map_err(db_error)?;
            digest.update(row.len().to_be_bytes());
            digest.update(row.as_bytes());
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}
impl NativeRestore {
    pub(super) fn run(
        &mut self,
        verified: &super::native_auth::VerifiedEnrollment,
        transport: &mut impl super::member_worker::BusinessTransport,
        cancelled: impl Fn() -> Result<()>,
    ) -> Result<()> {
        if !verified.is_administrator() {
            return Err("Verified Administrator recovery is required".into());
        }
        let mut isolated = Store::open(&self.file.0)?;
        // Only this disposable copy can bypass the restore guard. Binding still
        // requires the same existing API/gym/device and fresh verified authority.
        isolated
            .conn
            .execute(
                "DELETE FROM metadata WHERE key='restore_requires_reconciliation'",
                [],
            )
            .map_err(db_error)?;
        isolated.enroll_native(verified)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        for _ in 0..25 {
            cancelled()?;
            if std::time::Instant::now() >= deadline {
                break;
            }
            match isolated.run_business_sync(transport,Utc::now(),super::member_worker::Limits {pushes:10,pages:10})? {
                super::member_worker::Run::Complete {..} => {
                    cancelled()?;
                    isolated.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").map_err(db_error)?;
                    integrity(&isolated.conn)?;
                    self.complete=true;
                    self.verified_until=Some(verified.valid_until());
                    return Ok(());
                }
                super::member_worker::Run::Yielded {..} => continue,
                _ => return Err("Restore reconciliation did not complete. The restored database and recovery copies remain guarded; retry online or review the conflict.".into()),
            }
        }
        Err("Restore reconciliation reached its bounded limit; the restored database is retained. Retry online.".into())
    }
    pub(super) fn commit(self, store: &mut Store) -> Result<Value> {
        if !self.complete {
            return Err("Restore has no complete verified server reconciliation".into());
        }
        let check_expiry = || -> Result<()> {
            if self
                .verified_until
                .is_none_or(|expiry| expiry <= Utc::now())
            {
                return Err(
                    "Verified recovery session expired; the restored database remains guarded"
                        .into(),
                );
            }
            Ok(())
        };
        check_expiry()?;
        verify_file(&self.file.0, SCHEMA_VERSION)?;
        let isolated = Store::open(&self.file.0)?;
        let preview = store.preview_restore(isolated.backup_envelope()?)?;
        drop(isolated);
        store.restore_checked_with(preview["token"].as_str().ok_or("Missing native recovery token")?.into(),
            |conn| {
                check_expiry()?;
                if storage_fingerprint(conn)? != self.original_state { return Err("Restored database changed during online recovery; no records were replaced".into()); }
                Ok(())
            },
            |conn| {
                conn.execute("DELETE FROM metadata WHERE key='restore_requires_reconciliation'",[]).map_err(db_error)?;
                check_expiry()?;
                record(conn,"restore_reconciled",&id(),None,None,json!({"verifiedServerHistory":true}))
            })
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn io_error(error: std::io::Error) -> String {
    format!("Backup/export operation failed: {error}")
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    // Persist the directory entry before a recovery backup authorizes replacement.
    #[cfg(unix)]
    fs::File::open(path.parent().ok_or("Backup path has no parent")?)
        .and_then(|directory| directory.sync_all())
        .map_err(io_error)?;
    Ok(())
}
fn schema(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt=conn.prepare("SELECT name,sql FROM sqlite_master WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' ORDER BY name").map_err(db_error)?;
    let values = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error);
    values
}
fn verify_file(path: &Path, version: i64) -> Result<()> {
    if !(1..=SCHEMA_VERSION).contains(&version) {
        return Err("Unsupported backup schema version".into());
    }
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(db_error)?;
    if conn
        .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
        .map_err(db_error)?
        != version
    {
        return Err("Backup manifest/schema version mismatch".into());
    }
    integrity(&conn)?;
    let expected = Connection::open_in_memory().map_err(db_error)?;
    expected
        .execute_batch(include_str!("../migrations/001_foundation.sql"))
        .map_err(db_error)?;
    if version >= 2 {
        expected
            .execute_batch(include_str!("../migrations/002_local_operations.sql"))
            .map_err(db_error)?;
    }
    if version >= 3 {
        expected
            .execute_batch(include_str!("../migrations/003_finance.sql"))
            .map_err(db_error)?;
    }
    if version >= 4 {
        expected
            .execute_batch(include_str!("../migrations/004_removal.sql"))
            .map_err(db_error)?;
    }
    if version >= 5 {
        expected
            .execute_batch(include_str!("../migrations/005_member_sync.sql"))
            .map_err(db_error)?;
    }
    if version >= 6 {
        expected
            .execute_batch(include_str!("../migrations/006_member_conflicts.sql"))
            .map_err(db_error)?;
    }
    if version >= 7 {
        expected
            .execute_batch(include_str!("../migrations/007_business_sync.sql"))
            .map_err(db_error)?;
    }
    if version >= 8 {
        expected
            .execute_batch(include_str!("../migrations/008_staff_training.sql"))
            .map_err(db_error)?;
    }
    if version >= 9 {
        expected
            .execute_batch(include_str!("../migrations/009_attendance_profiles.sql"))
            .map_err(db_error)?;
    }
    if version >= 10 {
        expected
            .execute_batch(include_str!("../migrations/010_staff_removal.sql"))
            .map_err(db_error)?;
    }
    if version >= 11 {
        expected
            .execute_batch(include_str!("../migrations/011_admission.sql"))
            .map_err(db_error)?;
    }
    if schema(&conn)? != schema(&expected)? {
        return Err("Backup contains an unrecognized schema, index or trigger".into());
    }
    // Validate legacy calendar fields too; SQL integrity alone cannot establish valid business dates.
    let mut stmt=conn.prepare("SELECT joined_on FROM members UNION ALL SELECT starts_on FROM membership_periods UNION ALL SELECT ends_on FROM membership_periods").map_err(db_error)?;
    for value in stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(db_error)?
    {
        date(&value.map_err(db_error)?)?;
    }
    if version >= 8 {
        let mut dates=conn.prepare("SELECT starts_on FROM training_charges UNION ALL SELECT ends_on FROM training_charges UNION ALL SELECT salary_month||'-01' FROM staff_payouts").map_err(db_error)?;
        for value in dates
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?
        {
            date(&value.map_err(db_error)?)?;
        }
    }
    Ok(())
}
fn summary(snapshot: &Value) -> Value {
    json!({"members":snapshot["members"].as_array().map_or(0,Vec::len),"periods":snapshot["periods"].as_array().map_or(0,Vec::len),"payments":snapshot["payments"].as_array().map_or(0,Vec::len),"sales":snapshot["sales"].as_array().map_or(0,Vec::len),"expenses":snapshot["expenses"].as_array().map_or(0,Vec::len),"pending":snapshot["pending"],"auditCount":snapshot["auditCount"]})
}
impl Store {
    pub(super) fn requires_restore_reconciliation(&self) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='restore_requires_reconciliation')",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)
    }
    pub(super) fn prepare_restore_reconciliation(&self) -> Result<NativeRestore> {
        if !self.requires_restore_reconciliation()?
            || super::member_worker::scope(&self.conn)?.is_none()
        {
            return Err("A restored backup with verified server scope is required".into());
        }
        let original_state = storage_fingerprint(&self.conn)?;
        let snapshot = self.backup_envelope()?;
        let file = TemporaryFile(
            self.files_dir()?
                .join(format!("online-recovery-{}.sqlite3", id())),
        );
        write_new(&file.0, &snapshot.data)?;
        let copy = Store::open(&file.0)?;
        if storage_fingerprint(&copy.conn)? != original_state
            || storage_fingerprint(&self.conn)? != original_state
        {
            return Err("Restored database changed before recovery could start".into());
        }
        drop(copy);
        Ok(NativeRestore {
            file,
            original_state,
            complete: false,
            verified_until: None,
        })
    }
    fn files_dir(&self) -> Result<PathBuf> {
        let path = self.path.with_extension("backups");
        fs::create_dir_all(&path).map_err(io_error)?;
        Ok(path)
    }
    pub fn backup_envelope(&self) -> Result<BackupEnvelope> {
        let file = TemporaryFile(self.files_dir()?.join(format!("snapshot-{}.sqlite3", id())));
        write_new(&file.0, &[])?;
        self.conn
            .backup(DatabaseName::Main, &file.0, None)
            .map_err(db_error)?;
        verify_file(&file.0, SCHEMA_VERSION)?;
        let data = fs::read(&file.0).map_err(io_error)?;
        Ok(BackupEnvelope {
            format: "armstrong-sqlite-backup".into(),
            format_version: 1,
            schema_version: SCHEMA_VERSION,
            created_at: Utc::now().to_rfc3339(),
            sha256: hash(&data),
            data,
        })
    }
    pub fn export_backup(&self) -> Result<Value> {
        let envelope = self.backup_envelope()?;
        let path = self
            .files_dir()?
            .join(format!("armstrong-{}.armstrong-backup.json", id()));
        write_new(
            &path,
            &serde_json::to_vec(&envelope).map_err(|e| e.to_string())?,
        )?;
        Ok(json!({"path":path,"sha256":envelope.sha256}))
    }
    pub fn preview_restore(&mut self, envelope: BackupEnvelope) -> Result<Value> {
        self.restore_candidate = None;
        if envelope.format != "armstrong-sqlite-backup"
            || envelope.format_version != 1
            || envelope.data.is_empty()
            || envelope.data.len() > 64 * 1024 * 1024
        {
            return Err("Select a supported native SQLite backup (maximum 64 MiB database), not browser demo JSON".into());
        }
        if hash(&envelope.data) != envelope.sha256 {
            return Err("Backup checksum mismatch; no data was changed".into());
        }
        let file = TemporaryFile(
            self.files_dir()?
                .join(format!("restore-preview-{}.sqlite3", id())),
        );
        write_new(&file.0, &envelope.data)?;
        verify_file(&file.0, envelope.schema_version)?;
        let candidate = Store::open(&file.0)?; // Migrate only the isolated copy, never the live store at preview.
        let source_device: String = candidate
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let device: String = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let mut current = self.snapshot()?;
        // Session capability is runtime state, not a storage-change fingerprint.
        current
            .as_object_mut()
            .unwrap()
            .remove("removalAuthorization");
        current["memberSync"]
            .as_object_mut()
            .unwrap()
            .remove("reviewAuthorization");
        let backup = candidate.snapshot()?;
        if source_device != device && current["auditCount"].as_i64().unwrap_or(1) != 0 {
            return Err(
                "Backup is from another device; device-transfer reconciliation is not implemented"
                    .into(),
            );
        }
        // A single-file candidate must include migrated WAL contents before it can be used by restore.
        candidate
            .conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .map_err(db_error)?;
        drop(candidate);
        let token = id();
        let result = json!({"token":token,"current":summary(&current),"backup":summary(&backup),"schemaVersion":SCHEMA_VERSION,"sha256":envelope.sha256});
        self.restore_candidate = Some(RestoreCandidate {
            token,
            file,
            state: hash(current.to_string().as_bytes()),
            checksum: envelope.sha256,
        });
        Ok(result)
    }
    pub fn restore_backup(&mut self, token: String) -> Result<Value> {
        self.restore_checked(token, |_| Ok(()))
    }
    pub(super) fn restore_checked(
        &mut self,
        token: String,
        after_copy: impl FnOnce(&Connection) -> Result<()>,
    ) -> Result<Value> {
        self.restore_checked_with(token, |_| Ok(()), |tx| after_copy(tx))
    }
    fn restore_checked_with(
        &mut self,
        token: String,
        before_copy: impl FnOnce(&Connection) -> Result<()>,
        after_copy: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<()>,
    ) -> Result<Value> {
        let candidate = self
            .restore_candidate
            .take()
            .ok_or("Select and preview a backup before restoring")?;
        if token != candidate.token {
            return Err("Storage changed or confirmation expired. Preview the backup again; no data was replaced.".into());
        }
        verify_file(&candidate.file.0, SCHEMA_VERSION)?;
        // Prepare reconciliation marker/audit/outbox inside the candidate. They arrive atomically with replacement.
        let mut staged = Store::open(&candidate.file.0)?;
        let actor: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT user_id,label FROM temp.native_actor LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        if let Some((user, label)) = actor {
            let same: bool = staged
                .conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM users WHERE id=?1 AND subject=?2)",
                    params![
                        user,
                        self.conn
                            .query_row("SELECT subject FROM users WHERE id=?1", [&user], |r| r
                                .get::<_, String>(
                                0
                            ))
                            .map_err(db_error)?
                    ],
                    |r| r.get(0),
                )
                .map_err(db_error)?;
            if !same {
                return Err("Backup does not contain the verified restoration actor; administrator reconciliation is required before replacement".into());
            }
            staged
                .conn
                .execute(
                    "INSERT INTO temp.native_actor VALUES(?1,?2)",
                    params![user, label],
                )
                .map_err(db_error)?;
        }
        let tx = staged
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO metadata VALUES('restore_requires_reconciliation','1') ON CONFLICT(key) DO UPDATE SET value='1'",[]).map_err(db_error)?;
        let auth_required:bool=self.conn.query_row("SELECT EXISTS(SELECT 1 FROM metadata WHERE key IN ('native_auth_required','member_sync_scope'))",[],|r|r.get(0)).map_err(db_error)?;
        if auth_required {
            tx.execute("INSERT INTO metadata VALUES('native_auth_required','1') ON CONFLICT(key) DO NOTHING",[]).map_err(db_error)?;
        }
        record(
            &tx,
            "restore",
            &id(),
            None,
            None,
            json!({"backupSha256":candidate.checksum,"requiresReconciliation":true}),
        )?;
        tx.commit().map_err(db_error)?;
        staged
            .conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .map_err(db_error)?;
        drop(staged);
        verify_file(&candidate.file.0, SCHEMA_VERSION)?;
        // A read-only connection can capture the committed WAL snapshot while the live
        // IMMEDIATE transaction excludes writers, including other app processes.
        let recovery_store = Store {
            conn: Connection::open_with_flags(&self.path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(db_error)?,
            path: self.path.clone(),
            restore_candidate: None,
            removal_session: None,
        };
        self.conn
            .execute(
                "ATTACH DATABASE ?1 AS armstrong_restore",
                [candidate.file.0.to_string_lossy().as_ref()],
            )
            .map_err(db_error)?;
        let result = (|| {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db_error)?;
            if hash(snapshot_on(&tx)?.to_string().as_bytes()) != candidate.state {
                return Err("Storage changed or confirmation expired. Preview the backup again; no data was replaced.".into());
            }
            before_copy(&tx)?;
            let recovery = recovery_store.export_backup()?;
            let triggers = {
                let mut stmt = tx
                    .prepare(
                        "SELECT name,sql FROM sqlite_master WHERE type='trigger' ORDER BY name",
                    )
                    .map_err(db_error)?;
                let values = stmt
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(db_error)?
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(db_error)?;
                values
            };
            let mut tables = {
                let mut stmt = tx.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").map_err(db_error)?;
                let values = stmt
                    .query_map([], |row| row.get::<_, String>(0))
                    .map_err(db_error)?
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(db_error)?;
                values
            };
            // Insert parents first and delete children first. This also avoids deferred
            // FK counters spanning the schema changes that reinstate history triggers.
            let order = [
                "metadata",
                "plans",
                "gym_settings",
                "admission_settings",
                "roles",
                "users",
                "user_roles",
                "members",
                "nfc_cards",
                "membership_periods",
                "attendance",
                "payments",
                "products",
                "sales",
                "sale_items",
                "stock_movements",
                "expenses",
                "expense_voids",
                "invoices",
                "payment_allocations",
                "invoice_details",
                "payment_reversal_details",
                "allocation_reversals",
                "payment_receipts",
                "audit",
                "outbox",
                "member_deliveries",
                "member_remote_heads",
                "member_sync_cursor",
                "member_sync_conflicts",
                "member_conflict_resolutions",
                "member_resolved_conflicts",
                "member_resolved_operations",
                "local_operations",
                "business_dirty",
                "business_batches",
                "business_batch_operations",
                "business_cursor",
                "trainers",
                "member_trainers",
                "training_charges",
                "staff_payouts",
                "staff_payout_items",
                "member_profiles",
                "member_deletions",
                "staff_nfc_cards",
                "staff_attendance",
                "staff_deletions",
            ];
            if tables.len() != order.len()
                || tables.iter().any(|table| !order.contains(&table.as_str()))
            {
                return Err("Unrecognized restore table set; no records were replaced".into());
            }
            tables.sort_by_key(|table| order.iter().position(|name| *name == table).unwrap());
            // Explicit reviewed restore is the only operation allowed to replace history.
            // Both databases have canonical matching schemas, verified before this point.
            tx.execute_batch("PRAGMA defer_foreign_keys=ON")
                .map_err(db_error)?;
            for (name, _) in &triggers {
                tx.execute_batch(&format!("DROP TRIGGER {}", identifier(name)))
                    .map_err(db_error)?;
            }
            for table in tables.iter().rev() {
                tx.execute_batch(&format!("DELETE FROM {}", identifier(table)))
                    .map_err(db_error)?;
            }
            for table in &tables {
                let quoted = identifier(table);
                tx.execute_batch(&format!(
                    "INSERT INTO {quoted} SELECT * FROM armstrong_restore.{quoted}"
                ))
                .map_err(db_error)?;
            }
            for (_, sql) in &triggers {
                tx.execute_batch(sql).map_err(db_error)?;
            }
            integrity(&tx)?;
            after_copy(&tx)?; // Internal verification hook also exercises rollback after all tables copy.
            tx.commit()
                .map_err(|error| format!("Restore transaction could not commit: {error}"))?;
            Ok(json!({"recoveryPath":recovery["path"],"requiresReconciliation":true}))
        })();
        let detached = self
            .conn
            .execute_batch("DETACH DATABASE armstrong_restore")
            .map_err(db_error);
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                self.removal_session = None;
                detached.map_err(|error| {
                    format!(
                        "Restore committed, but cleanup failed: {error}. Recovery: {}",
                        value["recoveryPath"]
                    )
                })?;
                Ok(value)
            }
        }
    }
    pub fn export_report(&self, kind: String) -> Result<Value> {
        self.export_report_range(kind, ReportRange::default())
    }
    pub fn export_report_range(&self, kind: String, range: ReportRange) -> Result<Value> {
        let mut s = self.snapshot()?;
        super::reports::filter_snapshot(&mut s, &range)?;
        let mut records: Vec<Vec<String>> = vec![];
        let header: Vec<&str> = match kind.as_str() {
            "Attendance report" => {
                for row in s["attendance"].as_array().unwrap() {
                    records.push(vec![
                        text(row, "id"),
                        text(row, "memberId"),
                        text(row, "name"),
                        text(row, "businessOn"),
                        text(row, "occurredAt"),
                        text(row, "type"),
                        text(row, "source"),
                    ]);
                }
                vec![
                    "ID",
                    "Member ID",
                    "Member",
                    "Business date",
                    "UTC time",
                    "Type",
                    "Source",
                ]
            }
            "Membership report" => {
                for row in s["periods"].as_array().unwrap() {
                    records.push(vec![
                        text(row, "id"),
                        text(row, "memberId"),
                        text(row, "planName"),
                        text(row, "startsOn"),
                        text(row, "endsOn"),
                        minor(row, "priceMinor"),
                        text(row, "status"),
                    ]);
                }
                vec![
                    "ID",
                    "Member ID",
                    "Plan snapshot",
                    "Start date",
                    "Last valid day",
                    "Price LKR",
                    "Status",
                ]
            }
            "Income report" => {
                for (table, amount) in [("payments", "netAmountMinor"), ("sales", "totalMinor")] {
                    for row in s[table].as_array().unwrap() {
                        records.push(vec![
                            table.into(),
                            text(row, "id"),
                            text(row, "businessOn"),
                            text(row, "method"),
                            minor(row, amount),
                        ]);
                    }
                }
                vec!["Source", "ID", "Business date", "Method", "Received LKR"]
            }
            "Inventory report" => {
                for row in s["products"].as_array().unwrap() {
                    records.push(vec![
                        text(row, "id"),
                        text(row, "name"),
                        text(row, "sku"),
                        row["stock"].to_string(),
                        minor(row, "costMinor"),
                        minor(row, "priceMinor"),
                        row["reorderLevel"].to_string(),
                    ]);
                }
                vec![
                    "ID",
                    "Product",
                    "SKU",
                    "Stock",
                    "Cost LKR",
                    "Price LKR",
                    "Reorder level",
                ]
            }
            "Expense report" => {
                for row in s["expenses"].as_array().unwrap() {
                    records.push(vec![
                        text(row, "id"),
                        text(row, "title"),
                        text(row, "category"),
                        text(row, "businessOn"),
                        text(row, "method"),
                        minor(row, "amountMinor"),
                        minor(row, "effectiveAmountMinor"),
                        text(row, "status"),
                        text(row, "voidReason"),
                        text(row, "voidedAt"),
                        text(row, "voidedBy"),
                        text(row, "actor"),
                    ]);
                }
                vec![
                    "ID",
                    "Description",
                    "Category",
                    "Business date",
                    "Method",
                    "Amount LKR",
                    "Effective amount LKR",
                    "Status",
                    "Void reason",
                    "Voided at",
                    "Voided by",
                    "Recorded by",
                ]
            }
            "Audit report" => {
                for row in s["audit"].as_array().unwrap() {
                    records.push(vec![
                        text(row, "id"),
                        text(row, "action"),
                        text(row, "entity"),
                        text(row, "entityId"),
                        text(row, "user"),
                        text(row, "timestamp"),
                        text(row, "beforeJson"),
                        text(row, "afterJson"),
                    ]);
                }
                vec![
                    "ID",
                    "Action",
                    "Entity",
                    "Entity ID",
                    "Actor",
                    "UTC time",
                    "Before JSON",
                    "After JSON",
                ]
            }
            _ => return Err("Unknown report".into()),
        };
        let mut output = header.iter().map(|v| csv(v)).collect::<Vec<_>>().join(",") + "\r\n";
        for row in &records {
            output += &(row.iter().map(|v| csv(v)).collect::<Vec<_>>().join(",") + "\r\n");
        }
        let path = self.files_dir()?.join(format!("report-{}.csv", id()));
        write_new(&path, output.as_bytes())?;
        Ok(json!({"path":path,"rows":records.len(),"fromOn":range.from_on,"toOn":range.to_on}))
    }
}
fn text(row: &Value, key: &str) -> String {
    row[key].as_str().unwrap_or("").to_string()
}
fn minor(row: &Value, key: &str) -> String {
    let value = row[key].as_i64().unwrap_or(0);
    format!(
        "{}{}.{:02}",
        if value < 0 { "-" } else { "" },
        value.unsigned_abs() / 100,
        value.unsigned_abs() % 100
    )
}
fn csv(value: &str) -> String {
    let safe = if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_string()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}

fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
