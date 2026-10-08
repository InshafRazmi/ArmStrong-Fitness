use super::operations::{choice, money, receipt, replay, rows};
use super::*;
use rusqlite::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvoiceInput {
    pub request_id: String,
    pub member_id: String,
    pub membership_period_id: Option<String>,
    pub description: String,
    pub amount_minor: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AllocationInput {
    pub request_id: String,
    pub payment_id: String,
    pub invoice_id: String,
    pub amount_minor: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceivePaymentInput {
    pub request_id: String,
    pub member_id: String,
    pub amount_minor: i64,
    pub method: String,
    pub invoice_id: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenewalInput {
    pub request_id: String,
    pub member_id: String,
    pub plan_id: String,
    pub plan_version: i64,
    pub expected_last_period_id: Option<String>,
    pub starts_on: String,
    pub ends_on: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReversalInput {
    pub request_id: String,
    pub payment_id: String,
    pub reason: String,
}

fn number(conn: &Connection, kind: &str, document_id: &str) -> Result<String> {
    let device: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    Ok(format!("AF-{kind}-{device}-{document_id}"))
}
fn member_name(conn: &Connection, member_id: &str) -> Result<String> {
    conn.query_row("SELECT name FROM members WHERE id=?1", [member_id], |r| {
        r.get(0)
    })
    .optional()
    .map_err(db_error)?
    .ok_or("Select a saved member".into())
}
pub(super) fn insert_invoice(tx: &Transaction<'_>, input: &InvoiceInput) -> Result<Value> {
    money(input.amount_minor, true)?;
    let description = required(&input.description, "Invoice description", 254)?;
    let name = member_name(tx, &input.member_id)?;
    if let Some(period) = &input.membership_period_id {
        let saved: Option<(String, i64)> = tx
            .query_row(
                "SELECT member_id,price_minor FROM membership_periods WHERE id=?1",
                [period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (member, price) = saved.ok_or("Select a saved membership period")?;
        if member != input.member_id || price != input.amount_minor {
            return Err("Membership invoice must use that member's saved period price".into());
        }
    }
    let invoice = id();
    let invoice_number = number(tx, "I", &invoice)?;
    let now = Utc::now();
    let day = business_date(now);
    let instant = now.to_rfc3339();
    tx.execute(
        "INSERT INTO invoices VALUES(?1,?2,?3,NULL,?4,?5,?6)",
        params![
            invoice,
            input.member_id,
            input.membership_period_id,
            input.amount_minor,
            day,
            instant
        ],
    )
    .map_err(db_error)?;
    tx.execute(
        "INSERT INTO invoice_details VALUES(?1,?2,?3,?4,?5,0)",
        params![
            invoice,
            invoice_number,
            description,
            name,
            super::desktop_auth::actor_label(tx)?
        ],
    )
    .map_err(db_error)?;
    Ok(
        json!({"id":invoice,"number":invoice_number,"memberId":input.member_id,"memberName":name,"membershipPeriodId":input.membership_period_id,"amountMinor":input.amount_minor,"description":description,"issuedOn":day,"createdAt":instant,"actor":super::desktop_auth::actor_label(tx)?}),
    )
}
pub(super) fn insert_allocation(
    tx: &Transaction<'_>,
    payment: &str,
    invoice: &str,
    amount: i64,
) -> Result<Value> {
    money(amount, true)?;
    let allocation = id();
    tx.execute(
        "INSERT INTO payment_allocations VALUES(?1,?2,?3,?4)",
        params![allocation, payment, invoice, amount],
    )
    .map_err(db_error)?;
    Ok(json!({"id":allocation,"paymentId":payment,"invoiceId":invoice,"amountMinor":amount}))
}
pub(super) fn issue_receipt(tx: &Transaction<'_>, payment_id: &str, legacy: bool) -> Result<Value> {
    let payment:String=tx.query_row("SELECT json_object('id',id,'memberId',member_id,'memberName',member_name,'amountMinor',amount_minor,'method',method,'businessOn',business_on,'createdAt',created_at,'actor',actor,'reversesId',reverses_id) FROM payments WHERE id=?1",[payment_id],|r|r.get(0)).map_err(db_error)?;
    let payment: Value = serde_json::from_str(&payment).map_err(|e| e.to_string())?;
    let profile=rows(tx,"SELECT json_object('name',name,'location',location,'phone',phone,'email',email) FROM gym_settings")?.into_iter().next().ok_or("Gym profile is missing")?;
    let original = payment["reversesId"].as_str();
    let source = original.unwrap_or(payment_id);
    let mut stmt=tx.prepare("SELECT json_object('invoiceId',a.invoice_id,'invoiceNumber',COALESCE(d.number,a.invoice_id),'description',COALESCE(d.description,'Legacy invoice'),'amountMinor',a.amount_minor,'outstandingMinor',b.outstanding_minor,'released',EXISTS(SELECT 1 FROM allocation_reversals r WHERE r.allocation_id=a.id)) FROM payment_allocations a JOIN invoice_balances b ON b.id=a.invoice_id LEFT JOIN invoice_details d ON d.invoice_id=a.invoice_id WHERE a.payment_id=?1 ORDER BY a.rowid").map_err(db_error)?;
    let allocations = stmt
        .query_map([source], |r| r.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error)?;
    let allocations = allocations
        .into_iter()
        .map(|a| serde_json::from_str::<Value>(&a).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>>>()?;
    drop(stmt);
    let credit: i64 = tx
        .query_row(
            "SELECT unallocated_minor FROM payment_balances WHERE id=?1",
            [payment_id],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let reason: Option<String> = tx
        .query_row(
            "SELECT reason FROM payment_reversal_details WHERE payment_id=?1",
            [payment_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    let receipt_number = number(tx, "R", payment_id)?;
    let issued = Utc::now().to_rfc3339();
    let original_number = original.map(|p| number(tx, "R", p)).transpose()?;
    let snapshot = json!({"formatVersion":1,"number":receipt_number,"payment":payment,"gym":profile,"allocations":allocations,"unallocatedAtIssueMinor":credit,"originalReceiptNumber":original_number,"reason":reason,"issuedAt":issued,"legacy":legacy});
    tx.execute(
        "INSERT INTO payment_receipts VALUES(?1,?2,?3,?4,?5)",
        params![
            payment_id,
            receipt_number,
            snapshot.to_string(),
            issued,
            legacy
        ],
    )
    .map_err(db_error)?;
    Ok(snapshot)
}
pub(super) fn migrate_receipts(tx: &Transaction<'_>) -> Result<()> {
    // Preserve existing operation receipts/outbox. New document artifacts do not invent cash or debt.
    let mut stmt=tx.prepare("SELECT id FROM payments WHERE NOT EXISTS(SELECT 1 FROM payment_receipts r WHERE r.payment_id=payments.id) ORDER BY created_at,id").map_err(db_error)?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error)?;
    drop(stmt);
    for payment in ids {
        issue_receipt(tx, &payment, true)?;
    }
    Ok(())
}
pub(super) fn append_snapshot(conn: &Connection, result: &mut Value) -> Result<()> {
    result["invoices"]=json!(rows(conn,"SELECT json_object('id',i.id,'number',COALESCE(d.number,i.id),'memberId',i.member_id,'memberName',COALESCE(d.member_name,'Unassigned legacy invoice'),'membershipPeriodId',i.membership_period_id,'amountMinor',i.amount_minor,'paidMinor',i.paid_minor,'outstandingMinor',i.outstanding_minor,'description',COALESCE(d.description,'Legacy invoice'),'issuedOn',i.issued_on,'createdAt',i.created_at,'status',CASE WHEN i.outstanding_minor=0 THEN 'Paid' WHEN i.paid_minor=0 THEN 'Unpaid' ELSE 'Partial' END) FROM invoice_balances i LEFT JOIN invoice_details d ON d.invoice_id=i.id ORDER BY i.created_at DESC,i.id")?);
    result["payments"]=json!(rows(conn,"SELECT json_object('id',p.id,'memberId',p.member_id,'memberName',p.member_name,'amountMinor',p.amount_minor,'netAmountMinor',p.net_amount_minor,'allocatedMinor',p.allocated_minor,'unallocatedMinor',p.unallocated_minor,'status',p.status,'method',p.method,'businessOn',p.business_on,'createdAt',p.created_at,'actor',p.actor,'reversesId',p.reverses_id,'receiptNumber',r.number,'reversalReason',d.reason) FROM payment_balances p LEFT JOIN payment_receipts r ON r.payment_id=p.id LEFT JOIN payment_reversal_details d ON d.payment_id=p.id ORDER BY p.created_at DESC,p.id DESC")?);
    result["allocations"]=json!(rows(conn,"SELECT json_object('id',a.id,'paymentId',a.payment_id,'invoiceId',a.invoice_id,'amountMinor',a.amount_minor,'reversedBy',r.payment_reversal_id) FROM payment_allocations a LEFT JOIN allocation_reversals r ON r.allocation_id=a.id ORDER BY a.rowid")?);
    let accounts=rows(conn,"SELECT json_object('memberId',m.id,'memberName',m.name,'outstandingMinor',COALESCE((SELECT SUM(outstanding_minor) FROM invoice_balances WHERE member_id=m.id),0),'creditMinor',COALESCE((SELECT SUM(unallocated_minor) FROM payment_balances WHERE member_id=m.id),0)) FROM members m ORDER BY m.name,m.id")?;
    let mut accounts = accounts;
    for account in &mut accounts {
        let debt = account["outstandingMinor"]
            .as_i64()
            .ok_or("Invoice total overflow")?;
        let credit = account["creditMinor"]
            .as_i64()
            .ok_or("Credit total overflow")?;
        if debt > 9_007_199_254_740_991 || credit > 9_007_199_254_740_991 {
            return Err(
                "Finance totals exceed the exact display range; no records were changed".into(),
            );
        }
        account["netBalanceMinor"] = json!(debt - credit);
    }
    result["financialAccounts"] = json!(accounts);
    Ok(())
}
impl Store {
    pub fn create_invoice(&mut self, input: InvoiceInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "invoice", &input.request_id, &input)? {
            return Ok(previous);
        }
        let invoice = insert_invoice(&tx, &input)?;
        record(
            &tx,
            "invoice",
            invoice["id"].as_str().unwrap(),
            None,
            None,
            invoice.clone(),
        )?;
        let result = json!({"id":invoice["id"]});
        receipt(&tx, "invoice", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn allocate_payment(&mut self, input: AllocationInput) -> Result<Value> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "allocation", &input.request_id, &input)? {
            return Ok(previous);
        }
        let allocation = insert_allocation(
            &tx,
            &input.payment_id,
            &input.invoice_id,
            input.amount_minor,
        )?;
        record(
            &tx,
            "payment_allocation",
            allocation["id"].as_str().unwrap(),
            None,
            None,
            allocation.clone(),
        )?;
        let result = json!({"id":allocation["id"]});
        receipt(&tx, "allocation", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn receive_payment(&mut self, input: ReceivePaymentInput) -> Result<Value> {
        money(input.amount_minor, true)?;
        choice(
            &input.method,
            &["Cash", "Card", "Transfer"],
            "payment method",
        )?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "receive_payment", &input.request_id, &input)? {
            return Ok(previous);
        }
        let invoice_ids = input.invoice_id.iter().cloned().collect::<Vec<_>>();
        let result = receive_payment_on(&tx, &input, &invoice_ids)?;
        receipt(&tx, "receive_payment", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn renew_membership(&mut self, input: RenewalInput) -> Result<Value> {
        if date(&input.ends_on)? < date(&input.starts_on)? {
            return Err("Last valid day must be on or after the start".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "renewal", &input.request_id, &input)? {
            return Ok(previous);
        }
        member_name(&tx, &input.member_id)?;
        let latest:Option<(String,String)>=tx.query_row("SELECT id,ends_on FROM membership_periods WHERE member_id=?1 ORDER BY ends_on DESC,id DESC LIMIT 1",[&input.member_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_error)?;
        if latest.as_ref().map(|p| p.0.as_str()) != input.expected_last_period_id.as_deref() {
            return Err("Membership history changed. Refresh before renewing".into());
        }
        if latest.as_ref().is_some_and(|p| input.starts_on <= p.1) {
            return Err(
                "Choose a renewal start after the latest membership's last valid day".into(),
            );
        }
        let plan: Option<(String, i64, i64)> = tx
            .query_row(
                "SELECT name,price_minor,version FROM plans WHERE id=?1 AND active=1",
                [&input.plan_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (name, price, version) = plan.ok_or("Select an active plan")?;
        if version != input.plan_version {
            return Err("Plan price or details changed. Refresh before renewing".into());
        }
        money(price, true).map_err(|_| {
            "Zero-price memberships use the date-only entry; no invoice policy is defined"
                .to_string()
        })?;
        let period = id();
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO membership_periods VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                period,
                input.member_id,
                input.plan_id,
                name,
                price,
                input.starts_on,
                input.ends_on,
                now
            ],
        )
        .map_err(db_error)?;
        let invoice = insert_invoice(
            &tx,
            &InvoiceInput {
                request_id: input.request_id.clone(),
                member_id: input.member_id.clone(),
                membership_period_id: Some(period.clone()),
                description: format!("Membership: {name}"),
                amount_minor: price,
            },
        )?;
        record(
            &tx,
            "membership_renewal",
            &period,
            None,
            None,
            json!({"id":period,"memberId":input.member_id,"planId":input.plan_id,"planVersion":version,"expectedLastPeriodId":input.expected_last_period_id,"planName":name,"priceMinor":price,"startsOn":input.starts_on,"endsOn":input.ends_on,"createdAt":now,"invoice":invoice}),
        )?;
        let result = json!({"id":period,"invoiceId":invoice["id"]});
        receipt(&tx, "renewal", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn reverse_payment(&mut self, input: ReversalInput) -> Result<Value> {
        let reason = required(&input.reason, "Reversal reason", 254)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(previous) = replay(&tx, "payment_reversal", &input.request_id, &input)? {
            return Ok(previous);
        }
        let saved:Option<(String,String,i64,String,Option<String>)>=tx.query_row("SELECT member_id,member_name,amount_minor,method,reverses_id FROM payments WHERE id=?1",[&input.payment_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(db_error)?;
        let (member, name, amount, method, parent) =
            saved.ok_or("Select a saved received payment")?;
        let reversed: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM payments WHERE reverses_id=?1)",
                [&input.payment_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if parent.is_some() || reversed {
            return Err("This payment is already reversed or is itself a reversal".into());
        }
        let reversal = id();
        let now = Utc::now();
        let day = business_date(now);
        let instant = now.to_rfc3339();
        tx.execute(
            "INSERT INTO payments VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                reversal,
                member,
                name,
                amount,
                method,
                day,
                instant,
                super::desktop_auth::actor_label(&tx)?,
                input.payment_id
            ],
        )
        .map_err(db_error)?;
        tx.execute(
            "INSERT INTO payment_reversal_details VALUES(?1,?2)",
            params![reversal, reason],
        )
        .map_err(db_error)?;
        let mut stmt=tx.prepare("SELECT id FROM payment_allocations a WHERE payment_id=?1 AND NOT EXISTS(SELECT 1 FROM allocation_reversals r WHERE r.allocation_id=a.id) ORDER BY rowid").map_err(db_error)?;
        let allocations = stmt
            .query_map([&input.payment_id], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        drop(stmt);
        let mut releases = vec![];
        for allocation in allocations {
            let release = id();
            tx.execute(
                "INSERT INTO allocation_reversals VALUES(?1,?2,?3,?4)",
                params![release, allocation, reversal, instant],
            )
            .map_err(db_error)?;
            releases.push(json!({"id":release,"allocationId":allocation,"paymentReversalId":reversal,"createdAt":instant}));
        }
        let document = issue_receipt(&tx, &reversal, false)?;
        record(
            &tx,
            "payment_reversal",
            &reversal,
            None,
            None,
            json!({"id":reversal,"reversesId":input.payment_id,"memberId":member,"memberName":name,"amountMinor":amount,"method":method,"businessOn":day,"createdAt":instant,"actor":super::desktop_auth::actor_label(&tx)?,"reason":reason,"allocationReversals":releases,"receipt":document}),
        )?;
        let result = json!({"id":reversal,"receiptNumber":document["number"]});
        receipt(&tx, "payment_reversal", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn payment_receipt(&self, payment_id: String) -> Result<Value> {
        let tx = self.conn.unchecked_transaction().map_err(db_error)?;
        let saved: Option<(String, String)> = tx
            .query_row(
                "SELECT r.number,r.snapshot_json FROM payment_receipts r WHERE payment_id=?1",
                [&payment_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (number, snapshot) =
            saved.ok_or("Saved receipt is missing; no new number was issued")?;
        let status: String = tx
            .query_row(
                "SELECT status FROM payment_balances WHERE id=?1",
                [&payment_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let reversal:Option<String>=tx.query_row("SELECT r.number FROM payments p JOIN payment_receipts r ON r.payment_id=p.id WHERE p.reverses_id=?1",[&payment_id],|r|r.get(0)).optional().map_err(db_error)?;
        let snapshot: Value = serde_json::from_str(&snapshot).map_err(|e| e.to_string())?;
        tx.commit().map_err(db_error)?;
        Ok(
            json!({"number":number,"snapshot":snapshot,"currentStatus":status,"reversalReceiptNumber":reversal}),
        )
    }
}

pub(super) fn receive_payment_on(
    tx: &rusqlite::Transaction<'_>,
    input: &ReceivePaymentInput,
    invoice_ids: &[String],
) -> Result<Value> {
    let name = member_name(tx, &input.member_id)?;
    let payment = id();
    let now = Utc::now();
    let day = business_date(now);
    let instant = now.to_rfc3339();
    tx.execute(
        "INSERT INTO payments VALUES(?1,?2,?3,?4,?5,?6,?7,?8,NULL)",
        params![
            payment,
            input.member_id,
            name,
            input.amount_minor,
            input.method,
            day,
            instant,
            super::desktop_auth::actor_label(tx)?
        ],
    )
    .map_err(db_error)?;
    let mut allocations = vec![];
    let mut remaining = input.amount_minor;
    for invoice in invoice_ids {
        let target: Option<(Option<String>, i64)> = tx
            .query_row(
                "SELECT member_id,outstanding_minor FROM invoice_balances WHERE id=?1",
                [invoice],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (member, outstanding) = target.ok_or("Invoice no longer exists")?;
        if member.as_deref() != Some(&input.member_id) || outstanding <= 0 {
            return Err("Select an outstanding invoice belonging to this member".into());
        }
        if remaining == 0 {
            continue;
        }
        allocations.push(insert_allocation(
            tx,
            &payment,
            invoice,
            remaining.min(outstanding),
        )?);
        remaining -= remaining.min(outstanding);
    }
    let document = issue_receipt(tx, &payment, false)?;
    record(
        tx,
        "payment",
        &payment,
        None,
        None,
        json!({"id":payment,"memberId":input.member_id,"memberName":name,"amountMinor":input.amount_minor,"method":input.method,"businessOn":day,"createdAt":instant,"actor":super::desktop_auth::actor_label(tx)?,"allocations":allocations,"receipt":document}),
    )?;
    let result = json!({"id":payment,"receiptNumber":document["number"]});
    Ok(result)
}
