use super::*;

fn range(from: &str, to: &str) -> ReportRange {
    ReportRange {
        from_on: Some(from.into()),
        to_on: Some(to.into()),
    }
}
fn csv(store: &Store, kind: &str, range: ReportRange) -> (Value, String) {
    let result = store.export_report_range(kind.into(), range).unwrap();
    let contents = std::fs::read_to_string(result["path"].as_str().unwrap()).unwrap();
    (result, contents)
}

#[test]
fn report_range_financial_totals_and_csv_reconcile_posted_reversal_dates() {
    let mut f = Fixture::new();
    f.store.save_member(member("")).unwrap();
    let member = first_id(&f.store, "members");
    let original = id();
    f.store.conn.execute("INSERT INTO payments VALUES(?1,?2,'Test member',12345,'Cash','2026-10-01','2026-10-01T01:00:00Z','Test actor',NULL)", params![original,member]).unwrap();
    f.store.conn.execute("INSERT INTO payments VALUES(?1,?2,'Test member',12345,'Cash','2026-10-02','2026-10-02T01:00:00Z','Test actor',?3)", params![id(),member,original]).unwrap();
    f.store.conn.execute("INSERT INTO expenses VALUES(?1,'October electricity','Utilities',2345,'Bank','2026-10-01','2026-10-01T01:00:00Z','Test actor',NULL)", [id()]).unwrap();
    let first = f
        .store
        .report_summary(range("2026-10-01", "2026-10-01"))
        .unwrap();
    assert_eq!(first["incomeMinor"], 12345);
    assert_eq!(first["expenseMinor"], 2345);
    assert_eq!(first["netMinor"], 10000);
    let second = f
        .store
        .report_summary(range("2026-10-02", "2026-10-02"))
        .unwrap();
    assert_eq!(second["incomeMinor"], -12345);
    assert_eq!(second["expenseMinor"], 0);
    assert_eq!(second["netMinor"], -12345);
    let all = f.store.report_summary(ReportRange::default()).unwrap();
    assert_eq!(all["incomeMinor"], 0);
    assert_eq!(all["netMinor"], -2345);
    let (export, first_csv) = csv(&f.store, "Income report", range("2026-10-01", "2026-10-01"));
    assert_eq!(export["rows"], 1);
    assert!(first_csv.contains("123.45") && !first_csv.contains("2026-10-02"));
    let (_, second_csv) = csv(&f.store, "Income report", range("2026-10-02", "2026-10-02"));
    assert!(second_csv.contains("'-123.45")); // Spreadsheet-safe negative amount.
}

#[test]
fn report_range_voided_expenses_preserve_csv_history_and_do_not_add_expense() {
    let mut f = Fixture::new();
    let expense = f.store.record_expense(expense()).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    removal_admin(&mut f.store);
    f.store
        .void_expense(ExpenseVoidInput {
            request_id: id(),
            expense_id: expense,
            reason: "Duplicate bill".into(),
        })
        .unwrap();
    let today = business_date(Utc::now());
    let summary = f.store.report_summary(range(&today, &today)).unwrap();
    assert_eq!(summary["expenseMinor"], 0);
    let (result, contents) = csv(&f.store, "Expense report", range(&today, &today));
    assert_eq!(result["rows"], 1);
    assert!(
        contents.contains("1250.75")
            && contents.contains("Voided")
            && contents.contains("Duplicate bill")
    );
}

