use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{path::Path, time::Duration};
use uuid::Uuid;

mod attendance_profiles;
mod business_review;
mod business_sync;
mod desktop_auth;
mod finance;
mod initial_profile_recovery;
mod staff_training;
pub use attendance_profiles::StaffAttendanceInput;
pub use business_review::BusinessRetryInput;
pub use desktop_auth::{
    DesktopAuth, DesktopAuthStatus, InitialProfileRecoveryOutcome, MemberSyncOutcome,
    PendingInitialProfileRecovery, PendingMemberSync, PendingSessionRenewal, SessionRenewal,
};
pub use initial_profile_recovery::InitialProfileRecoveryInput;
pub use staff_training::{
    CombinedPaymentInput, StaffMemberInput, StaffPayoutInput, StaffRegisterInput, TrainerInput,
    TrainingChargeInput,
};
mod member_conflicts;
pub use member_conflicts::MemberConflictInput;
mod member_http;
mod member_sync;
mod member_worker;
mod native_auth;
mod native_credentials;
mod native_https;
mod native_process;
mod offline_access;
pub use native_credentials::DeviceApproval;
mod removal;
pub use removal::{ExpenseVoidInput, MemberRemovalInput, StaffRemovalInput};
mod operations;
mod registration;
pub use finance::{
    AllocationInput, InvoiceInput, ReceivePaymentInput, RenewalInput, ReversalInput,
};
pub use registration::RegisterMemberInput;
mod recovery;
mod reports;
pub use operations::{
    AttendanceInput, ExpenseInput, PaymentInput, ProductInput, ProfileInput, SaleInput, StockInput,
};
pub use recovery::BackupEnvelope;
pub use reports::ReportRange;

