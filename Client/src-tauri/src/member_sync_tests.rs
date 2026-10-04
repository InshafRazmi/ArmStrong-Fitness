use super::*;
use crate::member_sync::{Page, Receipt};
fn remote(member_id: &str, revision: i64, card: Option<&str>) -> Value {
    json!({"id":member_id,"revision":revision,"name":"Remote member","phone":"0771234567","email":"","nfcId":card,"joinedOn":"2026-10-03","archivedAt":null,"archivedBy":null})
}
fn page(after: i64, members: Vec<Value>) -> Page {
    serde_json::from_value(json!({"protocolVersion":1,"after":after,"nextCursor":after+members.len() as i64,"hasMore":false,"changes":members.iter().enumerate().map(|(i,m)|json!({"sequence":after+i as i64+1,"operationId":id(),"member":m})).collect::<Vec<_>>()})).unwrap()
}
fn ack(request: &Value, sequence: i64) -> Receipt {
    let mut member = request["member"].clone();
    member["id"] = request["memberId"].clone();
    member["revision"] = json!(request["expectedRevision"].as_i64().unwrap() + 1);
    member["archivedAt"] = Value::Null;
    member["archivedBy"] = Value::Null;
    serde_json::from_value(json!({"protocolVersion":1,"operationId":request["operationId"],"memberId":request["memberId"],"revision":member["revision"],"sequence":sequence,"member":member})).unwrap()
}
#[test]
fn frozen_retry_ack_only_one_member_and_retains_all_outbox_history() {
    let mut f = Fixture::new();
    f.store.save_plan(plan()).unwrap();
    f.store.save_member(member("ab")).unwrap();
    let first = f.store.prepare_member_push().unwrap().unwrap();
    let member_id = first["memberId"].as_str().unwrap();
    let mut edit = member("CD");
    edit.id = Some(member_id.into());
    edit.version = Some(1);
    edit.name = "Later local edit".into();
    f.store.save_member(edit).unwrap();
    assert_eq!(first, f.store.prepare_member_push().unwrap().unwrap());
    f.store.acknowledge_member(ack(&first, 1)).unwrap();
    f.store.acknowledge_member(ack(&first, 1)).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["pending"], 2);
    assert_eq!(
        f.store.snapshot().unwrap()["members"][0]["name"],
        "Later local edit"
    );
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM outbox", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    let second = f.store.prepare_member_push().unwrap().unwrap();
    assert_eq!(second["expectedRevision"], 1);
    assert_ne!(first["operationId"], second["operationId"]);
    f.store.acknowledge_member(ack(&second, 2)).unwrap();
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert!(f.store.prepare_member_push().unwrap().is_none());
    let reopened = Store::open(&f.path).unwrap();
    assert_eq!(
        reopened.snapshot().unwrap()["memberSync"]["acknowledged"],
        2
    );
}
#[test]
fn mismatched_receipt_does_not_acknowledge_or_replace_local_member() {
    let mut f = Fixture::new();
    f.store.save_member(member("")).unwrap();
    let request = f.store.prepare_member_push().unwrap().unwrap();
    let mut receipt = serde_json::to_value(ack(&request, 1)).unwrap();
    receipt["operationId"] = json!(id());
    assert!(f
        .store
        .acknowledge_member(serde_json::from_value(receipt).unwrap())
        .is_err());
    let mut receipt = serde_json::to_value(ack(&request, 1)).unwrap();
    receipt["member"]["name"] = json!("Unexpected name");
    assert!(f
        .store
        .acknowledge_member(serde_json::from_value(receipt).unwrap())
        .is_err());
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
}
#[test]
fn pulls_archive_with_inactive_actor_preserve_periods_and_cards() {
    let mut f = Fixture::new();
    let member_id = id();
    let first = remote(&member_id, 1, Some("ABC"));
    f.store
        .apply_member_page(page(0, vec![first.clone()]))
        .unwrap();
    f.store.save_plan(plan()).unwrap();
    f.store.add_period(period(&f.store)).unwrap();
    let mut archived = first;
    archived["revision"] = json!(2);
    archived["archivedAt"] = json!("2026-10-03T10:00:00.000Z");
    archived["archivedBy"] = json!({"id":id(),"name":"Verified server staff"});
    f.store.apply_member_page(page(1, vec![archived])).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["active"], false);
    assert_eq!(s["periods"].as_array().unwrap().len(), 1);
    assert_eq!(s["users"][0]["active"], 0);
    assert_eq!(s["users"][0]["roles"], json!([]));
    assert_eq!(s["removalAuthorization"]["allowed"], false);
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT count(*) FROM nfc_cards", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(s["memberSync"]["cursor"], 2);
}
#[test]
fn pull_conflicts_preserve_pending_edits_and_card_history() {
    let mut f = Fixture::new();
    f.store.save_member(member("ABC")).unwrap();
    let local_id = first_id(&f.store, "members");
    f.store
        .apply_member_page(page(
            0,
            vec![remote(&local_id, 1, None), remote(&id(), 1, Some("ABC"))],
        ))
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"].as_array().unwrap().len(), 1);
    assert_eq!(s["members"][0]["nfcId"], "ABC");
    assert_eq!(s["pending"], 1);
    assert_eq!(s["memberSync"]["conflicts"].as_array().unwrap().len(), 2);
    assert_eq!(s["memberSync"]["cursor"], 2);
    assert!(f.store.prepare_member_push().unwrap().is_none());
}
#[test]
fn malformed_page_rolls_back_all_members_history_and_cursor() {
    let mut f = Fixture::new();
    let good = remote(&id(), 1, Some("ABC"));
    let mut bad = remote(&id(), 1, None);
    bad["joinedOn"] = json!("2026-02-30");
    assert!(f.store.apply_member_page(page(0, vec![good, bad])).is_err());
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"], json!([]));
    assert_eq!(s["auditCount"], 0);
    assert_eq!(s["memberSync"]["cursor"], 0);
    let mut bad = serde_json::to_value(page(0, vec![remote(&id(), 1, None)])).unwrap();
    bad["nextCursor"] = json!(20);
    assert!(f
        .store
        .apply_member_page(serde_json::from_value(bad).unwrap())
        .is_err());
    assert_eq!(f.store.snapshot().unwrap()["members"], json!([]));
}
#[test]
fn remote_heads_do_not_replace_local_optimistic_versions() {
    let mut f = Fixture::new();
    let member_id = id();
    f.store
        .apply_member_page(page(0, vec![remote(&member_id, 17, None)]))
        .unwrap();
    assert_eq!(f.store.snapshot().unwrap()["members"][0]["version"], 1);
    let mut edit = member("");
    edit.id = Some(member_id);
    edit.version = Some(1);
    f.store.save_member(edit).unwrap();
    assert_eq!(
        f.store.prepare_member_push().unwrap().unwrap()["expectedRevision"],
        17
    );
}
#[test]
fn rejected_and_transient_deliveries_survive_restart_without_fake_ack() {
    let mut f = Fixture::new();
    f.store.save_member(member("")).unwrap();
    let request = f.store.prepare_member_push().unwrap().unwrap();
    let operation = request["operationId"].as_str().unwrap();
    f.store
        .reject_member_push(operation, "network timeout", &Value::Null, false)
        .unwrap();
    assert_eq!(f.store.prepare_member_push().unwrap().unwrap(), request);
    f.store
        .reject_member_push(
            operation,
            "revision conflict",
            &remote(request["memberId"].as_str().unwrap(), 2, None),
            true,
        )
        .unwrap();
    let mut reopened = Store::open(&f.path).unwrap();
    assert!(reopened.prepare_member_push().unwrap().is_none());
    assert_eq!(reopened.snapshot().unwrap()["pending"], 1);
    assert_eq!(
        reopened.snapshot().unwrap()["memberSync"]["conflicts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn backup_preserves_ack_cursor_conflicts_and_restore_blocks_all_sync() {
    let mut f = Fixture::new();
    f.store.save_member(member("ABC")).unwrap();
    let request = f.store.prepare_member_push().unwrap().unwrap();
    f.store.acknowledge_member(ack(&request, 1)).unwrap();
    f.store
        .apply_member_page(page(0, vec![remote(&id(), 1, Some("ABC"))]))
        .unwrap();
    let backup = f.store.backup_envelope().unwrap();
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["memberSync"]["acknowledged"], 1);
    assert_eq!(s["memberSync"]["cursor"], 1);
    assert_eq!(s["memberSync"]["conflicts"].as_array().unwrap().len(), 1);
    assert!(f.store.prepare_member_push().is_err());
    assert!(f.store.acknowledge_member(ack(&request, 1)).is_err());
    assert!(f.store.apply_member_page(page(1, vec![])).is_err());
}
#[test]
fn archive_ack_requires_original_enrolled_actor_and_preserves_local_history() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    let subject = id();
    f.store
        .conn
        .execute(
            "UPDATE users SET subject=?1 WHERE id='test-admin'",
            [&subject],
        )
        .unwrap();
    f.store.save_member(member("ABC")).unwrap();
    let create = f.store.prepare_member_push().unwrap().unwrap();
    f.store.acknowledge_member(ack(&create, 1)).unwrap();
    f.store
        .archive_member(removal_member_input(&f.store))
        .unwrap();
    let before = f.store.snapshot().unwrap()["members"][0].clone();
    let archive = f.store.prepare_member_push().unwrap().unwrap();
    assert_eq!(archive["action"], "archive");
    assert_eq!(archive["expectedRevision"], 1);
    let mut receipt = serde_json::to_value(ack(&create, 1)).unwrap();
    receipt["operationId"] = archive["operationId"].clone();
    receipt["revision"] = json!(2);
    receipt["sequence"] = json!(2);
    receipt["member"]["revision"] = json!(2);
    receipt["member"]["archivedAt"] = json!("2026-10-03T11:00:00.000Z");
    receipt["member"]["archivedBy"] = json!({"id":id(),"name":"Different uploader"});
    assert!(f
        .store
        .acknowledge_member(serde_json::from_value(receipt.clone()).unwrap())
        .is_err());
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    receipt["member"]["archivedBy"] = json!({"id":subject,"name":"Verified test administrator"});
    f.store
        .acknowledge_member(serde_json::from_value(receipt).unwrap())
        .unwrap();
    assert_eq!(f.store.snapshot().unwrap()["members"][0], before);
    assert_eq!(f.store.snapshot().unwrap()["pending"], 0);
}
#[test]
fn local_delete_is_retained_and_never_transmitted_as_a_member_create() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    f.store.save_member(member("")).unwrap();
    f.store
        .delete_member(removal_member_input(&f.store))
        .unwrap();
    assert!(f.store.prepare_member_push().unwrap().is_none());
    assert_eq!(f.store.snapshot().unwrap()["pending"], 2);
    assert_eq!(f.store.snapshot().unwrap()["members"], json!([]));
}
#[test]
fn remote_archive_maps_existing_subject_to_local_actor_fk_without_duplicate_user() {
    let mut f = Fixture::new();
    removal_admin(&mut f.store);
    let subject = id();
    f.store
        .conn
        .execute(
            "UPDATE users SET subject=?1 WHERE id='test-admin'",
            [&subject],
        )
        .unwrap();
    let mut archived = remote(&id(), 1, None);
    archived["archivedAt"] = json!("2026-10-03T11:00:00.000Z");
    archived["archivedBy"] = json!({"id":subject,"name":"Server staff name"});
    f.store.apply_member_page(page(0, vec![archived])).unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["archivedByUserId"], "test-admin");
    assert_eq!(s["users"].as_array().unwrap().len(), 1);
    assert_eq!(s["users"][0]["name"], "Verified test administrator");
}
#[test]
fn remote_card_reassignment_retains_original_attendance_card_fk() {
    let mut f = Fixture::new();
    let member_id = id();
    f.store
        .apply_member_page(page(0, vec![remote(&member_id, 1, Some("OLD"))]))
        .unwrap();
    f.store
        .record_attendance(AttendanceInput {
            request_id: id(),
            member_or_card: "OLD".into(),
            source: "NFC".into(),
        })
        .unwrap();
    let before = f.store.snapshot().unwrap()["attendance"].clone();
    f.store
        .apply_member_page(page(1, vec![remote(&member_id, 2, Some("NEW"))]))
        .unwrap();
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["attendance"], before);
    assert_eq!(s["members"][0]["nfcId"], "NEW");
    assert_eq!(s["pending"], 1);
    assert_eq!(
        f.store
            .conn
            .query_row(
                "SELECT count(*) FROM nfc_cards WHERE revoked_at IS NOT NULL",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
}
