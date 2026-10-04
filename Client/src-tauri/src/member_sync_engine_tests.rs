// SYNC-ENGINE TESTS: real temporary SQLite + a mock server transport.
// No HTTPS, JWT/Supabase, PostgreSQL, native GUI or live synchronization evidence.
use super::*;
use crate::member_sync::{Page, Receipt};
use crate::member_worker::{Limits, MemberTransport, Rejection, RemoteFailure, Run, SyncScope};
use std::collections::BTreeMap;

struct MockServer {
    scope: SyncScope,
    subject: String,
    members: BTreeMap<String, Value>,
    receipts: BTreeMap<String, (Value, Receipt)>,
    changes: Vec<Value>,
    events: Vec<String>,
    fail_push: Option<RemoteFailure>,
    drop_response: bool,
    fail_pull: Option<RemoteFailure>,
    after_push: Option<Box<dyn FnOnce()>>,
    after_pull: Option<Box<dyn FnOnce()>>,
    page_size: usize,
    malformed_pull: bool,
}
impl MockServer {
    fn fixture(store: &mut Store) -> Self {
        removal_admin(store);
        let subject = id();
        store
            .conn
            .execute(
                "UPDATE users SET subject=?1 WHERE id='test-admin'",
                [&subject],
            )
            .unwrap();
        let device: String = store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let scope = SyncScope::new("https://gym.example", &id(), &device).unwrap();
        store.bind_member_scope(scope.clone()).unwrap();
        Self {
            scope,
            subject,
            members: BTreeMap::new(),
            receipts: BTreeMap::new(),
            changes: vec![],
            events: vec![],
            fail_push: None,
            drop_response: false,
            fail_pull: None,
            after_push: None,
            after_pull: None,
            page_size: 100,
            malformed_pull: false,
        }
    }
    fn add_remote(&mut self) -> String {
        let member = id();
        let snapshot = json!({"id":member,"name":"Remote bootstrap member","phone":"0771234567","email":"","nfcId":null,"joinedOn":business_date(Utc::now()),"revision":1,"archivedAt":null,"archivedBy":null});
        self.members.insert(member.clone(), snapshot.clone());
        self.changes
            .push(json!({"sequence":self.changes.len()+1,"operationId":id(),"member":snapshot}));
        member
    }
}
impl MemberTransport for MockServer {
    fn scope(&self) -> &SyncScope {
        &self.scope
    }
    fn subject(&self) -> &str {
        &self.subject
    }
    fn push(&mut self, request: &Value) -> std::result::Result<Receipt, RemoteFailure> {
        self.events
            .push(format!("push:{}", request["operationId"].as_str().unwrap()));
        if let Some(failure) = self.fail_push.take() {
            return Err(failure);
        }
        let operation = request["operationId"].as_str().unwrap();
        if let Some((original, receipt)) = self.receipts.get(operation) {
            if original != request {
                return Err(RemoteFailure::Rejected {
                    kind: Rejection::OperationReuse,
                    member: Value::Null,
                });
            }
            return Ok(receipt.clone());
        }
        let member_id = request["memberId"].as_str().unwrap();
        let current = self.members.get(member_id).cloned();
        if current
            .as_ref()
            .map_or(0, |m| m["revision"].as_i64().unwrap())
            != request["expectedRevision"].as_i64().unwrap()
        {
            return Err(RemoteFailure::Rejected {
                kind: Rejection::Revision,
                member: current.unwrap_or(Value::Null),
            });
        }
        let mut member = if request["action"] == "archive" {
            current.unwrap()
        } else {
            request["member"].clone()
        };
        member["id"] = request["memberId"].clone();
        member["revision"] = json!(request["expectedRevision"].as_i64().unwrap() + 1);
        if request["action"] == "archive" {
            member["archivedAt"] = json!(Utc::now().to_rfc3339());
            member["archivedBy"] = json!({"id":self.subject,"name":"Mock authenticated staff"});
        } else {
            member["archivedAt"] = Value::Null;
            member["archivedBy"] = Value::Null;
        }
        let sequence = self.changes.len() + 1;
        let receipt:Receipt=serde_json::from_value(json!({"protocolVersion":1,"operationId":operation,"memberId":member_id,"revision":member["revision"],"sequence":sequence,"member":member})).unwrap();
        self.members.insert(member_id.into(), member.clone());
        self.changes
            .push(json!({"sequence":sequence,"operationId":operation,"member":member}));
        self.receipts
            .insert(operation.into(), (request.clone(), receipt.clone()));
        if let Some(callback) = self.after_push.take() {
            callback();
        }
        if self.drop_response {
            self.drop_response = false;
            return Err(RemoteFailure::Transient {
                retry_after_seconds: None,
            });
        }
        Ok(receipt)
    }
    fn pull(&mut self, after: i64) -> std::result::Result<Page, RemoteFailure> {
        self.events.push(format!("pull:{after}"));
        if let Some(failure) = self.fail_pull.take() {
            return Err(failure);
        }
        let changes: Vec<_> = self
            .changes
            .iter()
            .filter(|change| change["sequence"].as_i64().unwrap() > after)
            .take(self.page_size)
            .cloned()
            .collect();
        let next = changes
            .last()
            .map_or(after, |c| c["sequence"].as_i64().unwrap());
        let more = self
            .changes
            .iter()
            .any(|c| c["sequence"].as_i64().unwrap() > next);
        let mut value = json!({"protocolVersion":1,"after":after,"nextCursor":next,"hasMore":more,"changes":changes});
        if self.malformed_pull {
            self.malformed_pull = false;
            value["nextCursor"] = json!(next + 1);
        }
        if let Some(callback) = self.after_pull.take() {
            callback();
        }
        Ok(serde_json::from_value(value).unwrap())
    }
}
fn reauthenticate(store: &mut Store) {
    store.removal_session = Some(removal::Session {
        can_write: true,
        user_id: "test-admin".into(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    });
}
fn sync_status(store: &Store) -> Value {
    store.snapshot().unwrap()["memberSync"].clone()
}
#[test]
fn sync_engine_mock_server_binding_is_stable_and_rejects_cross_gym_device_and_insecure_origin() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    let scope = server.scope.clone();
    f.store.bind_member_scope(scope.clone()).unwrap();
    assert_eq!(
        Store::open(&f.path).unwrap().snapshot().unwrap()["memberSync"]["scope"],
        json!(scope)
    );
    assert!(f
        .store
        .bind_member_scope(
            SyncScope::new("https://other.example", &scope.gym_id, &scope.device_id).unwrap()
        )
        .is_err());
    assert!(f
        .store
        .bind_member_scope(SyncScope::new(&scope.server_origin, &id(), &scope.device_id).unwrap())
        .is_err());
    assert!(f
        .store
        .bind_member_scope(SyncScope::new(&scope.server_origin, &scope.gym_id, &id()).unwrap())
        .is_err());
    server.scope.gym_id = id();
    assert!(f
        .store
        .run_member_sync(&mut server, Utc::now(), Limits::default())
        .is_err());
    assert!(server.events.is_empty());
    for url in [
        "http://gym.example",
        "https://user:secret@gym.example",
        "https://gym.example/path",
        "https://gym.example?secret=x",
        "https://gym.example#x",
        " https://gym.example",
    ] {
        assert!(SyncScope::new(url, &scope.gym_id, &scope.device_id).is_err());
    }
    assert_eq!(
        SyncScope::new("https://GYM.example:443/", &scope.gym_id, &scope.device_id).unwrap(),
        scope
    );
}
#[test]
fn sync_engine_mock_server_dropped_push_response_persists_retry_and_prevents_all_pulls_until_ack() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("ABC")).unwrap();
    server.drop_response = true;
    let now = Utc::now();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, now, Limits::default())
            .unwrap(),
        Run::Failed
    );
    assert_eq!(server.changes.len(), 1);
    assert_eq!(server.events.len(), 1);
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert_eq!(sync_status(&f.store)["cursor"], 0);
    let first = server.receipts.values().next().unwrap().0.clone();
    let retry: DateTime<Utc> =
        serde_json::from_value(sync_status(&f.store)["retryOn"].clone()).unwrap();
    let mut reopened = Store::open(&f.path).unwrap();
    reauthenticate(&mut reopened);
    assert!(matches!(
        reopened
            .run_member_sync(&mut server, now, Limits::default())
            .unwrap(),
        Run::Deferred { .. }
    ));
    assert_eq!(server.events.len(), 1);
    assert_eq!(
        reopened
            .run_member_sync(&mut server, retry, Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
    assert_eq!(server.changes.len(), 1);
    assert_eq!(server.receipts.values().next().unwrap().0, first);
    assert_eq!(reopened.snapshot().unwrap()["pending"], 0);
    assert_eq!(sync_status(&reopened)["cursor"], 1);
    assert!(sync_status(&reopened)["conflicts"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        server
            .events
            .iter()
            .map(|v| v.split(':').next().unwrap())
            .collect::<Vec<_>>(),
        vec!["push", "push", "pull"]
    );
}
#[test]
fn sync_engine_mock_server_new_write_during_push_is_delivered_before_pull_without_overwrite() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("ABC")).unwrap();
    f.store.save_plan(plan()).unwrap();
    let path = f.path.clone();
    server.after_push = Some(Box::new(move || {
        let mut other = Store::open(&path).unwrap();
        let mut edit = member("NEW");
        edit.id = Some(first_id(&other, "members"));
        edit.version = Some(1);
        edit.name = "New local name".into();
        other.save_member(edit).unwrap();
    }));
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 2,
            pages: 1
        }
    );
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["members"][0]["name"], "New local name");
    assert_eq!(s["pending"], 1);
    assert_eq!(
        server
            .events
            .iter()
            .map(|v| v.split(':').next().unwrap())
            .collect::<Vec<_>>(),
        vec!["push", "push", "pull"]
    );
}
#[test]
fn sync_engine_mock_server_permanent_rejection_retains_operation_and_blocks_pull() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("")).unwrap();
    server.fail_push = Some(RemoteFailure::Rejected {
        kind: Rejection::Card,
        member: Value::Null,
    });
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Blocked
    );
    let s = f.store.snapshot().unwrap();
    assert_eq!(s["pending"], 1);
    assert_eq!(sync_status(&f.store)["cursor"], 0);
    assert_eq!(
        sync_status(&f.store)["conflicts"].as_array().unwrap().len(),
        1
    );
    assert_eq!(server.events.len(), 1);
    assert!(sync_status(&f.store)["lastSuccessOn"].is_null());
}
#[test]
fn sync_engine_mock_server_page_failure_rolls_back_projection_and_cursor_and_can_retry() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    server.add_remote();
    server.add_remote();
    server.malformed_pull = true;
    let now = Utc::now();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, now, Limits::default())
            .unwrap(),
        Run::Failed
    );
    assert_eq!(f.store.snapshot().unwrap()["members"], json!([]));
    assert_eq!(f.store.snapshot().unwrap()["auditCount"], 0);
    assert_eq!(sync_status(&f.store)["cursor"], 0);
    let retry: DateTime<Utc> =
        serde_json::from_value(sync_status(&f.store)["retryOn"].clone()).unwrap();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, retry, Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 0,
            pages: 1
        }
    );
    assert_eq!(
        f.store.snapshot().unwrap()["members"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(sync_status(&f.store)["cursor"], 2);
}
#[test]
fn sync_engine_mock_server_new_write_during_pull_defers_page_until_pushes_succeed() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    server.add_remote();
    let path = f.path.clone();
    server.after_pull = Some(Box::new(move || {
        let mut other = Store::open(&path).unwrap();
        other.save_member(member("NEW")).unwrap();
    }));
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Yielded {
            pushed: 0,
            pages: 0
        }
    );
    assert_eq!(sync_status(&f.store)["cursor"], 0);
    assert_eq!(
        f.store.snapshot().unwrap()["members"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
    assert_eq!(sync_status(&f.store)["cursor"], 2);
    assert_eq!(
        f.store.snapshot().unwrap()["members"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        server
            .events
            .iter()
            .map(|v| v.split(':').next().unwrap())
            .collect::<Vec<_>>(),
        vec!["pull", "push", "pull"]
    );
}
#[test]
fn sync_engine_mock_server_bounded_pagination_only_sets_success_at_completed_page() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    server.add_remote();
    server.add_remote();
    server.page_size = 1;
    let limits = Limits {
        pushes: 1,
        pages: 1,
    };
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), limits)
            .unwrap(),
        Run::Yielded {
            pushed: 0,
            pages: 1
        }
    );
    assert!(sync_status(&f.store)["lastSuccessOn"].is_null());
    assert_eq!(sync_status(&f.store)["cursor"], 1);
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), limits)
            .unwrap(),
        Run::Complete {
            pushed: 0,
            pages: 1
        }
    );
    assert!(sync_status(&f.store)["lastSuccessOn"].is_string());
    assert_eq!(sync_status(&f.store)["cursor"], 2);
    assert_eq!(sync_status(&f.store)["available"], false);
}
#[test]
fn sync_engine_mock_server_role_expiry_revocation_wrong_subject_and_remote_denial_fail_closed() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    let now = Utc::now();
    f.store.removal_session.as_mut().unwrap().expires_at = now - chrono::Duration::seconds(1);
    assert!(f
        .store
        .run_member_sync(&mut server, now, Limits::default())
        .is_err());
    reauthenticate(&mut f.store);
    f.store
        .conn
        .execute("UPDATE users SET active=0", [])
        .unwrap();
    assert!(f
        .store
        .run_member_sync(&mut server, now, Limits::default())
        .is_err());
    f.store
        .conn
        .execute("UPDATE users SET active=1", [])
        .unwrap();
    let original = server.subject.clone();
    server.subject = id();
    assert!(f
        .store
        .run_member_sync(&mut server, now, Limits::default())
        .is_err());
    server.subject = original;
    assert!(server.events.is_empty());
    server.fail_pull = Some(RemoteFailure::Authorization);
    assert_eq!(
        f.store
            .run_member_sync(&mut server, now, Limits::default())
            .unwrap(),
        Run::Failed
    );
    assert!(f.store.removal_session.is_none());
    assert!(sync_status(&f.store)["lastSuccessOn"].is_null());
}
#[test]
fn sync_engine_mock_server_restore_retains_binding_and_retry_but_never_resumes_sync() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("")).unwrap();
    server.fail_push = Some(RemoteFailure::Transient {
        retry_after_seconds: Some(60),
    });
    f.store
        .run_member_sync(&mut server, Utc::now(), Limits::default())
        .unwrap();
    let before = sync_status(&f.store);
    let backup = f.store.backup_envelope().unwrap();
    let preview = f.store.preview_restore(backup).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    reauthenticate(&mut f.store);
    let events = server.events.len();
    assert!(f
        .store
        .run_member_sync(
            &mut server,
            Utc::now() + chrono::Duration::hours(1),
            Limits::default()
        )
        .is_err());
    assert_eq!(server.events.len(), events);
    assert_eq!(sync_status(&f.store)["scope"], before["scope"]);
    assert_eq!(sync_status(&f.store)["retryOn"], before["retryOn"]);
}
#[test]
fn sync_engine_mock_server_session_revoked_during_push_preserves_unacknowledged_operation() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("")).unwrap();
    let path = f.path.clone();
    server.after_push = Some(Box::new(move || {
        let other = Store::open(&path).unwrap();
        other.conn.execute("UPDATE users SET active=0", []).unwrap();
    }));
    assert!(f
        .store
        .run_member_sync(&mut server, Utc::now(), Limits::default())
        .is_err());
    assert_eq!(server.events.len(), 1);
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert_eq!(sync_status(&f.store)["cursor"], 0);
    assert_eq!(sync_status(&f.store)["acknowledged"], 0);
}
#[test]
fn sync_engine_mock_server_retry_error_and_deadline_are_one_atomic_commit() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("")).unwrap();
    server.fail_push = Some(RemoteFailure::Transient {
        retry_after_seconds: None,
    });
    f.store.conn.execute_batch("CREATE TRIGGER fail_retry_metadata BEFORE INSERT ON metadata WHEN NEW.key='member_sync_runtime' BEGIN SELECT RAISE(ABORT,'Injected retry write failure'); END;").unwrap();
    assert!(f
        .store
        .run_member_sync(&mut server, Utc::now(), Limits::default())
        .is_err());
    let error: Option<String> = f
        .store
        .conn
        .query_row("SELECT last_error FROM member_deliveries", [], |r| r.get(0))
        .unwrap();
    assert!(error.is_none());
    assert!(sync_status(&f.store)["retryOn"].is_null());
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert_eq!(server.events.len(), 1);
    f.store
        .conn
        .execute_batch("DROP TRIGGER fail_retry_metadata;")
        .unwrap();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
}
#[test]
fn sync_engine_mock_server_unbound_database_cannot_grant_or_invent_gym_identity() {
    let mut f = Fixture::new();
    let device: String = f
        .store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let binding = SyncScope::new("https://gym.example", &id(), &device).unwrap();
    assert!(f.store.bind_member_scope(binding.clone()).is_err());
    assert!(sync_status(&f.store)["scope"].is_null());
    assert_eq!(
        Store::open(&f.path)
            .unwrap()
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        device
    );
    removal_admin(&mut f.store);
    assert!(f.store.bind_member_scope(binding.clone()).is_err());
    f.store
        .conn
        .execute("UPDATE users SET subject=?1", [id()])
        .unwrap();
    f.store
        .conn
        .execute("INSERT INTO member_remote_heads VALUES(?1,1,'{}')", [id()])
        .unwrap();
    assert!(f.store.bind_member_scope(binding).is_err());
    assert!(sync_status(&f.store)["scope"].is_null());
}
#[test]
fn sync_engine_mock_server_archive_requires_original_actor_and_current_administrator_role() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("")).unwrap();
    f.store
        .run_member_sync(&mut server, Utc::now(), Limits::default())
        .unwrap();
    f.store
        .archive_member(removal_member_input(&f.store))
        .unwrap();
    let calls = server.events.len();
    f.store
        .conn
        .execute(
            "UPDATE roles SET name='Reception' WHERE id='test-administrator'",
            [],
        )
        .unwrap();
    assert!(f
        .store
        .run_member_sync(&mut server, Utc::now(), Limits::default())
        .is_err());
    assert_eq!(server.events.len(), calls);
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    f.store
        .conn
        .execute(
            "UPDATE roles SET name='Administrator' WHERE id='test-administrator'",
            [],
        )
        .unwrap();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
    assert_eq!(f.store.snapshot().unwrap()["pending"], 0);
}
#[test]
fn sync_engine_mock_server_later_push_failure_keeps_prior_acknowledgement_and_stops_pull() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("")).unwrap();
    f.store.save_member(member("")).unwrap();
    assert_eq!(
        f.store
            .run_member_sync(
                &mut server,
                Utc::now(),
                Limits {
                    pushes: 1,
                    pages: 1
                }
            )
            .unwrap(),
        Run::Yielded {
            pushed: 1,
            pages: 0
        }
    );
    server.fail_push = Some(RemoteFailure::Transient {
        retry_after_seconds: None,
    });
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Failed
    );
    assert_eq!(sync_status(&f.store)["acknowledged"], 1);
    assert_eq!(sync_status(&f.store)["cursor"], 0);
    assert_eq!(f.store.snapshot().unwrap()["pending"], 1);
    assert!(server.events.iter().all(|e| e.starts_with("push:")));
    let retry: DateTime<Utc> =
        serde_json::from_value(sync_status(&f.store)["retryOn"].clone()).unwrap();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, retry, Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
    assert_eq!(sync_status(&f.store)["acknowledged"], 2);
    assert_eq!(sync_status(&f.store)["cursor"], 2);
}

