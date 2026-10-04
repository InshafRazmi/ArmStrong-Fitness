// Real adapter/SQLite with an HTTP mock. No TLS or live backend evidence.
use super::*;
use crate::member_worker::{Limits, Run};
use std::collections::VecDeque;

struct HttpMock {
    replies: VecDeque<std::result::Result<Response, ExchangeError>>,
    requests: Vec<Request>,
}
impl HttpsExchange for HttpMock {
    fn send(&mut self, request: Request) -> std::result::Result<Response, ExchangeError> {
        self.requests.push(request);
        self.replies.pop_front().expect("Unexpected HTTP request")
    }
}
fn response(status: u16, body: Value) -> Response {
    Response {
        status,
        content_type: Some("application/json; charset=utf-8".into()),
        retry_after: None,
        body: serde_json::to_vec(&body).unwrap(),
    }
}
fn api(replies: Vec<std::result::Result<Response, ExchangeError>>) -> MemberApi<HttpMock> {
    MemberApi::new(
        HttpMock {
            replies: replies.into(),
            requests: vec![],
        },
        SyncScope::new("https://api.example", &id(), &id()).unwrap(),
        id(),
        "mock-access-token".into(),
        "a".repeat(64),
        Utc::now() + chrono::Duration::minutes(10),
    )
    .unwrap()
}
fn operation(api: &MemberApi<HttpMock>) -> Value {
    json!({"protocolVersion":1,"operationId":id(),"deviceId":api.scope.device_id,"memberId":id(),"action":"create","expectedRevision":0,"member":{"name":"Test member","phone":"0771234567","email":"","nfcId":null,"joinedOn":"2026-10-04"}})
}
fn receipt(request: &Value) -> Value {
    let mut member = request["member"].clone();
    member["id"] = request["memberId"].clone();
    member["revision"] = json!(1);
    member["archivedAt"] = Value::Null;
    member["archivedBy"] = Value::Null;
    json!({"protocolVersion":1,"operationId":request["operationId"],"memberId":request["memberId"],"revision":1,"sequence":1,"member":member})
}
fn page() -> Value {
    json!({"protocolVersion":1,"after":0,"nextCursor":0,"hasMore":false,"changes":[]})
}

#[test]
fn http_mock_request_uses_enrolled_scope_and_frozen_retry_bytes() {
    let mut client = api(vec![]);
    let op = operation(&client);
    client.client.replies.extend([
        Ok(response(200, receipt(&op))),
        Ok(response(200, receipt(&op))),
        Ok(response(200, page())),
    ]);
    client.push(&op).ok().unwrap();
    client.push(&op).ok().unwrap();
    client.pull(0).ok().unwrap();
    let requests = &client.client.requests;
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].url, "https://api.example/v1/members/push");
    assert_eq!(requests[0].body, requests[1].body);
    assert_eq!(
        serde_json::from_slice::<Value>(requests[0].body.as_ref().unwrap()).unwrap(),
        op
    );
    assert_eq!(requests[0].response_limit, RESPONSE_LIMIT);
    assert_eq!(
        requests[0].headers,
        vec![
            ("authorization", "Bearer mock-access-token".into()),
            ("x-gym-id", client.scope.gym_id.clone()),
            ("x-device-id", client.scope.device_id.clone()),
            ("x-device-secret", "a".repeat(64)),
            ("accept", "application/json".into()),
            ("content-type", "application/json".into()),
        ]
    );
    assert_eq!(requests[2].method, "GET");
    assert_eq!(
        requests[2].url,
        "https://api.example/v1/members/changes?after=0"
    );
    assert!(requests[2].body.is_none());
}

