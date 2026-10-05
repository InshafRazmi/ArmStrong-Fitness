// Member API protocol adapter. desktop_auth constructs it from verified native
// enrollment with native_https. Its own unit tests inject HTTP mocks; actual
// desktop/API acceptance and the production sync scheduler remain pending.
#![allow(dead_code)]
use super::member_sync::{Page, Receipt};
use super::member_worker::{MemberTransport, Rejection, RemoteFailure, SyncScope};
use super::*;

pub(crate) const RESPONSE_LIMIT: usize = 256 * 1024;
const REQUEST_LIMIT: usize = 32 * 1024;
const MAX_CURSOR: i64 = 9_007_199_254_740_991;

// Deliberately no Debug/Serialize: requests contain credentials and member data.
pub(crate) struct Request {
    pub(crate) method: &'static str,
    pub(crate) url: String,
    pub(crate) headers: Vec<(&'static str, String)>,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) response_limit: usize,
}
pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) content_type: Option<String>,
    pub(crate) retry_after: Option<String>,
    pub(crate) body: Vec<u8>,
}
#[derive(Debug)]
pub(crate) enum ExchangeError {
    Unavailable,
    InvalidResponse,
}

// A native implementation MUST verify CA/hostname, reject redirects,
// enforce deadlines and response_limit while streaming, and redact errors.
// It must use the exact configured HTTPS origin and never log this request.
// native_https implements this; a URL parser is not TLS verification.
pub(crate) trait HttpsExchange {
    fn send(&mut self, request: Request) -> std::result::Result<Response, ExchangeError>;
}

// Private native memory only, no serde/Debug and no env or SQLite storage.
// The caller must obtain these from verified native login/enrollment and the OS
// credential store. This constructor checks shape/expiry, not live identity.
pub(crate) struct MemberApi<C> {
    client: C,
    scope: SyncScope,
    subject: String,
    token: String,
    secret: String,
    expires_at: DateTime<Utc>,
}
impl<C: HttpsExchange> MemberApi<C> {
    pub(crate) fn new(
        client: C,
        scope: SyncScope,
        subject: String,
        token: String,
        secret: String,
        expires_at: DateTime<Utc>,
    ) -> Result<Self> {
        let checked = SyncScope::new(&scope.server_origin, &scope.gym_id, &scope.device_id)?;
        if checked != scope {
            return Err("Member API requires the canonical enrolled HTTPS scope".into());
        }
        let parsed = Uuid::parse_str(&subject).map_err(|_| "Invalid native account identity")?;
        if parsed.to_string() != subject
            || !(1..=5).contains(&parsed.get_version_num())
            || parsed.get_variant() != uuid::Variant::RFC4122
        {
            return Err("Invalid native account identity".into());
        }
        if token.is_empty()
            || token.len() > 8192
            || !token
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._~-".contains(&c))
            || secret.len() != 64
            || !secret
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err("Invalid native session credentials; values withheld".into());
        }
        if expires_at <= Utc::now() {
            return Err("Native session expired; sign in again".into());
        }
        Ok(Self {
            client,
            scope,
            subject,
            token,
            secret,
            expires_at,
        })
    }
    fn exchange(
        &mut self,
        method: &'static str,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> std::result::Result<Response, RemoteFailure> {
        if self.expires_at <= Utc::now() {
            return Err(RemoteFailure::Authentication);
        }
        let business = path.starts_with("/v2/business/");
        let request_limit = if business {
            super::business_sync::REQUEST_LIMIT
        } else {
            REQUEST_LIMIT
        };
        if body
            .as_ref()
            .is_some_and(|bytes| bytes.len() > request_limit)
        {
            return Err(RemoteFailure::InvalidResponse);
        }
        let request = Request {
            method,
            url: format!("{}{path}", self.scope.server_origin),
            headers: vec![
                ("authorization", format!("Bearer {}", self.token)),
                ("x-gym-id", self.scope.gym_id.clone()),
                ("x-device-id", self.scope.device_id.clone()),
                ("x-device-secret", self.secret.clone()),
                ("accept", "application/json".into()),
                ("content-type", "application/json".into()),
            ],
            body,
            response_limit: if business {
                super::business_sync::REQUEST_LIMIT + 8192
            } else {
                RESPONSE_LIMIT
            },
        };
        match self.client.send(request) {
            Ok(response) => Ok(response),
            Err(ExchangeError::Unavailable) => Err(RemoteFailure::Transient {
                retry_after_seconds: None,
            }),
            Err(ExchangeError::InvalidResponse) => Err(RemoteFailure::InvalidResponse),
        }
    }
}

