// Real isolated SQLite and trusted test sessions; no live enrollment or HTTPS.
use super::*;
use crate::member_conflicts::ConflictChoice;
use crate::member_worker::SyncScope;

fn enrolled(f: &mut Fixture) {
    removal_admin(&mut f.store);
    f.store
        .conn
        .execute("UPDATE users SET subject=?1 WHERE id='test-admin'", [id()])
        .unwrap();
    let device: String = f
        .store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    f.store
        .bind_member_scope(SyncScope::new("https://gym.example", &id(), &device).unwrap())
        .unwrap();
}
fn remote(f: &Fixture) -> Value {
    let local = &f.store.snapshot().unwrap()["members"][0];
    json!({"id":local["id"],"name":"Recorded server member","phone":"0777654321","email":"server@example.test","nfcId":"SERVER-CARD","joinedOn":local["joinedOn"],"revision":3,"archivedAt":null,"archivedBy":null})
}
fn rejected() -> (Fixture, Value, Value) {
    let mut f = Fixture::new();
    enrolled(&mut f);
    f.store.save_member(member("LOCAL-CARD")).unwrap();
    let request = f.store.prepare_member_push().unwrap().unwrap();
    let remote = remote(&f);
    f.store
        .reject_member_push(
            request["operationId"].as_str().unwrap(),
            "Server revision changed",
            &remote,
            true,
        )
        .unwrap();
    (f, request, remote)
}
fn preview(f: &Fixture) -> Value {
    let conflict = f.store.snapshot().unwrap()["memberSync"]["conflicts"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.store.preview_member_conflict(&conflict).unwrap()
}
fn input(review: &Value, choice: ConflictChoice) -> MemberConflictInput {
    MemberConflictInput {
        request_id: id(),
        conflict_id: review["conflictId"].as_str().unwrap().into(),
        fingerprint: review["fingerprint"].as_str().unwrap().into(),
        choice,
        reason: "Administrator reviewed the recorded versions".into(),
    }
}
fn edit(f: &mut Fixture, name: &str) {
    let local = &f.store.snapshot().unwrap()["members"][0];
    let mut value = member(local["nfcId"].as_str().unwrap());
    value.id = Some(local["id"].as_str().unwrap().into());
    value.version = local["version"].as_i64();
    value.name = name.into();
    f.store.save_member(value).unwrap();
}
fn delivery(f: &Fixture, operation: &str) -> (String, String) {
    f.store
        .conn
        .query_row(
            "SELECT request_json,state FROM member_deliveries WHERE operation_id=?1",
            [operation],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}
fn count(f: &Fixture, table: &str) -> i64 {
    f.store
        .conn
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[test]
fn server_choice_preserves_money_dates_cards_original_edits_and_receipts() {
    let (mut f, request, _) = rejected();
    edit(&mut f, "Later local edit");
    f.store.save_plan(plan()).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    let payment = f
        .store
        .record_payment(PaymentInput {
            request_id: id(),
            member_id: first_id(&f.store, "members"),
            amount_minor: 12345,
            method: "Cash".into(),
        })
        .unwrap();
    let receipt = f
        .store
        .payment_receipt(payment["id"].as_str().unwrap().into())
        .unwrap();
    let before = f.store.snapshot().unwrap();
    let old_delivery = delivery(&f, request["operationId"].as_str().unwrap());
    let old_outbox = operations::rows(
        &f.store.conn,
        "SELECT json_object('id',id,'payload',payload_json) FROM outbox ORDER BY rowid",
    )
    .unwrap();
    let review = preview(&f);
    let command = input(&review, ConflictChoice::UseServer);
    let result = f.store.resolve_member_conflict(command.clone()).unwrap();
    let after = f.store.snapshot().unwrap();
    assert_eq!(result["serverConfirmed"], false);
    assert_eq!(result["superseded"], 2);
    assert_eq!(after["members"][0]["name"], "Recorded server member");
    assert_eq!(after["members"][0]["version"], 3);
    assert_eq!(after["members"][0]["nfcId"], "SERVER-CARD");
    for field in ["periods", "payments", "invoices", "allocations"] {
        assert_eq!(before[field], after[field], "{field}");
    }
    assert_eq!(
        f.store
            .payment_receipt(payment["id"].as_str().unwrap().into())
            .unwrap(),
        receipt
    );
    assert_eq!(
        operations::rows(
            &f.store.conn,
            "SELECT json_object('id',id,'payload',payload_json) FROM outbox ORDER BY rowid"
        )
        .unwrap(),
        old_outbox
    );
    assert_eq!(
        delivery(&f, request["operationId"].as_str().unwrap()),
        old_delivery
    );
    assert_eq!(count(&f, "nfc_cards"), 2);
    assert_eq!(after["memberSync"]["acknowledged"], 0);
    assert_eq!(after["memberSync"]["cursor"], 0);
    assert_eq!(after["memberSync"]["conflicts"], json!([]));
    assert_eq!(
        after["pending"].as_i64(),
        Some(before["pending"].as_i64().unwrap() - 2)
    );
    let actor: String = f
        .store
        .conn
        .query_row(
            "SELECT actor_user_id FROM audit WHERE action='resolve member conflict'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actor, "test-admin");
    assert_eq!(
        f.store.resolve_member_conflict(command.clone()).unwrap(),
        result
    );
    assert_eq!(f.store.snapshot().unwrap(), after);
    let mut reused = command;
    reused.reason = "Different choice reason".into();
    assert!(f
        .store
        .resolve_member_conflict(reused)
        .unwrap_err()
        .contains("reused"));
    assert!(f.store.prepare_member_push().unwrap().is_none());
}
#[test]
fn keep_local_queues_a_new_request_against_recorded_revision_without_rewriting_old_request() {
    let (mut f, request, _) = rejected();
    edit(&mut f, "Latest local name");
    let old_delivery = delivery(&f, request["operationId"].as_str().unwrap());
    let review = preview(&f);
    let result = f
        .store
        .resolve_member_conflict(input(&review, ConflictChoice::KeepLocal))
        .unwrap();
    let next = f.store.prepare_member_push().unwrap().unwrap();
    assert_eq!(next["operationId"], result["retryOperationId"]);
    assert_ne!(next["operationId"], request["operationId"]);
    assert_eq!(next["action"], "update");
    assert_eq!(next["expectedRevision"], 3);
    assert_eq!(next["member"]["name"], "Latest local name");
    assert_eq!(next["member"]["nfcId"], "LOCAL-CARD");
    assert_eq!(
        delivery(&f, request["operationId"].as_str().unwrap()),
        old_delivery
    );
    assert_eq!(count(&f, "member_resolved_operations"), 2);
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert_eq!(f.store.snapshot().unwrap()["memberSync"]["acknowledged"], 0);
    assert_eq!(f.store.prepare_member_push().unwrap().unwrap(), next);
    let mut member = remote(&f);
    member["revision"] = json!(1);
    let late=serde_json::from_value(json!({"protocolVersion":1,"operationId":request["operationId"],"memberId":request["memberId"],"revision":1,"sequence":1,"member":member})).unwrap();
    assert!(f
        .store
        .acknowledge_member(late)
        .unwrap_err()
        .contains("cannot be acknowledged"));
}
#[test]
fn changed_member_or_conflict_invalidates_the_review_atomically() {
    for new_conflict in [false, true] {
        let (mut f, request, remote) = rejected();
        let review = preview(&f);
        if new_conflict {
            f.store
                .conn
                .execute(
                    "INSERT INTO member_sync_conflicts VALUES(?1,?2,?3,'New server review',?4,?5)",
                    params![
                        id(),
                        request["memberId"].as_str(),
                        id(),
                        remote.to_string(),
                        Utc::now().to_rfc3339()
                    ],
                )
                .unwrap();
        } else {
            edit(&mut f, "Changed after review");
        }
        let before = f.store.snapshot().unwrap();
        assert!(f
            .store
            .resolve_member_conflict(input(&review, ConflictChoice::UseServer))
            .unwrap_err()
            .contains("changed"));
        assert_eq!(f.store.snapshot().unwrap(), before);
        assert_eq!(count(&f, "member_conflict_resolutions"), 0);
        let current = preview(&f);
        f.store
            .resolve_member_conflict(input(&current, ConflictChoice::KeepLocal))
            .unwrap();
    }
}
#[test]
fn native_role_scope_writer_expiry_and_restore_gates_are_checked_again_on_commit() {
    for gate in [
        "logout", "expiry", "readonly", "inactive", "role", "subject", "scope", "device", "restore",
    ] {
        let (mut f, _, _) = rejected();
        let review = preview(&f);
        match gate {
            "logout" => f.store.removal_session = None,
            "expiry" => {
                f.store.removal_session.as_mut().unwrap().expires_at =
                    Utc::now() - chrono::Duration::seconds(1)
            }
            "readonly" => f.store.removal_session.as_mut().unwrap().can_write = false,
            "inactive" => {
                f.store
                    .conn
                    .execute("UPDATE users SET active=0", [])
                    .unwrap();
            }
            "role" => {
                f.store
                    .conn
                    .execute("UPDATE roles SET name='Reception'", [])
                    .unwrap();
            }
            "subject" => {
                f.store
                    .conn
                    .execute("UPDATE users SET subject='unverified'", [])
                    .unwrap();
            }
            "scope" => {
                f.store
                    .conn
                    .execute("DELETE FROM metadata WHERE key='member_sync_scope'", [])
                    .unwrap();
            }
            "device" => {
                f.store
                    .conn
                    .execute("UPDATE metadata SET value=?1 WHERE key='device_id'", [id()])
                    .unwrap();
            }
            "restore" => {
                f.store
                    .conn
                    .execute(
                        "INSERT INTO metadata VALUES('restore_requires_reconciliation','true')",
                        [],
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            f.store
                .preview_member_conflict(review["conflictId"].as_str().unwrap())
                .is_err(),
            "{gate}"
        );
        assert!(
            f.store
                .resolve_member_conflict(input(&review, ConflictChoice::UseServer))
                .is_err(),
            "{gate}"
        );
        assert_eq!(count(&f, "member_conflict_resolutions"), 0, "{gate}");
    }
}
#[test]
fn incomplete_wrong_identity_inconsistent_and_changed_history_require_server_review() {
    for case in [
        "null",
        "identity",
        "joined",
        "invalid",
        "same_revision",
        "older_inconsistent",
    ] {
        let (mut f, request, mut remote) = rejected();
        match case {
            "null" => remote = Value::Null,
            "identity" => remote["id"] = json!(id()),
            "joined" => remote["joinedOn"] = json!("2025-01-01"),
            "invalid" => remote["nfcId"] = json!("lowercase"),
            "same_revision" | "older_inconsistent" => {
                let mut head = remote.clone();
                head["name"] = json!("Different snapshot");
                if case == "older_inconsistent" {
                    head["revision"] = json!(5);
                }
                f.store
                    .conn
                    .execute(
                        "INSERT INTO member_remote_heads VALUES(?1,?2,?3)",
                        params![
                            request["memberId"].as_str(),
                            head["revision"].as_i64(),
                            head.to_string()
                        ],
                    )
                    .unwrap();
                if case == "older_inconsistent" {
                    let mut second = remote.clone();
                    second["name"] = json!("Inconsistent old revision");
                    f.store.conn.execute("INSERT INTO member_sync_conflicts VALUES(?1,?2,NULL,'Older duplicate',?3,?4)",params![id(),request["memberId"].as_str(),second.to_string(),Utc::now().to_rfc3339()]).unwrap();
                }
            }
            _ => unreachable!(),
        }
        if !matches!(case, "same_revision" | "older_inconsistent") {
            // Initial rejection is immutable; a new conflicting server observation must be retained.
            f.store.conn.execute("INSERT INTO member_sync_conflicts VALUES(?1,?2,NULL,'Recorded observation',?3,?4)",params![id(),request["memberId"].as_str(),remote.to_string(),Utc::now().to_rfc3339()]).unwrap();
        }
        let review = preview(&f);
        assert_eq!(review["useServer"]["allowed"], false, "{case}");
        assert_eq!(review["keepLocal"]["allowed"], false, "{case}");
        assert!(f
            .store
            .resolve_member_conflict(input(&review, ConflictChoice::UseServer))
            .is_err());
        assert_eq!(count(&f, "member_conflict_resolutions"), 0);
    }
}
#[test]
fn lost_or_inflight_requests_cannot_be_retired_even_by_a_direct_sql_insert() {
    let (mut f, request, remote) = rejected();
    f.store
        .conn
        .execute(
            "UPDATE member_deliveries SET state='pending' WHERE operation_id=?1",
            [request["operationId"].as_str().unwrap()],
        )
        .unwrap();
    let review = preview(&f);
    assert!(review["useServer"]["reason"]
        .as_str()
        .unwrap()
        .contains("unconfirmed"));
    assert_eq!(review["keepLocal"]["allowed"], false);
    assert!(f
        .store
        .resolve_member_conflict(input(&review, ConflictChoice::KeepLocal))
        .is_err());
    let resolution = id();
    f.store.conn.execute("INSERT INTO member_conflict_resolutions VALUES(?1,?2,'test-admin','Test','use_server','Reviewed','{}',?3)",params![resolution,remote["id"].as_str(),Utc::now().to_rfc3339()]).unwrap();
    assert!(f
        .store
        .conn
        .execute(
            "INSERT INTO member_resolved_operations VALUES(?1,?2)",
            params![request["operationId"].as_str(), resolution]
        )
        .is_err());
    assert_eq!(count(&f, "member_resolved_operations"), 0);
}
#[test]
fn colliding_cards_block_server_choice_and_archives_cannot_be_reactivated_or_retried_as_edits() {
    let (mut f, _, _) = rejected();
    f.store.save_member(member("SERVER-CARD")).unwrap();
    let review = preview(&f);
    assert_eq!(review["useServer"]["allowed"], false);
    assert!(review["useServer"]["reason"]
        .as_str()
        .unwrap()
        .contains("another"));
    assert_eq!(review["keepLocal"]["allowed"], true);
    assert!(f
        .store
        .resolve_member_conflict(input(&review, ConflictChoice::UseServer))
        .is_err());
    let (mut f, _, _) = rejected();
    f.store
        .archive_member(removal_member_input(&f.store))
        .unwrap();
    let review = preview(&f);
    assert_eq!(review["useServer"]["allowed"], false);
    assert_eq!(review["keepLocal"]["allowed"], false);
    assert_eq!(count(&f, "member_conflict_resolutions"), 0);
}
#[test]
fn accepting_server_archive_imports_only_an_inactive_identity_reference() {
    let (mut f, request, mut remote) = rejected();
    let actor = id();
    remote["revision"] = json!(4);
    remote["archivedAt"] = json!(Utc::now().to_rfc3339());
    remote["archivedBy"] = json!({"id":actor,"name":"Remote administrator"});
    f.store
        .conn
        .execute(
            "INSERT INTO member_sync_conflicts VALUES(?1,?2,NULL,'Server archive',?3,?4)",
            params![
                id(),
                request["memberId"].as_str(),
                remote.to_string(),
                Utc::now().to_rfc3339()
            ],
        )
        .unwrap();
    let review = preview(&f);
    assert_eq!(review["useServer"]["allowed"], true);
    assert_eq!(review["keepLocal"]["allowed"], false);
    f.store
        .resolve_member_conflict(input(&review, ConflictChoice::UseServer))
        .unwrap();
    let after = f.store.snapshot().unwrap();
    assert_eq!(after["members"][0]["active"], false);
    assert_eq!(after["members"][0]["archivedByUserId"], actor);
    let user = after["users"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["id"] == actor)
        .unwrap();
    assert_eq!(user["active"], 0);
    assert_eq!(user["roles"], json!([]));
}
#[test]
fn resolution_history_is_immutable_and_cannot_retire_another_member_or_finance() {
    let (mut f, _, _) = rejected();
    let review = preview(&f);
    let result = f
        .store
        .resolve_member_conflict(input(&review, ConflictChoice::UseServer))
        .unwrap();
    for table in [
        "member_sync_conflicts",
        "member_conflict_resolutions",
        "member_resolved_conflicts",
        "member_resolved_operations",
    ] {
        assert!(
            f.store
                .conn
                .execute(&format!("DELETE FROM {table}"), [])
                .is_err(),
            "{table}"
        );
        assert!(
            f.store
                .conn
                .execute(&format!("UPDATE {table} SET rowid=rowid"), [])
                .is_err(),
            "{table}"
        );
    }
    f.store.save_member(member("OTHER")).unwrap();
    f.store.save_plan(plan()).unwrap();
    let other: String = f
        .store
        .conn
        .query_row(
            "SELECT id FROM outbox WHERE entity='member' ORDER BY rowid DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let finance: String = f
        .store
        .conn
        .query_row("SELECT id FROM outbox WHERE entity='plan'", [], |r| {
            r.get(0)
        })
        .unwrap();
    for operation in [other, finance] {
        assert!(f
            .store
            .conn
            .execute(
                "INSERT INTO member_resolved_operations VALUES(?1,?2)",
                params![operation, result["resolutionId"].as_str()]
            )
            .is_err());
    }
}
#[test]
fn failed_resolution_rolls_back_member_queue_audit_and_ledger_together() {
    let (mut f, _, _) = rejected();
    let review = preview(&f);
    let before = f.store.snapshot().unwrap();
    f.store.conn.execute_batch("CREATE TRIGGER fail_resolution BEFORE INSERT ON audit WHEN NEW.action='resolve member conflict' BEGIN SELECT RAISE(ABORT,'Injected failure'); END;").unwrap();
    assert!(f
        .store
        .resolve_member_conflict(input(&review, ConflictChoice::KeepLocal))
        .is_err());
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert_eq!(count(&f, "member_conflict_resolutions"), 0);
    assert_eq!(count(&f, "member_resolved_operations"), 0);
}
#[test]
fn conflict_resolution_survives_validated_backup_restore_and_still_requires_reconciliation() {
    let (mut f, _, _) = rejected();
    let review = preview(&f);
    f.store
        .resolve_member_conflict(input(&review, ConflictChoice::KeepLocal))
        .unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let mut restored = Fixture::new();
    let p = restored.store.preview_restore(backup).unwrap();
    restored
        .store
        .restore_backup(p["token"].as_str().unwrap().into())
        .unwrap();
    for table in [
        "member_conflict_resolutions",
        "member_resolved_conflicts",
        "member_resolved_operations",
        "member_sync_conflicts",
    ] {
        assert_eq!(count(&f, table), count(&restored, table), "{table}");
    }
    assert_eq!(count(&restored, "outbox"), count(&f, "outbox") + 1); // Explicit restore audit operation.
    assert_eq!(
        restored.store.snapshot().unwrap()["restoreRequiresReconciliation"],
        true
    );
    assert!(restored.store.prepare_member_push().is_err());
}

#[test]
fn populated_schema_five_migration_preserves_history_and_old_backups_migrate_in_isolation() {
    use sha2::{Digest, Sha256};
    let (path, conn) = v1_database();
    conn.execute_batch(include_str!("../migrations/002_local_operations.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../migrations/003_finance.sql"))
        .unwrap();
    finance::migrate_receipts(&conn.unchecked_transaction().unwrap()).unwrap();
    conn.execute_batch(include_str!("../migrations/004_removal.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../migrations/005_member_sync.sql"))
        .unwrap();
    conn.execute_batch("INSERT INTO member_deliveries(operation_id,request_json,state) VALUES('old-operation','{\"original\":true}','conflict'); INSERT INTO member_sync_conflicts VALUES('old-conflict','old-member','old-operation','Recorded rejection','null','2026-01-01T00:00:00Z'); INSERT INTO member_remote_heads VALUES('old-member',2,'{\"originalHead\":true}'); UPDATE member_sync_cursor SET sequence=7;").unwrap();
    drop(conn);
    let data = std::fs::read(&path).unwrap();
    let backup = BackupEnvelope {
        format: "armstrong-sqlite-backup".into(),
        format_version: 1,
        schema_version: 5,
        created_at: Utc::now().to_rfc3339(),
        sha256: format!("{:x}", Sha256::digest(&data)),
        data,
    };
    let f = Fixture {
        store: Store::open(&path).unwrap(),
        path,
    };
    assert_eq!(f.store.snapshot().unwrap()["members"][0]["version"], 3);
    assert_eq!(
        f.store.snapshot().unwrap()["periods"][0]["priceMinor"],
        500025
    );
    assert_eq!(f.store.snapshot().unwrap()["memberSync"]["cursor"], 7);
    assert_eq!(
        f.store.snapshot().unwrap()["memberSync"]["conflicts"][0]["id"],
        "old-conflict"
    );
    assert_eq!(
        delivery(&f, "old-operation"),
        ("{\"original\":true}".into(), "conflict".into())
    );
    assert_eq!(count(&f, "member_conflict_resolutions"), 0);
    let mut restored = Fixture::new();
    let preview = restored.store.preview_restore(backup).unwrap();
    restored
        .store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    assert_eq!(
        restored.store.snapshot().unwrap()["memberSync"]["cursor"],
        7
    );
    assert_eq!(
        delivery(&restored, "old-operation"),
        delivery(&f, "old-operation")
    );
    assert_eq!(
        restored.store.snapshot().unwrap()["members"],
        f.store.snapshot().unwrap()["members"]
    );
    let version: i64 = restored
        .store
        .conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 6);
}
