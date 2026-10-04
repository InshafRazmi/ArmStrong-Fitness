// Real SQLite enrollment + MOCK Auth/enrollment HTTP. No live credentials/TLS.
use super::super::member_http::ExchangeError;
use super::*;
use std::collections::VecDeque;

struct Mock {
    replies: VecDeque<std::result::Result<Response, ExchangeError>>,
    requests: Vec<Request>,
}
impl HttpsExchange for Mock {
    fn send(&mut self, request: Request) -> std::result::Result<Response, ExchangeError> {
        self.requests.push(request);
        self.replies.pop_front().expect("Unexpected HTTP request")
    }
}
fn reply(payload: Value) -> Response {
    Response {
        status: 200,
        content_type: Some("application/json".into()),
        retry_after: None,
        body: serde_json::to_vec(&payload).unwrap(),
    }
}
struct Fixture {
    path: std::path::PathBuf,
    store: Store,
    config: AuthConfig,
    device: String,
    subject: String,
    gym: String,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("armstrong-auth-{}.sqlite3", id()));
        let store = Store::open(&path).unwrap();
        let device = store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        Self {
            path,
            store,
            device,
            subject: id(),
            gym: id(),
            config: AuthConfig::new(
                "https://auth.example",
                "sb_publishable_mock",
                "https://api.example",
            )
            .unwrap(),
        }
    }
    fn responses(&self) -> Vec<Value> {
        vec![
            json!({"access_token":"mock-access-token","refresh_token":"unused-refresh-token","token_type":"bearer","expires_in":3600,"expires_at":Utc::now().timestamp()+3600,"user":{"id":self.subject,"email":"admin@example.test","user_metadata":{"role":"Administrator","gymId":"forged"}}}),
            json!({"id":self.subject,"email":"admin@example.test","user_metadata":{"role":"Administrator"}}),
            json!({"protocolVersion":1,"gym":{"id":self.gym,"name":"Test gym"},"staff":{"id":self.subject,"name":"Test administrator","role":"Administrator"},"device":{"id":self.device,"canWrite":true}}),
        ]
    }
    fn mock(&self) -> Mock {
        Mock {
            replies: self
                .responses()
                .into_iter()
                .map(|payload| Ok(reply(payload)))
                .collect(),
            requests: vec![],
        }
    }
    fn login(&self, mock: &mut Mock) -> Result<VerifiedEnrollment> {
        login(
            mock,
            &self.config,
            &self.device,
            "a".repeat(64),
            "admin@example.test",
            "private-test-password",
        )
    }
    fn count(&self, table: &str) -> i64 {
        self.store
            .conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
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

#[test]
fn native_auth_mock_checks_online_identity_then_enrolls_persisted_device_without_role_claims() {
    let mut f = Fixture::new();
    let mut mock = f.mock();
    let result = f.login(&mut mock).ok().unwrap();
    assert_eq!(mock.requests.len(), 3);
    assert_eq!(
        mock.requests[0].url,
        "https://auth.example/auth/v1/token?grant_type=password"
    );
    assert_eq!(mock.requests[0].method, "POST");
    assert_eq!(mock.requests[1].url, "https://auth.example/auth/v1/user");
    assert_eq!(mock.requests[1].method, "GET");
    assert!(mock.requests[1].body.is_none());
    assert_eq!(mock.requests[2].url, "https://api.example/v1/enrollment");
    let enrollment: Value =
        serde_json::from_slice(mock.requests[2].body.as_ref().unwrap()).unwrap();
    assert_eq!(
        enrollment,
        json!({"protocolVersion":1,"deviceId":f.device,"deviceSecret":"a".repeat(64)})
    );
    assert!(mock.requests[2]
        .headers
        .iter()
        .all(|(key, _)| *key != "apikey"));
    assert!(mock
        .requests
        .iter()
        .all(|request| !request.url.contains("private-test-password")
            && !request.url.contains("mock-access-token")
            && !request.url.contains(&"a".repeat(64))));
    f.store.enroll_native(&result).unwrap();
    assert_eq!(f.count("users"), 1);
    assert_eq!(f.count("user_roles"), 1);
    assert!(
        f.store.snapshot().unwrap()["removalAuthorization"]["allowed"]
            .as_bool()
            .unwrap()
    );
    let saved = f.store.snapshot().unwrap();
    assert_eq!(saved["memberSync"]["scope"]["gymId"], f.gym);
    assert_eq!(saved["memberSync"]["scope"]["deviceId"], f.device);
    assert_eq!(saved["memberSync"]["available"], false);
    let backup = f.store.backup_envelope().unwrap();
    for secret in [
        "private-test-password",
        "mock-access-token",
        "unused-refresh-token",
        &"a".repeat(64),
    ] {
        assert!(!saved.to_string().contains(secret));
        assert!(!String::from_utf8_lossy(&backup.data).contains(secret));
    }
    // A valid prior login is not a persistent offline grant.
    let reopened = Store::open(&f.path).unwrap();
    assert_eq!(
        reopened.snapshot().unwrap()["removalAuthorization"]["allowed"],
        false
    );
}

#[test]
fn native_auth_mock_subject_mismatch_stops_before_enrollment_and_never_changes_sqlite() {
    let f = Fixture::new();
    let mut values = f.responses();
    values[1]["id"] = json!(id());
    let mut mock = Mock {
        replies: values.into_iter().map(|v| Ok(reply(v))).collect(),
        requests: vec![],
    };
    assert!(f.login(&mut mock).is_err());
    assert_eq!(mock.requests.len(), 2);
    assert_eq!(f.count("users"), 0);
    assert_eq!(f.count("roles"), 0);
    assert_eq!(
        f.store.snapshot().unwrap()["memberSync"]["scope"],
        Value::Null
    );
}

#[test]
fn native_auth_mock_refuses_unknown_enrollment_fields_roles_devices_and_subjects() {
    let f = Fixture::new();
    for change in [
        json!({"staff":{"id":id(),"name":"Forged","role":"Administrator"}}),
        json!({"staff":{"id":f.subject,"name":"Forged","role":"Owner"}}),
        json!({"device":{"id":id(),"canWrite":true}}),
        json!({"device":{"id":f.device,"canWrite":"true"}}),
        json!({"protocolVersion":2}),
        json!({"role":"Administrator"}),
        json!({"gym":{"id":"forged","name":"Test gym"}}),
        json!({"gym":{"id":f.gym,"name":""}}),
    ] {
        let mut values = f.responses();
        for (key, value) in change.as_object().unwrap() {
            values[2][key] = value.clone();
        }
        let mut mock = Mock {
            replies: values.into_iter().map(|v| Ok(reply(v))).collect(),
            requests: vec![],
        };
        assert!(f.login(&mut mock).is_err());
        assert_eq!(f.count("users"), 0);
        assert!(f.store.removal_session.is_none());
    }
}

#[test]
fn native_auth_mock_bad_token_expiry_email_and_credentials_fail_before_identity_io() {
    let f = Fixture::new();
    for change in [
        json!({"access_token":"token\r\nforged"}),
        json!({"token_type":"service_role"}),
        json!({"expires_in":0}),
        json!({"expires_in":86401}),
        json!({"expires_at":Utc::now().timestamp()-1}),
        json!({"expires_at":i64::MAX}),
        json!({"user":{"id":f.subject,"email":"other@example.test"}}),
    ] {
        let mut values = f.responses();
        for (key, value) in change.as_object().unwrap() {
            values[0][key] = value.clone();
        }
        let mut mock = Mock {
            replies: values.into_iter().map(|v| Ok(reply(v))).collect(),
            requests: vec![],
        };
        assert!(f.login(&mut mock).is_err());
        assert_eq!(mock.requests.len(), 1);
    }
    for (device, secret, email, password) in [
        (
            "forged".into(),
            "a".repeat(64),
            "admin@example.test",
            "password",
        ),
        (
            f.device.clone(),
            "a".repeat(63),
            "admin@example.test",
            "password",
        ),
        (f.device.clone(), "a".repeat(64), "admin@example.test", ""),
        (f.device.clone(), "a".repeat(64), "", "password"),
    ] {
        let mut mock = Mock {
            replies: VecDeque::new(),
            requests: vec![],
        };
        assert!(login(&mut mock, &f.config, &device, secret, email, password).is_err());
        assert!(mock.requests.is_empty());
    }
}

#[test]
fn native_auth_mock_provider_errors_redirects_and_malformed_replies_are_redacted() {
    let f = Fixture::new();
    for stage in 0..3 {
        for status in [400, 401, 403, 422, 429, 500, 503, 302, 307] {
            let mut mock = f.mock();
            mock.replies[stage] = Ok(Response {
                status,
                content_type: Some("text/html".into()),
                retry_after: None,
                body: b"private-provider-password-token".to_vec(),
            });
            let error = f.login(&mut mock).err().unwrap();
            assert!(!error.contains("private-provider"));
            assert_eq!(mock.requests.len(), stage + 1);
        }
        for body in [b"{".to_vec(), vec![b' '; RESPONSE_LIMIT + 1]] {
            let mut mock = f.mock();
            mock.replies[stage] = Ok(Response {
                status: 200,
                content_type: Some("application/json".into()),
                retry_after: None,
                body,
            });
            assert!(f.login(&mut mock).is_err());
        }
        let mut mock = f.mock();
        mock.replies[stage] = Err(ExchangeError::Unavailable);
        assert!(f.login(&mut mock).is_err());
    }
    assert_eq!(f.count("audit"), 0);
}

#[test]
fn native_auth_config_refuses_privileged_keys_insecure_or_noncanonical_origins() {
    let anon = format!(
        "header.{}.signature",
        URL_SAFE_NO_PAD.encode(br#"{"role":"anon"}"#)
    );
    let service = format!(
        "header.{}.signature",
        URL_SAFE_NO_PAD.encode(br#"{"role":"service_role"}"#)
    );
    assert!(AuthConfig::new("https://auth.example", &anon, "https://api.example").is_ok());
    for key in [
        "sb_secret_private",
        "sb_publishable_",
        "anything",
        "sb_publishable_mock\r\nheader",
        &service,
    ] {
        assert!(AuthConfig::new("https://auth.example", key, "https://api.example").is_err());
    }
    for value in [
        "http://auth.example",
        "https://auth.example/path",
        "https://auth.example?key=secret",
        "https://user:password@auth.example",
        "https://AUTH.example",
        "https://auth.example/",
        "https://auth.example:443",
        " https://auth.example",
    ] {
        assert!(AuthConfig::new(value, "sb_publishable_mock", "https://api.example").is_err());
        assert!(AuthConfig::new("https://auth.example", "sb_publishable_mock", value).is_err());
    }
    assert!(AuthConfig::new(
        "https://auth.example",
        "sb_publishable_mock",
        "https://auth.example"
    )
    .is_err());
}

#[test]
fn native_auth_mock_retry_preserves_identity_id_and_server_role_demotion_relocks_removal() {
    let mut f = Fixture::new();
    let mut mock = f.mock();
    let verified = f.login(&mut mock).ok().unwrap();
    f.store.enroll_native(&verified).unwrap();
    f.store.enroll_native(&verified).unwrap();
    assert_eq!(f.count("users"), 1);
    let version: i64 = f
        .store
        .conn
        .query_row("SELECT version FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1);
    let mut values = f.responses();
    values[2]["staff"]["role"] = json!("Reception");
    let mut mock = Mock {
        replies: values.into_iter().map(|v| Ok(reply(v))).collect(),
        requests: vec![],
    };
    let reception = f.login(&mut mock).ok().unwrap();
    f.store.enroll_native(&reception).unwrap();
    assert_eq!(f.count("user_roles"), 1);
    assert_eq!(
        f.store.snapshot().unwrap()["removalAuthorization"]["allowed"],
        false
    );
    let admin_count: i64 = f.store.conn.query_row("SELECT count(*) FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE r.name='Administrator'", [], |r| r.get(0)).unwrap();
    assert_eq!(admin_count, 0);
}

#[test]
fn native_auth_mock_readonly_device_locks_removal_logout_and_expiry_do_not_erase_history() {
    let mut f = Fixture::new();
    let mut values = f.responses();
    values[2]["device"]["canWrite"] = json!(false);
    let mut mock = Mock {
        replies: values.into_iter().map(|v| Ok(reply(v))).collect(),
        requests: vec![],
    };
    let readonly = f.login(&mut mock).ok().unwrap();
    f.store.enroll_native(&readonly).unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["removalAuthorization"]["allowed"],
        false
    );
    let mut mock = f.mock();
    let mut verified = f.login(&mut mock).ok().unwrap();
    f.store.enroll_native(&verified).unwrap();
    assert_eq!(
        f.store.snapshot().unwrap()["removalAuthorization"]["allowed"],
        true
    );
    f.store.lock_native_session();
    assert_eq!(
        f.store.snapshot().unwrap()["removalAuthorization"]["allowed"],
        false
    );
    verified.expires_at = Utc::now() - chrono::Duration::seconds(1);
    assert!(f.store.enroll_native(&verified).is_err());
    assert_eq!(f.count("users"), 1);
    assert_eq!(f.count("audit"), 2);
}

#[test]
fn native_auth_mock_scope_failure_and_sql_failure_roll_back_roles_binding_and_session() {
    for mismatch in [true, false] {
        let mut f = Fixture::new();
        let mut mock = f.mock();
        let verified = f.login(&mut mock).ok().unwrap();
        if mismatch {
            let scope = SyncScope::new("https://other.example", &f.gym, &f.device).unwrap();
            f.store
                .conn
                .execute(
                    "INSERT INTO metadata VALUES('member_sync_scope',?1)",
                    [serde_json::to_string(&scope).unwrap()],
                )
                .unwrap();
        } else {
            f.store.conn.execute_batch("CREATE TRIGGER fail_login BEFORE INSERT ON audit BEGIN SELECT RAISE(ABORT,'Injected audit failure'); END;").unwrap();
        }
        assert!(f.store.enroll_native(&verified).is_err());
        assert_eq!(f.count("users"), 0);
        assert_eq!(f.count("roles"), 0);
        assert_eq!(f.count("user_roles"), 0);
        assert!(f.store.removal_session.is_none());
        if !mismatch {
            assert_eq!(
                f.store.snapshot().unwrap()["memberSync"]["scope"],
                Value::Null
            );
        }
    }
}

#[test]
fn native_auth_mock_rejects_restored_or_changed_device_without_creating_identity() {
    for restored in [true, false] {
        let mut f = Fixture::new();
        let mut mock = f.mock();
        let verified = f.login(&mut mock).ok().unwrap();
        if restored {
            f.store
                .conn
                .execute(
                    "INSERT INTO metadata VALUES('restore_requires_reconciliation','1')",
                    [],
                )
                .unwrap();
        } else {
            f.store
                .conn
                .execute("UPDATE metadata SET value=?1 WHERE key='device_id'", [id()])
                .unwrap();
        }
        assert!(f.store.enroll_native(&verified).is_err());
        assert_eq!(f.count("users"), 0);
        assert!(f.store.removal_session.is_none());
    }
}

#[test]
fn native_auth_mock_existing_actor_ids_are_preserved_and_email_collisions_refuse_remapping() {
    let mut f = Fixture::new();
    f.store
        .conn
        .execute(
            "INSERT INTO users VALUES('historic-actor',?1,'old@example.test','Historic actor',0,1)",
            [&f.subject],
        )
        .unwrap();
    let mut mock = f.mock();
    let verified = f.login(&mut mock).ok().unwrap();
    f.store.enroll_native(&verified).unwrap();
    assert_eq!(f.count("users"), 1);
    assert_eq!(
        f.store.removal_session.as_ref().unwrap().user_id,
        "historic-actor"
    );
    f.store
        .conn
        .execute("UPDATE users SET email='old@example.test'", [])
        .unwrap();
    f.store
        .conn
        .execute(
            "INSERT INTO users VALUES(?1,?1,'admin@example.test','Other account',1,1)",
            [id()],
        )
        .unwrap();
    let audit = f.count("audit");
    assert!(f.store.enroll_native(&verified).is_err());
    assert!(f.store.removal_session.is_none());
    assert_eq!(f.count("users"), 2);
    assert_eq!(f.count("audit"), audit);
}

#[test]
fn native_auth_mock_readonly_device_cannot_upload_or_acknowledge_pending_members() {
    let mut f = Fixture::new();
    let mut values = f.responses();
    values[2]["device"]["canWrite"] = json!(false);
    let mut mock = Mock {
        replies: values.into_iter().map(|v| Ok(reply(v))).collect(),
        requests: vec![],
    };
    let verified = f.login(&mut mock).ok().unwrap();
    f.store.enroll_native(&verified).unwrap();
    f.store
        .save_member(MemberInput {
            id: None,
            version: None,
            name: "Pending member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "".into(),
        })
        .unwrap();
    let pending = f.store.snapshot().unwrap()["pending"].clone();
    let mut transport = verified
        .member_transport(Mock {
            replies: VecDeque::new(),
            requests: vec![],
        })
        .unwrap();
    assert!(f
        .store
        .run_member_sync(
            &mut transport,
            Utc::now(),
            crate::member_worker::Limits::default()
        )
        .is_err());
    assert_eq!(f.store.snapshot().unwrap()["pending"], pending);
    assert_eq!(f.store.snapshot().unwrap()["memberSync"]["acknowledged"], 0);
    assert!(f.store.removal_session.is_some());
}
