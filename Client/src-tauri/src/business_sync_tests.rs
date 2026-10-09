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

#[test]
fn administrator_review_retries_identical_blocked_bytes_and_receipts_survive_restart() {
    let subject = id();
    let gym = id();
    let mut f = Fixture::new(&subject, &gym, true);
    let tx = f.store.conn.transaction().unwrap();
    capture(&tx).unwrap();
    tx.commit().unwrap();
    let (batch, encoded): (String, String) = f
        .store
        .conn
        .query_row(
            "SELECT id,request_json FROM business_batches ORDER BY ordinal LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    record_failure(
        &mut f.store,
        RemoteFailure::BusinessConflict {
            code: "business_constraint_conflict".into(),
        },
        Some(&batch),
        Utc::now(),
    )
    .unwrap();
    let review = f.store.preview_business_retry(&batch).unwrap();
    let input = BusinessRetryInput {
        request_id: id(),
        batch_id: batch.clone(),
        fingerprint: review["fingerprint"].as_str().unwrap().into(),
    };
    let mut stale = input.clone();
    stale.fingerprint = "different".into();
    assert!(f.store.retry_business_transaction(stale).is_err());
    let result = f.store.retry_business_transaction(input.clone()).unwrap();
    assert_eq!(
        f.store.retry_business_transaction(input.clone()).unwrap(),
        result
    );
    assert_eq!(
        f.store
            .conn
            .query_row(
                "SELECT request_json FROM business_batches WHERE id=?1",
                [&batch],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        encoded
    );
    let mut reopened = Store::open(&f.path).unwrap();
    reopened.removal_session = f.store.removal_session.clone();
    assert_eq!(reopened.retry_business_transaction(input).unwrap(), result);
    let mut remote = Mock::for_store(&reopened, &subject, &gym);
    let run = reopened
        .run_business_sync(
            &mut remote,
            Utc::now(),
            Limits {
                pushes: 10,
                pages: 10,
            },
        )
        .unwrap();
    assert!(matches!(run, Run::Complete { .. }));
    assert_eq!(
        remote.changes[0]["request"],
        serde_json::from_str::<Value>(&encoded).unwrap()
    );
    assert_eq!(status(&reopened.conn).unwrap()["pending"], 0);
    reopened.removal_session = None;
    assert!(reopened.preview_business_retry(&batch).is_err());
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
            let mut current = BTreeMap::new();
            for entry in &self.changes {
                for change in entry["request"]["changes"].as_array().unwrap() {
                    current.insert(
                        (
                            change["table"].clone().to_string(),
                            change["id"].clone().to_string(),
                        ),
                        change["after"].clone(),
                    );
                }
            }
            for change in request["changes"].as_array().unwrap() {
                let row = current
                    .get(&(change["table"].to_string(), change["id"].to_string()))
                    .unwrap_or(&Value::Null);
                if *row != change["before"]
                    && !(change["before"].is_null() && *row == change["after"])
                {
                    return Err(RemoteFailure::BusinessConflict {
                        code: "business_revision_conflict".into(),
                    });
                }
            }
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

// Preserve coverage for older audit-only seeds and the first-sign-in seed
// reported by Windows: one audit, the default profile and a user reference.
fn frozen_profile_fixture(
    subject: &str,
    gym: &str,
) -> (Fixture, InitialProfileRecoveryInput, String) {
    frozen_installation_fixture(subject, gym, false)
}
fn frozen_installation_fixture(
    subject: &str,
    gym: &str,
    with_user: bool,
) -> (Fixture, InitialProfileRecoveryInput, String) {
    let mut f = Fixture::new(subject, gym, true);
    if !with_user {
        f.store
            .conn
            .execute("DELETE FROM business_dirty WHERE table_name='users'", [])
            .unwrap();
    }
    let device: String = f
        .store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    for _ in 0..if with_user { 1 } else { 2 } {
        f.store.conn.execute("INSERT INTO audit(id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity,actor_user_id) VALUES(?1,'Administrator',?2,'Verified staff sign-in',?3,NULL,'{}',?4,'staff sign-in',?3)", params![id(), device, subject, Utc::now().to_rfc3339()]).unwrap();
    }
    let tx = f.store.conn.transaction().unwrap();
    capture(&tx).unwrap();
    tx.commit().unwrap();
    let (batch, encoded): (String, String) = f
        .store
        .conn
        .query_row(
            "SELECT id,request_json FROM business_batches ORDER BY ordinal LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&encoded).unwrap()["changes"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    record_failure(
        &mut f.store,
        RemoteFailure::BusinessConflict {
            code: "business_revision_conflict".into(),
        },
        Some(&batch),
        Utc::now(),
    )
    .unwrap();
    let preview = f.store.preview_business_retry(&batch).unwrap();
    assert_eq!(
        preview["initialProfileRecovery"]["allowed"], true,
        "{preview}"
    );
    let input = InitialProfileRecoveryInput {
        request_id: id(),
        batch_id: batch,
        fingerprint: preview["fingerprint"].as_str().unwrap().into(),
        confirmation: true,
    };
    (f, input, encoded)
}
fn newer_profile_cloud(store: &Store, subject: &str, gym: &str) -> Mock {
    let mut source = Fixture::new(subject, gym, true);
    let mut remote = Mock::for_store(&source.store, subject, gym);
    for version in [1, 2] {
        source
            .store
            .save_profile(ProfileInput {
                version,
                name: "Shared gym".into(),
                location: format!("Server location {version}"),
                phone: "0771234567".into(),
                email: String::new(),
            })
            .unwrap();
        assert!(matches!(
            source
                .store
                .run_business_sync(&mut remote, Utc::now(), Limits::default())
                .unwrap(),
            Run::Complete { .. }
        ));
    }
    remote.scope = Mock::for_store(store, subject, gym).scope;
    remote
}
fn retained_batch(store: &Store, batch: &str) -> (String, String, Option<String>) {
    store
        .conn
        .query_row(
            "SELECT request_json,state,response_json FROM business_batches WHERE id=?1",
            [batch],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
}

#[test]
fn initial_profile_recovery_keeps_original_bytes_audits_backup_and_real_receipts() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_profile_fixture(&subject, &gym);
    // Repeated old Retry attempts add benign, immutable audit-only transactions.
    for _ in 0..2 {
        let preview = f.store.preview_business_retry(&input.batch_id).unwrap();
        f.store
            .retry_business_transaction(BusinessRetryInput {
                request_id: id(),
                batch_id: input.batch_id.clone(),
                fingerprint: preview["fingerprint"].as_str().unwrap().into(),
            })
            .unwrap();
        record_failure(
            &mut f.store,
            RemoteFailure::BusinessConflict {
                code: "business_revision_conflict".into(),
            },
            Some(&input.batch_id),
            Utc::now(),
        )
        .unwrap();
    }
    let mut remote = newer_profile_cloud(&f.store, &subject, &gym);
    let original_audits: Vec<Value> = serde_json::from_str::<Value>(&encoded).unwrap()["changes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["table"] == "audit")
        .cloned()
        .collect();
    let before = f.store.backup_envelope().unwrap();
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    job.run(&mut remote, || Ok(())).unwrap();
    assert_eq!(
        f.store.backup_envelope().unwrap().sha256,
        before.sha256,
        "network work must not modify the original"
    );
    let result = job.commit(&mut f.store).unwrap();
    assert_eq!(result["superseded"], true);
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded.clone(), "conflict".into(), None)
    );
    assert_eq!(f.store.snapshot().unwrap()["profile"]["version"], 3);
    assert_eq!(
        f.store.snapshot().unwrap()["profile"]["location"],
        "Server location 2"
    );
    for change in original_audits {
        let table = tables()
            .unwrap()
            .into_iter()
            .find(|t| t.name == "audit")
            .unwrap();
        assert_eq!(
            row_on(&f.store.conn, &table, change["id"].as_str().unwrap()).unwrap(),
            Some(change["after"].clone())
        );
    }
    let original_backup: recovery::BackupEnvelope =
        serde_json::from_slice(&std::fs::read(result["recoveryPath"].as_str().unwrap()).unwrap())
            .unwrap();
    assert_eq!(original_backup.sha256, before.sha256);
    assert!(f
        .store
        .conn
        .execute(
            "DELETE FROM local_operations WHERE command='business_initial_profile_recovery'",
            []
        )
        .is_err());
    assert_eq!(
        status(&f.store.conn).unwrap()["recoveredInitialProfiles"],
        1
    );
    assert_eq!(status(&f.store.conn).unwrap()["conflicts"], json!([]));
    assert!(f.store.preview_business_retry(&input.batch_id).is_err());
    // Replacement and recovery audits each await their actual server receipt.
    assert_eq!(status(&f.store.conn).unwrap()["pending"], 2);
    let mut restarted = Store::open(&f.path).unwrap();
    restarted.removal_session = Some(removal::Session {
        user_id: subject.clone(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
        can_write: true,
        native_nonce: Some("nonce".into()),
    });
    let replay = restarted
        .prepare_initial_profile_recovery(input)
        .unwrap()
        .commit(&mut restarted)
        .unwrap();
    assert_eq!(replay["duplicate"], true);
    let count = remote.changes.len();
    assert!(matches!(
        restarted
            .run_business_sync(&mut remote, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete { .. }
    ));
    assert_eq!(remote.changes.len(), count + 2);
    assert_eq!(status(&restarted.conn).unwrap()["pending"], 0);
    assert_eq!(
        retained_batch(&restarted, replay["batchId"].as_str().unwrap()).1,
        "conflict"
    );
    if let Ok(path) = std::env::var("ARMSTRONG_BUSINESS_FIXTURE_PATH") {
        std::fs::write(format!("{path}.profile-recovery.json"), json!({"entries":remote.changes,"originalRequest":serde_json::from_str::<Value>(&encoded).unwrap(),"replacementBatchId":result["replacementBatchId"]}).to_string()).unwrap();
    }
}

#[test]
fn initial_profile_recovery_keeps_first_sign_in_identity_and_audit_without_authority_changes() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_installation_fixture(&subject, &gym, true);
    let original: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(original["changes"][2]["table"], "users");
    assert_eq!(original["changes"][2]["after"]["active"], 0);
    // Subsequent enrollment's local version increment is not an identity edit.
    f.store
        .conn
        .execute("UPDATE users SET version=2 WHERE id=?1", [&subject])
        .unwrap();
    assert_eq!(
        f.store.preview_business_retry(&input.batch_id).unwrap()["initialProfileRecovery"]
            ["allowed"],
        true
    );
    let before = f.store.backup_envelope().unwrap();
    let mut remote = newer_profile_cloud(&f.store, &subject, &gym);
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    job.run(&mut remote, || Ok(())).unwrap();
    assert_eq!(f.store.backup_envelope().unwrap().sha256, before.sha256);
    let result = job.commit(&mut f.store).unwrap();
    assert_eq!(result["superseded"], true);
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded.clone(), "conflict".into(), None)
    );
    let backup: recovery::BackupEnvelope =
        serde_json::from_slice(&std::fs::read(result["recoveryPath"].as_str().unwrap()).unwrap())
            .unwrap();
    assert_eq!(backup.sha256, before.sha256);
    let replacement = remote
        .changes
        .iter()
        .find(|entry| entry["request"]["operationId"] == result["replacementBatchId"])
        .unwrap();
    let preserved: Vec<_> = original["changes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["table"] != "gym_settings")
        .cloned()
        .collect();
    assert_eq!(replacement["request"]["changes"], json!(preserved));
    let user_table = tables()
        .unwrap()
        .into_iter()
        .find(|t| t.name == "users")
        .unwrap();
    let local_user = row_on(&f.store.conn, &user_table, &subject)
        .unwrap()
        .unwrap();
    assert_eq!(local_user["active"], 1);
    assert_eq!(local_user["version"], 2);
    assert_eq!(f.store.conn.query_row("SELECT r.name FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=?1", [&subject], |r| r.get::<_, String>(0)).unwrap(), "Administrator");
    assert_eq!(f.store.snapshot().unwrap()["profile"]["version"], 3);
    let mut restarted = Store::open(&f.path).unwrap();
    restarted.removal_session = Some(removal::Session {
        user_id: subject.clone(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
        can_write: true,
        native_nonce: Some("nonce".into()),
    });
    assert!(matches!(
        restarted
            .run_business_sync(&mut remote, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete { .. }
    ));
    assert_eq!(status(&restarted.conn).unwrap()["pending"], 0);
    assert_eq!(
        retained_batch(&restarted, &input.batch_id),
        (encoded, "conflict".into(), None)
    );
    if let Ok(path) = std::env::var("ARMSTRONG_BUSINESS_FIXTURE_PATH") {
        std::fs::write(format!("{path}.profile-recovery-user.json"), json!({"entries":remote.changes,"originalRequest":original,"replacementBatchId":result["replacementBatchId"]}).to_string()).unwrap();
    }
}

#[test]
fn initial_profile_recovery_does_not_overwrite_a_different_server_identity() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_installation_fixture(&subject, &gym, true);
    let mut remote = newer_profile_cloud(&f.store, &subject, &gym);
    for entry in &mut remote.changes {
        for change in entry["request"]["changes"].as_array_mut().unwrap() {
            if change["table"] == "users" {
                change["after"]["email"] = json!("different@example.invalid");
            }
        }
    }
    let server_history = remote.changes.clone();
    let before = f.store.backup_envelope().unwrap();
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    assert!(job.run(&mut remote, || Ok(())).is_err());
    assert!(job.commit(&mut f.store).is_err());
    assert_eq!(remote.changes, server_history);
    assert_eq!(f.store.backup_envelope().unwrap().sha256, before.sha256);
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded, "conflict".into(), None)
    );
}

