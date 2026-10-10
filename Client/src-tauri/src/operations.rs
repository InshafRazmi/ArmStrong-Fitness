use super::*;
use rusqlite::Transaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttendanceInput {
    pub request_id: String,
    pub member_or_card: String,
    pub source: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaymentInput {
    pub request_id: String,
    pub member_id: String,
    pub amount_minor: i64,
    pub method: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpenseInput {
    pub request_id: String,
    pub title: String,
    pub category: String,
    pub amount_minor: i64,
    pub method: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductInput {
    pub request_id: String,
    pub id: Option<String>,
    pub version: Option<i64>,
    pub name: String,
    pub sku: String,
    pub cost_minor: i64,
    pub price_minor: i64,
    pub reorder_level: i64,
    pub opening_stock: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StockInput {
    pub request_id: String,
    pub product_id: String,
    pub amount: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaleInput {
    pub request_id: String,
    pub product_id: String,
    pub quantity: i64,
    pub method: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileInput {
    #[serde(default)]
    pub admission_minor: Option<i64>,
    #[serde(default)]
    pub admission_version: Option<i64>,
    pub version: i64,
    pub name: String,
    pub location: String,
    pub phone: String,
    pub email: String,
}

pub(super) fn money(amount: i64, positive: bool) -> Result<()> {
    if !(i64::from(positive)..=100_000_000_000).contains(&amount) {
        return Err("Enter an amount within the supported LKR range".into());
    }
    Ok(())
}
pub(super) fn choice(value: &str, choices: &[&str], label: &str) -> Result<()> {
    if !choices.contains(&value) {
        return Err(format!("Invalid {label}"));
    }
    Ok(())
}
pub(super) fn replay(
    tx: &Transaction<'_>,
    command: &str,
    request_id: &str,
    input: &impl Serialize,
) -> Result<Option<Value>> {
    Uuid::parse_str(request_id).map_err(|_| "A valid operation UUID is required")?;
    let previous: Option<(String, String, String)> = tx
        .query_row(
            "SELECT command,request_json,result_json FROM local_operations WHERE request_id=?1",
            [request_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(db_error)?;
    if let Some((old_command, request, result)) = previous {
        if old_command != command
            || request != serde_json::to_string(input).map_err(|e| e.to_string())?
        {
            return Err("Operation ID was reused with a different request".into());
        }
        return serde_json::from_str(&result)
            .map(Some)
            .map_err(|e| e.to_string());
    }
    Ok(None)
}
pub(super) fn receipt(
    tx: &Transaction<'_>,
    command: &str,
    request_id: &str,
    input: &impl Serialize,
    result: &Value,
) -> Result<()> {
    tx.execute(
        "INSERT INTO local_operations VALUES (?1,?2,?3,?4,?5)",
        params![
            request_id,
            command,
            serde_json::to_string(input).map_err(|e| e.to_string())?,
            result.to_string(),
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(db_error)?;
    Ok(())
}
pub(super) fn rows(conn: &Connection, sql: &str) -> Result<Vec<Value>> {
    let mut stmt = conn.prepare(sql).map_err(db_error)?;
    let values = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error)?;
    values
        .into_iter()
        .map(|value| serde_json::from_str(&value).map_err(|e| e.to_string()))
        .collect()
}
pub(super) fn append_snapshot(conn: &Connection, result: &mut Value) -> Result<()> {
    result["profile"] = rows(conn, "SELECT json_object('version',version,'name',name,'location',location,'phone',phone,'email',email,'admissionMinor',COALESCE((SELECT amount_minor FROM admission_settings WHERE id=1),0),'admissionVersion',(SELECT version FROM admission_settings WHERE id=1)) FROM gym_settings WHERE id=1")?.into_iter().next().ok_or("Gym profile is missing; preserve the database for recovery")?;
    result["attendance"] = json!(rows(conn,"SELECT json_object('id',id,'memberId',member_id,'name',member_name,'cardId',card_id,'cardUid',card_uid,'type',kind,'source',source,'businessOn',business_on,'occurredAt',occurred_at,'voidsId',voids_id) FROM attendance ORDER BY occurred_at DESC,rowid DESC")?);
    result["payments"] = json!(rows(conn,"SELECT json_object('id',id,'memberId',member_id,'memberName',member_name,'amountMinor',amount_minor,'method',method,'businessOn',business_on,'createdAt',created_at,'actor',actor,'reversesId',reverses_id) FROM payments ORDER BY created_at DESC,rowid DESC")?);
    result["expenses"] = json!(rows(conn,"SELECT json_object('id',id,'title',title,'category',category,'amountMinor',amount_minor,'method',method,'businessOn',business_on,'createdAt',created_at,'actor',actor,'reversesId',reverses_id) FROM expenses ORDER BY created_at DESC,rowid DESC")?);
    result["products"] = json!(rows(conn,"SELECT json_object('id',id,'version',version,'name',name,'sku',sku,'costMinor',cost_minor,'priceMinor',price_minor,'reorderLevel',reorder_level,'stock',COALESCE((SELECT SUM(delta) FROM stock_movements WHERE product_id=products.id),0)) FROM products ORDER BY name")?);
    result["sales"] = json!(rows(conn,"SELECT json_object('id',id,'totalMinor',total_minor,'method',method,'businessOn',business_on,'createdAt',created_at,'actor',actor,'reversesId',reverses_id,'items',json(COALESCE((SELECT json_group_array(json_object('id',id,'productId',product_id,'name',name,'sku',sku,'quantity',quantity,'priceMinor',price_minor,'costMinor',cost_minor)) FROM sale_items WHERE sale_id=sales.id),'[]'))) FROM sales ORDER BY created_at DESC,rowid DESC")?);
    result["audit"] = json!(rows(conn,"SELECT json_object('id',id,'action',action,'entity',entity,'entityId',entity_id,'user',actor,'deviceId',device_id,'beforeJson',before_json,'afterJson',after_json,'timestamp',created_at) FROM audit ORDER BY created_at DESC,rowid DESC")?);
    result["users"] = json!(rows(conn,"SELECT json_object('id',id,'name',display_name,'email',email,'active',active,'roles',json(COALESCE((SELECT json_group_array(name) FROM roles JOIN user_roles ON role_id=roles.id WHERE user_id=users.id),'[]'))) FROM users ORDER BY display_name")?);
    result["restoreRequiresReconciliation"] = json!(
        conn.query_row(
            "SELECT value FROM metadata WHERE key='restore_requires_reconciliation'",
            [],
            |r| r.get::<_, String>(0)
        )
        .optional()
        .map_err(db_error)?
        .as_deref()
            == Some("1")
    );
    Ok(())
}
fn stock(tx: &Transaction<'_>, product_id: &str) -> Result<i64> {
    tx.query_row(
        "SELECT COALESCE(SUM(delta),0) FROM stock_movements WHERE product_id=?1",
        [product_id],
        |r| r.get(0),
    )
    .map_err(db_error)
}
fn movement(
    tx: &Transaction<'_>,
    product_id: &str,
    item: Option<&str>,
    delta: i64,
    kind: &str,
    reason: &str,
    now: &str,
) -> Result<Value> {
    let movement_id = id();
    tx.execute(
        "INSERT INTO stock_movements VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            movement_id,
            product_id,
            item,
            delta,
            kind,
            reason,
            now,
            super::desktop_auth::actor_label(tx)?
        ],
    )
    .map_err(db_error)?;
    Ok(
        json!({"id":movement_id,"productId":product_id,"saleItemId":item,"delta":delta,"kind":kind,"reason":reason,"createdAt":now,"actor":super::desktop_auth::actor_label(tx)?}),
    )
}
impl Store {
    pub fn save_profile(&mut self, input: ProfileInput) -> Result<Value> {
        if let Some(amount) = input.admission_minor {
            money(amount, false)?;
        }
        let name = required(&input.name, "Gym name", 120)?;
        let location = required(&input.location, "Location", 254)?;
        let phone = input.phone.trim();
        let email = input.email.trim();
        if phone.chars().count() > 40
            || email.len() > 254
            || (!email.is_empty()
                && (!email.contains('@') || email.chars().any(char::is_whitespace)))
        {
            return Err("Enter a valid phone/email or leave them blank".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(amount) = input.admission_minor {
            let admission_before = rows(&tx, "SELECT json_object('id',id,'amountMinor',amount_minor,'version',version) FROM admission_settings WHERE id=1")?.into_iter().next().map(|value| value.to_string());
            let current: Option<i64> = tx
                .query_row(
                    "SELECT version FROM admission_settings WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db_error)?;
            if current != input.admission_version {
                return Err("Admission fee changed. Reload the gym profile before saving.".into());
            }
            tx.execute("INSERT INTO admission_settings VALUES(1,?1,1) ON CONFLICT(id) DO UPDATE SET amount_minor=excluded.amount_minor,version=admission_settings.version+1", [amount]).map_err(db_error)?;
            record_uncaptured(
                &tx,
                "admission_settings",
                "1",
                current,
                admission_before,
                json!({"id":1,"amountMinor":amount,"version":current.unwrap_or(0)+1}),
            )?;
        }
        let before = rows(&tx,"SELECT json_object('name',name,'location',location,'phone',phone,'email',email,'version',version) FROM gym_settings")?[0].to_string();
        if tx.execute("UPDATE gym_settings SET name=?1,location=?2,phone=?3,email=?4,version=version+1 WHERE id=1 AND version=?5", params![name,location,phone,email,input.version]).map_err(db_error)?!=1 { return Err("Gym profile changed. Refresh before saving.".into()); }
        record(
            &tx,
            "gym_settings",
            "1",
            Some(input.version),
            Some(before),
            json!({"name":name,"location":location,"phone":phone,"email":email,"version":input.version+1}),
        )?;
        tx.commit().map_err(db_error)?;
        Ok(Value::Null)
    }
    pub fn record_attendance(&mut self, input: AttendanceInput) -> Result<Value> {
        self.attendance_at(input, Utc::now())
    }
    pub(super) fn attendance_at(
        &mut self,
        input: AttendanceInput,
        now: DateTime<Utc>,
    ) -> Result<Value> {
        choice(&input.source, &["NFC", "Manual"], "attendance source")?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(result) = replay(&tx, "attendance", &input.request_id, &input)? {
            return Ok(result);
        }
        let lookup = input.member_or_card.trim().to_ascii_uppercase();
        let member: Option<(String, String, Option<String>, String)> = if input.source == "NFC" {
            tx.query_row("SELECT m.id,m.name,c.id,c.uid FROM members m JOIN nfc_cards c ON c.member_id=m.id AND c.revoked_at IS NULL AND c.uid=m.nfc_id WHERE c.uid=?1 AND m.archived_at IS NULL", [&lookup], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db_error)?
        } else {
            tx.query_row(
                "SELECT id,name,NULL,'' FROM members WHERE id=?1 AND archived_at IS NULL",
                [&input.member_or_card],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()
            .map_err(db_error)?
        };
        let (member_id, name, card_id, card_uid) =
            member.ok_or("Card/member is not linked to a member")?;
        let day = business_date(now);
        let last: Option<(String,String,String,String)> = tx.query_row("SELECT id,kind,source,occurred_at FROM attendance WHERE member_id=?1 AND business_on=?2 ORDER BY occurred_at DESC,rowid DESC LIMIT 1", params![member_id,day], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db_error)?;
        if let Some((last_id, _, source, instant)) = &last {
            let previous = DateTime::parse_from_rfc3339(instant).map_err(|e| e.to_string())?;
            let elapsed = now.timestamp_millis() - previous.timestamp_millis();
            if input.source == "NFC" && source == "NFC" && (0..2000).contains(&elapsed) {
                let result = json!({"id":last_id,"duplicate":true});
                receipt(&tx, "attendance", &input.request_id, &input, &result)?;
                tx.commit().map_err(db_error)?;
                return Ok(result);
            }
        }
        let kind = if last
            .as_ref()
            .is_some_and(|(_, kind, _, _)| kind == "Check-in")
        {
            "Check-out"
        } else {
            "Check-in"
        };
        let event = id();
        let instant = now.to_rfc3339();
        tx.execute(
            "INSERT INTO attendance VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,NULL)",
            params![
                event,
                member_id,
                card_id,
                name,
                card_uid,
                kind,
                input.source,
                day,
                instant
            ],
        )
        .map_err(db_error)?;
        record(
            &tx,
            "attendance",
            &event,
            None,
            None,
            json!({"id":event,"memberId":member_id,"cardId":card_id,"name":name,"cardUid":card_uid,"type":kind,"source":input.source,"businessOn":day,"occurredAt":instant}),
        )?;
        let result = json!({"id":event,"duplicate":false});
        receipt(&tx, "attendance", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn record_payment(&mut self, input: PaymentInput) -> Result<Value> {
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
        if let Some(result) = replay(&tx, "payment", &input.request_id, &input)? {
            return Ok(result);
        }
        let name: String = tx
            .query_row(
                "SELECT name FROM members WHERE id=?1",
                [&input.member_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let payment = id();
        let now = Utc::now();
        let day = business_date(now);
        let instant = now.to_rfc3339();
        tx.execute(
            "INSERT INTO payments VALUES (?1,?2,?3,?4,?5,?6,?7,?8,NULL)",
            params![
                payment,
                input.member_id,
                name,
                input.amount_minor,
                input.method,
                day,
                instant,
                super::desktop_auth::actor_label(&tx)?
            ],
        )
        .map_err(db_error)?;
        let document = super::finance::issue_receipt(&tx, &payment, false)?;
        record(
            &tx,
            "payment",
            &payment,
            None,
            None,
            json!({"id":payment,"memberId":input.member_id,"memberName":name,"amountMinor":input.amount_minor,"method":input.method,"businessOn":day,"createdAt":instant,"actor":super::desktop_auth::actor_label(&tx)?,"receipt":document}),
        )?;
        let result = json!({"id":payment});
        receipt(&tx, "payment", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn record_expense(&mut self, input: ExpenseInput) -> Result<Value> {
        money(input.amount_minor, true)?;
        let title = required(&input.title, "Description", 254)?;
        choice(&input.method, &["Cash", "Card", "Bank"], "expense method")?;
        choice(
            &input.category,
            &["Operations", "Utilities", "Maintenance", "Salary", "Other"],
            "category",
        )?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(result) = replay(&tx, "expense", &input.request_id, &input)? {
            return Ok(result);
        }
        let expense = id();
        let now = Utc::now();
        let day = business_date(now);
        let instant = now.to_rfc3339();
        tx.execute(
            "INSERT INTO expenses VALUES (?1,?2,?3,?4,?5,?6,?7,?8,NULL)",
            params![
                expense,
                title,
                input.category,
                input.amount_minor,
                input.method,
                day,
                instant,
                super::desktop_auth::actor_label(&tx)?
            ],
        )
        .map_err(db_error)?;
        record(
            &tx,
            "expense",
            &expense,
            None,
            None,
            json!({"id":expense,"title":title,"category":input.category,"amountMinor":input.amount_minor,"method":input.method,"businessOn":day,"createdAt":instant,"actor":super::desktop_auth::actor_label(&tx)?}),
        )?;
        let result = json!({"id":expense});
        receipt(&tx, "expense", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn save_product(&mut self, input: ProductInput) -> Result<Value> {
        let name = required(&input.name, "Product name", 120)?;
        let sku = required(&input.sku, "SKU", 80)?.to_ascii_uppercase();
        money(input.cost_minor, false)?;
        money(input.price_minor, false)?;
        if !(0..=1_000_000).contains(&input.opening_stock)
            || !(0..=1_000_000).contains(&input.reorder_level)
            || (input.id.is_some() && input.opening_stock != 0)
        {
            return Err(
                "Invalid stock/reorder amount; adjust existing stock through movements".into(),
            );
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(result) = replay(&tx, "product", &input.request_id, &input)? {
            return Ok(result);
        }
        let product = input.id.clone().unwrap_or_else(id);
        let before: Option<String> = if input.id.is_some() {
            tx.query_row("SELECT json_object('name',name,'sku',sku,'costMinor',cost_minor,'priceMinor',price_minor,'reorderLevel',reorder_level,'version',version) FROM products WHERE id=?1",[&product],|r|r.get(0)).optional().map_err(db_error)?
        } else {
            None
        };
        if input.id.is_some() {
            if tx.execute("UPDATE products SET name=?1,sku=?2,cost_minor=?3,price_minor=?4,reorder_level=?5,version=version+1 WHERE id=?6 AND version=?7",params![name,sku,input.cost_minor,input.price_minor,input.reorder_level,product,input.version]).map_err(db_error)?!=1 { return Err("Product changed. Refresh before saving.".into()); }
        } else {
            if input.version.is_some() {
                return Err("New products cannot have a version".into());
            }
            tx.execute(
                "INSERT INTO products VALUES (?1,?2,?3,?4,?5,?6,1)",
                params![
                    product,
                    name,
                    sku,
                    input.cost_minor,
                    input.price_minor,
                    input.reorder_level
                ],
            )
            .map_err(db_error)?;
        }
        let mut movements = vec![];
        if input.opening_stock > 0 {
            movements.push(movement(
                &tx,
                &product,
                None,
                input.opening_stock,
                "Opening",
                "Opening stock",
                &Utc::now().to_rfc3339(),
            )?);
        }
        record(
            &tx,
            "product",
            &product,
            input.version,
            before,
            json!({"id":product,"name":name,"sku":sku,"costMinor":input.cost_minor,"priceMinor":input.price_minor,"reorderLevel":input.reorder_level,"version":input.version.unwrap_or(0)+1,"stockMovements":movements}),
        )?;
        let result = json!({"id":product});
        receipt(&tx, "product", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn adjust_stock(&mut self, input: StockInput) -> Result<Value> {
        if input.amount != 1 && input.amount != -1 {
            return Err("Use a +1 or -1 stock adjustment".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(result) = replay(&tx, "stock", &input.request_id, &input)? {
            return Ok(result);
        }
        let row = movement(
            &tx,
            &input.product_id,
            None,
            input.amount,
            "Adjustment",
            &format!("Manual {:+1} adjustment", input.amount),
            &Utc::now().to_rfc3339(),
        )?;
        let movement_id = row["id"].as_str().unwrap();
        record(&tx, "stock_movement", movement_id, None, None, row.clone())?;
        let result = json!({"id":movement_id});
        receipt(&tx, "stock", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn complete_sale(&mut self, input: SaleInput) -> Result<Value> {
        if !(1..=1_000_000).contains(&input.quantity) {
            return Err("Enter a positive integer quantity".into());
        }
        choice(&input.method, &["Cash", "Card", "Transfer"], "sale method")?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if let Some(result) = replay(&tx, "sale", &input.request_id, &input)? {
            return Ok(result);
        }
        let (name, sku, price, cost): (String, String, i64, i64) = tx
            .query_row(
                "SELECT name,sku,price_minor,cost_minor FROM products WHERE id=?1",
                [&input.product_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(db_error)?;
        if stock(&tx, &input.product_id)? < input.quantity {
            return Err("Not enough stock for this sale".into());
        }
        let total = price
            .checked_mul(input.quantity)
            .ok_or("Sale total exceeds range")?;
        money(total, true)?;
        let sale = id();
        let item = id();
        let now = Utc::now();
        let day = business_date(now);
        let instant = now.to_rfc3339();
        tx.execute(
            "INSERT INTO sales VALUES (?1,?2,?3,?4,?5,?6,NULL)",
            params![
                sale,
                total,
                input.method,
                day,
                instant,
                super::desktop_auth::actor_label(&tx)?
            ],
        )
        .map_err(db_error)?;
        tx.execute(
            "INSERT INTO sale_items VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                item,
                sale,
                input.product_id,
                name,
                sku,
                input.quantity,
                price,
                cost
            ],
        )
        .map_err(db_error)?;
        let movement = movement(
            &tx,
            &input.product_id,
            Some(&item),
            -input.quantity,
            "Sale",
            "Retail sale",
            &instant,
        )?;
        record(
            &tx,
            "sale",
            &sale,
            None,
            None,
            json!({"id":sale,"totalMinor":total,"method":input.method,"businessOn":day,"createdAt":instant,"actor":super::desktop_auth::actor_label(&tx)?,"items":[{"id":item,"productId":input.product_id,"name":name,"sku":sku,"quantity":input.quantity,"priceMinor":price,"costMinor":cost}],"stockMovements":[movement]}),
        )?;
        let result = json!({"id":sale});
        receipt(&tx, "sale", &input.request_id, &input, &result)?;
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
}