#[test]
fn report_range_attendance_and_audit_use_colombo_midnight_and_inclusive_dates() {
    let mut f = Fixture::new();
    f.store.save_member(member("")).unwrap();
    let member = first_id(&f.store, "members");
    for time in [
        "2026-10-02T18:29:59Z",
        "2026-10-02T18:30:00Z",
        "2026-10-03T18:29:59Z",
        "2026-10-03T18:30:00Z",
    ] {
        f.store
            .attendance_at(
                AttendanceInput {
                    request_id: id(),
                    member_or_card: member.clone(),
                    source: "Manual".into(),
                },
                time.parse().unwrap(),
            )
            .unwrap();
        f.store.conn.execute("INSERT INTO audit(id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity) VALUES(?1,'Test actor','Test device','Boundary test',?2,NULL,'{}',?3,'attendance')", params![id(),member,time]).unwrap();
    }
    let summary = f
        .store
        .report_summary(range("2026-10-03", "2026-10-03"))
        .unwrap();
    assert_eq!(summary["attendanceCount"], 2);
    let (export, contents) = csv(
        &f.store,
        "Attendance report",
        range("2026-10-03", "2026-10-03"),
    );
    assert_eq!(export["rows"], 2);
    assert!(contents.contains("2026-10-02T18:30:00") && contents.contains("2026-10-03T18:29:59"));
    assert!(!contents.contains("2026-10-02T18:29:59") && !contents.contains("2026-10-03T18:30:00"));
    let (_, audit) = csv(&f.store, "Audit report", range("2026-10-03", "2026-10-03"));
    assert_eq!(audit.matches("Boundary test").count(), 2);
}

#[test]
fn report_range_memberships_use_overlap_and_inventory_remains_current() {
    let mut f = Fixture::new();
    f.store.save_member(member("")).unwrap();
    f.store.save_plan(plan()).unwrap();
    let member = first_id(&f.store, "members");
    let plan = first_id(&f.store, "plans");
    for (from, to) in [
        ("2026-09-01", "2026-09-30"),
        ("2026-10-01", "2026-10-31"),
        ("2026-11-01", "2026-11-30"),
    ] {
        f.store
            .add_period(PeriodInput {
                member_id: member.clone(),
                plan_id: plan.clone(),
                starts_on: from.into(),
                ends_on: to.into(),
            })
            .unwrap();
    }
    f.store.save_product(product()).unwrap();
    let summary = f
        .store
        .report_summary(range("2026-10-31", "2026-11-01"))
        .unwrap();
    assert_eq!(summary["membershipPeriodCount"], 2);
    assert_eq!(summary["inventoryValueMinor"], 50125);
    let (periods, contents) = csv(
        &f.store,
        "Membership report",
        range("2026-10-31", "2026-11-01"),
    );
    assert_eq!(periods["rows"], 2);
    assert!(!contents.contains("2026-09"));
    let (inventory, _) = csv(
        &f.store,
        "Inventory report",
        range("1900-01-01", "1900-01-01"),
    );
    assert_eq!(inventory["rows"], 1);
    let empty = f
        .store
        .report_summary(range("1900-01-01", "1900-01-01"))
        .unwrap();
    assert_eq!(empty["membershipPeriodCount"], 0);
    assert_eq!(empty["inventoryValueMinor"], 50125);
}

#[test]
fn report_range_invalid_inputs_refuse_export_and_open_bounds_are_supported() {
    let f = Fixture::new();
    for bounds in [
        range("2026-10-04", "2026-10-03"),
        range("2026-02-30", "2026-10-03"),
        range("2026-1-01", "2026-10-03"),
        range("", "2026-10-03"),
        range("1899-12-31", "2026-10-03"),
    ] {
        assert!(f.store.report_summary(bounds.clone()).is_err());
        assert!(f
            .store
            .export_report_range("Income report".into(), bounds)
            .is_err());
    }
    for bounds in [
        ReportRange {
            from_on: Some("2026-10-01".into()),
            to_on: None,
        },
        ReportRange {
            from_on: None,
            to_on: Some("2026-10-01".into()),
        },
    ] {
        assert_eq!(f.store.report_summary(bounds).unwrap()["incomeMinor"], 0);
    }
    assert!(f.store.export_report("Not a report".into()).is_err());
    assert!(serde_json::from_value::<ReportRange>(json!({"gymId":id()})).is_err());
}

#[test]
fn report_range_unsafe_totals_fail_without_rounding_and_exports_do_not_mutate_history() {
    let mut f = Fixture::new();
    let mut product = product();
    product.cost_minor = 100_000_000_000;
    product.opening_stock = 1_000_000;
    f.store.save_product(product).unwrap();
    assert!(f.store.report_summary(ReportRange::default()).is_err());
    let before = f.store.snapshot().unwrap();
    f.store
        .export_report_range("Inventory report".into(), ReportRange::default())
        .unwrap();
    assert_eq!(f.store.snapshot().unwrap(), before);
}
