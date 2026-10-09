use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BusinessRetryInput {
    pub request_id: String,
    pub batch_id: String,
    pub fingerprint: String,
}

pub(super) fn preview_on(
    conn: &Connection,
    session: Option<&removal::Session>,
    batch: &str,
) -> Result<Value> {
    removal::authorized(conn, session)?;
    let (encoded, error): (String, String) = conn.query_row(
        &format!("SELECT request_json,COALESCE(last_error,'') FROM business_batches WHERE id=?1 AND state='conflict' AND {} AND ordinal=(SELECT MIN(ordinal) FROM business_batches WHERE state<>'confirmed' AND {})", business_sync::ACTIVE_BATCH, business_sync::ACTIVE_BATCH),
        [batch], |r| Ok((r.get(0)?,r.get(1)?)),
    ).optional().map_err(db_error)?.ok_or("This transaction is no longer the first blocked transaction. Refresh synchronization.")?;
    let request: Value =
        serde_json::from_str(&encoded).map_err(|_| "Invalid retained transaction")?;
    let subject = member_worker::subject_on(conn, session)?;
    if !request["actorSubject"].is_null() && request["actorSubject"] != subject {
        return Err("Sign in as the original transaction's Administrator to retry it".into());
    }
    let fingerprint = business_sync::hash(&json!({"request":request,"reason":error}))?;
    let changes = request["changes"].as_array().ok_or("Invalid retained transaction")?.iter().map(|c| {
        json!({"table":c["table"],"id":c["id"],"action":if c["before"].is_null(){"Add"}else{"Update"},"name":c["after"]["name"].as_str().or(c["after"]["member_name"].as_str()).or(c["after"]["staff_name"].as_str())})
    }).collect::<Vec<_>>();
    let recovery = match super::initial_profile_recovery::eligible(
        conn, session, batch, &request, &error,
    ) {
        Ok(()) => {
            json!({"allowed":true,"reason":"This transaction contains the unchanged installation profile and audit entries. Recover using the server profile while retaining the original transaction and audit history."})
        }
        Err(reason) => json!({"allowed":false,"reason":reason}),
    };
    Ok(
        json!({"batchId":batch,"fingerprint":fingerprint,"reason":error,"changes":changes,"initialProfileRecovery":recovery}),
    )
}

impl Store {
    pub fn preview_business_retry(&self, batch: &str) -> Result<Value> {
        preview_on(&self.conn, self.removal_session.as_ref(), batch)
    }
    pub fn retry_business_transaction(&mut self, input: BusinessRetryInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        removal::authorized(&tx, self.removal_session.as_ref())?;
        if let Some(saved) = operations::replay(&tx, "business_retry", &input.request_id, &input)? {
            return Ok(saved);
        }
        let preview = preview_on(&tx, self.removal_session.as_ref(), &input.batch_id)?;
        if preview["fingerprint"] != input.fingerprint {
            return Err("The blocked transaction changed. Review it again before retrying.".into());
        }
        // Re-send exactly the original operation and payload. No money, member
        // row, operation mapping or server receipt is rewritten or acknowledged.
        tx.execute("UPDATE business_batches SET state='pending',last_error=NULL WHERE id=?1 AND state='conflict'",[&input.batch_id]).map_err(db_error)?;
        tx.execute("DELETE FROM metadata WHERE key IN ('business_last_error','business_retry_on','business_failures')",[]).map_err(db_error)?;
        record(
            &tx,
            "retry business transaction",
            &input.batch_id,
            None,
            Some(preview.to_string()),
            json!({"batchId":input.batch_id,"fingerprint":input.fingerprint}),
        )?;
        let result = json!({"batchId":input.batch_id,"state":"pending"});
        operations::receipt(&tx, "business_retry", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}