#[test]
fn initial_profile_recovery_refuses_other_or_edited_identity_references() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_installation_fixture(&subject, &gym, true);
    let request: Value = serde_json::from_str(&encoded).unwrap();
    for variant in 0..8 {
        let mut edited = request.clone();
        match variant {
            0 => edited["changes"][2]["after"]["subject"] = json!(id()),
            1 => edited["changes"][2]["id"] = json!(id()),
            2 => edited["changes"][2]["after"]["display_name"] = json!("Changed identity"),
            3 => edited["changes"][2]["after"]["email"] = json!("changed@example.invalid"),
            4 => edited["changes"][2]["after"]["active"] = json!(1),
            5 => edited["changes"][2]["after"]["version"] = json!(2),
            6 => edited["changes"][2]["before"] = edited["changes"][2]["after"].clone(),
            _ => {
                let extra = edited["changes"][2].clone();
                edited["changes"].as_array_mut().unwrap().push(extra);
            }
        }
        assert!(
            crate::initial_profile_recovery::eligible(
                &f.store.conn,
                f.store.removal_session.as_ref(),
                &input.batch_id,
                &edited,
                "business_revision_conflict"
            )
            .is_err(),
            "variant {variant}"
        );
    }
    f.store
        .conn
        .execute(
            "UPDATE users SET display_name='Edited locally',version=version+1 WHERE id=?1",
            [&subject],
        )
        .unwrap();
    assert_eq!(
        f.store.preview_business_retry(&input.batch_id).unwrap()["initialProfileRecovery"]
            ["allowed"],
        false
    );
    assert!(f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .is_err());
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded, "conflict".into(), None)
    );
}

