use super::*;
const BUNDLED: &str = r#"{"authOrigin":"https://auth.example","apiOrigin":"https://api.example","publishableKey":"sb_publishable_mock"}"#;
#[derive(Default)]
struct AccessVault {
    value: Option<String>,
}
impl super::super::native_credentials::OfflineVault for AccessVault {
    fn read_access(&mut self, _: &str) -> Result<Option<String>> {
        Ok(self.value.clone())
    }
    fn save_access(&mut self, _: &str, value: &str) -> Result<()> {
        self.value = Some(value.into());
        Ok(())
    }
    fn clear_access(&mut self, _: &str) -> Result<()> {
        self.value = None;
        Ok(())
    }
}
#[test]
fn verified_native_login_saves_bounded_offline_access_without_bearer_or_password() {
    let mut f = Fixture::new();
    let auth = f.configured();
    let mut vault = AccessVault::default();
    auth.initialize(&mut f.store).unwrap();
    f.store
        .conn
        .execute(
            "INSERT INTO metadata VALUES('native_device_secret_sha256',?1)",
            [super::super::native_credentials::digest(&"a".repeat(64))],
        )
        .unwrap();
    let epoch = auth.begin(&mut f.store).unwrap();
    let verified = f.verified(&auth, "Administrator");
    let status = auth
        .finish_with_vault(
            &mut f.store,
            PendingDesktopLogin { epoch, verified },
            &mut vault,
        )
        .unwrap();
    assert!(status.authenticated && status.offline_until.is_some() && !status.offline);
    let saved = vault.value.unwrap();
    for secret in [
        "mock-private-access",
        "mock-private-password",
        "access_token",
        "refresh_token",
    ] {
        assert!(!saved.contains(secret));
    }
    assert!(saved.len() <= super::super::native_credentials::ACCESS_LIMIT);
    let mut snapshot = f.store.snapshot().unwrap();
    auth.member_sync_status(&f.store, &mut snapshot);
    assert_eq!(snapshot["memberSync"]["available"], true);
}
#[test]
fn native_logout_invalidates_dedicated_worker_and_queued_job_before_network() {
    let mut f = Fixture::new();
    let auth = f.configured();
    let mut vault = AccessVault::default();
    auth.initialize(&mut f.store).unwrap();
    let epoch = auth.begin(&mut f.store).unwrap();
    let verified = f.verified(&auth, "Administrator");
    auth.finish_with_vault(
        &mut f.store,
        PendingDesktopLogin { epoch, verified },
        &mut vault,
    )
    .unwrap();
    let job = auth.prepare_member_sync(&f.store).unwrap();
    let mut worker = Store::open(&job.path).unwrap();
    worker.removal_session = Some(job.session.clone());
    assert!(worker.native_identity(false).is_ok());
    auth.lock(&mut f.store).unwrap();
    assert!(worker.native_identity(false).is_err());
    assert!(auth.run_member_sync(job).is_err());
    assert!(!auth.online_transport.load(Ordering::SeqCst));
}

#[test]
fn logout_cancels_queued_and_completed_session_renewal_without_unlocking_or_network() {
    let mut f = Fixture::new();
    let auth = f.configured();
    let mut vault = AccessVault::default();
    auth.initialize(&mut f.store).unwrap();
    assert!(auth.prepare_session_renewal(&f.store).unwrap().is_none());
    let epoch = auth.begin(&mut f.store).unwrap();
    let verified = f.verified(&auth, "Administrator");
    auth.finish_with_vault(
        &mut f.store,
        PendingDesktopLogin { epoch, verified },
        &mut vault,
    )
    .unwrap();
    let queued = auth.prepare_session_renewal(&f.store).unwrap().unwrap();
    let completed = SessionRenewal {
        epoch,
        verified: Some(f.verified(&auth, "Administrator")),
        refused: false,
    };
    auth.lock(&mut f.store).unwrap();
    assert!(auth.run_session_renewal(queued).is_err());
    assert!(auth
        .finish_session_renewal(&mut f.store, completed)
        .is_err());
    assert!(!auth.status(&f.store).unwrap().authenticated);
    assert!(auth.renewal.lock().unwrap().is_none());
    assert!(auth.transport.lock().unwrap().is_none());
}

