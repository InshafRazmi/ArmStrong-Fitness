use super::operations::{choice, money, receipt, replay, rows};
use super::*;
use rusqlite::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerInput {
    pub request_id: String,
    pub id: Option<String>,
    pub version: Option<i64>,
    pub name: String,
    pub phone: String,
    pub nic: String,
    pub salary_minor: i64,
    pub training_fee_minor: i64,
    pub active: bool,
    pub nfc_id: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffRegisterInput {
    pub member: RegisterMemberInput,
    pub trainer_id: Option<String>,
    pub trainer_version: Option<i64>,
    pub gender: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffMemberInput {
    pub member: MemberInput,
    pub trainer_id: Option<String>,
    pub trainer_version: Option<i64>,
    pub assignment_version: Option<i64>,
    pub gender: Option<String>,
    pub gender_version: Option<i64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingChargeInput {
    pub request_id: String,
    pub member_id: String,
    pub trainer_id: String,
    pub trainer_version: i64,
    pub starts_on: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombinedPaymentInput {
    pub payment: ReceivePaymentInput,
    pub invoice_ids: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffPayoutInput {
    pub request_id: String,
    pub trainer_id: String,
    pub trainer_version: i64,
    pub salary_month: String,
    pub include_salary: bool,
    pub expected_salary_minor: i64,
    pub expected_training_minor: i64,
    pub allocation_ids: Vec<String>,
    pub method: String,
}

fn trainer(conn: &Connection, id: &str, version: i64, active: bool) -> Result<(String, i64, i64)> {
    conn.query_row(
        "SELECT name,salary_minor,training_fee_minor FROM trainers WHERE id=?1 AND version=?2 AND (?3=0 OR active=1)",
        params![id,version,active], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
    ).optional().map_err(db_error)?.ok_or("Staff details or rates changed. Refresh and select the staff again.".into())
}
fn assignment(
    tx: &Transaction<'_>,
    member: &str,
    trainer_id: Option<&str>,
    trainer_version: Option<i64>,
    expected: Option<i64>,
) -> Result<()> {
    let current: Option<(Option<String>, i64)> = tx
        .query_row(
            "SELECT trainer_id,version FROM member_trainers WHERE id=?1",
            [member],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(db_error)?;
    if let Some(selected) = trainer_id {
        trainer(
            tx,
            selected,
            trainer_version.ok_or("Select the staff again")?,
            current.as_ref().and_then(|v| v.0.as_deref()) != trainer_id,
        )?;
    } else if trainer_version.is_some() {
        return Err("Select a staff before supplying a rate version".into());
    }
    if current.as_ref().map(|v| v.1) != expected {
        return Err("This member's trainer assignment changed. Refresh before saving.".into());
    }
    if current.as_ref().and_then(|v| v.0.as_deref()) == trainer_id {
        return Ok(());
    }
    let before = current
        .as_ref()
        .map(|v| json!({"trainerId":v.0,"version":v.1}).to_string());
    let version = expected.unwrap_or(0) + 1;
    tx.execute("INSERT INTO member_trainers VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET trainer_id=excluded.trainer_id,version=excluded.version",params![member,trainer_id,version]).map_err(db_error)?;
    record_uncaptured(
        tx,
        "member_trainer",
        member,
        expected,
        before,
        json!({"id":member,"trainerId":trainer_id,"version":version}),
    )
}
fn charge(tx: &Transaction<'_>, input: &TrainingChargeInput) -> Result<Value> {
    let (name, _, fee) = trainer(tx, &input.trainer_id, input.trainer_version, true)?;
    money(fee, true).map_err(|_| "This staff has no paid training fee configured".to_string())?;
    let assigned: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM member_trainers a JOIN members m ON m.id=a.id WHERE a.id=?1 AND a.trainer_id=?2 AND m.archived_at IS NULL)",params![input.member_id,input.trainer_id],|r|r.get(0)).map_err(db_error)?;
    if !assigned {
        return Err("Select the active member's assigned staff before billing".into());
    }
    let ends = registration::package_end(&input.starts_on, 1)?;
    let invoice = finance::insert_invoice(
        tx,
        &InvoiceInput {
            request_id: input.request_id.clone(),
            member_id: input.member_id.clone(),
            membership_period_id: None,
            description: format!("Personal training: {name} · {} – {ends}", input.starts_on),
            amount_minor: fee,
        },
    )?;
    let charge_id = id();
    tx.execute(
        "INSERT INTO training_charges VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            charge_id,
            invoice["id"].as_str(),
            input.member_id,
            input.trainer_id,
            name,
            fee,
            input.starts_on,
            ends
        ],
    )
    .map_err(db_error)?;
    record_uncaptured(
        tx,
        "personal_training",
        &charge_id,
        None,
        None,
        json!({"id":charge_id,"invoiceId":invoice["id"],"memberId":input.member_id,"trainerId":input.trainer_id,"trainerName":name,"feeMinor":fee,"startsOn":input.starts_on,"endsOn":ends}),
    )?;
    Ok(json!({"id":charge_id,"invoiceId":invoice["id"]}))
}
impl Store {
    pub fn save_trainer(&mut self, input: TrainerInput) -> Result<Value> {
        let name = required(&input.name, "Staff name", 120)?;
        let phone = required(&input.phone, "Mobile number", 40)?;
        let digits = phone.chars().filter(char::is_ascii_digit).count();
        if !(7..=15).contains(&digits)
            || phone
                .chars()
                .any(|c| !c.is_ascii_digit() && !"+ -()".contains(c))
        {
            return Err("Enter a valid mobile number".into());
        }
        let nic = required(&input.nic, "NIC number", 24)?.to_ascii_uppercase();
        if !nic.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("NIC number must contain letters and digits without spaces".into());
        }
        money(input.salary_minor, false)?;
        money(input.training_fee_minor, false)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "staff", &input.request_id, &input)? {
            return Ok(previous);
        }
        let staff_id = input.id.clone().unwrap_or_else(id);
        let duplicate: Option<(String, bool)> = tx
            .query_row(
                "SELECT name,active FROM trainers t WHERE nic=?1 AND id<>?2 AND NOT EXISTS(SELECT 1 FROM staff_deletions d WHERE d.id=t.id) LIMIT 1",
                params![nic, staff_id],
                |r| Ok((r.get(0)?,r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        if let Some((owner, active)) = duplicate {
            return Err(format!("A staff member with this NIC number already exists: {owner} ({}). Edit the existing profile, or permanently delete it before creating a new one.", if active { "Active" } else { "Inactive" }));
        }
        if input.id.is_some() {
            let changed=tx.execute("UPDATE trainers SET name=?1,phone=?2,nic=?3,salary_minor=?4,training_fee_minor=?5,active=?6,version=version+1 WHERE id=?7 AND version=?8",params![name,phone,nic,input.salary_minor,input.training_fee_minor,input.active,staff_id,input.version]).map_err(db_error)?;
            if changed != 1 {
                return Err("Staff record changed. Refresh before saving.".into());
            }
        } else {
            if input.version.is_some() {
                return Err("New staff cannot have a saved version".into());
            }
            tx.execute(
                "INSERT INTO trainers VALUES(?1,?2,?3,?4,?5,?6,?7,1)",
                params![
                    staff_id,
                    name,
                    phone,
                    nic,
                    input.salary_minor,
                    input.training_fee_minor,
                    input.active
                ],
            )
            .map_err(db_error)?;
        }
        attendance_profiles::assign_staff_card(&tx, &staff_id, input.nfc_id.as_deref())?;
        record(
            &tx,
            "staff",
            &staff_id,
            input.version,
            None,
            json!({"id":staff_id,"name":name,"phone":phone,"nic":nic,"salaryMinor":input.salary_minor,"trainingFeeMinor":input.training_fee_minor,"active":input.active,"version":input.version.unwrap_or(0)+1}),
        )?;
        let result = json!({"id":staff_id});
        receipt(&tx, "staff", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn register_member_with_trainer(&mut self, input: StaffRegisterInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(
            &tx,
            "register_training_member",
            &input.member.request_id,
            &input,
        )? {
            return Ok(previous);
        }
        let mut result = registration::register_member_on(&tx, &input.member)?;
        let member = result["id"]
            .as_str()
            .ok_or("Member registration failed")?
            .to_owned();
        attendance_profiles::save_gender(&tx, &member, input.gender.as_deref(), None)?;
        assignment(
            &tx,
            &member,
            input.trainer_id.as_deref(),
            input.trainer_version,
            None,
        )?;
        if let Some(selected) = &input.trainer_id {
            let version = input.trainer_version.ok_or("Select the staff again")?;
            if trainer(&tx, selected, version, true)?.2 > 0 {
                result["training"] = charge(
                    &tx,
                    &TrainingChargeInput {
                        request_id: input.member.request_id.clone(),
                        member_id: member,
                        trainer_id: selected.clone(),
                        trainer_version: version,
                        starts_on: input
                            .member
                            .starts_on
                            .clone()
                            .unwrap_or_else(|| business_date(Utc::now())),
                    },
                )?;
            }
        }
        business_sync::capture(&tx)?;
        receipt(
            &tx,
            "register_training_member",
            &input.member.request_id,
            &input,
            &result,
        )?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn save_member_with_trainer(&mut self, input: StaffMemberInput) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let member = save_member_on(&tx, input.member)?;
        attendance_profiles::save_gender(
            &tx,
            &member,
            input.gender.as_deref(),
            input.gender_version,
        )?;
        assignment(
            &tx,
            &member,
            input.trainer_id.as_deref(),
            input.trainer_version,
            input.assignment_version,
        )?;
        business_sync::capture(&tx)?;
        tx.commit().map_err(db_error)
    }
    pub fn create_training_charge(&mut self, input: TrainingChargeInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "training_charge", &input.request_id, &input)? {
            return Ok(previous);
        }
        let result = charge(&tx, &input)?;
        business_sync::capture(&tx)?;
        receipt(&tx, "training_charge", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn receive_combined_payment(&mut self, input: CombinedPaymentInput) -> Result<Value> {
        money(input.payment.amount_minor, true)?;
        choice(
            &input.payment.method,
            &["Cash", "Card", "Transfer"],
            "payment method",
        )?;
        let ids = std::collections::BTreeSet::<&String>::from_iter(input.invoice_ids.iter());
        if input.invoice_ids.len() > 100
            || ids.len() != input.invoice_ids.len()
            || input.payment.invoice_id.is_some()
        {
            return Err("Select each invoice once for this payment".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "combined_payment", &input.payment.request_id, &input)?
        {
            return Ok(previous);
        }
        let result = finance::receive_payment_on(&tx, &input.payment, &input.invoice_ids)?;
        receipt(
            &tx,
            "combined_payment",
            &input.payment.request_id,
            &input,
            &result,
        )?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn pay_staff(&mut self, input: StaffPayoutInput) -> Result<Value> {
        choice(&input.method, &["Cash", "Card", "Bank"], "payment method")?;
        if input.salary_month.len() != 7 {
            return Err("Select a valid salary month".into());
        }
        date(&format!("{}-01", input.salary_month))?;
        if input.salary_month.as_str() > &business_date(Utc::now())[..7] {
            return Err("Select the current month or a past salary month".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "staff_payout", &input.request_id, &input)? {
            return Ok(previous);
        }
        let (name, salary, _) = trainer(&tx, &input.trainer_id, input.trainer_version, false)?;
        let already_paid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM active_staff_payouts WHERE trainer_id=?1 AND salary_month=?2 AND salary_minor>0)",params![input.trainer_id,input.salary_month],|r|r.get(0)).map_err(db_error)?;
        let salary = if input.include_salary && !already_paid {
            salary
        } else {
            0
        };
        let mut stmt=tx.prepare("SELECT id,amount_minor FROM unpaid_training_allocations WHERE trainer_id=?1 ORDER BY id").map_err(db_error)?;
        let allocations = stmt
            .query_map([&input.trainer_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        drop(stmt);
        let training = allocations.iter().try_fold(0_i64, |sum, a| {
            sum.checked_add(a.1)
                .ok_or("Training earnings exceed the supported range")
        })?;
        let mut expected_ids = input.allocation_ids.clone();
        expected_ids.sort();
        if salary != input.expected_salary_minor
            || training != input.expected_training_minor
            || allocations.iter().map(|a| a.0.clone()).collect::<Vec<_>>() != expected_ids
        {
            return Err("Salary or collected training earnings changed. Reopen the payment to review the current amount.".into());
        }
        let total = salary
            .checked_add(training)
            .ok_or("Staff payment exceeds the supported range")?;
        money(total, true)?;
        let expense = id();
        let payout = id();
        let now = Utc::now();
        let actor = desktop_auth::actor_label(&tx)?;
        tx.execute(
            "INSERT INTO expenses VALUES(?1,?2,'Salary',?3,?4,?5,?6,?7,NULL)",
            params![
                expense,
                format!("Staff payment: {name} · {}", input.salary_month),
                total,
                input.method,
                business_date(now),
                now.to_rfc3339(),
                actor
            ],
        )
        .map_err(db_error)?;
        tx.execute(
            "INSERT INTO staff_payouts VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                payout,
                input.trainer_id,
                expense,
                input.salary_month,
                salary,
                training
            ],
        )
        .map_err(db_error)?;
        for (allocation, amount) in allocations {
            tx.execute(
                "INSERT INTO staff_payout_items VALUES(?1,?2,?3,?4)",
                params![id(), payout, allocation, amount],
            )
            .map_err(db_error)?;
        }
        record(
            &tx,
            "staff_payout",
            &payout,
            None,
            None,
            json!({"id":payout,"trainerId":input.trainer_id,"expenseId":expense,"salaryMonth":input.salary_month,"salaryMinor":salary,"trainingMinor":training,"amountMinor":total,"actor":actor}),
        )?;
        let result = json!({"id":payout,"expenseId":expense});
        receipt(&tx, "staff_payout", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}
pub(super) fn append_snapshot(conn: &Connection, result: &mut Value) -> Result<()> {
    result["trainers"]=json!(rows(conn,"SELECT json_object('id',t.id,'name',t.name,'phone',t.phone,'nic',t.nic,'salaryMinor',t.salary_minor,'trainingFeeMinor',t.training_fee_minor,'active',json(CASE WHEN t.active=1 THEN 'true' ELSE 'false' END),'version',t.version,'assignedMembers',(SELECT count(*) FROM member_trainers a JOIN members m ON m.id=a.id WHERE a.trainer_id=t.id AND m.archived_at IS NULL),'unpaidTrainingMinor',COALESCE((SELECT sum(amount_minor) FROM unpaid_training_allocations a WHERE a.trainer_id=t.id),0)) FROM trainers t ORDER BY t.name,t.id")?);
    result["memberTrainers"]=json!(rows(conn,"SELECT json_object('memberId',a.id,'trainerId',a.trainer_id,'version',a.version) FROM member_trainers a")?);
    result["trainingCharges"]=json!(rows(conn,"SELECT json_object('id',c.id,'invoiceId',c.invoice_id,'memberId',c.member_id,'trainerId',c.trainer_id,'trainerName',c.trainer_name,'feeMinor',c.fee_minor,'startsOn',c.starts_on,'endsOn',c.ends_on) FROM training_charges c ORDER BY c.starts_on DESC,c.id")?);
    result["staffTrainingAllocations"]=json!(rows(conn,"SELECT json_object('id',id,'trainerId',trainer_id,'amountMinor',amount_minor) FROM unpaid_training_allocations ORDER BY id")?);
    result["staffPayouts"]=json!(rows(conn,"SELECT json_object('id',p.id,'trainerId',p.trainer_id,'trainerName',t.name,'expenseId',p.expense_id,'salaryMonth',p.salary_month,'salaryMinor',p.salary_minor,'trainingMinor',p.training_minor,'amountMinor',e.amount_minor,'businessOn',e.business_on,'method',e.method,'active',json(CASE WHEN EXISTS(SELECT 1 FROM active_staff_payouts a WHERE a.id=p.id) THEN 'true' ELSE 'false' END)) FROM staff_payouts p JOIN trainers t ON t.id=p.trainer_id JOIN expenses e ON e.id=p.expense_id ORDER BY e.created_at DESC,p.id")?);
    Ok(())
}

#[cfg(test)]
#[path = "staff_training_tests.rs"]
mod tests;