#[test]
fn initial_profile_recovery_lost_reply_uses_the_same_replacement_on_retry() {
    let subject = id();
    let gym = id();
    let (mut f, mut input, encoded) = frozen_installation_fixture(&subject, &gym, true);
    let mut remote = newer_profile_cloud(&f.store, &subject, &gym);
    let cloud_count = remote.changes.len();
    remote.drop_reply = true;
    let before = f.store.backup_envelope().unwrap().sha256;
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    assert!(job.run(&mut remote, || Ok(())).is_err());
    assert!(job.commit(&mut f.store).is_err());
    assert_eq!(f.store.backup_envelope().unwrap().sha256, before);
    assert_eq!(remote.changes.len(), cloud_count + 1);
    let accepted = remote.changes.last().unwrap()["request"].clone();
    input.request_id = id();
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    job.run(&mut remote, || Ok(())).unwrap();
    job.commit(&mut f.store).unwrap();
    assert_eq!(remote.changes.len(), cloud_count + 1);
    assert_eq!(remote.changes.last().unwrap()["request"], accepted);
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded, "conflict".into(), None)
    );
}

#[test]
fn initial_profile_recovery_obeys_original_receipt_before_a_stale_refusal() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_profile_fixture(&subject, &gym);
    let request: Value = serde_json::from_str(&encoded).unwrap();
    let mut remote = Mock::for_store(&f.store, &subject, &gym);
    remote.push_business(&request).unwrap();
    let default = default_profile();
    let mut updated = default.clone();
    updated["version"] = json!(2);
    updated["name"] = json!("Server changed after original acceptance");
    remote.push_business(&json!({"protocolVersion":2,"operationId":id(),"deviceId":id(),"actorSubject":subject,"operationIds":[],"changes":[{"table":"gym_settings","id":"1","before":default,"after":updated}]})).unwrap();
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    job.run(&mut remote, || Ok(())).unwrap();
    let result = job.commit(&mut f.store).unwrap();
    assert_eq!(result["superseded"], false);
    assert_eq!(
        status(&f.store.conn).unwrap()["recoveredInitialProfiles"],
        0
    );
    assert_eq!(retained_batch(&f.store, &input.batch_id).0, encoded);
    assert_eq!(retained_batch(&f.store, &input.batch_id).1, "confirmed");
    assert!(retained_batch(&f.store, &input.batch_id).2.is_some());
    assert_eq!(f.store.snapshot().unwrap()["profile"]["version"], 2);
    assert_eq!(remote.changes.len(), 2);
}

