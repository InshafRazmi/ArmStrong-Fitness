use super::*;

struct Fixture {
    store: Store,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("armstrong-registration-{}.sqlite3", id()));
        let mut store = Store::open(&path).unwrap();
        store
            .save_plan(PlanInput {
                id: None,
                version: None,
                name: "Monthly".into(),
                duration_months: 1,
                price_minor: 600050,
                active: true,
            })
            .unwrap();
        Self { store, path }
    }
    fn input(&self) -> RegisterMemberInput {
        let plan = &self.store.snapshot().unwrap()["plans"][0];
        RegisterMemberInput {
            expected_admission_minor: None,
            request_id: id(),
            name: "New member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: " card-1 ".into(),
            plan_id: Some(plan["id"].as_str().unwrap().into()),
            plan_version: Some(1),
            starts_on: Some("2026-01-31".into()),
        }
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
    }
}

#[test]
fn calendar_packages_include_last_day_and_handle_month_ends_leap_years_and_bounds() {
    for (start, months, end) in [
        ("2026-10-05", 1, "2026-11-04"),
        ("2026-10-01", 1, "2026-10-31"),
        ("2026-01-31", 1, "2026-02-28"),
        ("2024-01-31", 1, "2024-02-29"),
        ("2024-02-29", 12, "2025-02-28"),
        ("2026-12-15", 3, "2027-03-14"),
        ("2026-01-30", 3, "2026-04-29"),
        ("2026-01-31", 3, "2026-04-30"),
        ("2026-10-05", 60, "2031-10-04"),
        ("2200-11-01", 1, "2200-11-30"),
    ] {
        assert_eq!(package_end(start, months).unwrap(), end);
    }
    for (start, months) in [
        ("2026-02-30", 1),
        ("2026-2-01", 1),
        ("1899-12-01", 1),
        ("2200-12-31", 1),
        ("2026-10-05", 0),
        ("2026-10-05", 61),
    ] {
        assert!(package_end(start, months).is_err());
    }
}

