use super::operations::{choice, receipt, replay, rows};
use super::*;
#[cfg(test)]
#[path = "attendance_profiles_tests.rs"]
mod tests;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffAttendanceInput {
    pub request_id: String,
    pub staff_or_card: String,
    pub source: String,
}

pub(super) fn save_gender(
    tx: &rusqlite::Transaction<'_>,
    member: &str,
    gender: Option<&str>,
    expected: Option<i64>,
) -> Result<()> {
    // Older desktop commands do not alter a member's recorded gender.
    let Some(gender) = gender else {
        return Ok(());
    };
    choice(gender, &["Male", "Female"], "gender")?;
    let current: Option<(String, i64)> = tx
        .query_row(
            "SELECT gender,version FROM member_profiles WHERE id=?1",
            [member],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(db_error)?;
    if current.as_ref().map(|r| r.1) != expected {
        return Err("Member gender changed. Refresh before saving.".into());
    }
    if current.as_ref().is_some_and(|r| r.0 == gender) {
        return Ok(());
    }
    tx.execute("INSERT INTO member_profiles VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET gender=excluded.gender,version=excluded.version",
        params![member,gender,expected.unwrap_or(0)+1]).map_err(db_error)?;
    record_uncaptured(
        tx,
        "member_gender",
        member,
        expected,
        current.map(|r| json!({"gender":r.0,"version":r.1}).to_string()),
        json!({"gender":gender,"version":expected.unwrap_or(0)+1}),
    )
}

pub(super) fn assign_staff_card(
    tx: &rusqlite::Transaction<'_>,
    staff: &str,
    card: Option<&str>,
) -> Result<()> {
    let Some(card) = card else {
        return Ok(());
    };
    let uid = card.trim().to_ascii_uppercase();
    if uid.len() > 128
        || !uid.is_ascii()
        || uid.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("Card ID must be at most 128 ASCII characters without spaces".into());
    }
    let current: Option<String> = tx
        .query_row(
            "SELECT uid FROM staff_nfc_cards WHERE trainer_id=?1 AND revoked_at IS NULL",
            [staff],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    if current.as_deref().unwrap_or("") == uid {
        return Ok(());
    }
    let duplicate: Option<(String, String)> = tx.query_row("SELECT 'member',m.name FROM nfc_cards c JOIN members m ON m.id=c.member_id WHERE c.uid=?1 AND c.revoked_at IS NULL UNION ALL SELECT 'staff',t.name FROM staff_nfc_cards c JOIN trainers t ON t.id=c.trainer_id WHERE c.uid=?1 AND c.revoked_at IS NULL AND c.trainer_id<>?2 LIMIT 1",params![uid,staff],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_error)?;
    if let Some((kind, owner)) = duplicate.filter(|_| !uid.is_empty()) {
        return Err(format!("This NFC card is already assigned to {kind}: {owner}. Edit that profile and clear its NFC card before reusing it."));
    }
    let now = Utc::now().to_rfc3339();
    tx.execute(
        "UPDATE staff_nfc_cards SET revoked_at=?1 WHERE trainer_id=?2 AND revoked_at IS NULL",
        params![now, staff],
    )
    .map_err(db_error)?;
    if !uid.is_empty() {
        tx.execute(
            "INSERT INTO staff_nfc_cards VALUES(?1,?2,?3,?4,NULL)",
            params![id(), staff, uid, now],
        )
        .map_err(db_error)?;
    }
    Ok(())
}

pub(super) fn append_snapshot(conn: &Connection, result: &mut Value) -> Result<()> {
    for member in result["members"]
        .as_array_mut()
        .ok_or("Missing member snapshot")?
    {
        let profile: Option<(String, i64)> = conn
            .query_row(
                "SELECT gender,version FROM member_profiles WHERE id=?1",
                [member["id"].as_str().ok_or("Invalid member")?],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        member["gender"] = profile.as_ref().map(|r| json!(r.0)).unwrap_or(Value::Null);
        member["genderVersion"] = profile.map(|r| json!(r.1)).unwrap_or(Value::Null);
    }
    for event in result["attendance"]
        .as_array_mut()
        .ok_or("Missing attendance snapshot")?
    {
        let gender: Option<String> = conn
            .query_row(
                "SELECT gender FROM member_profiles WHERE id=?1",
                [event["memberId"]
                    .as_str()
                    .ok_or("Invalid attendance member")?],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        event["gender"] = json!(gender);
    }
    for staff in result["trainers"]
        .as_array_mut()
        .ok_or("Missing staff snapshot")?
    {
        let card: Option<String> = conn
            .query_row(
                "SELECT uid FROM staff_nfc_cards WHERE trainer_id=?1 AND revoked_at IS NULL",
                [staff["id"].as_str().ok_or("Invalid staff")?],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        staff["nfcId"] = json!(card.unwrap_or_default());
        let deleted_at: Option<String> = conn
            .query_row(
                "SELECT deleted_at FROM staff_deletions WHERE id=?1",
                [staff["id"].as_str().ok_or("Invalid staff")?],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        staff["deletedAt"] = json!(deleted_at);
    }
    result["staffAttendance"] = json!(rows(conn,"SELECT json_object('id',id,'staffId',trainer_id,'name',staff_name,'cardId',card_id,'cardUid',card_uid,'type',kind,'source',source,'businessOn',business_on,'occurredAt',occurred_at) FROM staff_attendance ORDER BY occurred_at DESC,rowid DESC")?);
    Ok(())
}

impl Store {
    pub fn record_staff_attendance(&mut self, input: StaffAttendanceInput) -> Result<Value> {
        self.staff_attendance_at(input, Utc::now())
    }
    pub fn record_nfc_attendance(&mut self, input: AttendanceInput) -> Result<Value> {
        if input.source != "NFC" {
            return Err("Use NFC for automatic card attendance".into());
        }
        // Route a retry to its original command even after card reassignment.
        // The command's durable receipt still verifies the complete input.
        let previous: Option<String> = self
            .conn
            .query_row(
                "SELECT command FROM local_operations WHERE request_id=?1",
                [&input.request_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let staff: bool = match previous.as_deref() {
            Some("staff_attendance") => true,
            Some("attendance") => false,
            Some(_) => return Err("Operation ID was reused with a different request".into()),
            None => self.conn.query_row("SELECT EXISTS(SELECT 1 FROM staff_nfc_cards WHERE uid=?1 AND revoked_at IS NULL)",[input.member_or_card.trim().to_ascii_uppercase()],|r|r.get(0)).map_err(db_error)?,
        };
        if staff {
            self.record_staff_attendance(StaffAttendanceInput {
                request_id: input.request_id,
                staff_or_card: input.member_or_card,
                source: input.source,
            })
        } else {
            let mut result = self.record_attendance(input)?;
            result["entity"] = json!("Member");
            Ok(result)
        }
    }
    pub(super) fn staff_attendance_at(
        &mut self,
        input: StaffAttendanceInput,
        now: DateTime<Utc>,
    ) -> Result<Value> {
        choice(&input.source, &["NFC", "Manual"], "attendance source")?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(saved) = replay(&tx, "staff_attendance", &input.request_id, &input)? {
            return Ok(saved);
        }
        let staff: Option<(String, String, Option<String>, String)> = if input.source == "NFC" {
            tx.query_row("SELECT t.id,t.name,c.id,c.uid FROM trainers t JOIN staff_nfc_cards c ON c.trainer_id=t.id AND c.revoked_at IS NULL WHERE c.uid=?1 AND t.active=1",[input.staff_or_card.trim().to_ascii_uppercase()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db_error)?
        } else {
            tx.query_row(
                "SELECT id,name,NULL,'' FROM trainers WHERE id=?1 AND active=1",
                [&input.staff_or_card],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()
            .map_err(db_error)?
        };
        let (staff_id, name, card_id, card_uid) =
            staff.ok_or("Card/staff is not linked to active staff")?;
        let day = business_date(now);
        let last: Option<(String,String,String,String)> = tx.query_row("SELECT id,kind,source,occurred_at FROM staff_attendance WHERE trainer_id=?1 AND business_on=?2 ORDER BY occurred_at DESC,rowid DESC LIMIT 1",params![staff_id,day],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db_error)?;
        if let Some((last_id, _, source, instant)) = &last {
            let previous = DateTime::parse_from_rfc3339(instant)
                .map_err(|_| "Invalid staff attendance time")?;
            if input.source == "NFC"
                && source == "NFC"
                && (0..2000).contains(&(now.timestamp_millis() - previous.timestamp_millis()))
            {
                let result = json!({"id":last_id,"duplicate":true,"entity":"Staff"});
                receipt(&tx, "staff_attendance", &input.request_id, &input, &result)?;
                tx.commit().map_err(db_error)?;
                return Ok(result);
            }
        }
        let kind = if last.as_ref().is_some_and(|r| r.1 == "Check-in") {
            "Check-out"
        } else {
            "Check-in"
        };
        let event = id();
        tx.execute(
            "INSERT INTO staff_attendance VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                event,
                staff_id,
                card_id,
                name,
                card_uid,
                kind,
                input.source,
                day,
                now.to_rfc3339()
            ],
        )
        .map_err(db_error)?;
        record(
            &tx,
            "staff_attendance",
            &event,
            None,
            None,
            json!({"id":event,"staffId":staff_id,"name":name,"type":kind,"source":input.source,"businessOn":day,"occurredAt":now.to_rfc3339()}),
        )?;
        let result = json!({"id":event,"duplicate":false,"entity":"Staff","type":kind,"name":name});
        receipt(&tx, "staff_attendance", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}