#[test]
fn initial_profile_recovery_cancellation_bad_receipt_and_changed_database_keep_original() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_profile_fixture(&subject, &gym);
    let mut remote = newer_profile_cloud(&f.store, &subject, &gym);
    let before = f.store.backup_envelope().unwrap().sha256;
    let cancel = std::rc::Rc::new(std::cell::Cell::new(false));
    let signal = cancel.clone();
    remote.after_pull = Some(Box::new(move || signal.set(true)));
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    assert!(job
        .run(&mut remote, || if cancel.get() {
            Err("Session changed".into())
        } else {
            Ok(())
        })
        .is_err());
    assert!(job.commit(&mut f.store).is_err());
    assert_eq!(f.store.backup_envelope().unwrap().sha256, before);
    remote.bad_receipt = true;
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    assert!(job.run(&mut remote, || Ok(())).is_err());
    assert!(job.commit(&mut f.store).is_err());
    assert_eq!(f.store.backup_envelope().unwrap().sha256, before);
    remote.bad_receipt = false;
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    job.run(&mut remote, || Ok(())).unwrap();
    f.store
        .conn
        .execute(
            "INSERT INTO metadata VALUES('concurrent-change','retained')",
            [],
        )
        .unwrap();
    assert!(job.commit(&mut f.store).unwrap_err().contains("changed"));
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded, "conflict".into(), None)
    );
    assert_eq!(f.store.snapshot().unwrap()["profile"]["version"], 1);
    assert_eq!(
        status(&f.store.conn).unwrap()["recoveredInitialProfiles"],
        0
    );
}

