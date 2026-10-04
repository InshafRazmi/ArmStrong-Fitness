// Internal protocol/storage boundary. No IPC accepts server receipts or pages.
// The native HTTPS transport exists; production scheduling awaits API acceptance.
#![allow(dead_code)]
use super::*;
use rusqlite::Transaction;
pub(crate) const PENDING_DURING_PULL: &str =
    "Pending member operations arrived during pull; retry pushes before pull";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RemoteActor {
    pub(super) id: String,
    pub(super) name: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RemoteMember {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) phone: String,
    pub(super) email: String,
    pub(super) nfc_id: Option<String>,
    pub(super) joined_on: String,
    pub(super) revision: i64,
    pub(super) archived_at: Option<String>,
    pub(super) archived_by: Option<RemoteActor>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Receipt {
    protocol_version: i64,
    operation_id: String,
    member_id: String,
    revision: i64,
    sequence: i64,
    member: RemoteMember,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Change {
    sequence: i64,
    operation_id: String,
    member: RemoteMember,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Page {
    protocol_version: i64,
    after: i64,
    next_cursor: i64,
    has_more: bool,
    changes: Vec<Change>,
}
fn uuid(value: &str) -> Result<()> {
    if Uuid::parse_str(value)
        .map_err(|_| "Invalid sync UUID")?
        .to_string()
        != value
    {
        return Err("Noncanonical sync UUID".into());
    }
    Ok(())
}
fn available(conn: &Connection) -> Result<()> {
    let blocked: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='restore_requires_reconciliation')",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if blocked {
        return Err("Restored database requires server reconciliation before sync".into());
    }
    Ok(())
}
fn pending(conn: &Connection, member: &str) -> Result<bool> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM outbox o WHERE entity='member' AND entity_id=?1 AND NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=o.id AND d.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=o.id))",[member],|r|r.get(0)).map_err(db_error)
}
pub(super) fn validate(member: &RemoteMember) -> Result<()> {
    uuid(&member.id)?;
    required(&member.name, "Remote name", 120)?;
    required(&member.phone, "Remote phone", 40)?;
    date(&member.joined_on)?;
    if member.revision < 1
        || member.revision > 9_007_199_254_740_991
        || member.email.len() > 254
        || (!member.email.is_empty()
            && (!member.email.contains('@') || member.email.chars().any(char::is_whitespace)))
    {
        return Err("Invalid remote member fields".into());
    }
    if let Some(card) = &member.nfc_id {
        if card.is_empty()
            || card.len() > 128
            || !card.bytes().all(|c| (33..=126).contains(&c))
            || card.to_ascii_uppercase() != *card
        {
            return Err("Invalid normalized remote card".into());
        }
    }
    match (&member.archived_at, &member.archived_by) {
        (None, None) => (),
        (Some(time), Some(actor)) => {
            DateTime::parse_from_rfc3339(time).map_err(|_| "Invalid archive timestamp")?;
            uuid(&actor.id)?;
            required(&actor.name, "Archive actor", 120)?;
        }
        _ => return Err("Remote archive must include its verified actor".into()),
    }
    Ok(())
}
fn conflict(
    tx: &Transaction<'_>,
    member: &str,
    operation: Option<&str>,
    reason: &str,
    remote: &Value,
) -> Result<()> {
    tx.execute(
        "INSERT INTO member_sync_conflicts VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            id(),
            member,
            operation,
            reason,
            remote.to_string(),
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(db_error)?;
    Ok(())
}
pub(super) fn head(tx: &Transaction<'_>, member: &RemoteMember) -> Result<()> {
    let value = serde_json::to_string(member).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO member_remote_heads VALUES(?1,?2,?3) ON CONFLICT(member_id) DO UPDATE SET revision=excluded.revision,snapshot_json=excluded.snapshot_json WHERE excluded.revision>member_remote_heads.revision",params![member.id,member.revision,value]).map_err(db_error)?;
    Ok(())
}
pub(super) fn append_snapshot(conn: &Connection, snapshot: &mut Value) -> Result<()> {
    snapshot["memberSync"] = json!({
        "available":false,
        "reason":"Member synchronization is disabled pending real API acceptance",
        "cursor":conn.query_row("SELECT sequence FROM member_sync_cursor WHERE id=1",[],|r|r.get::<_,i64>(0)).map_err(db_error)?,
        "resolved":conn.query_row("SELECT count(*) FROM member_conflict_resolutions",[],|r|r.get::<_,i64>(0)).map_err(db_error)?,
        "superseded":conn.query_row("SELECT count(*) FROM member_resolved_operations",[],|r|r.get::<_,i64>(0)).map_err(db_error)?,
        "acknowledged":conn.query_row("SELECT count(*) FROM member_deliveries WHERE state='acknowledged'",[],|r|r.get::<_,i64>(0)).map_err(db_error)?,
        "conflicts":operations::rows(conn,"SELECT json_object('id',id,'memberId',member_id,'operationId',operation_id,'reason',reason,'remote',json(remote_json),'createdAt',created_at) FROM member_active_conflicts ORDER BY created_at,id")?
    });
    super::member_worker::append_status(conn, snapshot)?;
    Ok(())
}
impl Store {
    pub(crate) fn prepare_member_push(&mut self) -> Result<Option<Value>> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        available(&tx)?;
        let candidates=operations::rows(&tx,"SELECT json_object('operationId',o.id,'deviceId',o.device_id,'memberId',o.entity_id,'action',o.action,'payload',json(o.payload_json)) FROM outbox o WHERE entity='member' AND action IN ('create','update','archive') AND NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=o.id AND d.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=o.id) AND NOT EXISTS(SELECT 1 FROM member_active_conflicts c WHERE c.member_id=o.entity_id) AND NOT EXISTS(SELECT 1 FROM outbox tomb WHERE tomb.entity='member' AND tomb.entity_id=o.entity_id AND tomb.action='delete') AND NOT EXISTS(SELECT 1 FROM outbox prev WHERE prev.entity='member' AND prev.entity_id=o.entity_id AND prev.rowid<o.rowid AND NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=prev.id AND d.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=prev.id)) ORDER BY o.rowid LIMIT 1")?;
        let Some(candidate) = candidates.first() else {
            return Ok(None);
        };
        let operation = candidate["operationId"]
            .as_str()
            .ok_or("Invalid outbox operation")?;
        let member = candidate["memberId"]
            .as_str()
            .ok_or("Invalid outbox member")?;
        uuid(operation)?;
        uuid(member)?;
        let existing:Option<String>=tx.query_row("SELECT request_json FROM member_deliveries WHERE operation_id=?1 AND state='pending'",[operation],|r|r.get(0)).optional().map_err(db_error)?;
        let request = if let Some(saved) = existing {
            tx.execute(
                "UPDATE member_deliveries SET attempts=attempts+1 WHERE operation_id=?1",
                [operation],
            )
            .map_err(db_error)?;
            serde_json::from_str(&saved).map_err(|e| e.to_string())?
        } else {
            let revision: Option<i64> = tx
                .query_row(
                    "SELECT revision FROM member_remote_heads WHERE member_id=?1",
                    [member],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db_error)?;
            let action = candidate["action"]
                .as_str()
                .ok_or("Invalid outbox action")?;
            if action == "archive" {
                let actor = candidate["payload"]["actorUserId"]
                    .as_str()
                    .ok_or("Archive has no verified local actor")?;
                let subject: String = tx
                    .query_row("SELECT subject FROM users WHERE id=?1", [actor], |r| {
                        r.get(0)
                    })
                    .map_err(db_error)?;
                uuid(&subject)
                    .map_err(|_| "Archive actor is not mapped to a Supabase staff identity")?;
            }
            let expected = revision.unwrap_or(0);
            if (action == "create") != (expected == 0) {
                return Err("Member has no matching remote base; reconciliation required".into());
            }
            let payload = &candidate["payload"];
            // Legacy v1 did not include joinedOn; joined_on is immutable locally.
            let joined: String = tx
                .query_row("SELECT joined_on FROM members WHERE id=?1", [member], |r| {
                    r.get(0)
                })
                .map_err(db_error)?;
            let projection = if action == "archive" {
                Value::Null
            } else {
                json!({"name":payload["name"],"phone":payload["phone"],"email":payload["email"],"nfcId":payload["nfcId"],"joinedOn":payload.get("joinedOn").cloned().unwrap_or(json!(joined))})
            };
            let request = json!({"protocolVersion":1,"operationId":operation,"deviceId":candidate["deviceId"],"memberId":member,"action":action,"expectedRevision":expected,"member":projection});
            tx.execute(
                "INSERT INTO member_deliveries(operation_id,request_json) VALUES(?1,?2)",
                params![operation, request.to_string()],
            )
            .map_err(db_error)?;
            request
        };
        tx.commit().map_err(db_error)?;
        Ok(Some(request))
    }
    #[cfg(test)]
    pub(crate) fn acknowledge_member(&mut self, receipt: Receipt) -> Result<()> {
        self.acknowledge_member_policy(receipt, None)
    }
    pub(crate) fn acknowledge_member_for_worker(
        &mut self,
        receipt: Receipt,
        subject: &str,
    ) -> Result<()> {
        self.acknowledge_member_policy(receipt, Some(subject))
    }
    fn acknowledge_member_policy(&mut self, receipt: Receipt, subject: Option<&str>) -> Result<()> {
        validate(&receipt.member)?;
        if receipt.protocol_version != 1
            || receipt.member_id != receipt.member.id
            || receipt.revision != receipt.member.revision
            || receipt.sequence < 1
        {
            return Err("Invalid member receipt".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        available(&tx)?;
        if let Some(subject) = subject {
            super::member_worker::authorize_reply(&tx, self.removal_session.as_ref(), subject)?;
        }
        let retired: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM member_resolved_operations WHERE operation_id=?1)",
                [&receipt.operation_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if retired {
            return Err("Resolved operation cannot be acknowledged".into());
        }
        let saved:Option<(String,String,Option<String>)>=tx.query_row("SELECT request_json,state,response_json FROM member_deliveries WHERE operation_id=?1",[&receipt.operation_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(db_error)?;
        let (request, state, response) =
            saved.ok_or("Receipt was not requested; outbox retained")?;
        let encoded = serde_json::to_string(&receipt).map_err(|e| e.to_string())?;
        if state == "acknowledged" {
            if response.as_deref() != Some(encoded.as_str()) {
                return Err("Server receipt changed".into());
            }
            return Ok(());
        }
        if state != "pending" {
            return Err("Conflicted delivery requires reconciliation".into());
        }
        let request: Value = serde_json::from_str(&request).map_err(|e| e.to_string())?;
        if request["memberId"] != receipt.member_id
            || request["expectedRevision"]
                .as_i64()
                .and_then(|v| v.checked_add(1))
                != Some(receipt.revision)
        {
            return Err("Mismatched member receipt; outbox retained".into());
        }
        if request["action"] != "archive" {
            let expected = &request["member"];
            let actual = json!({"name":receipt.member.name,"phone":receipt.member.phone,"email":receipt.member.email,"nfcId":receipt.member.nfc_id,"joinedOn":receipt.member.joined_on});
            if expected != &actual || receipt.member.archived_at.is_some() {
                return Err("Receipt member differs from sent operation".into());
            }
        } else {
            let subject:String=tx.query_row("SELECT u.subject FROM outbox o JOIN users u ON u.id=json_extract(o.payload_json,'$.actorUserId') WHERE o.id=?1",[&receipt.operation_id],|r|r.get(0)).map_err(db_error)?;
            if receipt
                .member
                .archived_by
                .as_ref()
                .map(|actor| actor.id.as_str())
                != Some(subject.as_str())
            {
                return Err(
                    "Archive receipt actor differs from the original enrolled staff identity"
                        .into(),
                );
            }
            let payload: String = tx
                .query_row(
                    "SELECT payload_json FROM outbox WHERE id=?1",
                    [&receipt.operation_id],
                    |r| r.get(0),
                )
                .map_err(db_error)?;
            let payload: Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
            let after = &payload["after"];
            let actual = json!({"name":receipt.member.name,"phone":receipt.member.phone,"email":receipt.member.email,"nfcId":receipt.member.nfc_id,"joinedOn":receipt.member.joined_on});
            for field in ["name", "phone", "email", "nfcId", "joinedOn"] {
                if after[field] != actual[field] {
                    return Err("Archive receipt differs from original member details".into());
                }
            }
            if receipt.member.archived_at.is_none() {
                return Err("Archive receipt missing archive".into());
            }
        }
        tx.execute("UPDATE member_deliveries SET state='acknowledged',response_json=?2,acknowledged_at=?3,last_error=NULL WHERE operation_id=?1",params![receipt.operation_id,encoded,Utc::now().to_rfc3339()]).map_err(db_error)?;
        head(&tx, &receipt.member)?;
        tx.commit().map_err(db_error)
    }
    pub(crate) fn reject_member_push(
        &mut self,
        operation: &str,
        reason: &str,
        remote: &Value,
        permanent: bool,
    ) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        available(&tx)?;
        let request:String=tx.query_row("SELECT request_json FROM member_deliveries WHERE operation_id=?1 AND state='pending'",[operation],|r|r.get(0)).map_err(db_error)?;
        let request: Value = serde_json::from_str(&request).map_err(|e| e.to_string())?;
        if permanent {
            conflict(
                &tx,
                request["memberId"].as_str().ok_or("Missing member")?,
                Some(operation),
                reason,
                remote,
            )?;
        }
        tx.execute(
            "UPDATE member_deliveries SET state=?2,last_error=?3 WHERE operation_id=?1",
            params![
                operation,
                if permanent { "conflict" } else { "pending" },
                reason
            ],
        )
        .map_err(db_error)?;
        tx.commit().map_err(db_error)
    }
    #[cfg(test)]
    pub(crate) fn apply_member_page(&mut self, page: Page) -> Result<()> {
        self.apply_member_page_policy(page, None)
    }
    pub(crate) fn apply_member_page_after_pushes(
        &mut self,
        page: Page,
        subject: &str,
    ) -> Result<()> {
        self.apply_member_page_policy(page, Some(subject))
    }
    fn apply_member_page_policy(&mut self, page: Page, subject: Option<&str>) -> Result<()> {
        if page.protocol_version != 1
            || page.changes.len() > 100
            || (page.has_more && page.changes.is_empty())
        {
            return Err("Invalid member page".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        available(&tx)?;
        if let Some(subject) = subject {
            super::member_worker::authorize_reply(&tx, self.removal_session.as_ref(), subject)?;
            let pending: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM outbox o WHERE o.entity='member' AND NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=o.id AND d.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=o.id))", [], |r|r.get(0)).map_err(db_error)?;
            if pending {
                return Err(PENDING_DURING_PULL.into());
            }
        }
        let cursor: i64 = tx
            .query_row(
                "SELECT sequence FROM member_sync_cursor WHERE id=1",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if page.after != cursor {
            return Err("Stale member cursor".into());
        }
        let mut sequence = cursor;
        for change in &page.changes {
            sequence = sequence.checked_add(1).ok_or("Invalid sync sequence")?;
            if change.sequence != sequence {
                return Err("Noncontiguous member page".into());
            }
            uuid(&change.operation_id)?;
            validate(&change.member)?;
            apply_change(&tx, change)?;
        }
        if page.next_cursor != sequence {
            return Err("Page cursor does not match changes".into());
        }
        tx.execute(
            "UPDATE member_sync_cursor SET sequence=?1 WHERE id=1",
            [sequence],
        )
        .map_err(db_error)?;
        tx.commit().map_err(db_error)
    }
}
fn apply_change(tx: &Transaction<'_>, change: &Change) -> Result<()> {
    let member = &change.member;
    let remote = serde_json::to_value(member).map_err(|e| e.to_string())?;
    let prior: Option<(i64, String)> = tx
        .query_row(
            "SELECT revision,snapshot_json FROM member_remote_heads WHERE member_id=?1",
            [&member.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(db_error)?;
    if let Some((revision, snapshot)) = prior {
        if member.revision < revision {
            return Ok(());
        }
        if member.revision == revision {
            if serde_json::from_str::<Value>(&snapshot).map_err(|e| e.to_string())? != remote {
                return Err("Remote revision has inconsistent snapshots".into());
            }
            return Ok(());
        }
    }
    if pending(tx, &member.id)? {
        conflict(
            tx,
            &member.id,
            Some(&change.operation_id),
            "Remote change overlaps pending local member edits",
            &remote,
        )?;
        head(tx, member)?;
        return Ok(());
    }
    let before:Option<String>=tx.query_row("SELECT json_object('id',id,'name',name,'phone',phone,'email',email,'nfcId',nfc_id,'joinedOn',joined_on,'version',version,'archivedAt',archived_at,'archivedByUserId',archived_by_user_id) FROM members WHERE id=?1",[&member.id],|r|r.get(0)).optional().map_err(db_error)?;
    let local: Value = before
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(Value::Null);
    let collision: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM members WHERE nfc_id=?1 AND id<>?2)",
            params![member.nfc_id, member.id],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let existing_conflict: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM member_active_conflicts WHERE member_id=?1)",
            [&member.id],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if collision
        || existing_conflict
        || local["archivedAt"].is_string()
        || (!local.is_null() && local["joinedOn"] != member.joined_on)
    {
        conflict(
            tx,
            &member.id,
            Some(&change.operation_id),
            "Remote card, archive, or member history requires reconciliation",
            &remote,
        )?;
        head(tx, member)?;
        return Ok(());
    }
    let local_archive_actor = match archive_identity(tx, member) {
        Ok(actor) => actor,
        Err(reason) if reason == "Archive actor identity mapping conflicts" => {
            conflict(tx, &member.id, Some(&change.operation_id), &reason, &remote)?;
            head(tx, member)?;
            return Ok(());
        }
        Err(reason) => return Err(reason),
    };
    write_remote(tx, member, local_archive_actor)?;
    let device: String = tx
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    tx.execute("INSERT INTO audit(id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity) VALUES(?1,'authenticated member sync',?2,'pull member',?3,?4,?5,?6,'member')",params![id(),device,member.id,before,remote.to_string(),Utc::now().to_rfc3339()]).map_err(db_error)?;
    head(tx, member)
}

// Validate identity references without importing privileges or writing rows.
pub(super) fn archive_identity(conn: &Connection, member: &RemoteMember) -> Result<Option<String>> {
    let Some(actor) = &member.archived_by else {
        return Ok(None);
    };
    let mut stmt = conn
        .prepare("SELECT id,subject FROM users WHERE id=?1 OR subject=?1")
        .map_err(db_error)?;
    let identities = stmt
        .query_map([&actor.id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error)?;
    if identities.len() > 1
        || identities
            .first()
            .is_some_and(|(_, subject)| subject != &actor.id)
    {
        return Err("Archive actor identity mapping conflicts".into());
    }
    Ok(Some(
        identities
            .first()
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| actor.id.clone()),
    ))
}
pub(super) fn write_remote(
    tx: &Transaction<'_>,
    member: &RemoteMember,
    local_archive_actor: Option<String>,
) -> Result<()> {
    if let Some(actor) = &member.archived_by {
        let local_id = local_archive_actor
            .as_ref()
            .ok_or("Missing archive actor identity")?;
        tx.execute("INSERT INTO users(id,subject,email,display_name,active,version) SELECT ?1,?2,?3,?4,0,1 WHERE NOT EXISTS(SELECT 1 FROM users WHERE id=?1)", params![local_id,actor.id,format!("{}@remote.invalid",actor.id),actor.name]).map_err(db_error)?;
    }
    tx.execute("INSERT INTO members(id,name,phone,email,nfc_id,joined_on,version,archived_at,archived_by_user_id) VALUES(?1,?2,?3,?4,?5,?6,1,?7,?8) ON CONFLICT(id) DO UPDATE SET name=excluded.name,phone=excluded.phone,email=excluded.email,nfc_id=excluded.nfc_id,version=members.version+1,archived_at=excluded.archived_at,archived_by_user_id=excluded.archived_by_user_id",params![member.id,member.name,member.phone,member.email,member.nfc_id,member.joined_on,member.archived_at,local_archive_actor]).map_err(db_error)?;
    Ok(())
}
