use super::member_worker::{BusinessTransport, RemoteFailure};
use super::*;
use rusqlite::types::Value as SqlValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(crate) const REQUEST_LIMIT: usize = 1024 * 1024;
// A reviewed initial-profile recovery retains its refused envelope. Its separate
// immutable local operation records that it was superseded, never confirmed.
pub(super) const ACTIVE_BATCH: &str = "NOT EXISTS(SELECT 1 FROM local_operations recovery WHERE recovery.command='business_initial_profile_recovery' AND json_extract(recovery.result_json,'$.superseded')=1 AND json_extract(recovery.result_json,'$.batchId')=business_batches.id)";
#[derive(Clone, Deserialize)]
pub(crate) struct Column {
    pub(crate) name: String,
    pub(crate) nullable: bool,
    #[serde(rename = "type")]
    pub(crate) kind: String,
}
#[derive(Clone, Deserialize)]
pub(crate) struct Table {
    pub(crate) name: String,
    pub(crate) key: String,
    pub(crate) columns: Vec<Column>,
    pub(crate) mutable: Vec<String>,
}
#[derive(Deserialize)]
struct Contract {
    tables: Vec<Table>,
}
pub(crate) fn tables() -> Result<Vec<Table>> {
    serde_json::from_str::<Contract>(include_str!("../business-schema.json"))
        .map(|c| c.tables)
        .map_err(|_| "Invalid bundled business row contract".into())
}
fn normalized(table: &str, mut row: Value) -> Value {
    if table == "users" {
        row["active"] = json!(0);
        row["version"] = json!(1);
    }
    row
}
fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value[name]
        .as_str()
        .ok_or_else(|| format!("Invalid business {name}"))
}
pub(crate) fn hash(value: &Value) -> Result<String> {
    // serde_json uses sorted map keys, matching the server canonical encoder.
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(|_| "Invalid business request")?)
    ))
}
pub(crate) fn capture(conn: &Connection) -> Result<()> {
    let downloading: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='business_initial_download')",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if downloading {
        return Err("This computer is downloading the existing gym records. Finish server synchronization before saving changes.".into());
    }
    let dirty = operations::rows(conn,"SELECT json_object('table',table_name,'id',record_id,'before',json(before_json),'after',json(after_json)) FROM business_dirty ORDER BY table_name,record_id")?;
    if dirty.is_empty() {
        return Ok(());
    }
    let changes: Vec<Value> = dirty
        .into_iter()
        .map(|mut c| {
            let table = c["table"].as_str().unwrap_or("").to_owned();
            if !c["before"].is_null() {
                c["before"] = normalized(&table, c["before"].clone());
            }
            c["after"] = normalized(&table, c["after"].clone());
            c
        })
        .filter(|c| c["before"] != c["after"])
        .collect();
    if changes.is_empty() {
        conn.execute("DELETE FROM business_dirty", [])
            .map_err(db_error)?;
        return Ok(());
    }
    let device: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get::<_, String>(0),
        )
        .map_err(db_error)?;
    let actor: Option<String> = conn
        .query_row(
            "SELECT u.subject FROM users u JOIN temp.native_actor a ON a.user_id=u.id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    let ops = operations::rows(conn,"SELECT json_quote(id) FROM outbox WHERE NOT EXISTS(SELECT 1 FROM business_batch_operations b WHERE b.operation_id=outbox.id) ORDER BY rowid")?;
    let batch = json!({"protocolVersion":2,"operationId":id(),"deviceId":device,"actorSubject":actor,"operationIds":ops,"changes":changes});
    let encoded = batch.to_string();
    if encoded.len() > REQUEST_LIMIT || batch["changes"].as_array().is_none_or(|c| c.len() > 2000) {
        return Err("This transaction exceeds the bounded business sync size; records were not partially committed".into());
    }
    conn.execute("INSERT INTO business_batches(id,ordinal,request_json,state) SELECT ?1,COALESCE(MAX(ordinal),0)+1,?2,'pending' FROM business_batches",params![batch["operationId"].as_str(),encoded]).map_err(db_error)?;
    for op in ops {
        conn.execute(
            "INSERT INTO business_batch_operations VALUES(?1,?2)",
            params![batch["operationId"].as_str(), op.as_str()],
        )
        .map_err(db_error)?;
    }
    conn.execute("DELETE FROM business_dirty", [])
        .map_err(db_error)?;
    Ok(())
}
pub(super) fn validate_receipt(
    receipt: &Value,
    request: &Value,
    scope: &super::member_worker::SyncScope,
    subject: Option<&str>,
) -> Result<()> {
    let keys = [
        "protocolVersion",
        "operationId",
        "deviceId",
        "gymId",
        "actorSubject",
        "sequence",
        "requestSha256",
    ];
    if receipt
        .as_object()
        .is_none_or(|v| v.len() != keys.len() || keys.iter().any(|k| !v.contains_key(*k)))
        || receipt["protocolVersion"] != 2
        || receipt["operationId"] != request["operationId"]
        || receipt["deviceId"] != request["deviceId"]
        || receipt["gymId"] != scope.gym_id
        || receipt["sequence"]
            .as_i64()
            .is_none_or(|n| !(1..=9_007_199_254_740_991).contains(&n))
        || receipt["requestSha256"] != hash(request)?
        || subject.is_some_and(|s| receipt["actorSubject"] != s)
    {
        return Err("Server business receipt does not match the immutable request".into());
    }
    Uuid::parse_str(field(receipt, "actorSubject")?)
        .map_err(|_| "Invalid confirmed business actor")?;
    Ok(())
}
fn sql_value(value: &Value) -> Result<SqlValue> {
    match value {
        Value::Null => Ok(SqlValue::Null),
        Value::String(s) if !s.contains('\0') => Ok(SqlValue::Text(s.clone())),
        Value::Number(n)
            if n.as_i64()
                .is_some_and(|v| v.unsigned_abs() <= 9_007_199_254_740_991) =>
        {
            Ok(SqlValue::Integer(n.as_i64().unwrap()))
        }
        _ => Err("Unsupported business row value".into()),
    }
}
pub(super) fn row_on(conn: &Connection, table: &Table, key: &str) -> Result<Option<Value>> {
    let fields = table
        .columns
        .iter()
        .map(|c| format!("'{}',{}", c.name, c.name))
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT json_object({fields}) FROM {} WHERE CAST({} AS TEXT)=?1",
        table.name, table.key
    );
    let row: Option<String> = conn
        .query_row(&query, [key], |r| r.get(0))
        .optional()
        .map_err(db_error)?;
    row.map(|r| serde_json::from_str(&r).map_err(|_| "Invalid local business row".into()))
        .transpose()
}
pub(super) fn default_profile() -> Value {
    json!({"id":1,"name":"Armstrong Fitness","location":"Matale, Sri Lanka","phone":"","email":"","version":1})
}
// Only an unused computer can download before freezing its baseline. Existing
// batches, including refused ones, must retain their original bytes and order.
fn prepare_initial_download(conn: &Connection) -> Result<bool> {
    let active: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='business_initial_download')",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if active {
        return Ok(true);
    }
    let used: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM metadata WHERE key='business_initial_download_complete') OR EXISTS(SELECT 1 FROM business_batches) OR EXISTS(SELECT 1 FROM outbox) OR EXISTS(SELECT 1 FROM local_operations) OR EXISTS(SELECT 1 FROM business_cursor WHERE sequence<>0) OR EXISTS(SELECT 1 FROM audit WHERE action<>'Verified staff sign-in')", [], |r| r.get(0)).map_err(db_error)?;
    if used {
        return Ok(false);
    }
    for table in tables()? {
        if table.name == "gym_settings" {
            if row_on(conn, &table, "1")? != Some(default_profile()) {
                return Ok(false);
            }
        } else if !matches!(table.name.as_str(), "users" | "audit") {
            let occupied: bool = conn
                .query_row(
                    &format!("SELECT EXISTS(SELECT 1 FROM {})", table.name),
                    [],
                    |r| r.get(0),
                )
                .map_err(db_error)?;
            if occupied {
                return Ok(false);
            }
        }
    }
    conn.execute(
        "INSERT INTO metadata VALUES('business_initial_download','1')",
        [],
    )
    .map_err(db_error)?;
    Ok(true)
}
fn apply_row(conn: &Connection, table: &Table, change: &Value) -> Result<()> {
    let after = &change["after"];
    let obj = after.as_object().ok_or("Invalid business row")?;
    if obj.len() != table.columns.len() || table.columns.iter().any(|c| !obj.contains_key(&c.name))
    {
        return Err("Business row columns do not match this release".into());
    }
    for c in &table.columns {
        let v = &obj[&c.name];
        if v.is_null() && c.nullable {
            continue;
        }
        if (c.kind == "integer" && !v.is_i64()) || (c.kind == "text" && !v.is_string()) {
            return Err("Invalid business column type".into());
        }
    }
    let key = field(change, "id")?;
    if after[&table.key]
        .as_str()
        .map(str::to_owned)
        .or_else(|| after[&table.key].as_i64().map(|n| n.to_string()))
        .as_deref()
        != Some(key)
    {
        return Err("Business row identity mismatch".into());
    }
    let existing = row_on(conn, table, key)?;
    let current = existing
        .as_ref()
        .map(|r| normalized(&table.name, r.clone()))
        .unwrap_or(Value::Null);
    if current == *after {
        return Ok(());
    }
    // Local role/activation always comes from verified enrollment, never a download.
    if table.name == "users" && existing.is_some() {
        if current["subject"] != after["subject"] {
            return Err("Historical user identity conflicts with this computer".into());
        }
        return Ok(());
    }
    let fresh_profile =
        table.name == "gym_settings" && change["before"].is_null() && current == default_profile();
    if current != change["before"] && !fresh_profile {
        return Err(format!(
            "Cloud {} conflicts with retained local history; no page was applied",
            table.name
        ));
    }
    if existing.is_some() && table.mutable.is_empty() {
        return Err("Downloaded financial history would replace an existing record".into());
    }
    if table.name == "users" && (after["active"] != 0 || after["version"] != 1) {
        return Err("Downloaded identities cannot grant access".into());
    }
    let columns = table
        .columns
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>();
    let values = columns
        .iter()
        .map(|c| sql_value(&after[*c]))
        .collect::<Result<Vec<_>>>()?;
    let marks = vec!["?"; columns.len()].join(",");
    let update = if existing.is_some() {
        format!(
            " DO UPDATE SET {}",
            table
                .mutable
                .iter()
                .map(|c| format!("{c}=excluded.{c}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    } else {
        " DO NOTHING".into()
    };
    let query = format!(
        "INSERT INTO {}({}) VALUES({marks}) ON CONFLICT({}){update}",
        table.name,
        columns.join(","),
        table.key
    );
    conn.execute(&query, rusqlite::params_from_iter(values))
        .map_err(db_error)?;
    Ok(())
}
fn apply_page(store: &mut Store, transport: &impl BusinessTransport, page: Value) -> Result<()> {
    let tx = store
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    super::member_worker::subject_on(&tx, store.removal_session.as_ref())?;
    let cursor: i64 = tx
        .query_row("SELECT sequence FROM business_cursor WHERE id=1", [], |r| {
            r.get(0)
        })
        .map_err(db_error)?;
    if page["protocolVersion"] != 2
        || page["gymId"] != transport.scope().gym_id
        || page["after"] != cursor
    {
        return Err("Business download scope/cursor does not match".into());
    }
    let changes = page["changes"]
        .as_array()
        .ok_or("Invalid business download")?;
    if changes.len() > 1 || !page["hasMore"].is_boolean() {
        return Err("Invalid bounded business page".into());
    }
    let pending: bool = tx
        .query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM business_batches WHERE state<>'confirmed' AND {ACTIVE_BATCH})"),
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if pending {
        return Err("Local business changes arrived during download; retry after pushing".into());
    }
    let mut next = cursor;
    for entry in changes {
        let sequence = entry["sequence"]
            .as_i64()
            .ok_or("Invalid business sequence")?;
        if sequence != cursor + 1 {
            return Err("Business download omitted an ordered transaction".into());
        }
        let request = &entry["request"];
        validate_receipt(&entry["receipt"], request, transport.scope(), None)?;
        if entry["receipt"]["sequence"] != sequence {
            return Err("Business receipt sequence mismatch".into());
        }
        let own: Option<String> = tx
            .query_row(
                "SELECT response_json FROM business_batches WHERE id=?1 AND state='confirmed'",
                [field(request, "operationId")?],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if let Some(saved) = own {
            if serde_json::from_str::<Value>(&saved).map_err(|_| "Invalid saved receipt")?
                != entry["receipt"]
            {
                return Err("Confirmed business history conflicts with server history".into());
            }
        } else {
            let rows = request["changes"]
                .as_array()
                .ok_or("Invalid business transaction")?;
            if rows.is_empty() || rows.len() > 2000 || request.to_string().len() > REQUEST_LIMIT {
                return Err("Invalid bounded business transaction".into());
            }
            let contract = tables()?;
            let by_name: BTreeMap<_, _> = contract.iter().map(|t| (t.name.as_str(), t)).collect();
            tx.execute("INSERT INTO metadata VALUES('business_import','1') ON CONFLICT(key) DO UPDATE SET value='1'",[]).map_err(db_error)?;
            tx.execute_batch("PRAGMA defer_foreign_keys=ON")
                .map_err(db_error)?;
            // Parents first, card revocations before newly assigned cards; releases
            // follow original allocations and the full reversing payment.
            let mut ordered = rows.iter().collect::<Vec<_>>();
            ordered.sort_by_key(|c| {
                let table = c["table"].as_str().unwrap_or("");
                let rank = contract
                    .iter()
                    .position(|t| t.name == table)
                    .unwrap_or(usize::MAX);
                let rank = if table == "payments" && !c["after"]["reverses_id"].is_null() {
                    17
                } else if table == "payment_reversal_details" {
                    18
                } else if table == "allocation_reversals" {
                    19
                } else {
                    rank
                };
                let revocation = matches!(table, "nfc_cards" | "staff_nfc_cards")
                    && !c["before"].is_null()
                    && !c["after"]["revoked_at"].is_null();
                (if revocation { 0 } else { rank + 1 }, c["before"].is_null())
            });
            for c in ordered {
                let t = by_name
                    .get(field(c, "table")?)
                    .ok_or("Unknown business table")?;
                apply_row(&tx, t, c)?;
                if t.name == "gym_settings" {
                    tx.execute("INSERT INTO metadata(key,value) SELECT 'business_initial_profile_received','1' WHERE EXISTS(SELECT 1 FROM metadata WHERE key='business_initial_download') ON CONFLICT(key) DO NOTHING", []).map_err(db_error)?;
                }
            }
            tx.execute("DELETE FROM metadata WHERE key='business_import'", [])
                .map_err(db_error)?;
            integrity(&tx)?;
        }
        next = sequence;
    }
    if page["nextCursor"] != next || (changes.is_empty() && page["hasMore"] != false) {
        return Err("Invalid business download cursor".into());
    }
    tx.execute("UPDATE business_cursor SET sequence=?1 WHERE id=1", [next])
        .map_err(db_error)?;
    tx.commit().map_err(db_error)
}

#[cfg(test)]
#[path = "business_sync_tests.rs"]
mod tests;
pub(crate) fn status(conn: &Connection) -> Result<Value> {
    let counts: (i64,i64)=conn.query_row(&format!("SELECT count(*) FILTER(WHERE state='confirmed'),count(*) FILTER(WHERE state<>'confirmed') FROM business_batches WHERE {ACTIVE_BATCH}"),[],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db_error)?;
    let conflicts=operations::rows(conn,&format!("SELECT json_object('id',id,'reason',last_error) FROM business_batches WHERE state='conflict' AND {ACTIVE_BATCH} ORDER BY ordinal"))?;
    let recovered: i64 = conn.query_row("SELECT count(*) FROM local_operations WHERE command='business_initial_profile_recovery' AND json_extract(result_json,'$.superseded')=1", [], |r| r.get(0)).map_err(db_error)?;
    let success: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='business_last_success'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    let error: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='business_last_error'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    Ok(
        json!({"acknowledged":counts.0,"pending":counts.1,"conflicts":conflicts,"recoveredInitialProfiles":recovered,"lastSuccessOn":success,"lastError":error,"available":false}),
    )
}
fn record_failure(
    store: &mut Store,
    failure: RemoteFailure,
    batch: Option<&str>,
    now: DateTime<Utc>,
) -> Result<super::member_worker::Run> {
    let denied = matches!(
        failure,
        RemoteFailure::Authentication | RemoteFailure::Authorization
    );
    let tx = store
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    super::member_worker::subject_on(&tx, store.removal_session.as_ref())?;
    let (conflicted, reason) = match &failure {
        RemoteFailure::BusinessConflict { code } => (
            true,
            match code.as_str() {
                "invalid_business_data" | "unsupported_business_table" => format!("Server refused the transaction: {code}. Check that the deployed server supports the latest staff and attendance records, then review and retry this transaction in Settings. Local history is retained."),
                "business_revision_conflict" => "Server refused the transaction: business_revision_conflict. The server record has changed since this computer's version. Review the affected records in Settings; retrying the same transaction cannot resolve a different version. Export a backup before reconciling the records. Local history is retained.".into(),
                _ => format!("Server refused the transaction: {code}. Local history is retained."),
            },
        ),
        _ if denied => (
            false,
            "Server denied access; sign in again. Local changes are retained.".into(),
        ),
        _ => (
            false,
            "Business synchronization failed. Unconfirmed changes are retained.".into(),
        ),
    };
    if let Some(id) = batch {
        tx.execute(
            "UPDATE business_batches SET state=?2,last_error=?3 WHERE id=?1 AND state='pending'",
            params![id, if conflicted { "conflict" } else { "pending" }, reason],
        )
        .map_err(db_error)?;
    }
    tx.execute("INSERT INTO metadata VALUES('business_last_error',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[&reason]).map_err(db_error)?;
    let failures: i64 = tx
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='business_failures'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .optional()
        .map_err(db_error)?
        .unwrap_or(0)
        .saturating_add(1)
        .min(9);
    let requested = match failure {
        RemoteFailure::Transient {
            retry_after_seconds,
        } => i64::from(retry_after_seconds.unwrap_or(0).min(3600)),
        _ => 0,
    };
    let retry =
        (now + chrono::Duration::seconds((1i64 << failures).min(300).max(requested))).to_rfc3339();
    for (key, value) in [
        ("business_retry_on", retry),
        ("business_failures", failures.to_string()),
    ] {
        tx.execute("INSERT INTO metadata VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value]).map_err(db_error)?;
    }
    tx.commit().map_err(db_error)?;
    if denied {
        store.removal_session = None;
    }
    Ok(if conflicted {
        super::member_worker::Run::Blocked
    } else {
        super::member_worker::Run::Failed
    })
}
impl Store {
    pub(crate) fn run_business_sync(
        &mut self,
        transport: &mut impl BusinessTransport,
        now: DateTime<Utc>,
        limits: super::member_worker::Limits,
    ) -> Result<super::member_worker::Run> {
        use super::member_worker::{authorized, Run};
        authorized(self, transport)?;
        if limits.pushes == 0 || limits.pages == 0 || limits.pushes > 1000 || limits.pages > 1000 {
            return Err("Invalid bounded business sync limits".into());
        }
        let retry: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='business_retry_on'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if let Some(retry) = retry {
            let retry_on = DateTime::parse_from_rfc3339(&retry)
                .map_err(|_| "Invalid business retry state")?
                .with_timezone(&Utc);
            if now < retry_on {
                return Ok(Run::Deferred { retry_on });
            }
        }
        let initial = {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db_error)?;
            super::member_worker::subject_on(&tx, self.removal_session.as_ref())?;
            let initial = prepare_initial_download(&tx)?;
            tx.commit().map_err(db_error)?;
            initial
        };
        if initial {
            for pages in 1..=limits.pages {
                authorized(self, transport)?;
                let cursor: i64 = self
                    .conn
                    .query_row("SELECT sequence FROM business_cursor WHERE id=1", [], |r| {
                        r.get(0)
                    })
                    .map_err(db_error)?;
                let page = match transport.pull_business(cursor) {
                    Ok(page) => page,
                    Err(failure) => {
                        authorized(self, transport)?;
                        return record_failure(self, failure, None, now);
                    }
                };
                authorized(self, transport)?;
                let more = page["hasMore"].as_bool().ok_or("Invalid business page")?;
                if apply_page(self, transport, page).is_err() {
                    return record_failure(
                        self,
                        RemoteFailure::BusinessConflict {
                            code: "download_conflicts_with_local_history".into(),
                        },
                        None,
                        now,
                    );
                }
                if !more {
                    let tx = self
                        .conn
                        .transaction_with_behavior(TransactionBehavior::Immediate)
                        .map_err(db_error)?;
                    super::member_worker::subject_on(&tx, self.removal_session.as_ref())?;
                    let received: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM metadata WHERE key='business_initial_profile_received')", [], |r| r.get(0)).map_err(db_error)?;
                    if received {
                        // Discard only the unfrozen installation default; signup
                        // audit and identity rows remain queued for confirmation.
                        let seed: Option<(Option<String>, String)> = tx.query_row("SELECT before_json,after_json FROM business_dirty WHERE table_name='gym_settings' AND record_id='1'", [], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(db_error)?;
                        if let Some((before, after)) = seed {
                            if before.is_some()
                                || serde_json::from_str::<Value>(&after)
                                    .map_err(|_| "Invalid initial gym profile")?
                                    != default_profile()
                            {
                                return Err("Initial gym profile changed during download; local history was retained".into());
                            }
                            tx.execute("DELETE FROM business_dirty WHERE table_name='gym_settings' AND record_id='1'", []).map_err(db_error)?;
                        }
                    }
                    tx.execute("DELETE FROM metadata WHERE key IN ('business_initial_download','business_initial_profile_received','business_last_error','business_retry_on','business_failures')", []).map_err(db_error)?;
                    tx.execute("INSERT INTO metadata VALUES('business_initial_download_complete','1') ON CONFLICT(key) DO NOTHING", []).map_err(db_error)?;
                    let read_only = !self.removal_session.as_ref().is_some_and(|s| s.can_write);
                    if read_only {
                        tx.execute("INSERT INTO metadata VALUES('business_last_success',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [now.to_rfc3339()]).map_err(db_error)?;
                    }
                    tx.commit().map_err(db_error)?;
                    return Ok(if read_only {
                        Run::Complete { pushed: 0, pages }
                    } else {
                        Run::Yielded { pushed: 0, pages }
                    });
                }
            }
            // Continue on the next bounded run, including after a process restart.
            return Ok(Run::Yielded {
                pushed: 0,
                pages: limits.pages,
            });
        }
        if self.removal_session.as_ref().is_some_and(|s| s.can_write) {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db_error)?;
            capture(&tx)?;
            tx.commit().map_err(db_error)?;
        }
        let mut pushed = 0;
        while pushed < limits.pushes {
            authorized(self, transport)?;
            let queued:Option<(String,String,String)>=self.conn.query_row(&format!("SELECT id,request_json,state FROM business_batches WHERE state<>'confirmed' AND {ACTIVE_BATCH} ORDER BY ordinal LIMIT 1"),[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(db_error)?;
            let Some((id, encoded, state)) = queued else {
                break;
            };
            if state == "conflict" {
                return Ok(Run::Blocked);
            }
            if !self.removal_session.as_ref().is_some_and(|s| s.can_write) {
                return Err("This computer cannot upload retained business changes; writer approval is required".into());
            }
            let request: Value =
                serde_json::from_str(&encoded).map_err(|_| "Invalid frozen business request")?;
            if request["deviceId"] != transport.scope().device_id {
                return Err(
                    "Business operation belongs to another computer; reconciliation required"
                        .into(),
                );
            }
            if !request["actorSubject"].is_null() && request["actorSubject"] != transport.subject()
            {
                return Err(
                    "Sign in as the original business actor before uploading this transaction"
                        .into(),
                );
            }
            match transport.push_business(&request) {
                Ok(receipt) => {
                    authorized(self, transport)?;
                    if validate_receipt(
                        &receipt,
                        &request,
                        transport.scope(),
                        Some(transport.subject()),
                    )
                    .is_err()
                    {
                        return record_failure(
                            self,
                            RemoteFailure::InvalidResponse,
                            Some(&id),
                            now,
                        );
                    }
                    let tx = self
                        .conn
                        .transaction_with_behavior(TransactionBehavior::Immediate)
                        .map_err(db_error)?;
                    super::member_worker::subject_on(&tx, self.removal_session.as_ref())?;
                    if tx.execute("UPDATE business_batches SET state='confirmed',response_json=?2,last_error=NULL WHERE id=?1 AND state='pending' AND request_json=?3",params![id,receipt.to_string(),encoded]).map_err(db_error)?!=1 {return Err("Business delivery changed before its receipt could commit".into());}
                    tx.commit().map_err(db_error)?;
                    pushed += 1;
                }
                Err(failure) => {
                    authorized(self, transport)?;
                    return record_failure(self, failure, Some(&id), now);
                }
            }
        }
        if pushed == limits.pushes {
            return Ok(Run::Yielded { pushed, pages: 0 });
        }
        for pages in 1..=limits.pages {
            authorized(self, transport)?;
            let cursor: i64 = self
                .conn
                .query_row("SELECT sequence FROM business_cursor WHERE id=1", [], |r| {
                    r.get(0)
                })
                .map_err(db_error)?;
            match transport.pull_business(cursor) {
                Ok(page) => {
                    authorized(self, transport)?;
                    let more = page["hasMore"].as_bool().ok_or("Invalid business page")?;
                    if let Err(error) = apply_page(self, transport, page) {
                        let failure = RemoteFailure::BusinessConflict {
                            code: if error.contains("arrived during download") {
                                "local_write_during_download".into()
                            } else {
                                "download_conflicts_with_local_history".into()
                            },
                        };
                        return record_failure(self, failure, None, now);
                    }
                    if !more {
                        let tx = self
                            .conn
                            .transaction_with_behavior(TransactionBehavior::Immediate)
                            .map_err(db_error)?;
                        super::member_worker::subject_on(&tx, self.removal_session.as_ref())?;
                        tx.execute("INSERT INTO metadata VALUES('business_last_success',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[now.to_rfc3339()]).map_err(db_error)?;
                        tx.execute("DELETE FROM metadata WHERE key IN ('business_last_error','business_retry_on','business_failures')",[]).map_err(db_error)?;
                        tx.commit().map_err(db_error)?;
                        return Ok(Run::Complete { pushed, pages });
                    }
                }
                Err(failure) => {
                    authorized(self, transport)?;
                    return record_failure(self, failure, None, now);
                }
            }
        }
        Ok(Run::Yielded {
            pushed,
            pages: limits.pages,
        })
    }
}