#[test]
fn initial_profile_recovery_refuses_edits_money_unchecked_confirmation_and_wrong_identity() {
    let subject = id();
    let gym = id();
    let (mut f, input, _) = frozen_profile_fixture(&subject, &gym);
    let mut unchecked = input.clone();
    unchecked.confirmation = false;
    assert!(f.store.prepare_initial_profile_recovery(unchecked).is_err());
    let mut stale = input.clone();
    stale.fingerprint = "old".into();
    assert!(f.store.prepare_initial_profile_recovery(stale).is_err());
    f.store.removal_session.as_mut().unwrap().can_write = false;
    assert!(f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .is_err());
    f.store.removal_session.as_mut().unwrap().can_write = true;
    f.store
        .conn
        .execute(
            "UPDATE metadata SET value='revoked' WHERE key='native_session_nonce'",
            [],
        )
        .unwrap();
    assert!(f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .is_err());
    f.store
        .conn
        .execute(
            "UPDATE metadata SET value='nonce' WHERE key='native_session_nonce'",
            [],
        )
        .unwrap();
    f.store
        .save_profile(ProfileInput {
            version: 1,
            name: "Locally edited gym".into(),
            location: "Matale".into(),
            phone: String::new(),
            email: String::new(),
        })
        .unwrap();
    let review = f.store.preview_business_retry(&input.batch_id).unwrap();
    assert_eq!(review["initialProfileRecovery"]["allowed"], false);
    assert!(f.store.prepare_initial_profile_recovery(input).is_err());
    let (mut f, input, encoded) = frozen_profile_fixture(&subject, &gym);
    f.store
        .record_expense(ExpenseInput {
            request_id: id(),
            title: "Retained rent".into(),
            category: "Operations".into(),
            amount_minor: 100_000,
            method: "Cash".into(),
        })
        .unwrap();
    assert_eq!(
        f.store.preview_business_retry(&input.batch_id).unwrap()["initialProfileRecovery"]
            ["allowed"],
        false
    );
    assert!(f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .is_err());
    assert_eq!(
        f.store.snapshot().unwrap()["expenses"][0]["amountMinor"],
        100_000
    );
    assert_eq!(retained_batch(&f.store, &input.batch_id).0, encoded);
}

#[test]
fn initial_profile_recovery_cannot_commit_after_nonce_revocation_or_before_network_completion() {
    let subject = id();
    let gym = id();
    let (mut f, input, encoded) = frozen_profile_fixture(&subject, &gym);
    let job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    assert!(job.commit(&mut f.store).unwrap_err().contains("complete"));
    let mut remote = newer_profile_cloud(&f.store, &subject, &gym);
    let mut job = f
        .store
        .prepare_initial_profile_recovery(input.clone())
        .unwrap();
    job.run(&mut remote, || Ok(())).unwrap();
    f.store
        .conn
        .execute(
            "UPDATE metadata SET value='new-session' WHERE key='native_session_nonce'",
            [],
        )
        .unwrap();
    assert!(job.commit(&mut f.store).is_err());
    assert_eq!(
        retained_batch(&f.store, &input.batch_id),
        (encoded, "conflict".into(), None)
    );
    assert_eq!(f.store.snapshot().unwrap()["profile"]["version"], 1);
}

#[test]
fn initial_profile_recovery_wrong_server_or_account_never_sends_a_request() {
    let subject = id();
    let gym = id();
    let (mut f, input, _) = frozen_profile_fixture(&subject, &gym);
    for wrong_account in [false, true] {
        let mut remote = Mock::for_store(&f.store, &subject, &gym);
        if wrong_account {
            remote.subject = id();
        } else {
            remote.scope.gym_id = id();
        }
        let mut job = f
            .store
            .prepare_initial_profile_recovery(input.clone())
            .unwrap();
        assert!(job.run(&mut remote, || Ok(())).is_err());
        assert!(remote.changes.is_empty());
        assert!(job.commit(&mut f.store).is_err());
    }
}