#[test]
fn registration_saves_package_dates_card_history_audit_and_one_sync_batch_together() {
    let mut f = Fixture::new();
    let input = f.input();
    let result = f.store.register_member(input).unwrap();
    let snapshot = f.store.snapshot().unwrap();
    assert_eq!(snapshot["members"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["members"][0]["nfcId"], "CARD-1");
    assert_eq!(snapshot["periods"][0]["memberId"], result["id"]);
    assert_eq!(snapshot["periods"][0]["id"], result["membershipPeriodId"]);
    assert_eq!(snapshot["periods"][0]["startsOn"], "2026-01-31");
    assert_eq!(snapshot["periods"][0]["endsOn"], "2026-02-28");
    assert_eq!(snapshot["periods"][0]["planName"], "Monthly");
    assert_eq!(snapshot["periods"][0]["priceMinor"], 600050);
    assert_eq!(snapshot["pending"], 4);
    assert_eq!(snapshot["auditCount"], 4);
    assert_eq!(snapshot["invoices"].as_array().unwrap().len(), 1);
    assert!(snapshot["payments"].as_array().unwrap().is_empty());
    let encoded: String = f
        .store
        .conn
        .query_row(
            "SELECT request_json FROM business_batches ORDER BY ordinal DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let batch: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(batch["operationIds"].as_array().unwrap().len(), 3);
    let changes = batch["changes"].as_array().unwrap();
    for table in ["members", "nfc_cards", "membership_periods"] {
        assert_eq!(
            changes
                .iter()
                .filter(|change| change["table"] == table)
                .count(),
            1
        );
    }
    assert_eq!(
        changes
            .iter()
            .filter(|change| change["table"] == "audit")
            .count(),
        3
    );
}

#[test]
fn exact_registration_retry_after_restart_is_idempotent_and_history_survives_plan_changes() {
    let mut f = Fixture::new();
    let input = f.input();
    let result = f.store.register_member(input.clone()).unwrap();
    f.store
        .save_plan(PlanInput {
            id: input.plan_id.clone(),
            version: Some(1),
            name: "New name".into(),
            duration_months: 3,
            price_minor: 999999,
            active: false,
        })
        .unwrap();
    let before = f.store.snapshot().unwrap();
    let mut reopened = Store::open(&f.path).unwrap();
    assert_eq!(reopened.register_member(input.clone()).unwrap(), result);
    assert_eq!(reopened.snapshot().unwrap(), before);
    let mut changed = input;
    changed.phone = "0779999999".into();
    assert!(reopened
        .register_member(changed)
        .unwrap_err()
        .contains("different request"));
    assert_eq!(reopened.snapshot().unwrap(), before);
}

#[test]
fn stale_inactive_missing_packages_and_invalid_dates_never_create_partial_members() {
    let mut f = Fixture::new();
    let input = f.input();
    let before = f.store.snapshot().unwrap();
    let mut invalid = Vec::new();
    let mut stale = input.clone();
    stale.plan_version = Some(2);
    invalid.push(stale);
    let mut missing = input.clone();
    missing.plan_id = Some(id());
    invalid.push(missing);
    let mut undated = input.clone();
    undated.starts_on = None;
    invalid.push(undated);
    let mut invalid_date = input.clone();
    invalid_date.starts_on = Some("2026-02-30".into());
    invalid.push(invalid_date);
    let mut out_of_range = input.clone();
    out_of_range.starts_on = Some("2200-12-31".into());
    invalid.push(out_of_range);
    let mut no_version = input.clone();
    no_version.plan_version = None;
    invalid.push(no_version);
    let mut bad_name = input.clone();
    bad_name.name.clear();
    invalid.push(bad_name);
    let mut bad_card = input.clone();
    bad_card.nfc_id = "HAS SPACE".into();
    invalid.push(bad_card);
    let mut bad_request = input.clone();
    bad_request.request_id = "invalid".into();
    invalid.push(bad_request);
    for request in invalid {
        assert!(f.store.register_member(request).is_err());
        assert_eq!(f.store.snapshot().unwrap(), before);
    }
    f.store
        .save_plan(PlanInput {
            id: input.plan_id.clone(),
            version: Some(1),
            name: "Monthly".into(),
            duration_months: 1,
            price_minor: 600050,
            active: false,
        })
        .unwrap();
    let before = f.store.snapshot().unwrap();
    let mut inactive = input;
    inactive.plan_version = Some(2);
    assert!(f.store.register_member(inactive).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
}

#[test]
fn late_membership_audit_sync_or_operation_receipt_failures_roll_back_all_registration_rows() {
    for trigger in [
        "CREATE TRIGGER injected_failure BEFORE INSERT ON membership_periods BEGIN SELECT RAISE(ABORT,'injected'); END;",
        "CREATE TRIGGER injected_failure BEFORE INSERT ON audit WHEN NEW.entity='membership_period' BEGIN SELECT RAISE(ABORT,'injected'); END;",
        "CREATE TRIGGER injected_failure BEFORE INSERT ON business_batches BEGIN SELECT RAISE(ABORT,'injected'); END;",
        "CREATE TRIGGER injected_failure BEFORE INSERT ON local_operations BEGIN SELECT RAISE(ABORT,'injected'); END;",
    ] {
        let mut f = Fixture::new(); let input = f.input(); let before = f.store.snapshot().unwrap();
        f.store.conn.execute_batch(trigger).unwrap();
        assert!(f.store.register_member(input.clone()).is_err()); assert_eq!(f.store.snapshot().unwrap(), before);
        for table in ["members", "nfc_cards", "membership_periods", "local_operations", "business_dirty"] {
            let count: i64 = f.store.conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0)).unwrap();
            assert_eq!(count, 0, "{table} must roll back");
        }
        assert_eq!(f.store.conn.query_row("SELECT COUNT(*) FROM business_batches", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
        f.store.conn.execute_batch("DROP TRIGGER injected_failure").unwrap();
        f.store.register_member(input).unwrap();
        assert_eq!(f.store.snapshot().unwrap()["members"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn duplicate_nfc_registration_keeps_existing_member_membership_and_history() {
    let mut f = Fixture::new();
    let input = f.input();
    f.store.register_member(input.clone()).unwrap();
    let before = f.store.snapshot().unwrap();
    let mut duplicate = input;
    duplicate.request_id = id();
    duplicate.name = "Duplicate".into();
    duplicate.nfc_id = "CaRd-1".into();
    assert!(f
        .store
        .register_member(duplicate)
        .unwrap_err()
        .contains("already assigned"));
    assert_eq!(f.store.snapshot().unwrap(), before);
}

#[test]
fn details_only_registration_has_no_membership_and_refuses_unmatched_package_fields() {
    let mut f = Fixture::new();
    let mut input = f.input();
    input.plan_id = None;
    assert!(f.store.register_member(input.clone()).is_err());
    input.plan_version = None;
    input.starts_on = None;
    let result = f.store.register_member(input.clone()).unwrap();
    assert_eq!(result["membershipPeriodId"], Value::Null);
    assert!(f.store.snapshot().unwrap()["periods"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(f.store.snapshot().unwrap()["pending"], 2);
    assert_eq!(f.store.register_member(input).unwrap(), result);
}

#[test]
fn registration_ipc_refuses_caller_supplied_expiry_and_duration() {
    let f = Fixture::new();
    let input = f.input();
    for field in ["endsOn", "durationMonths", "actor", "id"] {
        let mut value = serde_json::to_value(&input).unwrap();
        value[field] = json!("caller supplied");
        assert!(serde_json::from_value::<RegisterMemberInput>(value).is_err());
    }
}

#[test]
fn joining_dues_are_durable_partial_payments_reduce_balance_and_fee_changes_are_future_only() {
    let mut f = Fixture::new();
    let profile = |version, amount, admission_version| ProfileInput {
        version,
        name: "Armstrong Fitness".into(),
        location: "Matale".into(),
        phone: "".into(),
        email: "".into(),
        admission_minor: Some(amount),
        admission_version,
    };
    f.store.save_profile(profile(1, 150000, None)).unwrap();
    let input = f.input();
    let result = f.store.register_member(input.clone()).unwrap();
    let member = result["id"].as_str().unwrap().to_owned();
    let snapshot = f.store.snapshot().unwrap();
    assert_eq!(snapshot["financialAccounts"][0]["outstandingMinor"], 750050);
    assert_eq!(snapshot["invoices"].as_array().unwrap().len(), 2);
    assert_eq!(f.store.register_member(input).unwrap(), result);
    let invoices = snapshot["invoices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap().to_owned())
        .collect();
    let payment = CombinedPaymentInput {
        payment: ReceivePaymentInput {
            request_id: id(),
            member_id: member.clone(),
            amount_minor: 200000,
            method: "Cash".into(),
            invoice_id: None,
        },
        invoice_ids: invoices,
    };
    f.store.receive_combined_payment(payment.clone()).unwrap();
    f.store.receive_combined_payment(payment).unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["financialAccounts"][0]["outstandingMinor"],
        550050
    );
    f.store.save_profile(profile(2, 250000, Some(1))).unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["financialAccounts"][0]["outstandingMinor"],
        550050
    );
    let mut next = f.input();
    next.expected_admission_minor = Some(150000);
    let before = f.store.snapshot().unwrap();
    assert!(f
        .store
        .register_member(next.clone())
        .unwrap_err()
        .contains("Admission fee changed"));
    assert_eq!(f.store.snapshot().unwrap(), before);
    next.expected_admission_minor = Some(250000);
    next.nfc_id = "CARD-2".into();
    let second = f.store.register_member(next).unwrap();
    let reopened = Store::open(&f.path).unwrap().snapshot().unwrap();
    assert_eq!(reopened["profile"]["admissionMinor"], 250000);
    let accounts = reopened["financialAccounts"].as_array().unwrap();
    assert_eq!(
        accounts
            .iter()
            .find(|a| a["memberId"] == second["id"])
            .unwrap()["outstandingMinor"],
        850050
    );
    assert_eq!(reopened["payments"].as_array().unwrap().len(), 1);
    assert!(f.store.save_profile(profile(3, -1, Some(2))).is_err());
    assert!(f.store.save_profile(profile(3, 1, Some(1))).is_err());
    f.store.export_backup().unwrap();
}