fn business_reply(response: Response) -> std::result::Result<Value, RemoteFailure> {
    match response.status {
        401 => return Err(RemoteFailure::Authentication),
        403 => return Err(RemoteFailure::Authorization),
        429 | 500..=599 => {
            return Err(RemoteFailure::Transient {
                retry_after_seconds: retry_delay(response.retry_after.as_deref()),
            })
        }
        200 | 400 | 409 => (),
        _ => return Err(RemoteFailure::InvalidResponse),
    }
    if !response
        .content_type
        .as_deref()
        .and_then(|s| s.split(';').next())
        .is_some_and(|s| s.trim().eq_ignore_ascii_case("application/json"))
        || response.body.len() > super::business_sync::REQUEST_LIMIT + 8192
    {
        return Err(RemoteFailure::InvalidResponse);
    }
    let value: Value =
        serde_json::from_slice(&response.body).map_err(|_| RemoteFailure::InvalidResponse)?;
    if response.status == 200 {
        return Ok(value);
    }
    let code = value["error"]
        .as_str()
        .filter(|s| {
            !s.is_empty() && s.len() <= 80 && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        })
        .ok_or(RemoteFailure::InvalidResponse)?;
    Err(RemoteFailure::BusinessConflict { code: code.into() })
}
impl<C: HttpsExchange> super::member_worker::BusinessTransport for MemberApi<C> {
    fn push_business(&mut self, request: &Value) -> std::result::Result<Value, RemoteFailure> {
        if request["deviceId"] != self.scope.device_id {
            return Err(RemoteFailure::Authorization);
        }
        let response = self.exchange(
            "POST",
            "/v2/business/push",
            Some(serde_json::to_vec(request).map_err(|_| RemoteFailure::InvalidResponse)?),
        )?;
        business_reply(response)
    }
    fn pull_business(&mut self, after: i64) -> std::result::Result<Value, RemoteFailure> {
        if !(0..=MAX_CURSOR).contains(&after) {
            return Err(RemoteFailure::InvalidResponse);
        }
        let response =
            self.exchange("GET", &format!("/v2/business/changes?after={after}"), None)?;
        business_reply(response)
    }
}
fn retry_delay(value: Option<&str>) -> Option<u32> {
    let value = value?;
    if value.is_empty() || value.len() > 10 || !value.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    value.parse::<u32>().ok().map(|seconds| seconds.min(3600))
}
fn json_reply(response: Response, push: bool) -> std::result::Result<Vec<u8>, RemoteFailure> {
    // Access denial and transient statuses never parse/echo provider payloads.
    match response.status {
        401 => return Err(RemoteFailure::Authentication),
        403 => return Err(RemoteFailure::Authorization),
        429 | 500..=599 => {
            return Err(RemoteFailure::Transient {
                retry_after_seconds: retry_delay(response.retry_after.as_deref()),
            })
        }
        200 | 400 | 409 => (),
        _ => return Err(RemoteFailure::InvalidResponse),
    }
    let media = response
        .content_type
        .as_deref()
        .and_then(|value| value.split(';').next());
    if !media.is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
        || response.body.len() > RESPONSE_LIMIT
    {
        return Err(RemoteFailure::InvalidResponse);
    }
    if response.status == 200 {
        return Ok(response.body);
    }
    if !push {
        // Cursor/setup errors must preserve pending operations and cursor; they
        // are not member-operation conflicts and cannot acknowledge anything.
        return Err(RemoteFailure::InvalidResponse);
    }
    let payload: Value =
        serde_json::from_slice(&response.body).map_err(|_| RemoteFailure::InvalidResponse)?;
    let kind = match (response.status, payload["error"].as_str()) {
        (409, Some("revision_conflict")) => Rejection::Revision,
        (409, Some("card_or_member_conflict")) => Rejection::Card,
        (409, Some("member_archived")) => Rejection::Archived,
        (409, Some("operation_id_reused")) => Rejection::OperationReuse,
        (409, Some("joined_date_is_immutable")) => Rejection::InvalidOperation,
        (
            400,
            Some(
                "invalid_request"
                | "invalid_fields"
                | "invalid_protocol"
                | "invalid_operation"
                | "invalid_uuid"
                | "invalid_text"
                | "invalid_email"
                | "invalid_card"
                | "invalid_date"
                | "archive_has_no_member_payload",
            ),
        ) => Rejection::InvalidOperation,
        _ => return Err(RemoteFailure::InvalidResponse),
    };
    let member = payload.get("details").cloned().unwrap_or(Value::Null);
    if !member.is_null() && !member.is_object() {
        return Err(RemoteFailure::InvalidResponse);
    }
    Err(RemoteFailure::Rejected { kind, member })
}
impl<C: HttpsExchange> MemberTransport for MemberApi<C> {
    fn scope(&self) -> &SyncScope {
        &self.scope
    }
    fn subject(&self) -> &str {
        &self.subject
    }
    fn push(&mut self, operation: &Value) -> std::result::Result<Receipt, RemoteFailure> {
        if operation["deviceId"].as_str() != Some(self.scope.device_id.as_str()) {
            return Err(RemoteFailure::Authorization);
        }
        let body = serde_json::to_vec(operation).map_err(|_| RemoteFailure::InvalidResponse)?;
        let response = self.exchange("POST", "/v1/members/push", Some(body))?;
        serde_json::from_slice(&json_reply(response, true)?)
            .map_err(|_| RemoteFailure::InvalidResponse)
    }
    fn pull(&mut self, after: i64) -> std::result::Result<Page, RemoteFailure> {
        if !(0..=MAX_CURSOR).contains(&after) {
            return Err(RemoteFailure::InvalidResponse);
        }
        let response = self.exchange("GET", &format!("/v1/members/changes?after={after}"), None)?;
        serde_json::from_slice(&json_reply(response, false)?)
            .map_err(|_| RemoteFailure::InvalidResponse)
    }
}

#[cfg(test)]
#[path = "member_http_tests.rs"]
mod tests;
