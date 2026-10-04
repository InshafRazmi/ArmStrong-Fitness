use super::*;
#[path = "member_conflict_tests.rs"]
mod member_conflict_tests;
#[path = "member_sync_engine_tests.rs"]
mod member_sync_engine_tests;
#[path = "member_sync_tests.rs"]
mod member_sync_tests;
#[path = "report_tests.rs"]
mod report_tests;
struct Fixture {
    path: std::path::PathBuf,
    store: Store,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("armstrong-test-{}.sqlite3", id()));
        let store = Store::open(&path).unwrap();
        Self { path, store }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        drop(std::mem::replace(
            &mut self.store.conn,
            Connection::open_in_memory().unwrap(),
        ));
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(format!("{}-wal", self.path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", self.path.display()));
        let _ = std::fs::remove_dir_all(self.path.with_extension("backups"));
    }
}
fn plan() -> PlanInput {
    PlanInput {
        id: None,
        version: None,
        name: "Monthly".into(),
        duration_months: 1,
        price_minor: 600000,
        active: true,
    }
}
fn member(card: &str) -> MemberInput {
    MemberInput {
        id: None,
        version: None,
        name: "Test member".into(),
        phone: "0771234567".into(),
        email: "".into(),
        nfc_id: card.into(),
    }
}
fn first_id(store: &Store, kind: &str) -> String {
    store.snapshot().unwrap()[kind][0]["id"]
        .as_str()
        .unwrap()
        .into()
}
fn period(store: &Store) -> PeriodInput {
    PeriodInput {
        member_id: first_id(store, "members"),
        plan_id: first_id(store, "plans"),
        starts_on: "2026-01-31".into(),
        ends_on: "2026-02-28".into(),
    }
}
#[test]
fn empty_migration_and_restart_keep_records_and_pending_operations() {
    let mut f = Fixture::new();
    assert_eq!(f.store.snapshot().unwrap()["members"], json!([]));
    f.store.save_plan(plan()).unwrap();
    f.store.save_member(member(" abc123 ")).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    drop(std::mem::replace(
        &mut f.store.conn,
        Connection::open_in_memory().unwrap(),
    ));
    let reopened = Store::open(&f.path).unwrap();
    let s = reopened.snapshot().unwrap();
    assert_eq!(s["members"][0]["nfcId"], "ABC123");
    assert_eq!(s["periods"][0]["endsOn"], "2026-02-28");
    assert_eq!(s["pending"], 3);
    assert_eq!(s["auditCount"], 3);
    let version: i64 = reopened
        .conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
}
#[test]
fn cards_are_unique_case_insensitively_but_blank_cards_are_allowed() {
    let mut f = Fixture::new();
    f.store.save_member(member("aBc")).unwrap();
    assert!(f
        .store
        .save_member(member(" ABC "))
        .unwrap_err()
        .contains("already assigned"));
    f.store.save_member(member("")).unwrap();
    f.store.save_member(member(" ")).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["pending"], 3);
}
#[test]
fn outbox_failure_rolls_back_business_record_and_audit() {
    let mut f = Fixture::new();
    f.store.conn.execute_batch("CREATE TRIGGER fail_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT, 'injected failure'); END;").unwrap();
    assert!(f.store.save_member(member("X")).is_err());
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"], json!([]));
    assert_eq!(s["auditCount"], 0);
    assert_eq!(s["pending"], 0);
}
#[test]
fn stale_edit_is_rejected_without_losing_committed_edit() {
    let mut f = Fixture::new();
    f.store.save_member(member("X")).unwrap();
    let mut edit = member("X");
    edit.id = Some(first_id(&f.store, "members"));
    edit.version = Some(1);
    edit.name = "First edit".into();
    f.store.save_member(edit.clone()).unwrap();
    edit.name = "Stale edit".into();
    assert!(f.store.save_member(edit).is_err());
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["name"], "First edit");
    assert_eq!(s["pending"], 2);
}
#[test]
fn overlapping_invalid_and_orphan_memberships_are_rejected() {
    let mut f = Fixture::new();
    f.store.save_plan(plan()).unwrap();
    f.store.save_member(member("")).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    assert!(f.store.add_period(period(&f.store)).is_err());
    let mut p = period(&f.store);
    p.starts_on = "2026-02-29".into();
    assert!(f.store.add_period(p).is_err());
    let mut p = period(&f.store);
    p.member_id = "missing".into();
    assert!(f.store.add_period(p).is_err());
    let mut p = period(&f.store);
    p.starts_on = "2026-03-01".into();
    assert!(f.store.add_period(p).is_err());
    let mut p = period(&f.store);
    p.starts_on = "2026-03-01".into();
    p.ends_on = "2026-03-31".into();
    f.store.add_period(p).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["pending"], 4);
}
#[test]
fn renaming_plan_preserves_period_price_and_name_snapshot() {
    let mut f = Fixture::new();
    f.store.save_plan(plan()).unwrap();
    f.store.save_member(member("")).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    let mut edit = plan();
    edit.id = Some(first_id(&f.store, "plans"));
    edit.version = Some(1);
    edit.name = "New name".into();
    edit.price_minor = 900000;
    edit.active = false;
    f.store.save_plan(edit.clone()).unwrap();
    assert!(f.store.save_plan(edit).is_err());
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["periods"][0]["planName"], "Monthly");
    assert_eq!(s["periods"][0]["priceMinor"], 600000);
    let mut p = period(&f.store);
    p.starts_on = "2026-03-01".into();
    p.ends_on = "2026-03-31".into();
    assert!(f.store.add_period(p).is_err());
}
#[test]
fn local_dates_and_inclusive_status_boundaries() {
    let utc = "2026-10-02T19:00:00Z".parse::<DateTime<Utc>>().unwrap();
    assert_eq!(business_date(utc), "2026-10-03");
    assert_eq!(
        membership_status("2026-01-31", "2026-02-28", "2026-01-30").unwrap(),
        "Scheduled"
    );
    assert_eq!(
        membership_status("2026-01-31", "2026-02-28", "2026-01-31").unwrap(),
        "Active"
    );
    assert_eq!(
        membership_status("2026-01-31", "2026-02-28", "2026-02-28").unwrap(),
        "Expiring"
    );
    assert_eq!(
        membership_status("2026-01-31", "2026-02-28", "2026-03-01").unwrap(),
        "Expired"
    );
    assert!(date("2024-02-29").is_ok());
    assert!(date("2026-02-29").is_err());
}
#[test]
fn counts_are_derived_and_validation_prevents_invalid_money() {
    let mut f = Fixture::new();
    let mut bad = plan();
    bad.price_minor = -1;
    assert!(f.store.save_plan(bad).is_err());
    let mut bad = plan();
    bad.duration_months = 0;
    assert!(f.store.save_plan(bad).is_err());
    assert!(f.store.save_member(member("has spaces")).is_err());
    f.store.save_plan(plan()).unwrap();
    f.store.save_member(member("")).unwrap();
    let today = business_date(Utc::now());
    let mut p = period(&f.store);
    p.starts_on = today.clone();
    p.ends_on = today;
    f.store.add_period(p).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["plans"][0]["activeMembers"], 1);
}
#[test]
fn newer_or_corrupt_database_is_not_replaced() {
    let f = Fixture::new();
    f.store
        .conn
        .pragma_update(None, "user_version", 99)
        .unwrap();
    assert!(Store::open(&f.path).is_err());
    assert_eq!(
        f.store
            .conn
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        99
    );
    let path = std::env::temp_dir().join(format!("armstrong-corrupt-{}", id()));
    std::fs::write(&path, b"not a database").unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"not a database");
    std::fs::remove_file(path).unwrap();
}