#[test]
fn fresh_installed_app_requires_login_without_a_workstation_config_file() {
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
    let auth = DesktopAuth::load_with_bundled(&f.directory, Some(BUNDLED));
    auth.initialize(&mut f.store).unwrap();
    let status = auth.status(&f.store).unwrap();
    assert!(status.configured && status.requires_login && !status.authenticated);
    assert!(auth.authorize(&mut f.store, false).is_err());
    assert!(auth.authorize(&mut f.store, true).is_err());
    assert!(!f.directory.join("desktop-auth.json").exists());
    let unchanged: String = f
        .store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(device, unchanged);
}

#[test]
fn installed_app_preserves_matching_config_and_refuses_other_server_settings() {
    let mut f = Fixture::new();
    let path = f.directory.join("desktop-auth.json");
    std::fs::write(&path, BUNDLED).unwrap();
    let auth = DesktopAuth::load_with_bundled(&f.directory, Some(BUNDLED));
    assert!(auth.status(&f.store).unwrap().configured);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), BUNDLED);
    for other in [
        BUNDLED.replace("https://api.example", "https://other.example"),
        BUNDLED.replace("sb_publishable_mock", "sb_publishable_other"),
    ] {
        std::fs::write(&path, &other).unwrap();
        let auth = DesktopAuth::load_with_bundled(&f.directory, Some(BUNDLED));
        auth.initialize(&mut f.store).unwrap();
        let status = auth.status(&f.store).unwrap();
        assert!(status.requires_login && !status.configured && !status.authenticated);
        assert!(status.reason.contains("settings differ"));
        assert!(auth.authorize(&mut f.store, false).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), other);
    }
}

#[test]
fn malformed_or_privileged_bundled_settings_never_fall_back_to_local_access() {
    let mut f = Fixture::new();
    std::fs::write(f.directory.join("desktop-auth.json"), BUNDLED).unwrap();
    for invalid in [
        "private-invalid-config".into(),
        BUNDLED.replace("sb_publishable_mock", "sb_secret_private"),
        BUNDLED.replace("https://api.example", "http://api.example"),
        BUNDLED.replace("{", "{\"databaseUrl\":\"private-sql-password\","),
    ] {
        let auth = DesktopAuth::load_with_bundled(&f.directory, Some(&invalid));
        auth.initialize(&mut f.store).unwrap();
        let status = auth.status(&f.store).unwrap();
        assert!(status.requires_login && !status.configured && !status.authenticated);
        assert!(auth.begin(&mut f.store).is_err());
        assert!(!serde_json::to_string(&status).unwrap().contains("private"));
        assert_eq!(
            std::fs::read_to_string(f.directory.join("desktop-auth.json")).unwrap(),
            BUNDLED
        );
    }
}

#[test]
fn invalid_existing_settings_cannot_be_hidden_by_a_valid_installer_default() {
    let mut f = Fixture::new();
    for invalid in [
        "invalid-local".into(),
        BUNDLED.replace("{", "{\"apiOrigin\":\"https://api.example\","),
    ] {
        std::fs::write(f.directory.join("desktop-auth.json"), &invalid).unwrap();
        let auth = DesktopAuth::load_with_bundled(&f.directory, Some(BUNDLED));
        auth.initialize(&mut f.store).unwrap();
        assert!(!auth.status(&f.store).unwrap().configured);
        assert!(auth.authorize(&mut f.store, false).is_err());
        assert_eq!(
            std::fs::read_to_string(f.directory.join("desktop-auth.json")).unwrap(),
            invalid
        );
    }
}

#[cfg(feature = "packaged-auth")]
#[test]
fn packaged_build_has_the_approved_public_endpoints_and_starts_locked() {
    let mut f = Fixture::new();
    let auth = DesktopAuth::load_packaged(&f.directory);
    auth.initialize(&mut f.store).unwrap();
    assert_eq!(
        auth.config.as_ref().unwrap().api_origin,
        "https://armstrong-fitness.onrender.com"
    );
    assert_eq!(
        auth.config.as_ref().unwrap().auth_origin,
        "https://pxhnvhiaesapykivsopa.supabase.co"
    );
    assert!(auth.status(&f.store).unwrap().configured);
    assert!(auth.authorize(&mut f.store, false).is_err());
}

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
            native_nonce: None,
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