const SCHEMA_VERSION: i64 = 11;
type Result<T> = std::result::Result<T, String>;
const ACTOR: &str = "local-test-operator (unauthenticated)";
fn id() -> String {
    Uuid::new_v4().to_string()
}
fn db_error(e: rusqlite::Error) -> String {
    format!("Database operation failed: {e}")
}
fn required(value: &str, label: &str, max: usize) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max {
        return Err(format!("{label} must contain 1–{max} characters"));
    }
    Ok(value.to_string())
}
fn date(value: &str) -> Result<NaiveDate> {
    let parsed = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| "Use a valid YYYY-MM-DD date".to_string())?;
    if parsed.format("%Y-%m-%d").to_string() != value
        || !("1900-01-01"..="2200-12-31").contains(&value)
    {
        return Err("Date must be between 1900 and 2200".into());
    }
    Ok(parsed)
}
pub fn business_date(now: DateTime<Utc>) -> String {
    now.with_timezone(&FixedOffset::east_opt(19_800).unwrap())
        .format("%Y-%m-%d")
        .to_string()
}
pub fn membership_status(starts: &str, ends: &str, today: &str) -> Result<&'static str> {
    let (start, end, today) = (date(starts)?, date(ends)?, date(today)?);
    Ok(if today < start {
        "Scheduled"
    } else if today > end {
        "Expired"
    } else if (end - today).num_days() <= 7 {
        "Expiring"
    } else {
        "Active"
    })
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanInput {
    pub id: Option<String>,
    pub version: Option<i64>,
    pub name: String,
    pub duration_months: i64,
    pub price_minor: i64,
    pub active: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberInput {
    pub id: Option<String>,
    pub version: Option<i64>,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub nfc_id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeriodInput {
    pub member_id: String,
    pub plan_id: String,
    pub starts_on: String,
    pub ends_on: String,
}

pub struct Store {
    conn: Connection,
    path: std::path::PathBuf,
    restore_candidate: Option<recovery::RestoreCandidate>,
    removal_session: Option<removal::Session>,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = Connection::open(path).map_err(db_error)?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(db_error)?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
        )
        .map_err(db_error)?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let version: i64 = tx
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(db_error)?;
        if version > SCHEMA_VERSION {
            return Err("Database belongs to a newer release. No data was replaced.".into());
        }
        integrity(&tx)?;
        if version == 0 {
            let tables: i64 = tx.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r| r.get(0)).map_err(db_error)?;
            if tables != 0 {
                return Err("Unrecognized database. No data was replaced.".into());
            }
            tx.execute_batch(include_str!("../migrations/001_foundation.sql"))
                .map_err(db_error)?;
            tx.execute("INSERT INTO metadata VALUES ('device_id', ?1)", [id()])
                .map_err(db_error)?;
        }
        if version <= 1 {
            tx.execute_batch(include_str!("../migrations/002_local_operations.sql"))
                .map_err(db_error)?;
        }
        let invalid_money: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM plans WHERE typeof(price_minor)<>'integer' OR typeof(duration_months)<>'integer') OR EXISTS(SELECT 1 FROM membership_periods WHERE typeof(price_minor)<>'integer')", [], |r| r.get(0)).map_err(db_error)?;
        if invalid_money {
            return Err("Invalid existing price/duration. No records were replaced.".into());
        }
        if version <= 2 {
            let invalid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM payments r JOIN payments p ON p.id=r.reverses_id WHERE p.reverses_id IS NOT NULL OR p.member_id<>r.member_id OR p.member_name<>r.member_name OR p.amount_minor<>r.amount_minor OR p.method<>r.method)",[],|r|r.get(0)).map_err(db_error)?;
            if invalid {
                return Err("Existing reversal conflicts with its original payment; migration stopped without replacing records".into());
            }
            tx.execute_batch(include_str!("../migrations/003_finance.sql"))
                .map_err(db_error)?;
            finance::migrate_receipts(&tx)?;
        }
        if version <= 3 {
            let invalid: bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM expenses r JOIN expenses p ON p.id=r.reverses_id WHERE p.reverses_id IS NOT NULL OR p.amount_minor<>r.amount_minor OR p.title<>r.title OR p.category<>r.category OR p.method<>r.method)",[],|r|r.get(0)).map_err(db_error)?;
            if invalid {
                return Err("Existing expense reversal conflicts with its original; migration stopped without replacing records".into());
            }
            tx.execute_batch(include_str!("../migrations/004_removal.sql"))
                .map_err(db_error)?;
        }
        if version <= 4 {
            tx.execute_batch(include_str!("../migrations/005_member_sync.sql"))
                .map_err(db_error)?;
        }
        if version <= 5 {
            tx.execute_batch(include_str!("../migrations/006_member_conflicts.sql"))
                .map_err(db_error)?;
        }
        if version <= 6 {
            tx.execute_batch(include_str!("../migrations/007_business_sync.sql"))
                .map_err(db_error)?;
        }
        if version <= 7 {
            tx.execute_batch(include_str!("../migrations/008_staff_training.sql"))
                .map_err(db_error)?;
        }
        if version <= 8 {
            tx.execute_batch(include_str!("../migrations/009_attendance_profiles.sql"))
                .map_err(db_error)?;
        }
        if version <= 9 {
            tx.execute_batch(include_str!("../migrations/010_staff_removal.sql"))
                .map_err(db_error)?;
        }
        if version <= 10 {
            tx.execute_batch(include_str!("../migrations/011_admission.sql"))
                .map_err(db_error)?;
        }
        integrity(&tx)?;
        tx.commit().map_err(db_error)?;
        let integrity: String = conn
            .query_row("PRAGMA quick_check", [], |r| r.get(0))
            .map_err(db_error)?;
        if integrity != "ok" {
            return Err("Database integrity check failed. Keep the database for recovery.".into());
        }
        conn.execute_batch(
            "CREATE TEMP TABLE native_actor(user_id TEXT NOT NULL,label TEXT NOT NULL)",
        )
        .map_err(db_error)?;
        Ok(Self {
            conn,
            path: path.to_path_buf(),
            restore_candidate: None,
            removal_session: None,
        })
    }
    pub fn snapshot(&self) -> Result<Value> {
        let tx = self.conn.unchecked_transaction().map_err(db_error)?;
        let mut result = snapshot_on(&tx)?;
        result["removalAuthorization"] = removal::authorization(&tx, self.removal_session.as_ref());
        result["memberSync"]["reviewAuthorization"] = member_conflicts::authorization(self);
        tx.commit().map_err(db_error)?;
        Ok(result)
    }
    pub fn save_plan(&mut self, input: PlanInput) -> Result<()> {
        let name = required(&input.name, "Plan name", 80)?;
        if !(1..=60).contains(&input.duration_months)
            || !(0..=100_000_000_000).contains(&input.price_minor)
        {
            return Err("Invalid duration or price".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let plan_id = input.id.clone().unwrap_or_else(id);
        let before = if input.id.is_some() {
            tx.query_row("SELECT json_object('name',name,'durationMonths',duration_months,'priceMinor',price_minor,'active',active,'version',version) FROM plans WHERE id=?1", [&plan_id], |r| r.get::<_,String>(0)).optional().map_err(db_error)?
        } else {
            None
        };
        if input.id.is_some() {
            let changed = tx.execute("UPDATE plans SET name=?1,duration_months=?2,price_minor=?3,active=?4,version=version+1 WHERE id=?5 AND version=?6",params![name,input.duration_months,input.price_minor,input.active,plan_id,input.version]).map_err(db_error)?;
            if changed != 1 {
                return Err("Plan changed or no longer exists. Refresh and try again.".into());
            }
        } else {
            if input.version.is_some() {
                return Err("New plans cannot have a version".into());
            }
            tx.execute(
                "INSERT INTO plans VALUES (?1,?2,?3,?4,?5,1)",
                params![
                    plan_id,
                    name,
                    input.duration_months,
                    input.price_minor,
                    input.active
                ],
            )
            .map_err(db_error)?;
        }
        let payload = json!({"id":plan_id,"name":name,"durationMonths":input.duration_months,"priceMinor":input.price_minor,"active":input.active,"version":input.version.unwrap_or(0)+1});
        record(&tx, "plan", &plan_id, input.version, before, payload)?;
        tx.commit().map_err(db_error)
    }
    pub fn save_member(&mut self, input: MemberInput) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        save_member_on(&tx, input)?;
        business_sync::capture(&tx)?;
        tx.commit().map_err(db_error)
    }
    pub fn add_period(&mut self, input: PeriodInput) -> Result<()> {
        if date(&input.ends_on)? < date(&input.starts_on)? {
            return Err("End date must be on or after start date".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let plan: Option<(String, i64)> = tx
            .query_row(
                "SELECT name,price_minor FROM plans WHERE id=?1 AND active=1",
                [&input.plan_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (plan_name, price) = plan.ok_or("Select an active plan")?;
        let period_id = id();
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO membership_periods VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                period_id,
                input.member_id,
                input.plan_id,
                plan_name,
                price,
                input.starts_on,
                input.ends_on,
                now
            ],
        )
        .map_err(db_error)?;
        let payload = json!({"id":period_id,"memberId":input.member_id,"planId":input.plan_id,"planName":plan_name,"priceMinor":price,"startsOn":input.starts_on,"endsOn":input.ends_on,"createdAt":now});
        record(&tx, "membership_period", &period_id, None, None, payload)?;
        tx.commit().map_err(db_error)
    }
}
fn save_member_on(tx: &rusqlite::Transaction<'_>, input: MemberInput) -> Result<String> {
    let name = required(&input.name, "Name", 120)?;
    let phone = required(&input.phone, "Phone", 40)?;
    let email = input.email.trim();
    if email.len() > 254
        || (!email.is_empty() && (!email.contains('@') || email.chars().any(char::is_whitespace)))
    {
        return Err("Enter a valid email or leave it blank".into());
    }
    let nfc = input.nfc_id.trim().to_ascii_uppercase();
    if nfc.len() > 128
        || !nfc.is_ascii()
        || nfc.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("Card ID must be at most 128 ASCII characters without spaces".into());
    }
    let nfc = if nfc.is_empty() { None } else { Some(nfc) };
    let member_id = input.id.clone().unwrap_or_else(id);
    let duplicate: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM members WHERE nfc_id=?1 AND id<>?2)",
            params![nfc, member_id],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if duplicate {
        return Err("This NFC card is already assigned to another member".into());
    }
    let before = if input.id.is_some() {
        tx.query_row("SELECT json_object('name',name,'phone',phone,'email',email,'nfcId',nfc_id,'version',version) FROM members WHERE id=?1", [&member_id], |r| r.get::<_,String>(0)).optional().map_err(db_error)?
    } else {
        None
    };
    if input.id.is_some() {
        let changed = tx.execute("UPDATE members SET name=?1,phone=?2,email=?3,nfc_id=?4,version=version+1 WHERE id=?5 AND version=?6 AND archived_at IS NULL",params![name,phone,email,nfc,member_id,input.version]).map_err(db_error)?;
        if changed != 1 {
            return Err("Member changed or no longer exists. Refresh and try again.".into());
        }
    } else {
        if input.version.is_some() {
            return Err("New members cannot have a version".into());
        }
        tx.execute(
                "INSERT INTO members(id,name,phone,email,nfc_id,joined_on,version) VALUES (?1,?2,?3,?4,?5,?6,1)",
                params![
                    member_id,
                    name,
                    phone,
                    email,
                    nfc,
                    business_date(Utc::now())
                ],
            )
            .map_err(db_error)?;
    }
    let joined: String = tx
        .query_row(
            "SELECT joined_on FROM members WHERE id=?1",
            [&member_id],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let cards: String = tx.query_row("SELECT json_group_array(json_object('id',id,'uid',uid,'assignedAt',assigned_at,'revokedAt',revoked_at)) FROM nfc_cards WHERE member_id=?1",[&member_id],|r|r.get(0)).map_err(db_error)?;
    let cards: Value = serde_json::from_str(&cards).map_err(|e| e.to_string())?;
    let payload = json!({"id":member_id,"name":name,"phone":phone,"email":email,"nfcId":nfc,"joinedOn":joined,"version":input.version.unwrap_or(0)+1,"cards":cards});
    record_uncaptured(tx, "member", &member_id, input.version, before, payload)?;
    Ok(member_id)
}
fn record(
    tx: &rusqlite::Transaction<'_>,
    entity: &str,
    entity_id: &str,
    expected_version: Option<i64>,
    before: Option<String>,
    payload: Value,
) -> Result<()> {
    record_uncaptured(tx, entity, entity_id, expected_version, before, payload)?;
    business_sync::capture(tx)
}
fn record_uncaptured(
    tx: &rusqlite::Transaction<'_>,
    entity: &str,
    entity_id: &str,
    expected_version: Option<i64>,
    before: Option<String>,
    mut payload: Value,
) -> Result<()> {
    let device: String = tx
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let now = Utc::now().to_rfc3339();
    let action = if expected_version.is_some() {
        "update"
    } else {
        "create"
    };
    let actor = desktop_auth::actor_label(tx)?;
    let actor_user: Option<String> = tx
        .query_row("SELECT user_id FROM temp.native_actor LIMIT 1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(db_error)?;
    if let Some(user) = &actor_user {
        payload["actorUserId"] = json!(user);
        payload["actor"] = json!(actor);
    }
    let payload = payload.to_string();
    tx.execute(
        "INSERT INTO audit (id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity,actor_user_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            id(),
            actor,
            device,
            format!("{action} {entity}"),
            entity_id,
            before,
            payload,
            now,
            entity,
            actor_user
        ],
    )
    .map_err(db_error)?;
    tx.execute("INSERT INTO outbox (id,device_id,entity,entity_id,action,expected_version,payload_json,schema_version,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![id(),device,entity,entity_id,action,expected_version,payload,SCHEMA_VERSION,now]).map_err(db_error)?;
    Ok(())
}
fn integrity(conn: &Connection) -> Result<()> {
    let check: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(db_error)?;
    if check != "ok" {
        return Err(format!(
            "Database integrity failed: {check}. Preserve storage for recovery."
        ));
    }
    let mut stmt = conn.prepare("PRAGMA foreign_key_check").map_err(db_error)?;
    if stmt
        .query([])
        .map_err(db_error)?
        .next()
        .map_err(db_error)?
        .is_some()
    {
        return Err("Database has broken foreign keys. Preserve storage for recovery.".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests;

fn snapshot_on(conn: &Connection) -> Result<Value> {
    let today = business_date(Utc::now());
    let mut stmt = conn.prepare("SELECT id,name,duration_months,price_minor,active,version,(SELECT count(DISTINCT p.member_id) FROM membership_periods p JOIN members m ON m.id=p.member_id WHERE p.plan_id=plans.id AND p.starts_on<=?1 AND p.ends_on>=?1 AND m.archived_at IS NULL) FROM plans ORDER BY name").map_err(db_error)?;
    let plans = stmt.query_map([&today], |r| Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"durationMonths":r.get::<_,i64>(2)?,"priceMinor":r.get::<_,i64>(3)?,"active":r.get::<_,bool>(4)?,"version":r.get::<_,i64>(5)?,"activeMembers":r.get::<_,i64>(6)?}))).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
    drop(stmt);
    let mut stmt = conn
        .prepare("SELECT id,name,phone,email,nfc_id,joined_on,version FROM members ORDER BY name")
        .map_err(db_error)?;
    let members = stmt.query_map([], |r| Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"phone":r.get::<_,String>(2)?,"email":r.get::<_,String>(3)?,"nfcId":r.get::<_,Option<String>>(4)?.unwrap_or_default(),"joinedOn":r.get::<_,String>(5)?,"version":r.get::<_,i64>(6)?}))).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
    drop(stmt);
    let mut stmt = conn.prepare("SELECT id,member_id,plan_id,plan_name,price_minor,starts_on,ends_on FROM membership_periods ORDER BY starts_on DESC,id").map_err(db_error)?;
    let mut periods = stmt.query_map([], |r| Ok(json!({"id":r.get::<_,String>(0)?,"memberId":r.get::<_,String>(1)?,"planId":r.get::<_,String>(2)?,"planName":r.get::<_,String>(3)?,"priceMinor":r.get::<_,i64>(4)?,"startsOn":r.get::<_,String>(5)?,"endsOn":r.get::<_,String>(6)?}))).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
    drop(stmt);
    for p in &mut periods {
        p["status"] = json!(membership_status(
            p["startsOn"].as_str().unwrap(),
            p["endsOn"].as_str().unwrap(),
            &today
        )?);
    }
    let pending: i64 = conn
        .query_row("SELECT count(*) FROM outbox o WHERE NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=o.id AND d.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=o.id) AND NOT EXISTS(SELECT 1 FROM business_batch_operations bo JOIN business_batches b ON b.id=bo.batch_id WHERE bo.operation_id=o.id AND b.state='confirmed')", [], |r| r.get(0))
        .map_err(db_error)?;
    let audits: i64 = conn
        .query_row("SELECT count(*) FROM audit", [], |r| r.get(0))
        .map_err(db_error)?;
    let mut result = json!({"plans":plans,"members":members,"periods":periods,"pending":pending,"auditCount":audits,"today":today});
    operations::append_snapshot(conn, &mut result)?;
    finance::append_snapshot(conn, &mut result)?;
    staff_training::append_snapshot(conn, &mut result)?;
    removal::append_snapshot(conn, &mut result)?;
    attendance_profiles::append_snapshot(conn, &mut result)?;
    member_sync::append_snapshot(conn, &mut result)?;
    result["businessSync"] = business_sync::status(conn)?;
    Ok(result)
}