fn product() -> ProductInput {
    ProductInput {
        request_id: id(),
        id: None,
        version: None,
        name: "Water".into(),
        sku: "WATER-1".into(),
        cost_minor: 10025,
        price_minor: 20050,
        reorder_level: 2,
        opening_stock: 5,
    }
}
fn payment(store: &Store) -> PaymentInput {
    PaymentInput {
        request_id: id(),
        member_id: first_id(store, "members"),
        amount_minor: 600050,
        method: "Cash".into(),
    }
}
fn expense() -> ExpenseInput {
    ExpenseInput {
        request_id: id(),
        title: "Electricity".into(),
        category: "Utilities".into(),
        amount_minor: 125075,
        method: "Bank".into(),
    }
}
fn profile() -> ProfileInput {
    ProfileInput {
        version: 1,
        name: "Armstrong Fitness".into(),
        location: "Matale".into(),
        phone: "0661234567".into(),
        email: "gym@example.lk".into(),
    }
}
fn v1_database() -> (std::path::PathBuf, Connection) {
    let path = std::env::temp_dir().join(format!("armstrong-v1-{}.sqlite3", id()));
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(include_str!("../migrations/001_foundation.sql"))
        .unwrap();
    conn.execute_batch("INSERT INTO metadata VALUES('device_id','existing-device'); INSERT INTO plans VALUES('old-plan','Original',1,600050,1,2); INSERT INTO members VALUES('old-member','Existing member','0771234567','','OLD-CARD','2026-01-01',3); INSERT INTO membership_periods VALUES('old-period','old-member','old-plan','Original at sale',500025,'2026-01-01','2026-01-31','2026-01-01T00:00:00Z'); INSERT INTO audit VALUES('old-audit','old-actor','existing-device','create member','old-member',NULL,'{}','2026-01-01T00:00:00Z'); INSERT INTO outbox VALUES('old-operation','existing-device','member','old-member','create',NULL,'{\"oldPayload\":true}',1,'2026-01-01T00:00:00Z',4);").unwrap();
    (path, conn)
}
#[test]
fn populated_v1_migration_preserves_all_existing_data_and_is_idempotent() {
    let (path, conn) = v1_database();
    drop(conn);
    let f = Fixture {
        store: Store::open(&path).unwrap(),
        path,
    };
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["id"], "old-member");
    assert_eq!(s["members"][0]["version"], 3);
    assert_eq!(s["periods"][0]["priceMinor"], 500025);
    assert_eq!(s["pending"], 1);
    assert_eq!(s["auditCount"], 1);
    assert_eq!(
        f.store
            .conn
            .query_row(
                "SELECT payload_json FROM outbox WHERE id='old-operation'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "{\"oldPayload\":true}"
    );
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT schema_version FROM outbox", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT attempts FROM outbox", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT uid FROM nfc_cards", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "OLD-CARD"
    );
    let reopened = Store::open(&f.path).unwrap();
    assert_eq!(reopened.snapshot().unwrap(), s);
}
#[test]
fn migration_with_orphan_or_invalid_existing_money_rolls_back_without_replacing_data() {
    for bad_sql in ["INSERT INTO membership_periods VALUES('orphan','missing','old-plan','Old',1,'2027-01-01','2027-02-01','2026-01-01');", "UPDATE plans SET price_minor=6000.5;", "UPDATE membership_periods SET price_minor=5000.25;"] {
        let (path,conn)=v1_database(); conn.execute_batch("PRAGMA foreign_keys=OFF;").unwrap(); conn.execute_batch(bad_sql).unwrap();
        assert!(Store::open(&path).is_err());
        assert_eq!(conn.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(conn.query_row("SELECT count(*) FROM sqlite_master WHERE name='gym_settings'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(conn.query_row("SELECT name FROM members",[],|r|r.get::<_,String>(0)).unwrap(),"Existing member");
        drop(conn); let _=std::fs::remove_file(path);
    }
}
#[test]
fn all_profile_fields_persist_after_restart_and_stale_settings_are_rejected() {
    let mut f = Fixture::new();
    f.store.save_profile(profile()).unwrap();
    assert!(f.store.save_profile(profile()).is_err());
    let reopened = Store::open(&f.path).unwrap();
    let s = reopened.snapshot().unwrap();
    assert_eq!(
        s["profile"],
        json!({"version":2,"name":"Armstrong Fitness","location":"Matale","phone":"0661234567","email":"gym@example.lk"})
    );
    assert_eq!(s["auditCount"], 1);
    assert_eq!(s["pending"], 1);
}
#[test]
fn payments_expenses_sales_and_stock_persist_and_retries_are_idempotent() {
    let mut f = Fixture::new();
    f.store.save_member(member("CARD")).unwrap();
    let p = payment(&f.store);
    let e = expense();
    let prod = product();
    let product_id = f.store.save_product(prod.clone()).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let sale = SaleInput {
        request_id: id(),
        product_id: product_id.clone(),
        quantity: 2,
        method: "Card".into(),
    };
    let paid = f.store.record_payment(p.clone()).unwrap();
    let spent = f.store.record_expense(e.clone()).unwrap();
    let sold = f.store.complete_sale(sale.clone()).unwrap();
    let mut reopened = Store::open(&f.path).unwrap();
    assert_eq!(reopened.record_payment(p.clone()).unwrap(), paid);
    assert_eq!(reopened.record_expense(e).unwrap(), spent);
    assert_eq!(reopened.complete_sale(sale).unwrap(), sold);
    reopened.save_product(prod).unwrap();
    let s = reopened.snapshot().unwrap();
    assert_eq!(s["payments"][0]["amountMinor"], 600050);
    assert_eq!(s["expenses"][0]["amountMinor"], 125075);
    assert_eq!(s["sales"][0]["totalMinor"], 40100);
    assert_eq!(s["products"][0]["stock"], 3);
    assert_eq!(s["pending"], 5);
    let mut changed = p;
    changed.amount_minor = 700000;
    assert!(reopened
        .record_payment(changed)
        .unwrap_err()
        .contains("different request"));
    assert_eq!(s["payments"][0]["actor"], ACTOR);
    assert_eq!(s["expenses"][0]["actor"], ACTOR);
}
#[test]
fn failed_sale_payment_stock_and_profile_do_not_leave_partial_rows_or_receipts() {
    let mut f = Fixture::new();
    f.store.save_member(member("X")).unwrap();
    f.store.save_product(product()).unwrap();
    let before = f.store.snapshot().unwrap();
    let sale = SaleInput {
        request_id: id(),
        product_id: first_id(&f.store, "products"),
        quantity: 1,
        method: "Cash".into(),
    };
    f.store.conn.execute_batch("CREATE TRIGGER fail_new_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(f.store.complete_sale(sale).is_err());
    assert!(f.store.record_payment(payment(&f.store)).is_err());
    assert!(f.store.record_expense(expense()).is_err());
    assert!(f.store.save_profile(profile()).is_err());
    assert!(f
        .store
        .record_attendance(AttendanceInput {
            request_id: id(),
            member_or_card: first_id(&f.store, "members"),
            source: "Manual".into()
        })
        .is_err());
    assert!(f
        .store
        .adjust_stock(StockInput {
            request_id: id(),
            product_id: first_id(&f.store, "products"),
            amount: 1
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM sale_items", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM local_operations", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn attendance_dates_duplicate_scans_and_card_history_survive_reassignment() {
    let mut f = Fixture::new();
    f.store.save_member(member("ABC")).unwrap();
    let scan = AttendanceInput {
        request_id: id(),
        member_or_card: " abc ".into(),
        source: "NFC".into(),
    };
    let now = "2026-10-02T18:29:59Z".parse::<DateTime<Utc>>().unwrap();
    f.store.attendance_at(scan.clone(), now).unwrap();
    let mut repeat = scan.clone();
    repeat.request_id = id();
    assert_eq!(
        f.store.attendance_at(repeat, now).unwrap()["duplicate"],
        true
    );
    let mut tomorrow = scan;
    tomorrow.request_id = id();
    f.store
        .attendance_at(tomorrow, now + chrono::Duration::seconds(2))
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["attendance"].as_array().unwrap().len(), 2);
    assert_eq!(s["attendance"][0]["businessOn"], "2026-10-03");
    assert_eq!(s["attendance"][0]["type"], "Check-in");
    let mut edit = member("NEW");
    edit.id = Some(first_id(&f.store, "members"));
    edit.version = Some(1);
    f.store.save_member(edit).unwrap();
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM nfc_cards", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        f.store
            .conn
            .query_row(
                "SELECT count(*) FROM nfc_cards WHERE revoked_at IS NULL",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        f.store.snapshot().unwrap()["attendance"][0]["cardUid"],
        "ABC"
    );
    assert!(f
        .store
        .record_attendance(AttendanceInput {
            request_id: id(),
            member_or_card: "ABC".into(),
            source: "NFC".into()
        })
        .is_err());
    integrity(&f.store.conn).unwrap();
}
#[test]
fn history_foreign_keys_unique_skus_and_stock_constraints_are_enforced() {
    let mut f = Fixture::new();
    f.store.save_product(product()).unwrap();
    let mut duplicate = product();
    duplicate.sku = "water-1".into();
    assert!(f.store.save_product(duplicate).is_err());
    assert!(f
        .store
        .adjust_stock(StockInput {
            request_id: id(),
            product_id: "missing".into(),
            amount: 1
        })
        .is_err());
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO sale_items VALUES('x','missing','missing','n','s',1,1,1)",
            []
        )
        .is_err());
    assert!(f
        .store
        .conn
        .execute("DELETE FROM stock_movements", [])
        .is_err());
    assert!(f
        .store
        .conn
        .execute("UPDATE audit SET actor='fake'", [])
        .is_err());
    assert!(f.store.conn.execute("DELETE FROM outbox", []).is_err());
    assert!(f
        .store
        .conn
        .execute("UPDATE outbox SET payload_json='{}'", [])
        .is_err());
    assert!(f
        .store
        .complete_sale(SaleInput {
            request_id: id(),
            product_id: first_id(&f.store, "products"),
            quantity: 6,
            method: "Cash".into()
        })
        .is_err());
    let id = first_id(&f.store, "products");
    for _ in 0..5 {
        f.store
            .adjust_stock(StockInput {
                request_id: super::id(),
                product_id: id.clone(),
                amount: -1,
            })
            .unwrap();
    }
    assert!(f
        .store
        .adjust_stock(StockInput {
            request_id: super::id(),
            product_id: id,
            amount: -1
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap()["products"][0]["stock"], 0);
}
#[test]
fn concurrent_sellers_cannot_oversell_one_local_database() {
    let mut f = Fixture::new();
    let mut p = product();
    p.opening_stock = 1;
    f.store.save_product(p).unwrap();
    let product = first_id(&f.store, "products");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let jobs: Vec<_> = (0..2)
        .map(|_| {
            let path = f.path.clone();
            let product = product.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut store = Store::open(&path).unwrap();
                barrier.wait();
                store
                    .complete_sale(SaleInput {
                        request_id: id(),
                        product_id: product,
                        quantity: 1,
                        method: "Cash".into(),
                    })
                    .is_ok()
            })
        })
        .collect();
    assert_eq!(
        jobs.into_iter()
            .map(|job| job.join().unwrap())
            .filter(|saved| *saved)
            .count(),
        1
    );
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["sales"].as_array().unwrap().len(), 1);
    assert_eq!(s["products"][0]["stock"], 0);
}

#[test]
fn validated_backup_restore_keeps_settings_and_pending_operations_and_creates_recovery() {
    let mut f = Fixture::new();
    f.store.save_member(member("CARD")).unwrap();
    f.store.save_profile(profile()).unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let p = payment(&f.store);
    f.store.record_payment(p).unwrap();
    let preview = f.store.preview_restore(backup).unwrap();
    assert_eq!(preview["current"]["payments"], 1);
    assert_eq!(preview["backup"]["payments"], 0);
    let result = f
        .store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let path = std::path::Path::new(result["recoveryPath"].as_str().unwrap());
    assert!(path.is_file());
    let recovery: BackupEnvelope = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["profile"]["phone"], "0661234567");
    assert_eq!(s["payments"], json!([]));
    assert_eq!(s["pending"], 3);
    assert_eq!(s["restoreRequiresReconciliation"], true);
    let old_preview = f.store.preview_restore(recovery).unwrap();
    assert_eq!(old_preview["backup"]["payments"], 1);
    assert_eq!(
        Store::open(&f.path).unwrap().snapshot().unwrap()["restoreRequiresReconciliation"],
        true
    );
}
#[test]
fn bad_backup_checksum_schema_and_foreign_keys_do_not_replace_live_data() {
    use sha2::{Digest, Sha256};
    let mut f = Fixture::new();
    f.store.save_member(member("C")).unwrap();
    let original = f.store.snapshot().unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let mut bad = backup.clone();
    bad.data[0] ^= 1;
    assert!(f.store.preview_restore(bad).is_err());
    let mut bad = backup.clone();
    bad.schema_version = 99;
    assert!(f.store.preview_restore(bad).is_err());
    for sql in ["INSERT INTO attendance VALUES('orphan','missing',NULL,'n','','Check-in','Manual','2026-10-01','2026-10-01T00:00:00Z',NULL);", "CREATE TRIGGER unexpected_trigger AFTER INSERT ON members BEGIN SELECT 1; END;"] {
        let path=f.path.with_extension("tampered"); std::fs::write(&path,&backup.data).unwrap(); let conn=Connection::open(&path).unwrap(); conn.execute_batch("PRAGMA foreign_keys=OFF;").unwrap(); conn.execute_batch(sql).unwrap(); drop(conn);
        let mut bad=backup.clone(); bad.data=std::fs::read(&path).unwrap(); bad.sha256=format!("{:x}",Sha256::digest(&bad.data)); assert!(f.store.preview_restore(bad).is_err()); std::fs::remove_file(path).unwrap();
    }
    assert_eq!(f.store.snapshot().unwrap(), original);
}
#[test]
fn restore_confirmation_is_invalidated_by_new_writes_and_failed_recovery_preserves_live_data() {
    let mut f = Fixture::new();
    f.store.save_member(member("C")).unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let preview = f.store.preview_restore(backup.clone()).unwrap();
    f.store.record_expense(expense()).unwrap();
    let current = f.store.snapshot().unwrap();
    assert!(f
        .store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), current);
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .conn
        .execute_batch("CREATE INDEX unexpected_index ON members(phone);")
        .unwrap();
    assert!(f
        .store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), current);
}
#[test]
fn schema_one_backup_is_validated_then_migrated_in_isolation() {
    use sha2::{Digest, Sha256};
    let (path, conn) = v1_database();
    drop(conn);
    let data = std::fs::read(&path).unwrap();
    let backup = BackupEnvelope {
        format: "armstrong-sqlite-backup".into(),
        format_version: 1,
        schema_version: 1,
        created_at: Utc::now().to_rfc3339(),
        sha256: format!("{:x}", Sha256::digest(&data)),
        data,
    };
    let mut f = Fixture::new();
    let preview = f.store.preview_restore(backup).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["members"], json!([]));
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["id"], "old-member");
    assert_eq!(s["periods"][0]["priceMinor"], 500025);
    assert_eq!(s["pending"], 2);
    let _ = std::fs::remove_file(path);
}
#[test]
fn report_export_uses_native_minor_units_and_escapes_csv_formula_fields() {
    let mut f = Fixture::new();
    f.store.save_member(member("C")).unwrap();
    let mut e = expense();
    e.title = "=HYPERLINK(\"bad\")".into();
    f.store.record_expense(e).unwrap();
    let result = f.store.export_report("Expense report".into()).unwrap();
    let text = std::fs::read_to_string(result["path"].as_str().unwrap()).unwrap();
    assert!(text.contains("1250.75"));
    assert!(text.contains("'=HYPERLINK"));
    assert!(text.contains(ACTOR));
    assert_eq!(result["rows"], 1);
}

