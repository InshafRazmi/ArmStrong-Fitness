use super::*;
use chrono::{Datelike, Months};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterMemberInput {
    pub request_id: String,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub nfc_id: String,
    pub plan_id: Option<String>,
    pub plan_version: Option<i64>,
    pub starts_on: Option<String>,
}

// End dates are inclusive. A missing anniversary uses the month's last day.
pub(super) fn package_end(starts_on: &str, duration_months: i64) -> Result<String> {
    if !(1..=60).contains(&duration_months) {
        return Err("Package duration must be between 1 and 60 months".into());
    }
    let start = date(starts_on)?;
    let anniversary = start
        .checked_add_months(Months::new(duration_months as u32))
        .ok_or("Membership dates exceed the supported range")?;
    let end = if anniversary.day() == start.day() {
        anniversary
            .pred_opt()
            .ok_or("Invalid membership end date")?
    } else {
        anniversary
    };
    let result = end.format("%Y-%m-%d").to_string();
    date(&result)?;
    Ok(result)
}

impl Store {
    pub fn register_member(&mut self, input: RegisterMemberInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(result) = operations::replay(&tx, "register_member", &input.request_id, &input)?
        {
            return Ok(result);
        }
        let result = register_member_on(&tx, &input)?;
        // One frozen sync transaction includes member, NFC history, period and audit.
        business_sync::capture(&tx)?;
        operations::receipt(&tx, "register_member", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}

pub(super) fn register_member_on(
    tx: &rusqlite::Transaction<'_>,
    input: &RegisterMemberInput,
) -> Result<Value> {
    // Capture the reviewed package version, duration, price and name together.
    let package = if let Some(plan_id) = &input.plan_id {
        let version = input
            .plan_version
            .ok_or("Select a membership package again")?;
        let starts_on = input
            .starts_on
            .as_deref()
            .ok_or("Select a membership start date")?;
        let plan: Option<(String, i64, i64)> = tx.query_row(
                "SELECT name,price_minor,duration_months FROM plans WHERE id=?1 AND version=?2 AND active=1",
                params![plan_id, version], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).optional().map_err(db_error)?;
        let (name, price, months) =
            plan.ok_or("Membership package changed or is inactive. Refresh and select it again.")?;
        let ends_on = package_end(starts_on, months)?;
        Some((plan_id, name, price, starts_on, ends_on))
    } else {
        if input.plan_version.is_some() || input.starts_on.is_some() {
            return Err("Select a package before supplying membership dates".into());
        }
        None
    };
    let member_id = save_member_on(
        tx,
        MemberInput {
            id: None,
            version: None,
            name: input.name.clone(),
            phone: input.phone.clone(),
            email: input.email.clone(),
            nfc_id: input.nfc_id.clone(),
        },
    )?;
    let mut period_id = None;
    if let Some((plan_id, name, price, starts_on, ends_on)) = package {
        let period = id();
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO membership_periods VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![period, member_id, plan_id, name, price, starts_on, ends_on, now],
        )
        .map_err(db_error)?;
        record_uncaptured(
            tx,
            "membership_period",
            &period,
            None,
            None,
            json!({"id":period,"memberId":member_id,"planId":plan_id,"planName":name,"priceMinor":price,"startsOn":starts_on,"endsOn":ends_on,"createdAt":now}),
        )?;
        period_id = Some(period);
    }
    Ok(json!({"id":member_id,"membershipPeriodId":period_id}))
}

#[cfg(test)]
#[path = "registration_tests.rs"]
mod tests;