#[test]
fn initial_profile_recovery_requires_an_unedited_insert_not_an_update_or_mixed_transaction() {
    let subject = id();
    let gym = id();
    let f = Fixture::new(&subject, &gym, true);
    let request = json!({"protocolVersion":2,"operationId":id(),"deviceId":Mock::for_store(&f.store,&subject,&gym).scope.device_id,"actorSubject":subject,"operationIds":[],"changes":[{"table":"gym_settings","id":"1","before":null,"after":default_profile()},{"table":"audit","id":id(),"before":null,"after":{}}]});
    let batch = request["operationId"].as_str().unwrap();
    for variant in 0..5 {
        let mut edited = request.clone();
        match variant {
            0 => edited["changes"][0]["before"] = default_profile(),
            1 => edited["changes"][0]["after"]["phone"] = json!("0771234567"),
            2 => edited["operationIds"] = json!([id()]),
            3 => edited["changes"][1]["table"] = json!("payments"),
            _ => edited["actorSubject"] = json!(id()),
        }
        assert!(crate::initial_profile_recovery::eligible(
            &f.store.conn,
            f.store.removal_session.as_ref(),
            batch,
            &edited,
            "business_revision_conflict"
        )
        .is_err());
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
    let plan = store.snapshot().unwrap()["plans"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let registration = store
        .register_member(RegisterMemberInput {
            request_id: id(),
            name: "Member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "CARD1".into(),
            plan_id: Some(plan),
            plan_version: Some(1),
            starts_on: Some("2026-09-05".into()),
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
            expected_last_period_id: registration["membershipPeriodId"]
                .as_str()
                .map(str::to_owned),
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
    assert_eq!(b["periods"].as_array().unwrap().len(), 2);
    assert!(b["periods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|period| period["startsOn"] == "2026-09-05" && period["endsOn"] == "2026-10-04"));
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
fn shared_permanent_removal_preserves_billing_and_attendance_on_another_device() {
    let subject = id();
    let gym = id();
    let mut writer = Fixture::new(&subject, &gym, true);
    let (member, payment) = setup_operations(&mut writer.store);
    let before = writer.store.snapshot().unwrap();
    writer
        .store
        .delete_member(MemberRemovalInput {
            request_id: id(),
            member_id: member.clone(),
            version: 1,
        })
        .unwrap();
    let mut cloud = Mock::for_store(&writer.store, &subject, &gym);
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
    let after = reader.store.snapshot().unwrap();
    assert_eq!(after["members"], json!([]));
    for key in [
        "attendance",
        "periods",
        "payments",
        "invoices",
        "allocations",
    ] {
        assert_eq!(
            before[key], after[key],
            "{key} preserved on sharing computer"
        );
    }
    assert_eq!(
        writer.store.payment_receipt(payment.clone()).unwrap(),
        reader.store.payment_receipt(payment).unwrap()
    );
    assert_eq!(
        reader
            .store
            .conn
            .query_row("SELECT count(*) FROM member_deletions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn business_staff_download_preserves_rates_receipts_payouts_and_refund_guards() {
    let subject = id();
    let gym = id();
    let mut writer = Fixture::new(&subject, &gym, true);
    let mut staff_input = TrainerInput {
        request_id: id(),
        id: None,
        version: None,
        name: "Synthetic trainer".into(),
        phone: "0771234567".into(),
        nic: "900000001V".into(),
        salary_minor: 3_000_000,
        training_fee_minor: 500_025,
        active: true,
        nfc_id: None,
    };
    let staff = writer.store.save_trainer(staff_input.clone()).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let member = writer
        .store
        .register_member_with_trainer(StaffRegisterInput {
            member: RegisterMemberInput {
                request_id: id(),
                name: "Synthetic training member".into(),
                phone: "0771234567".into(),
                email: "".into(),
                nfc_id: "".into(),
                plan_id: None,
                plan_version: None,
                starts_on: None,
            },
            trainer_id: Some(staff.clone()),
            trainer_version: Some(1),
            gender: None,
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let snapshot = writer.store.snapshot().unwrap();
    let invoice = snapshot["invoices"][0]["id"].as_str().unwrap().to_owned();
    let payment = writer
        .store
        .receive_combined_payment(CombinedPaymentInput {
            payment: ReceivePaymentInput {
                request_id: id(),
                member_id: member,
                amount_minor: 500_025,
                method: "Cash".into(),
                invoice_id: None,
            },
            invoice_ids: vec![invoice],
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let snapshot = writer.store.snapshot().unwrap();
    let payout = writer
        .store
        .pay_staff(StaffPayoutInput {
            request_id: id(),
            trainer_id: staff.clone(),
            trainer_version: 1,
            salary_month: snapshot["today"].as_str().unwrap()[..7].into(),
            include_salary: true,
            expected_salary_minor: 3_000_000,
            expected_training_minor: 500_025,
            allocation_ids: snapshot["staffTrainingAllocations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a["id"].as_str().unwrap().into())
                .collect(),
            method: "Cash".into(),
        })
        .unwrap();
    staff_input.request_id = id();
    staff_input.id = Some(staff);
    staff_input.version = Some(1);
    staff_input.training_fee_minor = 600_025;
    writer.store.save_trainer(staff_input).unwrap();
    let mut cloud = Mock::for_store(&writer.store, &subject, &gym);
    let limits = || Limits {
        pushes: 100,
        pages: 100,
    };
    assert!(matches!(
        writer
            .store
            .run_business_sync(&mut cloud, Utc::now(), limits())
            .unwrap(),
        Run::Complete { .. }
    ));
    let mut reader = Fixture::new(&subject, &gym, false);
    let mut download = Mock::for_store(&reader.store, &subject, &gym);
    download.changes = cloud.changes.clone();
    assert!(matches!(
        reader
            .store
            .run_business_sync(&mut download, Utc::now(), limits())
            .unwrap(),
        Run::Complete { .. }
    ));
    let a = writer.store.snapshot().unwrap();
    let b = reader.store.snapshot().unwrap();
    for key in [
        "trainers",
        "memberTrainers",
        "trainingCharges",
        "staffTrainingAllocations",
        "staffPayouts",
        "invoices",
        "payments",
        "allocations",
        "expenses",
        "financialAccounts",
        "audit",
    ] {
        assert_eq!(a[key], b[key], "{key}");
    }
    assert_eq!(b["trainers"][0]["version"], 2);
    assert_eq!(b["trainers"][0]["trainingFeeMinor"], 600_025);
    assert_eq!(b["trainingCharges"][0]["feeMinor"], 500_025);
    assert_eq!(b["trainers"][0]["unpaidTrainingMinor"], 0);
    assert_eq!(
        writer.store.payment_receipt(payment.clone()).unwrap(),
        reader.store.payment_receipt(payment.clone()).unwrap()
    );
    let reversal = ReversalInput {
        request_id: id(),
        payment_id: payment,
        reason: "Synthetic refund".into(),
    };
    assert!(
        reader.store.reverse_payment(reversal.clone()).is_err(),
        "downloaded payout must still prevent refunding paid earnings"
    );
    writer
        .store
        .void_expense(ExpenseVoidInput {
            request_id: id(),
            expense_id: payout["expenseId"].as_str().unwrap().into(),
            reason: "Synthetic payout correction".into(),
        })
        .unwrap();
    writer.store.reverse_payment(reversal).unwrap();
    writer
        .store
        .delete_staff(StaffRemovalInput {
            request_id: id(),
            staff_id: writer.store.snapshot().unwrap()["trainers"][0]["id"]
                .as_str()
                .unwrap()
                .into(),
            version: 2,
        })
        .unwrap();
    assert!(matches!(
        writer
            .store
            .run_business_sync(&mut cloud, Utc::now(), limits())
            .unwrap(),
        Run::Complete { .. }
    ));
    download.changes = cloud.changes;
    assert!(matches!(
        reader
            .store
            .run_business_sync(&mut download, Utc::now(), limits())
            .unwrap(),
        Run::Complete { .. }
    ));
    let a = writer.store.snapshot().unwrap();
    let b = reader.store.snapshot().unwrap();
    for key in [
        "trainers",
        "staffPayouts",
        "staffTrainingAllocations",
        "payments",
        "allocations",
        "expenses",
        "financialAccounts",
        "audit",
    ] {
        assert_eq!(a[key], b[key], "after refund: {key}");
    }
    assert_eq!(b["staffPayouts"][0]["active"], false);
    assert_eq!(b["trainers"][0]["unpaidTrainingMinor"], 0);
    assert_eq!(b["trainers"][0]["active"], false);
    assert!(b["trainers"][0]["deletedAt"].as_str().is_some());
    assert!(b["memberTrainers"][0]["trainerId"].is_null());
    assert!(reader
        .store
        .conn
        .execute("UPDATE trainers SET active=1,version=version+1", [])
        .is_err());
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
fn fresh_writer_downloads_newer_gym_profile_before_uploading_its_default() {
    let subject = id();
    let gym = id();
    let mut original = Fixture::new(&subject, &gym, true);
    original
        .store
        .save_profile(ProfileInput {
            version: 1,
            name: "Shared gym".into(),
            location: "Matale".into(),
            phone: "0771234567".into(),
            email: String::new(),
        })
        .unwrap();
    let mut cloud = Mock::for_store(&original.store, &subject, &gym);
    assert!(matches!(
        original
            .store
            .run_business_sync(&mut cloud, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete { .. }
    ));
    original
        .store
        .save_profile(ProfileInput {
            version: 2,
            name: "Shared gym".into(),
            location: "Updated location".into(),
            phone: "0771234567".into(),
            email: String::new(),
        })
        .unwrap();
    original
        .store
        .run_business_sync(&mut cloud, Utc::now(), Limits::default())
        .unwrap();

    let mut fresh = Fixture::new(&subject, &gym, true);
    let mut remote = Mock::for_store(&fresh.store, &subject, &gym);
    remote.changes = cloud.changes;
    let before = remote.changes.len();
    assert!(matches!(
        fresh
            .store
            .run_business_sync(
                &mut remote,
                Utc::now(),
                Limits {
                    pushes: 10,
                    pages: 1
                }
            )
            .unwrap(),
        Run::Yielded {
            pushed: 0,
            pages: 1
        }
    ));
    assert_eq!(remote.changes.len(), before);
    assert!(fresh
        .store
        .save_plan(PlanInput {
            id: None,
            version: None,
            name: "While downloading".into(),
            duration_months: 1,
            price_minor: 100,
            active: true
        })
        .unwrap_err()
        .contains("downloading"));
    assert_eq!(fresh.store.snapshot().unwrap()["plans"], json!([]));
    let mut restarted = Store::open(&fresh.path).unwrap();
    restarted.removal_session = fresh.store.removal_session.clone();
    assert!(matches!(
        restarted
            .run_business_sync(
                &mut remote,
                Utc::now(),
                Limits {
                    pushes: 10,
                    pages: 1
                }
            )
            .unwrap(),
        Run::Yielded {
            pushed: 0,
            pages: 1
        }
    ));
    assert_eq!(restarted.snapshot().unwrap()["profile"]["version"], 3);
    assert_eq!(
        restarted.snapshot().unwrap()["profile"]["location"],
        "Updated location"
    );
    assert!(matches!(
        restarted
            .run_business_sync(&mut remote, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete { .. }
    ));
    assert!(remote.changes[before]["request"]["changes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["table"] != "gym_settings"));
    assert_eq!(status(&restarted.conn).unwrap()["pending"], 0);
    assert_eq!(status(&restarted.conn).unwrap()["conflicts"], json!([]));
}

#[test]
fn first_computer_keeps_its_default_when_cloud_is_empty() {
    let subject = id();
    let gym = id();
    let mut f = Fixture::new(&subject, &gym, true);
    let mut cloud = Mock::for_store(&f.store, &subject, &gym);
    assert!(matches!(
        f.store
            .run_business_sync(&mut cloud, Utc::now(), Limits::default())
            .unwrap(),
        Run::Yielded {
            pushed: 0,
            pages: 1
        }
    ));
    assert!(matches!(
        f.store
            .run_business_sync(&mut cloud, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete { .. }
    ));
    assert!(cloud.changes[0]["request"]["changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["table"] == "gym_settings" && c["after"] == default_profile()));
    assert_eq!(status(&f.store.conn).unwrap()["pending"], 0);
}

#[test]
fn frozen_default_conflict_preserves_request_and_does_not_start_initial_download() {
    let subject = id();
    let gym = id();
    let mut original = Fixture::new(&subject, &gym, true);
    original
        .store
        .save_profile(ProfileInput {
            version: 1,
            name: "Existing gym".into(),
            location: "Matale".into(),
            phone: String::new(),
            email: String::new(),
        })
        .unwrap();
    let mut cloud = Mock::for_store(&original.store, &subject, &gym);
    original
        .store
        .run_business_sync(&mut cloud, Utc::now(), Limits::default())
        .unwrap();
    let mut f = Fixture::new(&subject, &gym, true);
    capture(&f.store.conn).unwrap();
    let encoded: String = f
        .store
        .conn
        .query_row("SELECT request_json FROM business_batches", [], |r| {
            r.get(0)
        })
        .unwrap();
    let mut remote = Mock::for_store(&f.store, &subject, &gym);
    remote.changes = cloud.changes;
    assert_eq!(
        f.store
            .run_business_sync(&mut remote, Utc::now(), Limits::default())
            .unwrap(),
        Run::Blocked
    );
    assert_eq!(
        f.store
            .conn
            .query_row("SELECT request_json FROM business_batches", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        encoded
    );
    assert_eq!(f.store.snapshot().unwrap()["profile"]["version"], 1);
    assert!(status(&f.store.conn).unwrap()["conflicts"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("retrying the same transaction cannot resolve"));
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