#[test]
fn http_mock_expired_sessions_wrong_devices_and_oversized_requests_never_send() {
    let mut client = api(vec![]);
    let mut op = operation(&client);
    op["deviceId"] = json!(id());
    assert!(matches!(
        client.push(&op),
        Err(RemoteFailure::Authorization)
    ));
    op["deviceId"] = json!(client.scope.device_id);
    op["member"]["name"] = json!("x".repeat(REQUEST_LIMIT));
    assert!(matches!(
        client.push(&op),
        Err(RemoteFailure::InvalidResponse)
    ));
    assert!(matches!(
        client.pull(-1),
        Err(RemoteFailure::InvalidResponse)
    ));
    assert!(matches!(
        client.pull(MAX_CURSOR + 1),
        Err(RemoteFailure::InvalidResponse)
    ));
    client.expires_at = Utc::now() - chrono::Duration::seconds(1);
    assert!(matches!(client.pull(0), Err(RemoteFailure::Authentication)));
    assert!(client.client.requests.is_empty());
}

#[test]
fn http_mock_access_denials_and_outages_do_not_parse_provider_payloads() {
    for status in [401, 403, 429, 500, 502, 503] {
        let mut reply = response(status, Value::Null);
        reply.content_type = Some("text/html".into());
        reply.body = b"private provider error, token, SQL, credentials".to_vec();
        reply.retry_after = Some("9999".into());
        let mut client = api(vec![Ok(reply)]);
        let error = client.pull(0).err().unwrap();
        assert!(match status {
            401 => matches!(error, RemoteFailure::Authentication),
            403 => matches!(error, RemoteFailure::Authorization),
            _ => matches!(
                error,
                RemoteFailure::Transient {
                    retry_after_seconds: Some(3600)
                }
            ),
        });
    }
    for error in [ExchangeError::Unavailable, ExchangeError::InvalidResponse] {
        let transient = matches!(error, ExchangeError::Unavailable);
        let mut client = api(vec![Err(error)]);
        let result = client.pull(0);
        assert!(if transient {
            matches!(
                result,
                Err(RemoteFailure::Transient {
                    retry_after_seconds: None
                })
            )
        } else {
            matches!(result, Err(RemoteFailure::InvalidResponse))
        });
    }
}

#[test]
fn http_mock_retry_after_only_accepts_bounded_integer_seconds() {
    for (value, expected) in [
        ("0", Some(0)),
        ("120", Some(120)),
        ("3601", Some(3600)),
        ("-1", None),
        ("1.5", None),
        ("Wed, 21 Oct 2026 07:28:00 GMT", None),
        ("", None),
        ("4294967296", None),
    ] {
        assert_eq!(retry_delay(Some(value)), expected);
    }
    assert_eq!(retry_delay(None), None);
}

#[test]
fn http_mock_redirects_malformed_media_json_and_large_replies_cannot_acknowledge() {
    let mut replies = vec![];
    for status in [201, 204, 301, 302, 307, 308, 404] {
        replies.push(response(status, page()));
    }
    let mut wrong_media = response(200, page());
    wrong_media.content_type = Some("text/html".into());
    replies.push(wrong_media);
    let mut missing_media = response(200, page());
    missing_media.content_type = None;
    replies.push(missing_media);
    let mut malformed = response(200, page());
    malformed.body = b"{".to_vec();
    replies.push(malformed);
    let mut large = response(200, page());
    large.body = vec![b' '; RESPONSE_LIMIT + 1];
    replies.push(large);
    let mut unknown = page();
    unknown["callerRole"] = json!("Administrator");
    replies.push(response(200, unknown));
    for reply in replies {
        let mut client = api(vec![Ok(reply)]);
        assert!(matches!(
            client.pull(0),
            Err(RemoteFailure::InvalidResponse)
        ));
    }
}

#[test]
fn http_mock_push_rejections_are_explicit_and_pull_errors_are_not_member_conflicts() {
    for (status, code) in [
        (409, "revision_conflict"),
        (409, "card_or_member_conflict"),
        (409, "member_archived"),
        (409, "operation_id_reused"),
        (409, "joined_date_is_immutable"),
        (400, "invalid_fields"),
    ] {
        let payload = json!({"error":code,"details":{"id":"remote snapshot"}});
        let mut client = api(vec![
            Ok(response(status, payload.clone())),
            Ok(response(status, payload)),
        ]);
        let op = operation(&client);
        assert!(
            matches!(client.push(&op), Err(RemoteFailure::Rejected { member, .. }) if member["id"] == "remote snapshot")
        );
        assert!(matches!(
            client.pull(0),
            Err(RemoteFailure::InvalidResponse)
        ));
    }
    for payload in [
        json!({"error":"unknown_error"}),
        json!({"error":"revision_conflict","details":"private provider text"}),
    ] {
        let mut client = api(vec![Ok(response(409, payload))]);
        let op = operation(&client);
        assert!(matches!(
            client.push(&op),
            Err(RemoteFailure::InvalidResponse)
        ));
    }
}