#[test]
fn product_edits_preserve_sale_snapshots_and_stock_retries_are_durable() {
    let mut f = Fixture::new();
    let input = product();
    let product_id = f.store.save_product(input.clone()).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.store
        .complete_sale(SaleInput {
            request_id: id(),
            product_id: product_id.clone(),
            quantity: 1,
            method: "Cash".into(),
        })
        .unwrap();
    let mut edit = input;
    edit.request_id = id();
    edit.id = Some(product_id.clone());
    edit.version = Some(1);
    edit.name = "Renamed".into();
    edit.cost_minor = 30000;
    edit.price_minor = 50000;
    edit.opening_stock = 0;
    f.store.save_product(edit.clone()).unwrap();
    let mut stale = edit;
    stale.request_id = id();
    assert!(f.store.save_product(stale).is_err());
    let adjustment = StockInput {
        request_id: id(),
        product_id: product_id.clone(),
        amount: 1,
    };
    f.store.adjust_stock(adjustment.clone()).unwrap();
    let mut reopened = Store::open(&f.path).unwrap();
    reopened.adjust_stock(adjustment).unwrap();
    let s = reopened.snapshot().unwrap();
    assert_eq!(s["products"][0]["stock"], 5);
    assert_eq!(s["products"][0]["version"], 2);
    assert_eq!(s["sales"][0]["totalMinor"], 20050);
    assert_eq!(s["sales"][0]["items"][0]["name"], "Water");
    assert_eq!(s["sales"][0]["items"][0]["costMinor"], 10025);
    assert_eq!(s["pending"], 4);
}
#[test]
fn invoice_ownership_allocation_limits_and_append_only_links_are_enforced() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    f.store.save_member(member("B")).unwrap();
    let s = f.store.snapshot().unwrap();
    let a = s["members"][0]["id"].as_str().unwrap();
    let b = s["members"][1]["id"].as_str().unwrap();
    f.store.save_plan(plan()).unwrap();
    let period = PeriodInput {
        member_id: a.into(),
        plan_id: first_id(&f.store, "plans"),
        starts_on: "2026-01-01".into(),
        ends_on: "2026-01-31".into(),
    };
    f.store.add_period(period).unwrap();
    let period_id = first_id(&f.store, "periods");
    let p = f
        .store
        .record_payment(PaymentInput {
            request_id: id(),
            member_id: a.into(),
            amount_minor: 100,
            method: "Cash".into(),
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.store
        .conn
        .execute(
            "INSERT INTO invoices VALUES('i1',?1,?2,NULL,80,'2026-01-01','2026-01-01T00:00:00Z')",
            params![a, period_id],
        )
        .unwrap();
    assert!(f.store.conn.execute("INSERT INTO invoices VALUES('wrong',?1,?2,NULL,80,'2026-01-01','2026-01-01T00:00:00Z')",params![b,period_id]).is_err());
    f.store.conn.execute("INSERT INTO invoices VALUES('i2',?1,NULL,NULL,100,'2026-01-01','2026-01-01T00:00:00Z')",[a]).unwrap();
    f.store.conn.execute("INSERT INTO invoices VALUES('i3',?1,NULL,NULL,100,'2026-01-01','2026-01-01T00:00:00Z')",[b]).unwrap();
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO payment_allocations VALUES('too-invoice',?1,'i1',81)",
            [&p]
        )
        .is_err());
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO payment_allocations VALUES('cross-member',?1,'i3',20)",
            [&p]
        )
        .is_err());
    f.store
        .conn
        .execute(
            "INSERT INTO payment_allocations VALUES('a1',?1,'i1',80)",
            [&p],
        )
        .unwrap();
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO payment_allocations VALUES('too-payment',?1,'i2',21)",
            [&p]
        )
        .is_err());
    f.store
        .conn
        .execute(
            "INSERT INTO payment_allocations VALUES('a2',?1,'i2',20)",
            [&p],
        )
        .unwrap();
    assert!(f
        .store
        .conn
        .execute("DELETE FROM payment_allocations", [])
        .is_err());
    integrity(&f.store.conn).unwrap();
}
#[test]
fn attendance_rejects_a_card_link_from_another_member() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    f.store.save_member(member("B")).unwrap();
    let s = f.store.snapshot().unwrap();
    let a = s["members"][0]["id"].as_str().unwrap();
    let b = s["members"][1]["id"].as_str().unwrap();
    let (card, uid): (String, String) = f
        .store
        .conn
        .query_row(
            "SELECT id,uid FROM nfc_cards WHERE member_id=?1",
            [a],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(f.store.conn.execute("INSERT INTO attendance VALUES('wrong',?1,?2,'Test',?3,'Check-in','NFC','2026-10-03','2026-10-03T00:00:00Z',NULL)",params![b,card,uid]).is_err());
    assert_eq!(f.store.snapshot().unwrap()["attendance"], json!([]));
}
#[test]
fn native_validation_rejects_invalid_money_method_and_untrusted_actor() {
    let mut f = Fixture::new();
    f.store.save_member(member("X")).unwrap();
    let before = f.store.snapshot().unwrap();
    let mut p = payment(&f.store);
    p.amount_minor = 0;
    assert!(f.store.record_payment(p).is_err());
    let mut p = payment(&f.store);
    p.amount_minor = 100000000001;
    assert!(f.store.record_payment(p).is_err());
    let mut p = payment(&f.store);
    p.method = "Other".into();
    assert!(f.store.record_payment(p).is_err());
    let mut e = expense();
    e.category = "Invented category".into();
    assert!(f.store.record_expense(e).is_err());
    let mut e = expense();
    e.method = "Transfer".into();
    assert!(f.store.record_expense(e).is_err());
    assert!(serde_json::from_value::<PaymentInput>(
        json!({"requestId":id(),"memberId":"x","amountMinor":1.5,"method":"Cash"})
    )
    .is_err());
    assert!(serde_json::from_value::<ExpenseInput>(json!({"requestId":id(),"title":"x","category":"Operations","amountMinor":100,"method":"Cash","actor":"Administrator"})).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
}

#[test]
fn restore_holds_the_writer_lock_and_rolls_back_all_tables_on_copy_failure() {
    let mut f = Fixture::new();
    f.store.save_member(member("CARD")).unwrap();
    let backup = f.store.backup_envelope().unwrap();
    f.store.record_payment(payment(&f.store)).unwrap();
    f.store.save_product(product()).unwrap();
    f.store.record_expense(expense()).unwrap();
    let before = f.store.snapshot().unwrap();
    let preview = f.store.preview_restore(backup.clone()).unwrap();
    let second = Connection::open(&f.path).unwrap();
    second.busy_timeout(Duration::from_millis(10)).unwrap();
    let result = f
        .store
        .restore_checked(preview["token"].as_str().unwrap().into(), |_| {
            assert!(second
                .execute(
                    "INSERT INTO metadata VALUES('concurrent-writer','lost')",
                    []
                )
                .is_err());
            Err("Injected failure after table replacement".into())
        });
    assert!(result.unwrap_err().contains("Injected failure"));
    assert_eq!(f.store.snapshot().unwrap(), before);
    integrity(&f.store.conn).unwrap();
    // Trigger removals must roll back too.
    assert!(f.store.conn.execute("DELETE FROM payments", []).is_err());
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    assert_eq!(f.store.snapshot().unwrap()["payments"], json!([]));
}

fn finance_invoice(store: &mut Store, member_id: &str, amount: i64) -> String {
    store
        .create_invoice(InvoiceInput {
            request_id: id(),
            member_id: member_id.into(),
            membership_period_id: None,
            description: "Membership charge".into(),
            amount_minor: amount,
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
fn finance_receive(
    store: &mut Store,
    member_id: &str,
    amount: i64,
    invoice: Option<&str>,
) -> String {
    store
        .receive_payment(ReceivePaymentInput {
            request_id: id(),
            member_id: member_id.into(),
            amount_minor: amount,
            method: "Cash".into(),
            invoice_id: invoice.map(String::from),
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
fn finance_renewal(store: &Store) -> RenewalInput {
    let s = store.snapshot().unwrap();
    let latest = s["periods"]
        .as_array()
        .unwrap()
        .iter()
        .max_by_key(|p| p["endsOn"].as_str().unwrap())
        .map(|p| p["id"].as_str().unwrap().to_string());
    RenewalInput {
        request_id: id(),
        member_id: first_id(store, "members"),
        plan_id: first_id(store, "plans"),
        plan_version: 1,
        expected_last_period_id: latest,
        starts_on: "2026-03-01".into(),
        ends_on: "2026-03-31".into(),
    }
}
#[test]
fn finance_partial_payment_overpayment_credit_and_outstanding_are_exact() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 10000);
    finance_receive(&mut f.store, &m, 3000, Some(&invoice));
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["invoices"][0]["status"], "Partial");
    assert_eq!(s["invoices"][0]["outstandingMinor"], 7000);
    let payment = finance_receive(&mut f.store, &m, 9000, Some(&invoice));
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["invoices"][0]["status"], "Paid");
    assert_eq!(s["invoices"][0]["paidMinor"], 10000);
    assert_eq!(s["financialAccounts"][0]["creditMinor"], 2000);
    assert_eq!(s["financialAccounts"][0]["netBalanceMinor"], -2000);
    let other = finance_invoice(&mut f.store, &m, 5000);
    assert_eq!(
        f.store.snapshot().unwrap()["financialAccounts"][0]["netBalanceMinor"],
        3000
    );
    f.store
        .allocate_payment(AllocationInput {
            request_id: id(),
            payment_id: payment,
            invoice_id: other,
            amount_minor: 2000,
        })
        .unwrap();
    let s = Store::open(&f.path).unwrap().snapshot().unwrap();
    assert_eq!(s["financialAccounts"][0]["outstandingMinor"], 3000);
    assert_eq!(s["financialAccounts"][0]["creditMinor"], 0);
}
#[test]
fn finance_repeated_partial_allocations_and_operation_retries_do_not_duplicate() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 10000);
    let payment = finance_receive(&mut f.store, &m, 10000, None);
    let input = AllocationInput {
        request_id: id(),
        payment_id: payment,
        invoice_id: invoice,
        amount_minor: 3000,
    };
    let result = f.store.allocate_payment(input.clone()).unwrap();
    let mut second = input.clone();
    second.request_id = id();
    second.amount_minor = 2000;
    f.store.allocate_payment(second).unwrap();
    let before = f.store.snapshot().unwrap();
    assert_eq!(f.store.allocate_payment(input.clone()).unwrap(), result);
    let mut reopened = Store::open(&f.path).unwrap();
    assert_eq!(reopened.allocate_payment(input.clone()).unwrap(), result);
    let mut changed = input;
    changed.amount_minor = 100;
    assert!(reopened.allocate_payment(changed).is_err());
    assert_eq!(reopened.snapshot().unwrap(), before);
    assert_eq!(before["allocations"].as_array().unwrap().len(), 2);
    assert_eq!(before["invoices"][0]["paidMinor"], 5000);
}
#[test]
fn finance_member_ownership_and_allocation_caps_reject_without_posting_cash() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let a = first_id(&f.store, "members");
    f.store.save_member(member("B")).unwrap();
    let b = f.store.snapshot().unwrap()["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] != a)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let invoice = finance_invoice(&mut f.store, &a, 100);
    let payment = finance_receive(&mut f.store, &b, 100, None);
    let before = f.store.snapshot().unwrap();
    assert!(f
        .store
        .allocate_payment(AllocationInput {
            request_id: id(),
            payment_id: payment,
            invoice_id: invoice.clone(),
            amount_minor: 50
        })
        .is_err());
    assert!(f
        .store
        .receive_payment(ReceivePaymentInput {
            request_id: id(),
            member_id: b,
            amount_minor: 50,
            method: "Cash".into(),
            invoice_id: Some(invoice.clone())
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    let payment = finance_receive(&mut f.store, &a, 200, None);
    let before = f.store.snapshot().unwrap();
    assert!(f
        .store
        .allocate_payment(AllocationInput {
            request_id: id(),
            payment_id: payment,
            invoice_id: invoice,
            amount_minor: 101
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
}
#[test]
fn finance_renewal_uses_explicit_inclusive_dates_and_snapshotted_price_atomically() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    f.store.save_plan(plan()).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    let input = finance_renewal(&f.store);
    let result = f.store.renew_membership(input.clone()).unwrap();
    let before = f.store.snapshot().unwrap();
    assert_eq!(f.store.renew_membership(input.clone()).unwrap(), result);
    assert_eq!(before["periods"][0]["startsOn"], "2026-03-01");
    assert_eq!(before["periods"][0]["endsOn"], "2026-03-31");
    assert_eq!(before["invoices"][0]["amountMinor"], 600000);
    assert_eq!(before["invoices"][0]["membershipPeriodId"], result["id"]);
    assert_eq!(before["invoices"][0]["status"], "Unpaid");
    let mut stale = input.clone();
    stale.request_id = id();
    assert!(f.store.renew_membership(stale).is_err());
    let mut bad = finance_renewal(&f.store);
    bad.starts_on = "2026-03-31".into();
    bad.ends_on = "2026-04-30".into();
    assert!(f.store.renew_membership(bad).is_err());
    let mut bad = finance_renewal(&f.store);
    bad.starts_on = "2026-02-29".into();
    assert!(f.store.renew_membership(bad).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    let mut edit = plan();
    edit.id = Some(first_id(&f.store, "plans"));
    edit.version = Some(1);
    edit.price_minor = 900000;
    edit.name = "Changed".into();
    f.store.save_plan(edit).unwrap();
    let mut old_price = finance_renewal(&f.store);
    old_price.starts_on = "2026-04-01".into();
    old_price.ends_on = "2026-04-30".into();
    assert!(f.store.renew_membership(old_price).is_err());
    assert_eq!(
        f.store.snapshot().unwrap()["invoices"][0]["amountMinor"],
        600000
    );
}
#[test]
fn finance_linked_invoice_requires_saved_membership_price_and_is_unique() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    f.store.save_plan(plan()).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    let mut input = InvoiceInput {
        request_id: id(),
        member_id: first_id(&f.store, "members"),
        membership_period_id: Some(first_id(&f.store, "periods")),
        description: "Saved period".into(),
        amount_minor: 1,
    };
    assert!(f.store.create_invoice(input.clone()).is_err());
    input.amount_minor = 600000;
    let result = f.store.create_invoice(input.clone()).unwrap();
    assert_eq!(f.store.create_invoice(input.clone()).unwrap(), result);
    input.request_id = id();
    assert!(f.store.create_invoice(input).is_err());
    assert_eq!(
        f.store.snapshot().unwrap()["invoices"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn finance_full_reversal_preserves_original_receipt_releases_allocations_and_rejects_duplicates() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    f.store.save_plan(plan()).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    let m = first_id(&f.store, "members");
    let one = finance_invoice(&mut f.store, &m, 10000);
    let two = finance_invoice(&mut f.store, &m, 5000);
    let payment = finance_receive(&mut f.store, &m, 12000, Some(&one));
    f.store
        .allocate_payment(AllocationInput {
            request_id: id(),
            payment_id: payment.clone(),
            invoice_id: two,
            amount_minor: 1000,
        })
        .unwrap();
    let document = f.store.payment_receipt(payment.clone()).unwrap();
    let input = ReversalInput {
        request_id: id(),
        payment_id: payment.clone(),
        reason: "Duplicate received record corrected".into(),
    };
    let result = f.store.reverse_payment(input.clone()).unwrap();
    assert_eq!(f.store.reverse_payment(input.clone()).unwrap(), result);
    let before = f.store.snapshot().unwrap();
    assert_eq!(before["financialAccounts"][0]["outstandingMinor"], 15000);
    assert_eq!(before["financialAccounts"][0]["creditMinor"], 0);
    assert_eq!(before["periods"].as_array().unwrap().len(), 1);
    assert_eq!(before["allocations"].as_array().unwrap().len(), 2);
    assert!(before["allocations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|a| a["reversedBy"] == result["id"]));
    assert_eq!(
        before["payments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["netAmountMinor"].as_i64().unwrap())
            .sum::<i64>(),
        0
    );
    let reprint = f.store.payment_receipt(payment.clone()).unwrap();
    assert_eq!(reprint["snapshot"], document["snapshot"]);
    assert_eq!(reprint["number"], document["number"]);
    assert_eq!(reprint["currentStatus"], "Reversed");
    let mut duplicate = input;
    duplicate.request_id = id();
    assert!(f.store.reverse_payment(duplicate).is_err());
    assert!(f
        .store
        .reverse_payment(ReversalInput {
            request_id: id(),
            payment_id: result["id"].as_str().unwrap().into(),
            reason: "Reverse reversal".into()
        })
        .is_err());
    assert!(f
        .store
        .allocate_payment(AllocationInput {
            request_id: id(),
            payment_id: payment,
            invoice_id: one,
            amount_minor: 1
        })
        .is_err());
    assert!(f.store.conn.execute("DELETE FROM payments", []).is_err());
    assert!(f
        .store
        .conn
        .execute("DELETE FROM payment_allocations", [])
        .is_err());
    assert!(f
        .store
        .conn
        .execute("DELETE FROM allocation_reversals", [])
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
}
#[test]
fn finance_failures_roll_back_cash_allocations_receipts_renewals_and_audit_together() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    f.store.save_plan(plan()).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 10000);
    let payment = finance_receive(&mut f.store, &m, 3000, None);
    let before = f.store.snapshot().unwrap();
    f.store.conn.execute_batch("CREATE TRIGGER finance_fail_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'injected finance failure'); END;").unwrap();
    assert!(f
        .store
        .create_invoice(InvoiceInput {
            request_id: id(),
            member_id: m.clone(),
            membership_period_id: None,
            description: "Another".into(),
            amount_minor: 500
        })
        .is_err());
    assert!(f
        .store
        .allocate_payment(AllocationInput {
            request_id: id(),
            payment_id: payment.clone(),
            invoice_id: invoice.clone(),
            amount_minor: 1000
        })
        .is_err());
    assert!(f
        .store
        .receive_payment(ReceivePaymentInput {
            request_id: id(),
            member_id: m.clone(),
            amount_minor: 5000,
            method: "Cash".into(),
            invoice_id: Some(invoice.clone())
        })
        .is_err());
    assert!(f.store.renew_membership(finance_renewal(&f.store)).is_err());
    assert!(f
        .store
        .reverse_payment(ReversalInput {
            request_id: id(),
            payment_id: payment,
            reason: "Correction".into()
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM payment_receipts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    f.store.conn.execute_batch("DROP TRIGGER finance_fail_outbox; CREATE TRIGGER finance_fail_receipt BEFORE INSERT ON payment_receipts BEGIN SELECT RAISE(ABORT,'receipt unavailable'); END;").unwrap();
    assert!(f
        .store
        .receive_payment(ReceivePaymentInput {
            request_id: id(),
            member_id: m,
            amount_minor: 5000,
            method: "Cash".into(),
            invoice_id: Some(invoice)
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
}
fn finance_v2_fixture() -> (std::path::PathBuf, Connection, PaymentInput) {
    let (path, conn) = v1_database();
    conn.execute_batch(include_str!("../migrations/002_local_operations.sql"))
        .unwrap();
    conn.execute_batch("INSERT INTO payments VALUES('legacy-pay','old-member','Existing member',600050,'Cash','2026-01-01','2026-01-01T00:00:00Z','old-actor',NULL); INSERT INTO invoices VALUES('legacy-invoice','old-member',NULL,NULL,400000,'2026-01-01','2026-01-01T00:00:00Z'); INSERT INTO payment_allocations VALUES('legacy-allocation','legacy-pay','legacy-invoice',200000);").unwrap();
    let input = PaymentInput {
        request_id: id(),
        member_id: "old-member".into(),
        amount_minor: 600050,
        method: "Cash".into(),
    };
    conn.execute(
        "INSERT INTO local_operations VALUES(?1,'payment',?2,?3,'2026-01-01T00:00:00Z')",
        params![
            input.request_id,
            serde_json::to_string(&input).unwrap(),
            "{\"id\":\"legacy-pay\"}"
        ],
    )
    .unwrap();
    (path, conn, input)
}
#[test]
fn finance_v2_migration_preserves_received_amounts_allocations_and_old_retry_receipts() {
    let (path, conn, input) = finance_v2_fixture();
    drop(conn);
    let mut f = Fixture {
        store: Store::open(&path).unwrap(),
        path,
    };
    let before = f.store.snapshot().unwrap();
    assert_eq!(before["payments"][0]["id"], "legacy-pay");
    assert_eq!(before["payments"][0]["amountMinor"], 600050);
    assert_eq!(before["allocations"][0]["id"], "legacy-allocation");
    assert_eq!(before["invoices"][0]["id"], "legacy-invoice");
    assert_eq!(before["financialAccounts"][0]["outstandingMinor"], 200000);
    assert_eq!(before["financialAccounts"][0]["creditMinor"], 400050);
    assert_eq!(
        f.store.record_payment(input).unwrap(),
        json!({"id":"legacy-pay"})
    );
    assert_eq!(f.store.snapshot().unwrap(), before);
    let document = f.store.payment_receipt("legacy-pay".into()).unwrap();
    assert_eq!(document["snapshot"]["legacy"], true);
    assert_eq!(document["snapshot"]["payment"]["actor"], "old-actor");
    assert_eq!(
        document["snapshot"]["payment"]["createdAt"],
        "2026-01-01T00:00:00Z"
    );
    assert_eq!(document["number"], "AF-R-existing-device-legacy-pay");
    assert_eq!(before["pending"], 1);
    assert_eq!(before["auditCount"], 1);
    let reopened = Store::open(&f.path).unwrap();
    assert_eq!(
        reopened.payment_receipt("legacy-pay".into()).unwrap(),
        document
    );
}
#[test]
fn finance_conflicting_legacy_reversal_stops_migration_without_changing_v2_records() {
    let (path, conn, _) = finance_v2_fixture();
    conn.execute_batch("INSERT INTO payments VALUES('bad-reversal','old-member','Existing member',1,'Cash','2026-01-02','2026-01-02T00:00:00Z','old-actor','legacy-pay');").unwrap();
    assert!(Store::open(&path).err().unwrap().contains("conflicts"));
    assert_eq!(
        conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row(
            "SELECT amount_minor FROM payments WHERE id='legacy-pay'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        600050
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='payment_receipts'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    drop(conn);
    let _ = std::fs::remove_file(path);
}
#[test]
fn finance_receipt_reprint_uses_saved_profile_and_member_and_never_mutates_history() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let payment = finance_receive(&mut f.store, &m, 12345, None);
    let document = f.store.payment_receipt(payment.clone()).unwrap();
    f.store.save_profile(profile()).unwrap();
    let mut edited = member("A");
    edited.id = Some(m);
    edited.version = Some(1);
    edited.name = "New member name".into();
    f.store.save_member(edited).unwrap();
    let before = f.store.snapshot().unwrap();
    assert_eq!(f.store.payment_receipt(payment.clone()).unwrap(), document);
    assert_eq!(f.store.payment_receipt(payment).unwrap(), document);
    assert_eq!(document["snapshot"]["gym"]["location"], "Matale, Sri Lanka");
    assert_eq!(document["snapshot"]["payment"]["memberName"], "Test member");
    assert_eq!(f.store.snapshot().unwrap(), before);
}
#[test]
fn finance_concurrent_allocations_cannot_overdraw_credit_or_overpay_invoice() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 100);
    let payment = finance_receive(&mut f.store, &m, 100, None);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let jobs: Vec<_> = (0..2)
        .map(|_| {
            let path = f.path.clone();
            let invoice = invoice.clone();
            let payment = payment.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut store = Store::open(&path).unwrap();
                barrier.wait();
                store
                    .allocate_payment(AllocationInput {
                        request_id: id(),
                        payment_id: payment,
                        invoice_id: invoice,
                        amount_minor: 60,
                    })
                    .is_ok()
            })
        })
        .collect();
    assert_eq!(
        jobs.into_iter()
            .map(|j| j.join().unwrap())
            .filter(|ok| *ok)
            .count(),
        1
    );
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["financialAccounts"][0]["outstandingMinor"], 40);
    assert_eq!(s["financialAccounts"][0]["creditMinor"], 40);
}
#[test]
fn finance_income_export_subtracts_reversal_minor_units_and_allocations_add_no_cash() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 50);
    let payment = finance_receive(&mut f.store, &m, 50, Some(&invoice));
    f.store
        .reverse_payment(ReversalInput {
            request_id: id(),
            payment_id: payment,
            reason: "Correction".into(),
        })
        .unwrap();
    let report = f.store.export_report("Income report".into()).unwrap();
    let text = std::fs::read_to_string(report["path"].as_str().unwrap()).unwrap();
    assert!(text.contains("\"'-0.50\""));
    assert!(text.contains("\"0.50\""));
    assert_eq!(report["rows"], 2);
}
#[test]
fn finance_backup_restore_retains_reversal_links_balances_and_stable_receipts() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 10000);
    let payment = finance_receive(&mut f.store, &m, 5000, Some(&invoice));
    let reversal = f
        .store
        .reverse_payment(ReversalInput {
            request_id: id(),
            payment_id: payment.clone(),
            reason: "Correction".into(),
        })
        .unwrap();
    let document = f
        .store
        .payment_receipt(reversal["id"].as_str().unwrap().into())
        .unwrap();
    let backup = f.store.backup_envelope().unwrap();
    finance_receive(&mut f.store, &m, 500, None);
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["financialAccounts"][0]["outstandingMinor"], 10000);
    assert_eq!(s["financialAccounts"][0]["creditMinor"], 0);
    assert_eq!(
        f.store
            .payment_receipt(reversal["id"].as_str().unwrap().into())
            .unwrap(),
        document
    );
    assert_eq!(s["allocations"][0]["reversedBy"], reversal["id"]);
    assert_eq!(s["restoreRequiresReconciliation"], true);
}

#[test]
fn finance_valid_legacy_reversal_migrates_original_cash_and_releases_without_erasing_history() {
    let (path, conn, _) = finance_v2_fixture();
    conn.execute_batch("INSERT INTO payments VALUES('legacy-reverse','old-member','Existing member',600050,'Cash','2026-01-02','2026-01-02T00:00:00Z','old-actor','legacy-pay');").unwrap();
    drop(conn);
    let store = Store::open(&path).unwrap();
    let snapshot = store.snapshot().unwrap();
    assert_eq!(snapshot["payments"].as_array().unwrap().len(), 2);
    assert_eq!(snapshot["allocations"][0]["id"], "legacy-allocation");
    assert_eq!(snapshot["allocations"][0]["reversedBy"], "legacy-reverse");
    assert_eq!(snapshot["invoices"][0]["outstandingMinor"], 400000);
    assert_eq!(snapshot["financialAccounts"][0]["creditMinor"], 0);
    assert_eq!(
        snapshot["payments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["netAmountMinor"].as_i64().unwrap())
            .sum::<i64>(),
        0
    );
    let original = store.payment_receipt("legacy-pay".into()).unwrap();
    let reversal = store.payment_receipt("legacy-reverse".into()).unwrap();
    assert_eq!(original["snapshot"]["legacy"], true);
    assert_eq!(
        reversal["snapshot"]["originalReceiptNumber"],
        original["number"]
    );
    assert_eq!(
        reversal["snapshot"]["reason"],
        "Legacy reversal: original reason unavailable"
    );
    assert_eq!(reversal["snapshot"]["allocations"][0]["released"], 1);
    assert!(store
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.snapshot().unwrap(), snapshot);
    assert_eq!(
        reopened.payment_receipt("legacy-reverse".into()).unwrap(),
        reversal
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn finance_document_and_release_constraints_reject_orphans_mismatches_and_history_edits() {
    let mut f = Fixture::new();
    f.store.save_member(member("A")).unwrap();
    let m = first_id(&f.store, "members");
    let invoice = finance_invoice(&mut f.store, &m, 100);
    let payment = finance_receive(&mut f.store, &m, 50, Some(&invoice));
    let before = f.store.snapshot().unwrap();
    for sql in [
        "INSERT INTO invoice_details VALUES('missing','duplicate','No owner','A','actor',0)",
        "INSERT INTO payment_reversal_details VALUES('missing','No parent')",
        "INSERT INTO allocation_reversals VALUES('orphan','missing','missing','2026-01-01T00:00:00Z')",
        "INSERT INTO payment_receipts VALUES('missing','invalid','{}','2026-01-01T00:00:00Z',0)",
        "UPDATE payment_receipts SET number='changed'",
        "DELETE FROM payment_receipts",
        "UPDATE invoice_details SET description='changed'",
        "DELETE FROM invoice_details",
    ] { assert!(f.store.conn.execute_batch(sql).is_err(), "{sql}"); }
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO payment_reversal_details VALUES(?1,'Original is not a reversal')",
            [&payment]
        )
        .is_err());
    assert!(f.store.conn.execute("INSERT INTO payments VALUES('wrong',?1,'A',1,'Cash','2026-01-01','2026-01-01T00:00:00Z','actor',?2)",params![m,payment]).is_err());
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO payment_allocations VALUES('orphan','missing',?1,1)",
            [&invoice]
        )
        .is_err());
    assert!(f
        .store
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
    assert_eq!(f.store.snapshot().unwrap(), before);
}

fn removal_admin(store: &mut Store) {
    // Test-only trusted session setup. This helper is never built into the desktop binary.
    store.conn.execute_batch("INSERT INTO users VALUES('test-admin','verified-test-subject','admin@example.test','Verified test administrator',1,1); INSERT INTO roles VALUES('test-administrator','Administrator'); INSERT INTO user_roles VALUES('test-admin','test-administrator');").unwrap();
    store.removal_session = Some(removal::Session {
        can_write: true,
        user_id: "test-admin".into(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    });
}
fn removal_member_input(store: &Store) -> MemberRemovalInput {
    let s = store.snapshot().unwrap();
    MemberRemovalInput {
        request_id: id(),
        member_id: s["members"][0]["id"].as_str().unwrap().into(),
        version: s["members"][0]["version"].as_i64().unwrap(),
    }
}
fn removal_expense(store: &mut Store) -> String {
    store
        .record_expense(ExpenseInput {
            request_id: id(),
            title: "Removal electricity".into(),
            category: "Utilities".into(),
            amount_minor: 12345,
            method: "Bank".into(),
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
fn removal_storage(store: &Store) -> Value {
    let mut result = store.snapshot().unwrap();
    result
        .as_object_mut()
        .unwrap()
        .remove("removalAuthorization");
    result["memberSync"]
        .as_object_mut()
        .unwrap()
        .remove("reviewAuthorization");
    result
}
#[test]
fn removal_linked_member_archive_retains_all_history_and_excludes_active_membership_counts() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    f.store.save_member(member("ARCHIVE-CARD")).unwrap();
    f.store.save_plan(plan()).unwrap();
    let member_id = first_id(&f.store, "members");
    let today = business_date(Utc::now());
    f.store
        .add_period(PeriodInput {
            member_id: member_id.clone(),
            plan_id: first_id(&f.store, "plans"),
            starts_on: today.clone(),
            ends_on: today,
        })
        .unwrap();
    f.store
        .record_attendance(AttendanceInput {
            request_id: id(),
            member_or_card: member_id.clone(),
            source: "Manual".into(),
        })
        .unwrap();
    let invoice = finance_invoice(&mut f.store, &member_id, 10000);
    finance_receive(&mut f.store, &member_id, 5000, Some(&invoice));
    let before = f.store.snapshot().unwrap();
    let input = removal_member_input(&f.store);
    let result = f.store.archive_member(input.clone()).unwrap();
    assert_eq!(f.store.archive_member(input).unwrap(), result);
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["active"], false);
    assert_eq!(s["members"][0]["archivedByUserId"], "test-admin");
    assert_eq!(s["members"][0]["canDelete"], false);
    assert_eq!(s["members"][0]["version"], 2);
    assert_eq!(s["plans"][0]["activeMembers"], 0);
    for key in [
        "periods",
        "attendance",
        "invoices",
        "payments",
        "allocations",
        "financialAccounts",
    ] {
        assert_eq!(s[key], before[key], "{key} history changed");
    }
    assert_eq!(
        s["pending"].as_i64().unwrap(),
        before["pending"].as_i64().unwrap() + 1
    );
    assert_eq!(s["audit"][0]["action"], "archive member");
    assert_eq!(
        s["audit"][0]["user"],
        "Verified test administrator (test-admin)"
    );
    let actor: String = f
        .store
        .conn
        .query_row(
            "SELECT actor_user_id FROM audit WHERE action='archive member'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actor, "test-admin");
    assert_eq!(
        f.store
            .conn
            .query_row(
                "SELECT count(*) FROM nfc_cards WHERE uid='ARCHIVE-CARD' AND revoked_at IS NULL",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    for source in ["Manual", "NFC"] {
        assert!(f
            .store
            .record_attendance(AttendanceInput {
                request_id: id(),
                member_or_card: if source == "NFC" {
                    "ARCHIVE-CARD".into()
                } else {
                    member_id.clone()
                },
                source: source.into()
            })
            .is_err());
    }
    let mut edit = member("CHANGED");
    edit.id = Some(member_id.clone());
    edit.version = Some(2);
    assert!(f.store.save_member(edit).is_err());
    let mut next = period(&f.store);
    next.starts_on = "2199-01-01".into();
    next.ends_on = "2199-01-31".into();
    assert!(f.store.add_period(next).is_err());
    assert!(f
        .store
        .delete_member(removal_member_input(&f.store))
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), s);
    let reopened = Store::open(&f.path).unwrap();
    assert_eq!(reopened.snapshot().unwrap()["members"][0]["active"], false);
    assert_eq!(
        reopened.snapshot().unwrap()["removalAuthorization"]["allowed"],
        false
    );
    integrity(&reopened.conn).unwrap();
}
#[test]
fn removal_unlinked_member_deletion_retains_actor_audit_outbox_and_durable_retry() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    f.store.save_member(member("")).unwrap();
    let input = removal_member_input(&f.store);
    assert_eq!(f.store.snapshot().unwrap()["members"][0]["canDelete"], true);
    let result = f.store.delete_member(input.clone()).unwrap();
    assert_eq!(f.store.delete_member(input.clone()).unwrap(), result);
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"], json!([]));
    assert_eq!(s["auditCount"], 2);
    assert_eq!(s["pending"], 2);
    assert_eq!(s["audit"][0]["action"], "delete member");
    let before: Value =
        serde_json::from_str(s["audit"][0]["beforeJson"].as_str().unwrap()).unwrap();
    assert_eq!(before["name"], "Test member");
    let after: Value = serde_json::from_str(s["audit"][0]["afterJson"].as_str().unwrap()).unwrap();
    assert_eq!(after["deleted"], true);
    assert_eq!(after["deletedByUserId"], "test-admin");
    let action: String = f
        .store
        .conn
        .query_row("SELECT action FROM outbox WHERE action='delete'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(action, "delete");
    let mut reopened = Store::open(&f.path).unwrap();
    assert!(reopened.delete_member(input.clone()).is_err());
    reopened.removal_session = Some(removal::Session {
        can_write: true,
        user_id: "test-admin".into(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    });
    assert_eq!(reopened.delete_member(input.clone()).unwrap(), result);
    let mut repeated = input;
    repeated.request_id = id();
    assert!(reopened.delete_member(repeated).is_err());
    assert_eq!(reopened.snapshot().unwrap()["auditCount"], 2);
}
#[test]
fn removal_every_member_business_link_including_revoked_cards_blocks_deletion() {
    for kind in [
        "membership",
        "attendance",
        "invoice",
        "payment",
        "card",
        "revoked-card",
    ] {
        let mut f = Fixture::new();
        removal_admin(&mut f.store);
        f.store
            .save_member(member(if kind.contains("card") {
                "LINK-CARD"
            } else {
                ""
            }))
            .unwrap();
        let m = first_id(&f.store, "members");
        match kind {
            "membership" => {
                f.store.save_plan(plan()).unwrap();
                f.store.add_period(period(&f.store)).unwrap();
            }
            "attendance" => {
                f.store
                    .record_attendance(AttendanceInput {
                        request_id: id(),
                        member_or_card: m.clone(),
                        source: "Manual".into(),
                    })
                    .unwrap();
            }
            "invoice" => {
                finance_invoice(&mut f.store, &m, 100);
            }
            "payment" => {
                finance_receive(&mut f.store, &m, 100, None);
            }
            "revoked-card" => {
                let mut edit = member("");
                edit.id = Some(m.clone());
                edit.version = Some(1);
                f.store.save_member(edit).unwrap();
            }
            _ => {}
        }
        let before = f.store.snapshot().unwrap();
        assert_eq!(before["members"][0]["canDelete"], false);
        assert!(
            f.store
                .delete_member(removal_member_input(&f.store))
                .is_err(),
            "{kind} linked deletion"
        );
        assert_eq!(f.store.snapshot().unwrap(), before);
        assert!(
            f.store
                .conn
                .execute("DELETE FROM members WHERE id=?1", [m])
                .is_err(),
            "{kind} SQL bypass"
        );
        integrity(&f.store.conn).unwrap();
    }
}
#[test]
fn removal_expense_void_preserves_amount_requires_reason_audits_and_excludes_report_totals() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    let expense_id = removal_expense(&mut f.store);
    let original = f.store.snapshot().unwrap()["expenses"][0].clone();
    let mut input = ExpenseVoidInput {
        request_id: id(),
        expense_id: expense_id.clone(),
        reason: "  Duplicate electricity entry  ".into(),
    };
    let mut blank = input.clone();
    blank.reason = " ".into();
    assert!(f.store.void_expense(blank).is_err());
    let result = f.store.void_expense(input.clone()).unwrap();
    assert_eq!(f.store.void_expense(input.clone()).unwrap(), result);
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["expenses"].as_array().unwrap().len(), 1);
    assert_eq!(s["expenses"][0]["status"], "Voided");
    assert_eq!(s["expenses"][0]["effectiveAmountMinor"], 0);
    assert_eq!(
        s["expenses"][0]["voidReason"],
        "Duplicate electricity entry"
    );
    assert_eq!(s["expenses"][0]["voidedByUserId"], "test-admin");
    for key in [
        "amountMinor",
        "title",
        "category",
        "method",
        "businessOn",
        "createdAt",
        "actor",
        "reversesId",
    ] {
        assert_eq!(s["expenses"][0][key], original[key]);
    }
    assert_eq!(s["audit"][0]["action"], "void expense");
    assert_eq!(
        s["audit"][0]["user"],
        "Verified test administrator (test-admin)"
    );
    assert_eq!(s["auditCount"], 2);
    assert_eq!(s["pending"], 2);
    input.request_id = id();
    assert!(f.store.void_expense(input).is_err());
    assert_eq!(f.store.snapshot().unwrap(), s);
    let csv = f.store.export_report("Expense report".into()).unwrap();
    let text = std::fs::read_to_string(csv["path"].as_str().unwrap()).unwrap();
    assert!(text.contains("\"123.45\",\"0.00\",\"Voided\""));
    assert!(text.contains("Duplicate electricity entry"));
    assert!(text.contains("Verified test administrator"));
    assert_eq!(csv["rows"], 1);
    assert!(f.store.conn.execute_batch("DELETE FROM expenses").is_err());
    assert!(f
        .store
        .conn
        .execute_batch("UPDATE expense_voids SET reason='overwrite'")
        .is_err());
    assert!(f
        .store
        .conn
        .execute_batch("DELETE FROM expense_voids")
        .is_err());
    let reopened = Store::open(&f.path).unwrap();
    assert_eq!(reopened.snapshot().unwrap()["expenses"], s["expenses"]);
    integrity(&reopened.conn).unwrap();
}
#[test]
fn removal_authorization_denies_unauthenticated_expired_inactive_nonadmin_revoked_and_spoofed_requests(
) {
    let mut f = Fixture::new();
    f.store.save_member(member("")).unwrap();
    let expense_id = removal_expense(&mut f.store);
    let member_input = removal_member_input(&f.store);
    let expense_input = ExpenseVoidInput {
        request_id: id(),
        expense_id,
        reason: "Correction".into(),
    };
    let before = removal_storage(&f.store);
    assert!(f
        .store
        .archive_member(member_input.clone())
        .unwrap_err()
        .contains("authenticated"));
    assert!(f.store.delete_member(member_input.clone()).is_err());
    assert!(f.store.void_expense(expense_input.clone()).is_err());
    assert_eq!(removal_storage(&f.store), before);
    assert!(serde_json::from_value::<MemberRemovalInput>(json!({"requestId":id(),"memberId":member_input.member_id,"version":1,"actor":"Administrator"})).is_err());
    assert!(serde_json::from_value::<ExpenseVoidInput>(json!({"requestId":id(),"expenseId":expense_input.expense_id,"reason":"Correction","userId":"test-admin"})).is_err());
    removal_admin(&mut f.store);
    for denied in ["expired", "inactive", "nonadmin"] {
        f.store.removal_session = Some(removal::Session {
            can_write: true,
            user_id: "test-admin".into(),
            expires_at: if denied == "expired" {
                Utc::now() - chrono::Duration::seconds(1)
            } else {
                Utc::now() + chrono::Duration::hours(1)
            },
        });
        if denied == "inactive" {
            f.store
                .conn
                .execute("UPDATE users SET active=0 WHERE id='test-admin'", [])
                .unwrap();
        }
        if denied == "nonadmin" {
            f.store
                .conn
                .execute("UPDATE users SET active=1 WHERE id='test-admin'", [])
                .unwrap();
            f.store
                .conn
                .execute_batch("DELETE FROM user_roles")
                .unwrap();
        }
        let before = removal_storage(&f.store);
        assert!(f.store.archive_member(member_input.clone()).is_err());
        assert!(f.store.delete_member(member_input.clone()).is_err());
        assert!(f.store.void_expense(expense_input.clone()).is_err());
        assert_eq!(removal_storage(&f.store), before, "{denied}");
    }
    f.store
        .conn
        .execute_batch("INSERT INTO user_roles VALUES('test-admin','test-administrator')")
        .unwrap();
    f.store.void_expense(expense_input.clone()).unwrap();
    f.store
        .conn
        .execute_batch("DELETE FROM user_roles")
        .unwrap();
    assert!(
        f.store.void_expense(expense_input).is_err(),
        "replay cannot bypass revoked permission"
    );
}
#[test]
fn removal_stale_versions_and_outbox_failures_roll_back_archive_delete_and_void_atomically() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    f.store.save_member(member("")).unwrap();
    let expense_id = removal_expense(&mut f.store);
    let input = removal_member_input(&f.store);
    let mut edit = member("");
    edit.id = Some(input.member_id.clone());
    edit.version = Some(1);
    edit.phone = "0779999999".into();
    f.store.save_member(edit).unwrap();
    let before = f.store.snapshot().unwrap();
    assert!(f.store.archive_member(input.clone()).is_err());
    assert!(f.store.delete_member(input).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    f.store.conn.execute_batch("CREATE TRIGGER removal_fail_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'Injected removal failure'); END").unwrap();
    assert!(f
        .store
        .archive_member(removal_member_input(&f.store))
        .is_err());
    assert!(f
        .store
        .delete_member(removal_member_input(&f.store))
        .is_err());
    assert!(f
        .store
        .void_expense(ExpenseVoidInput {
            request_id: id(),
            expense_id,
            reason: "Correction".into()
        })
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM expense_voids", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    integrity(&f.store.conn).unwrap();
}
fn removal_v3_fixture(reverse_amount: i64) -> (std::path::PathBuf, Connection) {
    let (path, mut conn, _) = finance_v2_fixture();
    conn.execute_batch(include_str!("../migrations/003_finance.sql"))
        .unwrap();
    let tx = conn.transaction().unwrap();
    finance::migrate_receipts(&tx).unwrap();
    tx.commit().unwrap();
    conn.execute_batch("INSERT INTO expenses VALUES('legacy-expense','Old electricity','Utilities',12345,'Bank','2026-01-01','2026-01-01T00:00:00Z','old-actor',NULL)").unwrap();
    conn.execute("INSERT INTO expenses VALUES('legacy-void','Old electricity','Utilities',?1,'Bank','2026-01-02','2026-01-02T00:00:00Z','old-reversing-actor','legacy-expense')",[reverse_amount]).unwrap();
    (path, conn)
}
#[test]
fn removal_v3_migration_preserves_legacy_members_expenses_reversals_audit_outbox_and_repeat_reopen()
{
    let (path, conn) = removal_v3_fixture(12345);
    let history: Vec<(String, i64)> = conn
        .prepare("SELECT id,amount_minor FROM expenses ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    drop(conn);
    let store = Store::open(&path).unwrap();
    let s = store.snapshot().unwrap();
    assert_eq!(s["members"][0]["active"], true);
    assert_eq!(s["members"][0]["version"], 3);
    assert_eq!(
        s["expenses"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["effectiveAmountMinor"].as_i64().unwrap())
            .sum::<i64>(),
        0
    );
    let original = s["expenses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "legacy-expense")
        .unwrap();
    assert_eq!(original["status"], "Voided");
    assert_eq!(original["voidedBy"], "old-reversing-actor");
    assert_eq!(
        original["voidReason"],
        "Legacy reversal: original reason unavailable"
    );
    let after: Vec<(String, i64)> = store
        .conn
        .prepare("SELECT id,amount_minor FROM expenses ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    assert_eq!(history, after);
    assert_eq!(s["pending"], 1);
    assert_eq!(s["auditCount"], 1);
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.snapshot().unwrap(), s);
    integrity(&reopened.conn).unwrap();
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn removal_conflicting_legacy_expense_reversal_stops_migration_without_replacing_v3() {
    let (path, conn) = removal_v3_fixture(1);
    drop(conn);
    assert!(Store::open(&path).is_err());
    let conn = Connection::open(&path).unwrap();
    assert_eq!(
        conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='expense_voids'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT amount_minor FROM expenses WHERE id='legacy-expense'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        12345
    );
    drop(conn);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn removal_backup_restore_preserves_archived_user_fk_void_history_and_never_restores_authentication(
) {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    f.store.save_member(member("RESTORE-CARD")).unwrap();
    let expense_id = removal_expense(&mut f.store);
    f.store
        .archive_member(removal_member_input(&f.store))
        .unwrap();
    f.store
        .void_expense(ExpenseVoidInput {
            request_id: id(),
            expense_id,
            reason: "Restored void".into(),
        })
        .unwrap();
    let s = f.store.snapshot().unwrap();
    let backup = f.store.backup_envelope().unwrap();
    removal_expense(&mut f.store);
    let preview = f.store.preview_restore(backup).unwrap();
    // Expiry changes runtime capability, not the persisted confirmation fingerprint.
    f.store.removal_session = Some(removal::Session {
        can_write: true,
        user_id: "test-admin".into(),
        expires_at: Utc::now() - chrono::Duration::seconds(1),
    });
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let restored = f.store.snapshot().unwrap();
    assert_eq!(restored["members"], s["members"]);
    assert_eq!(restored["expenses"], s["expenses"]);
    assert_eq!(restored["removalAuthorization"]["allowed"], false);
    assert!(f.store.removal_session.is_none());
    integrity(&f.store.conn).unwrap();
}
