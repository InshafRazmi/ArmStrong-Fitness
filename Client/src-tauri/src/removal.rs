use super::operations::{receipt, replay, rows};
use super::*;
use rusqlite::Transaction;

// This is a native-only session seam for verified enrollment/login integration.
// Store::open and restore never grant it. No IPC accepts a user ID or grants a session.
#[derive(Clone)]
pub(super) struct Session {
    pub(super) user_id: String,
    pub(super) expires_at: DateTime<Utc>,
    pub(super) can_write: bool,
    pub(super) native_nonce: Option<String>,
}
pub(super) fn current_session(conn: &Connection, session: &Session) -> Result<()> {
    if let Some(expected) = &session.native_nonce {
        let actual: Option<String> = conn
            .query_row(
                "SELECT value FROM metadata WHERE key='native_session_nonce'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if actual.as_ref() != Some(expected) {
            return Err("Native account session changed; sign in again".into());
        }
    }
    Ok(())
}
pub(super) struct Actor {
    pub(super) user_id: String,
    pub(super) label: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberRemovalInput {
    pub request_id: String,
    pub member_id: String,
    pub version: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffRemovalInput {
    pub request_id: String,
    pub staff_id: String,
    pub version: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpenseVoidInput {
    pub request_id: String,
    pub expense_id: String,
    pub reason: String,
}
pub(super) fn authorized(conn: &Connection, session: Option<&Session>) -> Result<Actor> {
    let session = session.ok_or("Removal requires an authenticated Administrator session")?;
    current_session(conn, session)?;
    if session.expires_at <= Utc::now() {
        return Err("Staff session expired; authenticate again".into());
    }
    if !session.can_write {
        return Err("This enrolled device has read-only access".into());
    }
    let saved: Option<String> = conn.query_row(
        "SELECT u.display_name FROM users u WHERE u.id=?1 AND u.active=1 AND EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=u.id AND r.name='Administrator' COLLATE NOCASE)",
        [&session.user_id], |r|r.get(0)).optional().map_err(db_error)?;
    let name = saved.ok_or("An active Administrator account is required for removal")?;
    Ok(Actor {
        user_id: session.user_id.clone(),
        label: format!("{name} ({})", session.user_id),
    })
}
pub(super) fn authorization(conn: &Connection, session: Option<&Session>) -> Value {
    match authorized(conn, session) {
        Ok(actor) => json!({"allowed":true,"userId":actor.user_id,"user":actor.label,"reason":""}),
        Err(reason) => json!({"allowed":false,"userId":null,"user":null,"reason":reason}),
    }
}
pub(super) fn member_record(conn: &Connection, member: &str) -> Result<Value> {
    let saved: Option<String> = conn.query_row(
        "SELECT json_object('id',id,'name',name,'phone',phone,'email',email,'nfcId',nfc_id,'joinedOn',joined_on,'version',version,'archivedAt',archived_at,'archivedByUserId',archived_by_user_id) FROM members WHERE id=?1",
        [member],|r|r.get(0)).optional().map_err(db_error)?;
    serde_json::from_str(&saved.ok_or("Member no longer exists")?).map_err(|e| e.to_string())
}
pub(super) fn record_action(
    tx: &Transaction<'_>,
    actor: &Actor,
    action: &str,
    target: (&str, &str),
    version: Option<i64>,
    before: &Value,
    after: &Value,
) -> Result<()> {
    let (entity, entity_id) = target;
    let device: String = tx
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let now = Utc::now().to_rfc3339();
    tx.execute("INSERT INTO audit(id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity,actor_user_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![id(),actor.label,device,format!("{action} {entity}"),entity_id,before.to_string(),after.to_string(),now,entity,actor.user_id]).map_err(db_error)?;
    let payload =
        json!({"before":before,"after":after,"actorUserId":actor.user_id,"actor":actor.label});
    tx.execute("INSERT INTO outbox(id,device_id,entity,entity_id,action,expected_version,payload_json,schema_version,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![id(),device,entity,entity_id,action,version,payload.to_string(),SCHEMA_VERSION,now]).map_err(db_error)?;
    business_sync::capture(tx)?;
    Ok(())
}
pub(super) fn append_snapshot(conn: &Connection, value: &mut Value) -> Result<()> {
    let members=rows(conn,"SELECT json_object('id',id,'name',name,'phone',phone,'email',email,'nfcId',COALESCE(nfc_id,''),'joinedOn',joined_on,'version',version,'active',json(CASE WHEN archived_at IS NULL THEN 'true' ELSE 'false' END),'archivedAt',archived_at,'archivedByUserId',archived_by_user_id) FROM members WHERE NOT EXISTS(SELECT 1 FROM member_deletions d WHERE d.id=members.id) ORDER BY name,id")?;
    let mut members = members;
    for member in &mut members {
        member["canDelete"] = json!(true);
    }
    value["members"] = json!(members);
    value["expenses"]=json!(rows(conn,"SELECT json_object('id',id,'title',title,'category',category,'amountMinor',amount_minor,'effectiveAmountMinor',effective_amount_minor,'status',status,'method',method,'businessOn',business_on,'createdAt',created_at,'actor',actor,'reversesId',reverses_id,'voidReason',void_reason,'voidedAt',voided_at,'voidedBy',voided_by,'voidedByUserId',voided_by_user_id) FROM expense_history ORDER BY created_at DESC,id DESC")?);
    Ok(())
}
impl Store {
    pub fn delete_staff(&mut self, input: StaffRemovalInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let actor = authorized(&tx, self.removal_session.as_ref())?;
        let bound = json!({"input":input,"actorUserId":actor.user_id});
        if let Some(result) = replay(&tx, "delete_staff", &input.request_id, &bound)? {
            return Ok(result);
        }
        let before: Option<String> = tx.query_row(
            "SELECT json_object('id',id,'name',name,'active',active,'version',version) FROM trainers WHERE id=?1",
            [&input.staff_id], |r| r.get(0)).optional().map_err(db_error)?;
        let before: Value = serde_json::from_str(&before.ok_or("Staff profile no longer exists")?)
            .map_err(|e| e.to_string())?;
        if before["version"].as_i64() != Some(input.version) {
            return Err("Staff record changed. Refresh before deleting.".into());
        }
        let deleted: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM staff_deletions WHERE id=?1)",
                [&input.staff_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if deleted {
            return Err("Staff profile was already deleted".into());
        }
        // Retain the identity referenced by salary, invoices and attendance.
        // Only current operational assignments and card access are removed.
        tx.execute(
            "UPDATE trainers SET active=0,version=version+1 WHERE id=?1 AND version=?2",
            params![input.staff_id, input.version],
        )
        .map_err(db_error)?;
        attendance_profiles::assign_staff_card(&tx, &input.staff_id, Some(""))?;
        tx.execute(
            "UPDATE member_trainers SET trainer_id=NULL,version=version+1 WHERE trainer_id=?1",
            [&input.staff_id],
        )
        .map_err(db_error)?;
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO staff_deletions VALUES(?1,?2,?3)",
            params![input.staff_id, now, actor.user_id],
        )
        .map_err(db_error)?;
        let after = json!({"id":input.staff_id,"name":before["name"],"active":0,"version":input.version+1,"deletedAt":now});
        record_action(
            &tx,
            &actor,
            "delete",
            ("staff", &input.staff_id),
            Some(input.version),
            &before,
            &after,
        )?;
        let result = json!({"id":input.staff_id});
        receipt(&tx, "delete_staff", &input.request_id, &bound, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn archive_member(&mut self, input: MemberRemovalInput) -> Result<Value> {
        self.remove_member(input, false)
    }
    pub fn delete_member(&mut self, input: MemberRemovalInput) -> Result<Value> {
        self.remove_member(input, true)
    }
    fn remove_member(&mut self, input: MemberRemovalInput, delete: bool) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let actor = authorized(&tx, self.removal_session.as_ref())?;
        let command = if delete {
            "delete_member"
        } else {
            "archive_member"
        };
        let bound = json!({"input":input,"actorUserId":actor.user_id});
        if let Some(result) = replay(&tx, command, &input.request_id, &bound)? {
            return Ok(result);
        }
        let deleted: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM member_deletions WHERE id=?1)",
                [&input.member_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if deleted {
            return Err("Member was already permanently removed".into());
        }
        let before = member_record(&tx, &input.member_id)?;
        if before["version"].as_i64() != Some(input.version) {
            return Err("Member changed. Refresh before removal".into());
        }
        let after = if delete {
            let now = Utc::now().to_rfc3339();
            tx.execute("UPDATE members SET archived_at=?1,archived_by_user_id=?2,version=version+1 WHERE id=?3 AND archived_at IS NULL",params![now,actor.user_id,input.member_id]).map_err(db_error)?;
            tx.execute(
                "INSERT INTO member_deletions VALUES(?1,?2,?3)",
                params![input.member_id, now, actor.user_id],
            )
            .map_err(db_error)?;
            json!({"id":input.member_id,"deleted":true,"deletedAt":now,"deletedByUserId":actor.user_id})
        } else {
            if before["archivedAt"].is_string() {
                return Err("Member is already archived".into());
            }
            tx.execute("UPDATE members SET archived_at=?1,archived_by_user_id=?2,version=version+1 WHERE id=?3 AND version=?4",params![Utc::now().to_rfc3339(),actor.user_id,input.member_id,input.version]).map_err(db_error)?;
            member_record(&tx, &input.member_id)?
        };
        record_action(
            &tx,
            &actor,
            if delete { "delete" } else { "archive" },
            ("member", &input.member_id),
            Some(input.version),
            &before,
            &after,
        )?;
        let result = json!({"id":input.member_id});
        receipt(&tx, command, &input.request_id, &bound, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn void_expense(&mut self, input: ExpenseVoidInput) -> Result<Value> {
        let reason = required(&input.reason, "Void reason", 254)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let actor = authorized(&tx, self.removal_session.as_ref())?;
        let bound = json!({"input":input,"actorUserId":actor.user_id});
        if let Some(result) = replay(&tx, "void_expense", &input.request_id, &bound)? {
            return Ok(result);
        }
        let saved:Option<String>=tx.query_row("SELECT json_object('id',id,'title',title,'category',category,'amountMinor',amount_minor,'method',method,'businessOn',business_on,'createdAt',created_at,'actor',actor,'reversesId',reverses_id,'status',status) FROM expense_history WHERE id=?1",[&input.expense_id],|r|r.get(0)).optional().map_err(db_error)?;
        let before: Value = serde_json::from_str(&saved.ok_or("Expense no longer exists")?)
            .map_err(|e| e.to_string())?;
        if before["status"] != "Recorded" {
            return Err("Expense is already voided or is a legacy reversal".into());
        }
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO expense_voids VALUES(?1,?2,?3,?4,?5,NULL)",
            params![input.expense_id, reason, now, actor.label, actor.user_id],
        )
        .map_err(db_error)?;
        let after = json!({"id":input.expense_id,"status":"Voided","reason":reason,"voidedAt":now,"voidedBy":actor.label,"voidedByUserId":actor.user_id,"effectiveAmountMinor":0});
        record_action(
            &tx,
            &actor,
            "void",
            ("expense", &input.expense_id),
            None,
            &before,
            &after,
        )?;
        let result = json!({"id":input.expense_id});
        receipt(&tx, "void_expense", &input.request_id, &bound, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}