#[test]
fn http_mock_credential_shapes_and_noncanonical_scope_fail_before_io() {
    let valid_scope = SyncScope::new("https://api.example", &id(), &id()).unwrap();
    for (token, secret, subject, expiry) in [
        (
            "injected\r\nheader".into(),
            "a".repeat(64),
            id(),
            Utc::now() + chrono::Duration::hours(1),
        ),
        (
            "token".into(),
            "A".repeat(64),
            id(),
            Utc::now() + chrono::Duration::hours(1),
        ),
        (
            "token".into(),
            "a".repeat(64),
            "forged-subject".into(),
            Utc::now() + chrono::Duration::hours(1),
        ),
        (
            "token".into(),
            "a".repeat(64),
            id(),
            Utc::now() - chrono::Duration::seconds(1),
        ),
    ] {
        assert!(MemberApi::new(
            HttpMock {
                replies: VecDeque::new(),
                requests: vec![]
            },
            valid_scope.clone(),
            subject,
            token,
            secret,
            expiry
        )
        .is_err());
    }
    let mut scope = valid_scope;
    scope.server_origin = "https://API.example:443/".into();
    assert!(MemberApi::new(
        HttpMock {
            replies: VecDeque::new(),
            requests: vec![]
        },
        scope,
        id(),
        "token".into(),
        "a".repeat(64),
        Utc::now() + chrono::Duration::hours(1)
    )
    .is_err());
}

#[test]
fn http_mock_adapter_with_sqlite_acknowledges_only_matching_receipt_and_pulls_after_push() {
    let path = std::env::temp_dir().join(format!("armstrong-http-{}.sqlite3", id()));
    let mut store = Store::open(&path).unwrap();
    let mut client = api(vec![]);
    let device: String = store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    client.scope.device_id = device;
    store
        .conn
        .execute(
            "INSERT INTO users VALUES(?1,?1,'mock@example.test','HTTP mock administrator',1,1)",
            [&client.subject],
        )
        .unwrap();
    store
        .conn
        .execute("INSERT INTO roles VALUES('http-admin','Administrator')", [])
        .unwrap();
    store
        .conn
        .execute(
            "INSERT INTO user_roles VALUES(?1,'http-admin')",
            [&client.subject],
        )
        .unwrap();
    store.removal_session = Some(crate::removal::Session {
        can_write: true,
        user_id: client.subject.clone(),
        expires_at: client.expires_at,
    });
    store.bind_member_scope(client.scope.clone()).unwrap();
    store
        .save_member(MemberInput {
            id: None,
            version: None,
            name: "HTTP mock member".into(),
            phone: "0771234567".into(),
            email: "".into(),
            nfc_id: "".into(),
        })
        .unwrap();
    let op = store.prepare_member_push().unwrap().unwrap();
    let mut changes = page();
    changes["nextCursor"] = json!(1);
    changes["changes"] =
        json!([{"sequence":1,"operationId":op["operationId"],"member":receipt(&op)["member"]}]);
    client
        .client
        .replies
        .extend([Ok(response(200, receipt(&op))), Ok(response(200, changes))]);
    assert_eq!(
        store
            .run_member_sync(&mut client, Utc::now(), Limits::default())
            .unwrap(),
        Run::Complete {
            pushed: 1,
            pages: 1
        }
    );
    assert_eq!(store.snapshot().unwrap()["memberSync"]["acknowledged"], 1);
    assert_eq!(client.client.requests[0].method, "POST");
    assert_eq!(client.client.requests[1].method, "GET");
    drop(store);
    std::fs::remove_file(path).unwrap();
}
