use super::*;
use sha2::{Digest, Sha256};

pub(super) const COMMAND: &str = "business_initial_profile_recovery";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialProfileRecoveryInput {
    pub request_id: String,
    pub batch_id: String,
    pub fingerprint: String,
    pub confirmation: bool,
}

fn audit_only(request: &Value) -> bool {
    request["changes"].as_array().is_some_and(|changes| {
        !changes.is_empty()
            && changes
                .iter()
                .all(|change| change["table"] == "audit" && change["before"].is_null())
    })
}

// This deliberately excludes edits, financial data, other master records and
// computers that have already downloaded history. Their conflicts need review.
pub(super) fn eligible(
    conn: &Connection,
    session: Option<&removal::Session>,
    batch: &str,
    request: &Value,
    error: &str,
) -> Result<()> {
    let actor = removal::authorized(conn, session)?;
    let identity = member_worker::review_scope(conn, session)?;
    if !error.contains("business_revision_conflict") {
        return Err("Initial profile recovery applies only to a refused default profile.".into());
    }
    if request["protocolVersion"] != 2
        || request["operationId"] != batch
        || request["deviceId"] != identity["scope"]["deviceId"]
        || (!request["actorSubject"].is_null() && request["actorSubject"] != identity["subject"])
        || request["operationIds"] != json!([])
    {
        return Err("This transaction has saved operations or a different identity; separate review is required.".into());
    }
    let changes = request["changes"]
        .as_array()
        .ok_or("Invalid retained transaction")?;
    let profiles: Vec<_> = changes
        .iter()
        .filter(|c| c["table"] == "gym_settings")
        .collect();
    if profiles.len() != 1
        || profiles[0]["id"] != "1"
        || !profiles[0]["before"].is_null()
        || profiles[0]["after"] != business_sync::default_profile()
        || changes.len() < 2
        || changes.iter().any(|c| {
            !c["before"].is_null()
                || !matches!(
                    c["table"].as_str(),
                    Some("gym_settings" | "audit" | "users")
                )
        })
    {
        return Err("This transaction contains an edited profile or other business changes; separate review is required.".into());
    }
    // Native first sign-in journals an inactive identity reference alongside the
    // seed profile and audit. Only that same verified actor's unchanged insert
    // is eligible; this reference carries no activation or role authority.
    let users: Vec<_> = changes.iter().filter(|c| c["table"] == "users").collect();
    if users.len() > 1
        || users.iter().any(|c| {
            c["id"] != actor.user_id
                || c["after"]["id"] != actor.user_id
                || c["after"]["subject"] != identity["subject"]
                || c["after"]["active"] != 0
                || c["after"]["version"] != 1
        })
    {
        return Err("The retained identity is not the unchanged signed-in account; separate review is required.".into());
    }
    let cursor: i64 = conn
        .query_row("SELECT sequence FROM business_cursor WHERE id=1", [], |r| {
            r.get(0)
        })
        .map_err(db_error)?;
    if cursor != 0 {
        return Err(
            "This computer already has downloaded history; separate review is required.".into(),
        );
    }
    for table in business_sync::tables()? {
        if table.name == "gym_settings" {
            if business_sync::row_on(conn, &table, "1")? != Some(business_sync::default_profile()) {
                return Err(
                    "The local gym profile has been edited; no profile was replaced.".into(),
                );
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
                return Err("This computer has business records; export a backup for separate reconciliation.".into());
            }
        }
        for change in changes.iter().filter(|c| c["table"] == table.name) {
            let key = change["id"]
                .as_str()
                .ok_or("Invalid retained record identity")?;
            let local = business_sync::row_on(conn, &table, key)?
                .map(|row| business_sync::normalized(&table.name, row));
            if local != Some(change["after"].clone()) {
                return Err(
                    "The retained record differs from local history; separate review is required."
                        .into(),
                );
            }
        }
    }
    let unsupported: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM outbox WHERE entity<>'retry business transaction') OR EXISTS(SELECT 1 FROM local_operations WHERE command<>'business_retry') OR EXISTS(SELECT 1 FROM business_dirty WHERE table_name NOT IN ('audit','users'))",
        [], |r| r.get(0),
    ).map_err(db_error)?;
    if unsupported {
        return Err(
            "This computer has other saved changes; separate reconciliation is required.".into(),
        );
    }
    let queued = operations::rows(conn, &format!("SELECT json_object('id',id,'state',state,'request',json(request_json)) FROM business_batches WHERE state<>'confirmed' AND {} ORDER BY ordinal", business_sync::ACTIVE_BATCH))?;
    if queued
        .first()
        .is_none_or(|q| q["id"] != batch || q["state"] != "conflict")
    {
        return Err("The blocked transaction changed. Review it again.".into());
    }
    if queued.iter().skip(1).any(|q| {
        q["state"] != "pending"
            || !audit_only(&q["request"])
            || q["request"]["deviceId"] != identity["scope"]["deviceId"]
            || (!q["request"]["actorSubject"].is_null()
                && q["request"]["actorSubject"] != identity["subject"])
    }) {
        return Err(
            "Later transactions contain other business changes; separate review is required."
                .into(),
        );
    }
    // Enrollment can journal only a local activation/version change. Capture
    // normalizes those fields; any actual identity edit remains a refusal.
    for change in operations::rows(conn, "SELECT json_object('before',json(before_json),'after',json(after_json)) FROM business_dirty WHERE table_name='users'")? {
        let mut before = change["before"].clone();
        let mut after = change["after"].clone();
        if !before.is_object() || !after.is_object() {
            return Err("A new identity is pending; separate review is required.".into());
        }
        before["active"] = json!(0); before["version"] = json!(1);
        after["active"] = json!(0); after["version"] = json!(1);
        if before != after {
            return Err("An identity change is pending; separate review is required.".into());
        }
    }
    Ok(())
}

pub(super) fn replacement(request: &Value, subject: &str) -> Result<Value> {
    let source = request["operationId"]
        .as_str()
        .ok_or("Invalid retained transaction")?;
    // The original random operation UUID gives this derived ID stable identity
    // across lost replies, cancellations, restarts and a fresh recovery request.
    let digest = Sha256::digest(format!("armstrong-initial-profile-recovery:{source}"));
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let changes: Vec<_> = request["changes"]
        .as_array()
        .ok_or("Invalid retained transaction")?
        .iter()
        .filter(|c| c["table"] == "audit" || c["table"] == "users")
        .cloned()
        .collect();
    Ok(
        json!({"protocolVersion":2,"operationId":Uuid::from_bytes(bytes).to_string(),"deviceId":request["deviceId"],"actorSubject":subject,"operationIds":[],"changes":changes}),
    )
}

pub(super) fn bound(
    conn: &Connection,
    session: Option<&removal::Session>,
    input: &InitialProfileRecoveryInput,
) -> Result<Value> {
    if !input.confirmation {
        return Err("Confirm recovery of the unchanged installation profile first.".into());
    }
    removal::authorized(conn, session)?;
    Ok(json!({"input":input,"identity":member_worker::review_scope(conn, session)?}))
}
