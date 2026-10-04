use super::member_sync::{self, RemoteMember};
use super::operations::{receipt, replay};
use super::*;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictChoice {
    UseServer,
    KeepLocal,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberConflictInput {
    pub request_id: String,
    pub conflict_id: String,
    pub fingerprint: String,
    pub choice: ConflictChoice,
    pub reason: String,
}
fn rows(conn: &Connection, sql: &str, member: &str) -> Result<Vec<Value>> {
    let mut stmt = conn.prepare(sql).map_err(db_error)?;
    let result = stmt
        .query_map([member], |r| r.get::<_, String>(0))
        .map_err(db_error)?
        .map(|r| serde_json::from_str(&r.map_err(db_error)?).map_err(|e| e.to_string()))
        .collect();
    result
}
fn remote_record(member: &str, conflicts: &[Value], head: &Value) -> Result<RemoteMember> {
    let mut latest: Option<RemoteMember> = None;
    let mut revisions = std::collections::BTreeMap::new();
    for value in conflicts
        .iter()
        .map(|c| &c["remote"])
        .chain(head.get("snapshot"))
    {
        let remote: RemoteMember = serde_json::from_value(value.clone())
            .map_err(|_| "A complete server member record is required; obtain a verified server response before review")?;
        member_sync::validate(&remote)?;
        if remote.id != member {
            return Err("Recorded server member identity does not match this conflict".into());
        }
        if let Some(previous) = revisions.insert(remote.revision, value.clone()) {
            if previous != *value {
                return Err(
                    "Remote revision has inconsistent snapshots; server review is required".into(),
                );
            }
        }
        if latest
            .as_ref()
            .is_none_or(|old| remote.revision > old.revision)
        {
            latest = Some(remote);
        }
    }
    if let Some(snapshot) = head.get("snapshot") {
        if head["revision"] != snapshot["revision"] {
            return Err("Saved server head revision is inconsistent".into());
        }
    }
    latest.ok_or_else(|| "A recorded server member is required".into())
}
fn review(conn: &Connection, conflict: &str, identity: &Value) -> Result<Value> {
    let member: String = conn
        .query_row(
            "SELECT member_id FROM member_active_conflicts WHERE id=?1",
            [conflict],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?
        .ok_or("Conflict has already been resolved or no longer exists")?;
    let local = conn.query_row("SELECT json_object('id',id,'name',name,'phone',phone,'email',email,'nfcId',nfc_id,'joinedOn',joined_on,'version',version,'archivedAt',archived_at,'archivedByUserId',archived_by_user_id) FROM members WHERE id=?1", [&member], |r|r.get::<_,String>(0))
        .optional().map_err(db_error)?.map(|v|serde_json::from_str::<Value>(&v).map_err(|e|e.to_string())).transpose()?.unwrap_or(Value::Null);
    let conflicts = rows(conn, "SELECT json_object('id',id,'operationId',operation_id,'reason',reason,'remote',json(remote_json),'createdAt',created_at) FROM member_active_conflicts WHERE member_id=?1 ORDER BY created_at,id", &member)?;
    let operations = rows(conn, "SELECT json_object('id',o.id,'deviceId',o.device_id,'action',o.action,'version',o.expected_version,'payload',json(o.payload_json),'createdAt',o.created_at,'deliveryState',d.state,'request',json(d.request_json)) FROM outbox o LEFT JOIN member_deliveries d ON d.operation_id=o.id WHERE o.entity='member' AND o.entity_id=?1 AND NOT EXISTS(SELECT 1 FROM member_deliveries a WHERE a.operation_id=o.id AND a.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=o.id) ORDER BY o.rowid", &member)?;
    let head = conn.query_row("SELECT json_object('revision',revision,'snapshot',json(snapshot_json)) FROM member_remote_heads WHERE member_id=?1", [&member], |r|r.get::<_,String>(0))
        .optional().map_err(db_error)?.map(|v|serde_json::from_str::<Value>(&v).map_err(|e|e.to_string())).transpose()?.unwrap_or(Value::Null);
    let mut state = json!({"conflictId":conflict,"memberId":member,"identity":identity,"local":local,"conflicts":conflicts,"operations":operations,"head":head});
    let fingerprint = format!("{:x}", Sha256::digest(state.to_string().as_bytes()));
    let remote = remote_record(&member, &conflicts, &head);
    let common = if operations.iter().any(|o| o["deliveryState"] == "pending") {
        Some("Retry the unconfirmed member request before resolving; it may already have reached the server".to_string())
    } else if operations.iter().any(|o| o["action"] == "delete") {
        Some("Deleted member history requires server reconciliation".to_string())
    } else if operations
        .iter()
        .any(|o| o["deviceId"] != identity["scope"]["deviceId"])
    {
        Some(
            "Pending operation belongs to another device; server reconciliation is required"
                .to_string(),
        )
    } else {
        match &remote {
            Err(reason) => Some(reason.clone()),
            Ok(remote) if !local.is_null() && local["joinedOn"] != remote.joined_on => Some("Joined date differs from immutable local history; server reconciliation is required".into()),
            _ => None,
        }
    };
    let mut server_reason = common.clone();
    let mut local_reason = common;
    if let Ok(remote) = &remote {
        if server_reason.is_none() {
            let collision: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM members WHERE nfc_id=?1 AND id<>?2)",
                    params![remote.nfc_id, member],
                    |r| r.get(0),
                )
                .map_err(db_error)?;
            server_reason = if collision {
                Some("Server card is assigned to another local member".into())
            } else if local["archivedAt"].is_string() && remote.archived_at.is_none() {
                Some("An archived local member cannot be reactivated by conflict review".into())
            } else {
                member_sync::archive_identity(conn, remote).err()
            };
        }
        if local_reason.is_none() {
            local_reason = if local.is_null() {
                Some("There is no local member version to retry".into())
            } else if local["archivedAt"].is_string() || remote.archived_at.is_some() {
                Some("Keeping an archived member requires server reconciliation; archive intent cannot be replaced by an edit".into())
            } else {
                None
            };
        }
    }
    state["remote"] = remote
        .ok()
        .map(|v| serde_json::to_value(v).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(Value::Null);
    state["fingerprint"] = json!(fingerprint);
    state["useServer"] =
        json!({"allowed":server_reason.is_none(),"reason":server_reason.unwrap_or_default()});
    state["keepLocal"] =
        json!({"allowed":local_reason.is_none(),"reason":local_reason.unwrap_or_default()});
    Ok(state)
}
pub(super) fn authorization(store: &Store) -> Value {
    let checked = removal::authorized(&store.conn, store.removal_session.as_ref())
        .and_then(|_| member_worker::review_scope(&store.conn, store.removal_session.as_ref()));
    match checked {
        Ok(_) => json!({"allowed":true,"reason":""}),
        Err(reason) => json!({"allowed":false,"reason":reason}),
    }
}
impl Store {
    pub fn preview_member_conflict(&self, conflict_id: &str) -> Result<Value> {
        let tx = self.conn.unchecked_transaction().map_err(db_error)?;
        let actor = removal::authorized(&tx, self.removal_session.as_ref())?;
        let mut identity = member_worker::review_scope(&tx, self.removal_session.as_ref())?;
        identity["actorUserId"] = json!(actor.user_id);
        let result = review(&tx, conflict_id, &identity)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn resolve_member_conflict(&mut self, input: MemberConflictInput) -> Result<Value> {
        let reason = required(&input.reason, "Review reason", 500)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let actor = removal::authorized(&tx, self.removal_session.as_ref())?;
        let mut identity = member_worker::review_scope(&tx, self.removal_session.as_ref())?;
        identity["actorUserId"] = json!(actor.user_id);
        let bound = json!({"input":input,"identity":identity});
        if let Some(result) = replay(&tx, "resolve_member_conflict", &input.request_id, &bound)? {
            return Ok(result);
        }
        let review = review(&tx, &input.conflict_id, &identity)?;
        if review["fingerprint"] != input.fingerprint {
            return Err("Member or conflict changed. Review the current versions again; no records were replaced".into());
        }
        let (choice, permission) = match input.choice {
            ConflictChoice::UseServer => ("use_server", "useServer"),
            ConflictChoice::KeepLocal => ("keep_local", "keepLocal"),
        };
        if review[permission]["allowed"] != true {
            return Err(review[permission]["reason"]
                .as_str()
                .unwrap_or("Server reconciliation is required")
                .into());
        }
        let remote: RemoteMember =
            serde_json::from_value(review["remote"].clone()).map_err(|e| e.to_string())?;
        let resolution = id();
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO member_conflict_resolutions VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                resolution,
                remote.id,
                actor.user_id,
                actor.label,
                choice,
                reason,
                review.to_string(),
                now
            ],
        )
        .map_err(db_error)?;
        for conflict in review["conflicts"]
            .as_array()
            .ok_or("Invalid reviewed conflicts")?
        {
            tx.execute(
                "INSERT INTO member_resolved_conflicts VALUES(?1,?2)",
                params![conflict["id"].as_str(), resolution],
            )
            .map_err(db_error)?;
        }
        for operation in review["operations"]
            .as_array()
            .ok_or("Invalid reviewed operations")?
        {
            tx.execute(
                "INSERT INTO member_resolved_operations VALUES(?1,?2)",
                params![operation["id"].as_str(), resolution],
            )
            .map_err(db_error)?;
        }
        member_sync::head(&tx, &remote)?;
        let mut retry_operation: Option<String> = None;
        match input.choice {
            ConflictChoice::UseServer => {
                let archive_actor = member_sync::archive_identity(&tx, &remote)?;
                member_sync::write_remote(&tx, &remote, archive_actor)?;
            }
            ConflictChoice::KeepLocal => {
                tx.execute(
                    "UPDATE members SET version=version+1 WHERE id=?1",
                    [&remote.id],
                )
                .map_err(db_error)?;
                let mut payload = removal::member_record(&tx, &remote.id)?;
                payload["actorUserId"] = json!(actor.user_id);
                payload["actor"] = json!(actor.label);
                let operation = id();
                tx.execute("INSERT INTO outbox(id,device_id,entity,entity_id,action,expected_version,payload_json,schema_version,created_at) VALUES(?1,?2,'member',?3,'update',?4,?5,?6,?7)", params![operation,identity["scope"]["deviceId"].as_str(),remote.id,review["local"]["version"].as_i64(),payload.to_string(),SCHEMA_VERSION,now]).map_err(db_error)?;
                retry_operation = Some(operation);
            }
        }
        let after = removal::member_record(&tx, &remote.id)?;
        let result = json!({"resolutionId":resolution,"memberId":remote.id,"choice":choice,"superseded":review["operations"].as_array().unwrap().len(),"retryOperationId":retry_operation,"serverConfirmed":false});
        tx.execute("INSERT INTO audit(id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity,actor_user_id) VALUES(?1,?2,?3,'resolve member conflict',?4,?5,?6,?7,'member',?8)", params![id(),actor.label,identity["scope"]["deviceId"].as_str(),remote.id,review.to_string(),json!({"member":after,"resolution":result,"reason":reason}).to_string(),now,actor.user_id]).map_err(db_error)?;
        receipt(
            &tx,
            "resolve_member_conflict",
            &input.request_id,
            &bound,
            &result,
        )?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}