#[test]
fn sync_engine_mock_resolution_sends_only_fresh_retry_then_acknowledges_and_pulls() {
    let mut f = Fixture::new();
    let mut server = MockServer::fixture(&mut f.store);
    f.store.save_member(member("LOCAL")).unwrap();
    let local = f.store.snapshot().unwrap()["members"][0].clone();
    let member_id = local["id"].as_str().unwrap().to_string();
    let remote = json!({"id":member_id,"name":"Server version","phone":"0771234567","email":"","nfcId":"REMOTE","joinedOn":local["joinedOn"],"revision":3,"archivedAt":null,"archivedBy":null});
    server.members.insert(member_id.clone(), remote.clone());
    server
        .changes
        .push(json!({"sequence":1,"operationId":id(),"member":remote}));
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Blocked
    );
    let conflict = sync_status(&f.store)["conflicts"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let review = f.store.preview_member_conflict(&conflict).unwrap();
    let old_operation = review["operations"][0]["id"].as_str().unwrap().to_string();
    let old_request: String = f
        .store
        .conn
        .query_row(
            "SELECT request_json FROM member_deliveries WHERE operation_id=?1",
            [&old_operation],
            |r| r.get(0),
        )
        .unwrap();
    let result = f
        .store
        .resolve_member_conflict(MemberConflictInput {
            request_id: id(),
            conflict_id: conflict,
            fingerprint: review["fingerprint"].as_str().unwrap().into(),
            choice: crate::member_conflicts::ConflictChoice::KeepLocal,
            reason: "Retain the reviewed local member details".into(),
        })
        .unwrap();
    server.events.clear();
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
    assert_eq!(
        server.events,
        vec![
            format!("push:{}", result["retryOperationId"].as_str().unwrap()),
            "pull:0".into()
        ]
    );
    assert_eq!(server.members[&member_id]["revision"], 4);
    assert_eq!(server.members[&member_id]["nfcId"], "LOCAL");
    let after = f.store.snapshot().unwrap();
    assert_eq!(after["pending"], 0);
    assert_eq!(after["memberSync"]["acknowledged"], 1);
    assert_eq!(after["memberSync"]["superseded"], 1);
    assert_eq!(after["memberSync"]["cursor"], 2);
    assert_eq!(after["members"][0]["nfcId"], "LOCAL");
    let stored: (String, String) = f
        .store
        .conn
        .query_row(
            "SELECT request_json,state FROM member_deliveries WHERE operation_id=?1",
            [&old_operation],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(stored, (old_request, "conflict".into()));
    assert_eq!(
        f.store
            .run_member_sync(&mut server, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 0,
            pages: 1
        }
    );
    assert_eq!(f.store.snapshot().unwrap()["memberSync"]["acknowledged"], 1);
}
