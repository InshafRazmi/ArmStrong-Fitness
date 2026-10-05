// Real temporary SQLite and mock transport. Live Auth/HTTPS is tested separately.
use super::*;
use crate::member_sync::{Page, Receipt};
use crate::member_worker::{
    BusinessTransport, Limits, MemberTransport, RemoteFailure, Run, SyncScope,
};

struct Fixture {
    store: Store,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new(subject: &str, gym: &str, write: bool) -> Self {
        let path = std::env::temp_dir().join(format!("armstrong-business-{}.sqlite3", id()));
        let mut store = Store::open(&path).unwrap();
        store
            .conn
            .execute(
                "INSERT INTO users VALUES(?1,?1,'actor@example.invalid','Administrator',1,1)",
                [subject],
            )
            .unwrap();
        store
            .conn
            .execute("INSERT INTO roles VALUES('admin','Administrator')", [])
            .unwrap();
        store
            .conn
            .execute("INSERT INTO user_roles VALUES(?1,'admin')", [subject])
            .unwrap();
        store
            .conn
            .execute(
                "INSERT INTO temp.native_actor VALUES(?1,?2)",
                params![subject, format!("Administrator ({subject})")],
            )
            .unwrap();
        store
            .conn
            .execute(
                "INSERT INTO metadata VALUES('native_session_nonce','nonce')",
                [],
            )
            .unwrap();
        store.removal_session = Some(removal::Session {
            user_id: subject.into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            can_write: write,
            native_nonce: Some("nonce".into()),
        });
        let device: String = store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        store
            .bind_member_scope(SyncScope::new("https://gym.example", gym, &device).unwrap())
            .unwrap();
        Self { store, path }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(format!("{}-wal", self.path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", self.path.display()));
        let _ = std::fs::remove_dir_all(self.path.with_extension("backups"));
    }
}
struct Mock {
    scope: SyncScope,
    subject: String,
    changes: Vec<Value>,
    drop_reply: bool,
    bad_receipt: bool,
    after_push: Option<Box<dyn FnOnce()>>,
    after_pull: Option<Box<dyn FnOnce()>>,
}
impl Mock {
    fn for_store(store: &Store, subject: &str, gym: &str) -> Self {
        let device: String = store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        Self {
            scope: SyncScope::new("https://gym.example", gym, &device).unwrap(),
            subject: subject.into(),
            changes: vec![],
            drop_reply: false,
            bad_receipt: false,
            after_push: None,
            after_pull: None,
        }
    }
}
impl MemberTransport for Mock {
    fn scope(&self) -> &SyncScope {
        &self.scope
    }
    fn subject(&self) -> &str {
        &self.subject
    }
    fn push(&mut self, _: &Value) -> std::result::Result<Receipt, RemoteFailure> {
        panic!("Business sync must not use protocol 1")
    }
    fn pull(&mut self, _: i64) -> std::result::Result<Page, RemoteFailure> {
        panic!("Business sync must not use protocol 1")
    }
}
impl BusinessTransport for Mock {
    fn push_business(&mut self, request: &Value) -> std::result::Result<Value, RemoteFailure> {
        let prior = self
            .changes
            .iter()
            .find(|c| c["request"]["operationId"] == request["operationId"]);
        let receipt = if let Some(prior) = prior {
            assert_eq!(prior["request"], *request);
            prior["receipt"].clone()
        } else {
            let sequence = self.changes.len() + 1;
            let receipt = json!({"protocolVersion":2,"operationId":request["operationId"],"deviceId":request["deviceId"],"gymId":self.scope.gym_id,"actorSubject":self.subject,"sequence":sequence,"requestSha256":hash(request).unwrap()});
            self.changes
                .push(json!({"sequence":sequence,"request":request,"receipt":receipt}));
            receipt
        };
        if let Some(callback) = self.after_push.take() {
            callback();
        }
        if std::mem::take(&mut self.drop_reply) {
            return Err(RemoteFailure::Transient {
                retry_after_seconds: None,
            });
        }
        let mut receipt = receipt;
        if self.bad_receipt {
            receipt["requestSha256"] = json!("0".repeat(64));
        }
        Ok(receipt)
    }
    fn pull_business(&mut self, after: i64) -> std::result::Result<Value, RemoteFailure> {
        let changes = self
            .changes
            .iter()
            .filter(|c| c["sequence"].as_i64().unwrap() > after)
            .take(1)
            .cloned()
            .collect::<Vec<_>>();
        let next = changes
            .last()
            .and_then(|c| c["sequence"].as_i64())
            .unwrap_or(after);
        let page = json!({"protocolVersion":2,"gymId":self.scope.gym_id,"after":after,"nextCursor":next,"hasMore":next<(self.changes.len() as i64),"changes":changes});
        if let Some(callback) = self.after_pull.take() {
            callback();
        }
        Ok(page)
    }
}
fn setup_operations(store: &mut Store) -> (String, String) {
    store
        .save_plan(PlanInput {
            id: None,
            version: None,
            name: "Monthly".into(),
            duration_months: 1,
            price_minor: 600_000,
            active: true,
        })
        .unwrap();
    store
        .save_member(MemberInput {
            id: None,
            version: None,
            name: "Member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "CARD1".into(),
        })
        .unwrap();
    let snap = store.snapshot().unwrap();
    let member = snap["members"][0]["id"].as_str().unwrap().to_string();
    let plan = snap["plans"][0]["id"].as_str().unwrap().to_string();
    let renewal = store
        .renew_membership(RenewalInput {
            request_id: id(),
            member_id: member.clone(),
            plan_id: plan,
            plan_version: 1,
            expected_last_period_id: None,
            starts_on: "2026-10-05".into(),
            ends_on: "2026-11-04".into(),
        })
        .unwrap();
    let payment = store
        .receive_payment(ReceivePaymentInput {
            request_id: id(),
            member_id: member.clone(),
            amount_minor: 300_000,
            method: "Cash".into(),
            invoice_id: Some(renewal["invoiceId"].as_str().unwrap().into()),
        })
        .unwrap();
    store
        .record_attendance(AttendanceInput {
            request_id: id(),
            member_or_card: "CARD1".into(),
            source: "NFC".into(),
        })
        .unwrap();
    let product = store
        .save_product(ProductInput {
            request_id: id(),
            id: None,
            version: None,
            name: "Bottle".into(),
            sku: "BOTTLE".into(),
            cost_minor: 5000,
            price_minor: 10_000,
            reorder_level: 1,
            opening_stock: 2,
        })
        .unwrap();
    store
        .complete_sale(SaleInput {
            request_id: id(),
            product_id: product["id"].as_str().unwrap().into(),
            quantity: 1,
            method: "Cash".into(),
        })
        .unwrap();
    let expense = store
        .record_expense(ExpenseInput {
            request_id: id(),
            title: "Lights".into(),
            category: "Utilities".into(),
            amount_minor: 10_000,
            method: "Cash".into(),
        })
        .unwrap();
    store
        .void_expense(ExpenseVoidInput {
            request_id: id(),
            expense_id: expense["id"].as_str().unwrap().into(),
            reason: "Entered twice".into(),
        })
        .unwrap();
    store
        .reverse_payment(ReversalInput {
            request_id: id(),
            payment_id: payment["id"].as_str().unwrap().into(),
            reason: "Correction".into(),
        })
        .unwrap();
    (member, payment["id"].as_str().unwrap().into())
}
#[test]
fn business_all_modules_download_exact_receipts_ledgers_and_audit_without_granting_roles() {
    let subject = id();
    let gym = id();
    let mut writer = Fixture::new(&subject, &gym, true);
    let mut cloud = Mock::for_store(&writer.store, &subject, &gym);
    let (member, payment) = setup_operations(&mut writer.store);
    assert!(matches!(
        writer
            .store
            .run_business_sync(
                &mut cloud,
                Utc::now(),
                Limits {
                    pushes: 100,
                    pages: 100
                }
            )
            .unwrap(),
        Run::Complete { .. }
    ));
    assert_eq!(writer.store.snapshot().unwrap()["pending"], 0);
    let mut reader = Fixture::new(&subject, &gym, false);
    let mut download = Mock::for_store(&reader.store, &subject, &gym);
    download.changes = cloud.changes;
    assert!(matches!(
        reader
            .store
            .run_business_sync(
                &mut download,
                Utc::now(),
                Limits {
                    pushes: 100,
                    pages: 100
                }
            )
            .unwrap(),
        Run::Complete { .. }
    ));
    let a = writer.store.snapshot().unwrap();
    let b = reader.store.snapshot().unwrap();
    for key in [
        "members",
        "plans",
        "periods",
        "invoices",
        "payments",
        "allocations",
        "financialAccounts",
        "products",
        "sales",
        "attendance",
        "expenses",
        "audit",
    ] {
        assert_eq!(a[key], b[key], "{key}");
    }
    assert_eq!(
        writer.store.payment_receipt(payment.clone()).unwrap(),
        reader.store.payment_receipt(payment).unwrap()
    );
    assert_eq!(b["products"][0]["stock"], 1);
    assert_eq!(b["members"][0]["id"], member);
    assert_eq!(b["users"][0]["active"], 1); // Existing real enrollment, not cloud activation.
    assert_eq!(
        reader
            .store
            .conn
            .query_row("SELECT count(*) FROM user_roles", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    if let Ok(path) = std::env::var("ARMSTRONG_BUSINESS_FIXTURE_PATH") {
        std::fs::write(path, serde_json::to_vec(&download.changes).unwrap()).unwrap();
    }
}
#[test]
fn business_dropped_response_restart_retries_same_request_and_keeps_outbox_history() {
    let subject = id();
    let gym = id();
    let mut f = Fixture::new(&subject, &gym, true);
    let mut cloud = Mock::for_store(&f.store, &subject, &gym);
    f.store
        .save_member(MemberInput {
            id: None,
            version: None,
            name: "Retry member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "".into(),
        })
        .unwrap();
    cloud.drop_reply = true;
    assert_eq!(
        f.store
            .run_business_sync(&mut cloud, Utc::now(), Limits::default())
            .unwrap(),
        Run::Failed
    );
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert_eq!(cloud.changes.len(), 1);
    let session = f.store.removal_session.clone();
    let mut reopened = Store::open(&f.path).unwrap();
    reopened.removal_session = session;
    assert!(matches!(
        reopened
            .run_business_sync(
                &mut cloud,
                Utc::now() + chrono::Duration::seconds(10),
                Limits::default()
            )
            .unwrap(),
        Run::Complete { .. }
    ));
    assert_eq!(cloud.changes.len(), 1);
    assert_eq!(reopened.snapshot().unwrap()["pending"], 0);
    assert_eq!(
        reopened
            .conn
            .query_row("SELECT count(*) FROM outbox", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn business_mismatched_receipt_and_logout_during_io_cannot_acknowledge() {
    let subject = id();
    let gym = id();
    let mut f = Fixture::new(&subject, &gym, true);
    let mut cloud = Mock::for_store(&f.store, &subject, &gym);
    f.store
        .save_plan(PlanInput {
            id: None,
            version: None,
            name: "Monthly".into(),
            duration_months: 1,
            price_minor: 600_000,
            active: true,
        })
        .unwrap();
    cloud.bad_receipt = true;
    assert_eq!(
        f.store
            .run_business_sync(&mut cloud, Utc::now(), Limits::default())
            .unwrap(),
        Run::Failed
    );
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    cloud.bad_receipt = false;
    let path = f.path.clone();
    cloud.after_push = Some(Box::new(move || {
        Connection::open(path)
            .unwrap()
            .execute(
                "UPDATE metadata SET value='logout' WHERE key='native_session_nonce'",
                [],
            )
            .unwrap();
    }));
    assert!(f
        .store
        .run_business_sync(
            &mut cloud,
            Utc::now() + chrono::Duration::seconds(10),
            Limits::default()
        )
        .is_err());
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
}
#[test]
fn business_failure_before_capture_commit_rolls_back_records_audit_and_queue() {
    let subject = id();
    let gym = id();
    let mut f = Fixture::new(&subject, &gym, true);
    f.store.conn.execute_batch("CREATE TRIGGER fail_business BEFORE INSERT ON business_batches BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(f
        .store
        .save_plan(PlanInput {
            id: None,
            version: None,
            name: "Monthly".into(),
            duration_months: 1,
            price_minor: 600_000,
            active: true
        })
        .is_err());
    let snapshot = f.store.snapshot().unwrap();
    assert_eq!(snapshot["plans"], json!([]));
    assert_eq!(snapshot["pending"], 0);
    assert_eq!(snapshot["auditCount"], 0);
}
