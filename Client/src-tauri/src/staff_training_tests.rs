use super::*;

struct Fixture {
    store: Store,
    path: std::path::PathBuf,
    staff: String,
    plan: String,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("armstrong-staff-{}.sqlite3", id()));
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
        let staff = store.save_trainer(Self::staff_input()).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let plan = store.snapshot().unwrap()["plans"][0]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            store,
            path,
            staff,
            plan,
        }
    }
    fn staff_input() -> TrainerInput {
        TrainerInput {
            request_id: id(),
            id: None,
            version: None,
            name: "Synthetic trainer".into(),
            phone: "0771234567".into(),
            nic: "900000001V".into(),
            salary_minor: 3000000,
            training_fee_minor: 500025,
            active: true,
            nfc_id: None,
        }
    }
    fn registration(&self) -> StaffRegisterInput {
        StaffRegisterInput {
            member: RegisterMemberInput {
                request_id: id(),
                name: "Synthetic training member".into(),
                phone: "0771234567".into(),
                email: "".into(),
                nfc_id: "".into(),
                plan_id: Some(self.plan.clone()),
                plan_version: Some(1),
                starts_on: Some("2026-01-31".into()),
            },
            trainer_id: Some(self.staff.clone()),
            trainer_version: Some(1),
            gender: None,
        }
    }
    fn register(&mut self) -> String {
        self.store
            .register_member_with_trainer(self.registration())
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }
    fn payment(&self, member: &str, amount: i64) -> CombinedPaymentInput {
        let s = self.store.snapshot().unwrap();
        let mut invoices = s["invoices"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["memberId"] == member && i["outstandingMinor"].as_i64().unwrap() > 0)
            .collect::<Vec<_>>();
        invoices.sort_by_key(|i| i["membershipPeriodId"].is_null());
        CombinedPaymentInput {
            payment: ReceivePaymentInput {
                request_id: id(),
                member_id: member.into(),
                amount_minor: amount,
                method: "Cash".into(),
                invoice_id: None,
            },
            invoice_ids: invoices
                .iter()
                .map(|i| i["id"].as_str().unwrap().into())
                .collect(),
        }
    }
    fn payout(&self) -> StaffPayoutInput {
        let s = self.store.snapshot().unwrap();
        let allocations = s["staffTrainingAllocations"].as_array().unwrap();
        StaffPayoutInput {
            request_id: id(),
            trainer_id: self.staff.clone(),
            trainer_version: 1,
            salary_month: business_date(Utc::now())[..7].into(),
            include_salary: true,
            expected_salary_minor: 3000000,
            expected_training_minor: allocations
                .iter()
                .map(|a| a["amountMinor"].as_i64().unwrap())
                .sum(),
            allocation_ids: allocations
                .iter()
                .map(|a| a["id"].as_str().unwrap().into())
                .collect(),
            method: "Bank".into(),
        }
    }
    fn admin(&mut self) {
        self.store.conn.execute_batch("INSERT INTO users VALUES('test-admin','verified-test-subject','admin@example.test','Verified administrator',1,1); INSERT INTO roles VALUES('staff-test-admin','Administrator'); INSERT INTO user_roles VALUES('test-admin','staff-test-admin');").unwrap();
        self.store.removal_session = Some(removal::Session {
            native_nonce: None,
            can_write: true,
            user_id: "test-admin".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        });
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
fn staff_schema_upgrade_preserves_old_device_member_queue_and_frozen_envelope_bytes() {
    let path = std::env::temp_dir().join(format!("armstrong-staff-upgrade-{}.sqlite3", id()));
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    for migration in [
        include_str!("../migrations/001_foundation.sql"),
        include_str!("../migrations/002_local_operations.sql"),
        include_str!("../migrations/003_finance.sql"),
        include_str!("../migrations/004_removal.sql"),
        include_str!("../migrations/005_member_sync.sql"),
        include_str!("../migrations/006_member_conflicts.sql"),
        include_str!("../migrations/007_business_sync.sql"),
    ] {
        conn.execute_batch(migration).unwrap();
    }
    let device = id();
    let member = id();
    let operation = id();
    conn.execute("INSERT INTO metadata VALUES('device_id',?1)", [&device])
        .unwrap();
    conn.execute("INSERT INTO members(id,name,phone,email,nfc_id,joined_on,version) VALUES(?1,'Existing member','0771234567','',NULL,'2026-01-01',1)",[&member]).unwrap();
    conn.execute("INSERT INTO outbox(id,device_id,entity,entity_id,action,expected_version,payload_json,schema_version,created_at) VALUES(?1,?2,'member',?3,'create',NULL,'{}',7,'2026-01-01T00:00:00Z')",params![operation,device,member]).unwrap();
    conn.execute_batch("CREATE TEMP TABLE native_actor(user_id TEXT,label TEXT)")
        .unwrap();
    business_sync::capture(&conn).unwrap();
    let frozen: String = conn
        .query_row("SELECT request_json FROM business_batches", [], |r| {
            r.get(0)
        })
        .unwrap();
    drop(conn);
    let mut store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .conn
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        SCHEMA_VERSION
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        device
    );
    assert_eq!(
        store
            .conn
            .query_row("SELECT request_json FROM business_batches", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        frozen
    );
    assert_eq!(store.snapshot().unwrap()["members"][0]["id"], member);
    assert_eq!(store.snapshot().unwrap()["trainers"], json!([]));
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT schema_version FROM outbox WHERE id=?1",
                [&operation],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
    drop(std::mem::replace(
        &mut store.conn,
        Connection::open_in_memory().unwrap(),
    ));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

#[test]
fn staff_backup_restore_retains_assignments_rates_invoices_payouts_and_recovery_guard() {
    let mut f = Fixture::new();
    f.admin();
    let member = f.register();
    let payment = f.payment(&member, 1100075);
    f.store.receive_combined_payment(payment).unwrap();
    f.store.pay_staff(f.payout()).unwrap();
    let before = f.store.snapshot().unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let restored = f.store.snapshot().unwrap();
    for key in [
        "trainers",
        "memberTrainers",
        "trainingCharges",
        "staffPayouts",
        "staffTrainingAllocations",
        "invoices",
        "allocations",
        "payments",
        "expenses",
    ] {
        assert_eq!(restored[key], before[key], "{key}");
    }
    assert_eq!(restored["restoreRequiresReconciliation"], true);
    assert!(f.store.removal_session.is_none());
}

#[test]
fn staff_registration_bills_membership_and_one_training_month_atomically_and_retries_after_restart()
{
    let mut f = Fixture::new();
    let input = f.registration();
    let result = f.store.register_member_with_trainer(input.clone()).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["invoices"].as_array().unwrap().len(), 2);
    assert_eq!(s["trainingCharges"][0]["feeMinor"], 500025);
    assert_eq!(s["trainingCharges"][0]["endsOn"], "2026-02-28");
    assert_eq!(s["trainers"][0]["assignedMembers"], 1);
    assert_eq!(
        f.store.register_member_with_trainer(input.clone()).unwrap(),
        result
    );
    assert_eq!(f.store.snapshot().unwrap(), s);
    let mut reopened = Store::open(&f.path).unwrap();
    assert_eq!(
        reopened
            .register_member_with_trainer(input.clone())
            .unwrap(),
        result
    );
    assert_eq!(reopened.snapshot().unwrap(), s);
    let mut changed = input;
    changed.trainer_id = None;
    changed.trainer_version = None;
    assert!(reopened.register_member_with_trainer(changed).is_err());
    assert_eq!(reopened.snapshot().unwrap(), s);
}
#[test]
fn staff_nic_mobile_rates_versions_and_inactive_assignment_are_validated_without_partial_writes() {
    let mut f = Fixture::new();
    let before = f.store.snapshot().unwrap();
    for patch in [
        json!({"nic":""}),
        json!({"phone":"not a mobile"}),
        json!({"salaryMinor":-1}),
        json!({"trainingFeeMinor":100000000001_i64}),
        json!({"nic":"900000001v"}),
    ] {
        let mut v = serde_json::to_value(Fixture::staff_input()).unwrap();
        for (k, value) in patch.as_object().unwrap() {
            v[k] = value.clone();
        }
        assert!(f
            .store
            .save_trainer(serde_json::from_value(v).unwrap())
            .is_err());
        assert_eq!(f.store.snapshot().unwrap(), before);
    }
    let mut request = f.registration();
    request.trainer_version = Some(99);
    assert!(f.store.register_member_with_trainer(request).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    let mut trainer = Fixture::staff_input();
    trainer.id = Some(f.staff.clone());
    trainer.version = Some(1);
    trainer.active = false;
    f.store.save_trainer(trainer).unwrap();
    let mut request = f.registration();
    request.trainer_version = Some(2);
    let before = f.store.snapshot().unwrap();
    assert!(f.store.register_member_with_trainer(request).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    let mut request = f.registration();
    request.trainer_id = None;
    request.trainer_version = None;
    f.store.register_member_with_trainer(request).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["trainingCharges"], json!([]));
}
#[test]
fn combined_payment_creates_one_receipt_and_only_collected_training_allocations_become_earnings() {
    let mut f = Fixture::new();
    let member = f.register();
    let before = f.store.snapshot().unwrap();
    assert_eq!(before["trainers"][0]["unpaidTrainingMinor"], 0);
    let partial = f.payment(&member, 600050);
    let paid = f.store.receive_combined_payment(partial.clone()).unwrap();
    assert_eq!(f.store.receive_combined_payment(partial).unwrap(), paid);
    assert_eq!(
        f.store.snapshot().unwrap()["trainers"][0]["unpaidTrainingMinor"],
        0
    );
    let payment = f.payment(&member, 500025);
    let result = f.store.receive_combined_payment(payment.clone()).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["trainers"][0]["unpaidTrainingMinor"], 500025);
    assert!(s["invoices"]
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i["outstandingMinor"] == 0));
    assert_eq!(f.store.receive_combined_payment(payment).unwrap(), result);
    assert_eq!(f.store.snapshot().unwrap(), s);
    f.store
        .reverse_payment(ReversalInput {
            request_id: id(),
            payment_id: result["id"].as_str().unwrap().into(),
            reason: "Synthetic refund".into(),
        })
        .unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["trainers"][0]["unpaidTrainingMinor"],
        0
    );
    let member2 = f.register();
    let combined = f.payment(&member2, 1100075);
    let result = f.store.receive_combined_payment(combined).unwrap();
    let doc = f
        .store
        .payment_receipt(result["id"].as_str().unwrap().into())
        .unwrap();
    assert_eq!(doc["snapshot"]["allocations"].as_array().unwrap().len(), 2);
}
#[test]
fn staff_payouts_pay_salary_once_and_collected_fees_once_with_durable_receipts_and_void_before_refund(
) {
    let mut f = Fixture::new();
    let member = f.register();
    let payment = f.payment(&member, 1100075);
    let received = f.store.receive_combined_payment(payment).unwrap();
    let payout = f.payout();
    let paid = f.store.pay_staff(payout.clone()).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["staffPayouts"][0]["salaryMinor"], 3000000);
    assert_eq!(s["staffPayouts"][0]["trainingMinor"], 500025);
    assert_eq!(s["expenses"][0]["amountMinor"], 3500025);
    assert_eq!(s["trainers"][0]["unpaidTrainingMinor"], 0);
    assert_eq!(f.store.pay_staff(payout).unwrap(), paid);
    assert_eq!(f.store.snapshot().unwrap(), s);
    assert!(f.store.pay_staff(f.payout()).is_err());
    assert_eq!(f.store.snapshot().unwrap(), s);
    let reversal = ReversalInput {
        request_id: id(),
        payment_id: received["id"].as_str().unwrap().into(),
        reason: "Synthetic refund".into(),
    };
    assert!(f
        .store
        .reverse_payment(reversal.clone())
        .unwrap_err()
        .contains("Void the linked"));
    assert_eq!(f.store.snapshot().unwrap(), s);
    f.admin();
    f.store
        .void_expense(ExpenseVoidInput {
            request_id: id(),
            expense_id: paid["expenseId"].as_str().unwrap().into(),
            reason: "Staff payout corrected before refund".into(),
        })
        .unwrap();
    f.store.reverse_payment(reversal).unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["trainers"][0]["unpaidTrainingMinor"],
        0
    );
}
#[test]
fn staff_billing_rejects_overlap_and_stale_earnings_and_freezes_past_rates() {
    let mut f = Fixture::new();
    let member = f.register();
    let before = f.store.snapshot().unwrap();
    let bill = TrainingChargeInput {
        request_id: id(),
        member_id: member.clone(),
        trainer_id: f.staff.clone(),
        trainer_version: 1,
        starts_on: "2026-02-01".into(),
    };
    assert!(f.store.create_training_charge(bill).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    let stale = f.payout();
    let payment = f.payment(&member, 1100075);
    f.store.receive_combined_payment(payment).unwrap();
    let before = f.store.snapshot().unwrap();
    assert!(f.store.pay_staff(stale).is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    let mut edited = Fixture::staff_input();
    edited.id = Some(f.staff.clone());
    edited.version = Some(1);
    edited.training_fee_minor = 700050;
    f.store.save_trainer(edited).unwrap();
    let bill = TrainingChargeInput {
        request_id: id(),
        member_id: member,
        trainer_id: f.staff.clone(),
        trainer_version: 2,
        starts_on: "2026-03-01".into(),
    };
    f.store.create_training_charge(bill).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["trainingCharges"][0]["feeMinor"], 700050);
    assert_eq!(s["trainingCharges"][1]["feeMinor"], 500025);
}
#[test]
fn staff_native_envelopes_export_complete_training_payment_and_payout_groups() {
    let mut f = Fixture::new();
    let member = f.register();
    let payment = f.payment(&member, 1100075);
    f.store.receive_combined_payment(payment).unwrap();
    f.store.pay_staff(f.payout()).unwrap();
    let requests = rows(
        &f.store.conn,
        "SELECT json(request_json) FROM business_batches ORDER BY ordinal",
    )
    .unwrap();
    assert!(requests.iter().any(|b| b["changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["table"] == "staff_payout_items")));
    if let Ok(path) = std::env::var("ARMSTRONG_STAFF_FIXTURE_PATH") {
        std::fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
    }
}

#[test]
fn staff_removal_retains_paid_history_revokes_card_clears_assignments_and_replays_after_restart() {
    let mut f = Fixture::new();
    let member = f.register();
    f.store
        .receive_combined_payment(f.payment(&member, 1100075))
        .unwrap();
    f.store.pay_staff(f.payout()).unwrap();
    let mut card = Fixture::staff_input();
    card.id = Some(f.staff.clone());
    card.version = Some(1);
    card.nfc_id = Some("REMOVED-STAFF-CARD".into());
    f.store.save_trainer(card).unwrap();
    f.store
        .record_staff_attendance(StaffAttendanceInput {
            request_id: id(),
            staff_or_card: "REMOVED-STAFF-CARD".into(),
            source: "NFC".into(),
        })
        .unwrap();
    f.admin();
    let before = f.store.snapshot().unwrap();
    let input = StaffRemovalInput {
        request_id: id(),
        staff_id: f.staff.clone(),
        version: 2,
    };
    let result = f.store.delete_staff(input.clone()).unwrap();
    assert_eq!(f.store.delete_staff(input.clone()).unwrap(), result);
    let after = f.store.snapshot().unwrap();
    assert_eq!(after["trainers"][0]["active"], false);
    assert_eq!(after["trainers"][0]["version"], 3);
    assert!(after["trainers"][0]["deletedAt"].as_str().is_some());
    assert_eq!(after["trainers"][0]["nfcId"], "");
    assert_eq!(after["trainers"][0]["assignedMembers"], 0);
    assert!(after["memberTrainers"][0]["trainerId"].is_null());
    for key in [
        "members",
        "periods",
        "invoices",
        "payments",
        "allocations",
        "trainingCharges",
        "staffPayouts",
        "expenses",
        "staffAttendance",
    ] {
        assert_eq!(before[key], after[key], "Removal changed historical {key}");
    }
    assert!(f
        .store
        .record_staff_attendance(StaffAttendanceInput {
            request_id: id(),
            staff_or_card: "REMOVED-STAFF-CARD".into(),
            source: "NFC".into(),
        })
        .is_err());
    assert_eq!(after["audit"][0]["action"], "delete staff");
    assert_eq!(
        after["audit"][0]["user"],
        "Verified administrator (test-admin)"
    );
    let requests = rows(
        &f.store.conn,
        "SELECT json(request_json) FROM business_batches ORDER BY ordinal",
    )
    .unwrap();
    if let Ok(path) = std::env::var("ARMSTRONG_STAFF_REMOVAL_FIXTURE_PATH") {
        std::fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
    }
    let mut reopened = Store::open(&f.path).unwrap();
    assert!(reopened.delete_staff(input.clone()).is_err());
    reopened.removal_session = f.store.removal_session.clone();
    assert_eq!(reopened.delete_staff(input).unwrap(), result);
    assert_eq!(reopened.snapshot().unwrap()["trainers"], after["trainers"]);
    let mut edit = Fixture::staff_input();
    edit.id = Some(f.staff.clone());
    edit.version = Some(3);
    assert!(reopened
        .save_trainer(edit)
        .unwrap_err()
        .contains("cannot be edited"));
    assert!(reopened
        .conn
        .execute(
            "UPDATE staff_deletions SET deleted_at='2026-10-09T00:00:00Z'",
            []
        )
        .is_err());
    assert!(reopened
        .conn
        .execute("DELETE FROM staff_deletions", [])
        .is_err());
    let backup = reopened.backup_envelope().unwrap();
    let preview = reopened.preview_restore(backup).unwrap();
    reopened
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    assert_eq!(reopened.snapshot().unwrap()["trainers"], after["trainers"]);
}

#[test]
fn inactive_staff_can_be_permanently_deleted_and_cannot_be_deleted_twice() {
    let mut f = Fixture::new();
    let mut inactive = Fixture::staff_input();
    inactive.id = Some(f.staff.clone());
    inactive.version = Some(1);
    inactive.active = false;
    f.store.save_trainer(inactive).unwrap();
    f.admin();
    f.store
        .delete_staff(StaffRemovalInput {
            request_id: id(),
            staff_id: f.staff.clone(),
            version: 2,
        })
        .unwrap();
    let after = f.store.snapshot().unwrap();
    assert_eq!(after["trainers"][0]["active"], false);
    assert!(after["trainers"][0]["deletedAt"].as_str().is_some());
    assert!(f
        .store
        .delete_staff(StaffRemovalInput {
            request_id: id(),
            staff_id: f.staff.clone(),
            version: 3
        })
        .unwrap_err()
        .contains("already deleted"));
    assert_eq!(f.store.snapshot().unwrap(), after);
}

#[test]
fn staff_removal_denies_unauthorized_stale_and_failed_outbox_without_partial_changes() {
    let mut f = Fixture::new();
    f.register();
    let input = StaffRemovalInput {
        request_id: id(),
        staff_id: f.staff.clone(),
        version: 1,
    };
    let before = f.store.snapshot().unwrap();
    assert!(f
        .store
        .delete_staff(input.clone())
        .unwrap_err()
        .contains("authenticated Administrator"));
    assert_eq!(f.store.snapshot().unwrap(), before);
    f.admin();
    let before = f.store.snapshot().unwrap();
    let mut stale = input.clone();
    stale.version = 2;
    assert!(f.store.delete_staff(stale).unwrap_err().contains("Refresh"));
    assert_eq!(f.store.snapshot().unwrap(), before);
    let session = f.store.removal_session.clone().unwrap();
    f.store.removal_session.as_mut().unwrap().can_write = false;
    assert!(f.store.delete_staff(input.clone()).is_err());
    f.store.removal_session = Some(session.clone());
    f.store.removal_session.as_mut().unwrap().expires_at =
        Utc::now() - chrono::Duration::seconds(1);
    assert!(f.store.delete_staff(input.clone()).is_err());
    f.store.removal_session = Some(session);
    f.store.conn.execute_batch("CREATE TRIGGER reject_staff_removal_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'Synthetic outbox failure'); END;").unwrap();
    assert!(f
        .store
        .delete_staff(input)
        .unwrap_err()
        .contains("Synthetic outbox failure"));
    assert_eq!(f.store.snapshot().unwrap(), before);
}

#[test]
fn removed_staff_retains_collected_earnings_and_can_receive_a_final_payout() {
    let mut f = Fixture::new();
    let member = f.register();
    f.store
        .receive_combined_payment(f.payment(&member, 1100075))
        .unwrap();
    f.admin();
    f.store
        .delete_staff(StaffRemovalInput {
            request_id: id(),
            staff_id: f.staff.clone(),
            version: 1,
        })
        .unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["trainers"][0]["unpaidTrainingMinor"],
        500025
    );
    let mut payout = f.payout();
    payout.trainer_version = 2;
    payout.include_salary = false;
    payout.expected_salary_minor = 0;
    let paid = f.store.pay_staff(payout.clone()).unwrap();
    assert_eq!(f.store.pay_staff(payout).unwrap(), paid);
    let after = f.store.snapshot().unwrap();
    assert_eq!(after["trainers"][0]["active"], false);
    assert_eq!(after["trainers"][0]["unpaidTrainingMinor"], 0);
    assert_eq!(after["staffPayouts"][0]["salaryMinor"], 0);
    assert_eq!(after["staffPayouts"][0]["trainingMinor"], 500025);
}
