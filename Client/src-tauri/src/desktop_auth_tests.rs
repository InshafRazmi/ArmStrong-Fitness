use super::*;
struct Fixture {
    directory: std::path::PathBuf,
    store: Store,
}
impl Fixture {
    fn verified(&self, auth: &DesktopAuth, role: &str) -> VerifiedEnrollment {
        use crate::member_http::{ExchangeError, HttpsExchange, Request, Response};
        struct Mock(std::collections::VecDeque<Value>);
        impl HttpsExchange for Mock {
            fn send(&mut self, _: Request) -> std::result::Result<Response, ExchangeError> {
                Ok(Response {
                    status: 200,
                    content_type: Some("application/json".into()),
                    retry_after: None,
                    body: serde_json::to_vec(&self.0.pop_front().unwrap()).unwrap(),
                })
            }
        }
        let subject = id();
        let device: String = self
            .store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let mut mock=Mock([json!({"access_token":"mock-private-access","token_type":"bearer","expires_in":3600,"user":{"id":subject,"email":"admin@example.test"}}),json!({"id":subject,"email":"admin@example.test"}),json!({"protocolVersion":1,"gym":{"id":id(),"name":"Test gym"},"staff":{"id":subject,"name":"Verified Admin","role":role},"device":{"id":device,"canWrite":true}})].into());
        super::super::native_auth::login(
            &mut mock,
            auth.config.as_ref().unwrap(),
            &device,
            "a".repeat(64),
            "admin@example.test",
            "mock-private-password",
        )
        .unwrap()
    }
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("armstrong-desktop-auth-{}", id()));
        std::fs::create_dir(&directory).unwrap();
        let store = Store::open(&directory.join("test.sqlite3")).unwrap();
        Self { directory, store }
    }
    fn configured(&self) -> DesktopAuth {
        std::fs::write(self.directory.join("desktop-auth.json"),r#"{"authOrigin":"https://auth.example","apiOrigin":"https://api.example","publishableKey":"sb_publishable_mock"}"#).unwrap();
        DesktopAuth::load(&self.directory)
    }
    fn session(&mut self, can_write: bool) {
        let user = id();
        let role = id();
        self.store
            .conn
            .execute(
                "INSERT INTO users VALUES(?1,?1,'admin@example.test','Verified Admin',1,1)",
                [&user],
            )
            .unwrap();
        self.store
            .conn
            .execute("INSERT INTO roles VALUES(?1,'Administrator')", [&role])
            .unwrap();
        self.store
            .conn
            .execute("INSERT INTO user_roles VALUES(?1,?2)", params![user, role])
            .unwrap();
        self.store.removal_session = Some(removal::Session {
            user_id: user,
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            can_write,
        });
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
#[test]
fn configured_or_invalid_configuration_requires_native_login_and_survives_file_removal() {
    let mut f = Fixture::new();
    let auth = f.configured();
    auth.initialize(&mut f.store).unwrap();
    assert!(auth.status(&f.store).unwrap().requires_login);
    assert!(auth.authorize(&mut f.store, false).is_err());
    std::fs::remove_file(f.directory.join("desktop-auth.json")).unwrap();
    assert!(
        DesktopAuth::load(&f.directory)
            .status(&f.store)
            .unwrap()
            .requires_login
    );
    std::fs::write(
        f.directory.join("desktop-auth.json"),
        b"private-invalid-config",
    )
    .unwrap();
    let bad = DesktopAuth::load(&f.directory);
    let status = bad.status(&f.store).unwrap();
    assert!(status.requires_login && !status.configured && !status.authenticated);
    assert!(!serde_json::to_string(&status)
        .unwrap()
        .contains("private-invalid-config"));
}
#[test]
fn native_gate_rechecks_expiry_role_activity_and_read_only_device() {
    let mut f = Fixture::new();
    let auth = f.configured();
    auth.initialize(&mut f.store).unwrap();
    f.session(false);
    assert!(auth.authorize(&mut f.store, false).is_ok());
    assert!(auth.authorize(&mut f.store, true).is_err());
    f.store.removal_session.as_mut().unwrap().can_write = true;
    assert!(auth.authorize(&mut f.store, true).is_ok());
    f.store
        .conn
        .execute("UPDATE users SET active=0", [])
        .unwrap();
    assert!(auth.authorize(&mut f.store, false).is_err());
    f.store
        .conn
        .execute("UPDATE users SET active=1", [])
        .unwrap();
    f.store.removal_session.as_mut().unwrap().expires_at =
        Utc::now() - chrono::Duration::seconds(1);
    assert!(auth.authorize(&mut f.store, false).is_err());
    assert!(!auth.status(&f.store).unwrap().authenticated);
    f.store.removal_session.as_mut().unwrap().expires_at =
        Utc::now() + chrono::Duration::minutes(1);
    f.store
        .conn
        .execute("UPDATE roles SET name='Reception'", [])
        .unwrap();
    assert!(auth.authorize(&mut f.store, true).is_err());
}
#[test]
fn logout_and_superseding_login_invalidate_pending_native_results_and_actor_context() {
    let mut f = Fixture::new();
    let auth = f.configured();
    auth.initialize(&mut f.store).unwrap();
    f.session(true);
    auth.authorize(&mut f.store, true).unwrap();
    assert!(actor_label(&f.store.conn)
        .unwrap()
        .contains("Verified Admin"));
    let first = auth.begin(&mut f.store).unwrap();
    assert!(!auth.status(&f.store).unwrap().authenticated);
    let second = auth.begin(&mut f.store).unwrap();
    assert_ne!(first, second);
    auth.lock(&mut f.store).unwrap();
    assert_ne!(second, auth.epoch.load(Ordering::SeqCst));
    assert_eq!(actor_label(&f.store.conn).unwrap(), ACTOR);
    assert!(auth.transport.lock().unwrap().is_none());
}
#[test]
fn unconfigured_local_test_remains_explicit_and_bound_store_cannot_bypass_auth() {
    let mut f = Fixture::new();
    let auth = DesktopAuth::load(&f.directory);
    auth.initialize(&mut f.store).unwrap();
    assert!(!auth.status(&f.store).unwrap().requires_login);
    auth.authorize(&mut f.store, true).unwrap();
    assert_eq!(actor_label(&f.store.conn).unwrap(), ACTOR);
    f.store
        .conn
        .execute(
            "INSERT INTO metadata VALUES('member_sync_scope','private-binding')",
            [],
        )
        .unwrap();
    assert!(auth.status(&f.store).unwrap().requires_login);
    assert!(auth.authorize(&mut f.store, false).is_err());
}
#[test]
fn trusted_config_rejects_server_secrets_oversize_and_unexpected_fields() {
    let f = Fixture::new();
    for text in [r#"{"authOrigin":"https://auth.example","apiOrigin":"https://api.example","publishableKey":"sb_secret_private"}"#.to_owned(),r#"{"authOrigin":"http://auth.example","apiOrigin":"https://api.example","publishableKey":"sb_publishable_mock"}"#.into(),"a".repeat(16385),r#"{"authOrigin":"https://auth.example","apiOrigin":"https://api.example","publishableKey":"sb_publishable_mock","role":"Administrator"}"#.into()] {
        std::fs::write(f.directory.join("desktop-auth.json"),text).unwrap(); let auth=DesktopAuth::load(&f.directory);
        assert!(auth.config.is_none() && auth.config_error.is_some()); assert!(!auth.status(&f.store).unwrap().authenticated);
    }
}
#[test]
fn verified_login_attributes_member_payment_receipt_and_audit_to_native_actor() {
    let mut f = Fixture::new();
    let auth = f.configured();
    auth.initialize(&mut f.store).unwrap();
    let epoch = auth.begin(&mut f.store).unwrap();
    let verified = f.verified(&auth, "Administrator");
    let status = auth
        .finish(&mut f.store, PendingDesktopLogin { epoch, verified })
        .unwrap();
    assert!(status.authenticated && status.can_write);
    assert!(!serde_json::to_string(&status)
        .unwrap()
        .contains("mock-private"));
    auth.authorize(&mut f.store, true).unwrap();
    f.store
        .save_member(MemberInput {
            id: None,
            version: None,
            name: "Synthetic Member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "".into(),
        })
        .unwrap();
    let user = f.store.removal_session.as_ref().unwrap().user_id.clone();
    let (member, actor): (String, String) = f
        .store
        .conn
        .query_row(
            "SELECT entity_id,actor_user_id FROM audit WHERE entity='member'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(actor, user);
    let payload: String = f
        .store
        .conn
        .query_row(
            "SELECT payload_json FROM outbox WHERE entity='member'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&payload).unwrap()["actorUserId"],
        user
    );
    auth.authorize(&mut f.store, true).unwrap();
    let payment = f
        .store
        .record_payment(PaymentInput {
            request_id: id(),
            member_id: member,
            amount_minor: 12345,
            method: "Cash".into(),
        })
        .unwrap();
    let receipt = f
        .store
        .payment_receipt(payment["id"].as_str().unwrap().into())
        .unwrap();
    assert_eq!(
        receipt["snapshot"]["payment"]["actor"],
        format!("Verified Admin ({user})")
    );
    let fingerprint = f.store.snapshot().unwrap();
    auth.lock(&mut f.store).unwrap();
    assert!(auth.authorize(&mut f.store, true).is_err());
    assert_eq!(
        f.store.snapshot().unwrap()["auditCount"],
        fingerprint["auditCount"]
    );
    let mut reopened = Store::open(&f.store.path).unwrap();
    assert!(auth.authorize(&mut reopened, false).is_err());
}
#[test]
fn cancelled_login_and_reception_do_not_bind_or_unlock_the_database() {
    for cancel in [true, false] {
        let mut f = Fixture::new();
        let auth = f.configured();
        auth.initialize(&mut f.store).unwrap();
        let epoch = auth.begin(&mut f.store).unwrap();
        let verified = f.verified(&auth, if cancel { "Administrator" } else { "Reception" });
        if cancel {
            auth.lock(&mut f.store).unwrap();
        }
        assert!(auth
            .finish(&mut f.store, PendingDesktopLogin { epoch, verified })
            .is_err());
        assert!(!auth.status(&f.store).unwrap().authenticated);
        let bound: bool = f
            .store
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='member_sync_scope')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!bound);
        assert_eq!(f.store.snapshot().unwrap()["auditCount"], 0);
    }
}
#[test]
fn authenticated_restore_preserves_actor_auth_requirement_and_locks_session() {
    let mut f = Fixture::new();
    let auth = f.configured();
    auth.initialize(&mut f.store).unwrap();
    let epoch = auth.begin(&mut f.store).unwrap();
    let verified = f.verified(&auth, "Administrator");
    auth.finish(&mut f.store, PendingDesktopLogin { epoch, verified })
        .unwrap();
    auth.authorize(&mut f.store, true).unwrap();
    let user = f.store.removal_session.as_ref().unwrap().user_id.clone();
    let exported = f.store.export_backup().unwrap();
    let envelope: crate::BackupEnvelope =
        serde_json::from_slice(&std::fs::read(exported["path"].as_str().unwrap()).unwrap())
            .unwrap();
    let preview = f.store.preview_restore(envelope).unwrap();
    f.store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap();
    assert!(!auth.status(&f.store).unwrap().authenticated);
    assert!(auth.authorize(&mut f.store, false).is_err());
    let actor: String = f
        .store
        .conn
        .query_row(
            "SELECT actor_user_id FROM audit WHERE entity='restore'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actor, user);
    std::fs::remove_file(f.directory.join("desktop-auth.json")).unwrap();
    assert!(
        DesktopAuth::load(&f.directory)
            .status(&f.store)
            .unwrap()
            .requires_login
    );
    assert!(f.store.snapshot().unwrap()["restoreRequiresReconciliation"]
        .as_bool()
        .unwrap());
}
#[test]
fn backup_without_restoration_actor_refuses_replacement_and_preserves_records() {
    let mut f = Fixture::new();
    let auth = f.configured();
    auth.initialize(&mut f.store).unwrap();
    let exported = f.store.export_backup().unwrap();
    let envelope: crate::BackupEnvelope =
        serde_json::from_slice(&std::fs::read(exported["path"].as_str().unwrap()).unwrap())
            .unwrap();
    let epoch = auth.begin(&mut f.store).unwrap();
    let verified = f.verified(&auth, "Administrator");
    auth.finish(&mut f.store, PendingDesktopLogin { epoch, verified })
        .unwrap();
    auth.authorize(&mut f.store, true).unwrap();
    let preview = f.store.preview_restore(envelope).unwrap();
    let before = f.store.snapshot().unwrap();
    assert!(f
        .store
        .restore_backup(preview["token"].as_str().unwrap().into())
        .unwrap_err()
        .contains("restoration actor"));
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert!(auth.status(&f.store).unwrap().authenticated);
}
