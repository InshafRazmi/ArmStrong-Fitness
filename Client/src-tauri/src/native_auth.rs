// Native Auth/enrollment contract and atomic local authorization binding.
// Private to Rust; desktop_auth now wires OS storage/HTTPS and narrow login IPC.
// Tests use HTTP mocks, never a decoded JWT or webview-supplied role grant.
#![allow(dead_code)]
use super::member_http::{HttpsExchange, MemberApi, Request, Response, RESPONSE_LIMIT};
use super::member_worker::{bind_scope_on, SyncScope};
use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use url::Url;

pub(crate) struct AuthConfig {
    pub(super) auth_origin: String,
    publishable_key: String,
    pub(super) api_origin: String,
}
fn origin(value: &str) -> Result<String> {
    let parsed = Url::parse(value).map_err(|_| "Configure a canonical HTTPS origin")?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.origin().ascii_serialization() != value
    {
        return Err("Configure a canonical HTTPS origin without credentials or paths".into());
    }
    Ok(value.into())
}
fn canonical_uuid(value: &str) -> Result<()> {
    let parsed = Uuid::parse_str(value).map_err(|_| "Invalid native identity UUID")?;
    if parsed.to_string() != value
        || !(1..=5).contains(&parsed.get_version_num())
        || parsed.get_variant() != uuid::Variant::RFC4122
    {
        return Err("Invalid native identity UUID".into());
    }
    Ok(())
}
fn public_key(key: &str) -> bool {
    if key.is_empty()
        || key.len() > 8192
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._~-".contains(&b))
    {
        return false;
    }
    if key.starts_with("sb_publishable_") {
        return key.len() > "sb_publishable_".len();
    }
    // Decode only the public key's role to reject privileged legacy keys. This
    // never verifies an identity or grants authorization from JWT claims.
    let parts: Vec<_> = key.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return false;
    }
    URL_SAFE_NO_PAD
        .decode(parts[1])
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .is_some_and(|payload| payload["role"] == "anon")
}
impl AuthConfig {
    pub(crate) fn new(auth_origin: &str, publishable_key: &str, api_origin: &str) -> Result<Self> {
        if !public_key(publishable_key) {
            return Err(
                "Desktop Auth requires a publishable or legacy anon key; values withheld".into(),
            );
        }
        let auth_origin = origin(auth_origin)?;
        let api_origin = origin(api_origin)?;
        if auth_origin == api_origin {
            return Err("Configure the Armstrong API origin separately from Supabase Auth".into());
        }
        Ok(Self {
            auth_origin,
            publishable_key: publishable_key.into(),
            api_origin,
        })
    }
}
#[derive(Deserialize)]
struct Identity {
    id: String,
    email: String,
}
#[derive(Deserialize)]
struct Tokens {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: i64,
    expires_at: Option<i64>,
    user: Identity,
}
#[derive(Clone, Copy, Deserialize)]
enum Role {
    Administrator,
    Reception,
}
impl Role {
    fn name(self) -> &'static str {
        match self {
            Self::Administrator => "Administrator",
            Self::Reception => "Reception",
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Gym {
    id: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Staff {
    id: String,
    name: String,
    role: Role,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Device {
    id: String,
    can_write: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Enrollment {
    protocol_version: i64,
    gym: Gym,
    staff: Staff,
    device: Device,
}

// Fields are private, deliberately no Deserialize/Serialize/Debug/Clone. Only a
// native Auth + online identity + authoritative API sequence can construct this.
pub(crate) struct VerifiedEnrollment {
    scope: SyncScope,
    subject: String,
    email: String,
    name: String,
    role: Role,
    can_write: bool,
    token: String,
    refresh_token: Option<String>,
    secret: String,
    expires_at: DateTime<Utc>,
}
// Rotating Auth credentials remain in native memory. They are never serialized
// into SQLite, a backup, the OS offline grant, IPC or diagnostic output.
pub(super) struct RefreshState {
    scope: SyncScope,
    subject: String,
    email: String,
    secret: String,
    refresh_token: String,
    pub(super) expires_at: DateTime<Utc>,
}
pub(super) enum RefreshFailure {
    Refused,
    Unavailable,
}
struct RenewalExchange<'a, C> {
    client: &'a mut C,
    refused: bool,
}
impl<C: HttpsExchange> HttpsExchange for RenewalExchange<'_, C> {
    fn send(
        &mut self,
        request: Request,
    ) -> std::result::Result<Response, super::member_http::ExchangeError> {
        let response = self.client.send(request)?;
        self.refused |= matches!(response.status, 400 | 401 | 403 | 422);
        Ok(response)
    }
}
fn credential(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 8192
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._~-".contains(&c))
}
fn json(response: Response) -> Result<Vec<u8>> {
    match response.status {
        200 => (),
        400 | 401 | 403 | 422 => {
            return Err(
                "Sign-in or device enrollment was refused; check approved account/device setup"
                    .into(),
            )
        }
        429 => return Err("Sign-in is rate limited; wait before trying again".into()),
        _ => return Err("Sign-in service unavailable; local permissions remain locked".into()),
    }
    if response.body.len() > RESPONSE_LIMIT
        || !response
            .content_type
            .as_deref()
            .and_then(|value| value.split(';').next())
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err("Invalid sign-in service response; values withheld".into());
    }
    Ok(response.body)
}
fn send(
    exchange: &mut impl HttpsExchange,
    method: &'static str,
    url: String,
    headers: Vec<(&'static str, String)>,
    body: Option<Value>,
) -> Result<Vec<u8>> {
    let body = body
        .map(|value| serde_json::to_vec(&value))
        .transpose()
        .map_err(|_| "Invalid sign-in request; values withheld")?;
    let reply = exchange
        .send(Request {
            method,
            url,
            headers,
            body,
            response_limit: RESPONSE_LIMIT,
        })
        .map_err(|_| "Sign-in connection unavailable; values withheld")?;
    json(reply)
}

// Runs without a database transaction/UI mutex. The native caller must obtain
// the existing local device and secret from SQLite/OS storage, not webview IDs.
pub(crate) fn login(
    exchange: &mut impl HttpsExchange,
    config: &AuthConfig,
    device: &str,
    secret: String,
    email: &str,
    password: &str,
) -> Result<VerifiedEnrollment> {
    canonical_uuid(device)?;
    let email = email.trim();
    if email.is_empty()
        || email.len() > 254
        || !email.contains('@')
        || email.chars().any(char::is_whitespace)
        || password.is_empty()
        || password.len() > 4096
        || secret.len() != 64
        || !secret
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err("Invalid account or native device credentials; values withheld".into());
    }
    let started = Utc::now();
    let bytes = send(
        exchange,
        "POST",
        format!("{}/auth/v1/token?grant_type=password", config.auth_origin),
        vec![
            ("apikey", config.publishable_key.clone()),
            ("content-type", "application/json".into()),
            ("accept", "application/json".into()),
        ],
        Some(json!({"email":email,"password":password})),
    )?;
    let tokens: Tokens =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid account response; values withheld")?;
    complete(exchange, config, device, secret, email, tokens, started)
}
fn complete(
    exchange: &mut impl HttpsExchange,
    config: &AuthConfig,
    device: &str,
    secret: String,
    email: &str,
    tokens: Tokens,
    started: DateTime<Utc>,
) -> Result<VerifiedEnrollment> {
    canonical_uuid(&tokens.user.id)?;
    if tokens.token_type != "bearer"
        || !(1..=86400).contains(&tokens.expires_in)
        || !credential(&tokens.access_token)
        || tokens
            .refresh_token
            .as_deref()
            .is_some_and(|token| !credential(token))
        || tokens.user.email.to_lowercase() != email.to_lowercase()
    {
        return Err("Invalid account response; values withheld".into());
    }
    // Count lifetime from BEFORE I/O, cap at absolute expiry when supplied, and
    // reserve 30s for clock skew. Never extend a session from an unverified JWT.
    let relative = started + chrono::Duration::seconds(tokens.expires_in);
    let absolute = tokens
        .expires_at
        .map(|value| {
            DateTime::from_timestamp(value, 0).ok_or("Invalid account expiry; values withheld")
        })
        .transpose()?;
    let expires_at =
        absolute.map_or(relative, |time| time.min(relative)) - chrono::Duration::seconds(30);
    if expires_at <= Utc::now() {
        return Err("Account session expired; sign in again".into());
    }
    let bytes = send(
        exchange,
        "GET",
        format!("{}/auth/v1/user", config.auth_origin),
        vec![
            ("apikey", config.publishable_key.clone()),
            ("authorization", format!("Bearer {}", tokens.access_token)),
            ("accept", "application/json".into()),
        ],
        None,
    )?;
    let verified: Identity = serde_json::from_slice(&bytes)
        .map_err(|_| "Invalid online identity response; values withheld")?;
    if verified.id != tokens.user.id || verified.email.to_lowercase() != email.to_lowercase() {
        return Err("Sign-in and online identity mismatch; values withheld".into());
    }
    let bytes = send(
        exchange,
        "POST",
        format!("{}/v1/enrollment", config.api_origin),
        vec![
            ("authorization", format!("Bearer {}", tokens.access_token)),
            ("content-type", "application/json".into()),
            ("accept", "application/json".into()),
        ],
        Some(json!({"protocolVersion":1,"deviceId":device,"deviceSecret":secret})),
    )?;
    let enrollment: Enrollment = serde_json::from_slice(&bytes)
        .map_err(|_| "Invalid device enrollment response; values withheld")?;
    if enrollment.protocol_version != 1
        || enrollment.staff.id != verified.id
        || enrollment.device.id != device
    {
        return Err("Enrollment identity/device mismatch; values withheld".into());
    }
    let scope = SyncScope::new(&config.api_origin, &enrollment.gym.id, device)?;
    required(&enrollment.gym.name, "Enrolled gym name", 120)?;
    let name = required(&enrollment.staff.name, "Enrolled account name", 120)?;
    if expires_at <= Utc::now() {
        return Err("Account session expired during enrollment; sign in again".into());
    }
    Ok(VerifiedEnrollment {
        scope,
        subject: verified.id,
        email: verified.email,
        name,
        role: enrollment.staff.role,
        can_write: enrollment.device.can_write,
        token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        secret,
        expires_at,
    })
}
pub(super) fn renew(
    exchange: &mut impl HttpsExchange,
    config: &AuthConfig,
    state: &RefreshState,
) -> std::result::Result<VerifiedEnrollment, RefreshFailure> {
    let mut exchange = RenewalExchange {
        client: exchange,
        refused: false,
    };
    let result = (|| {
        let started = Utc::now();
        let bytes = send(
            &mut exchange,
            "POST",
            format!(
                "{}/auth/v1/token?grant_type=refresh_token",
                config.auth_origin
            ),
            vec![
                ("apikey", config.publishable_key.clone()),
                ("content-type", "application/json".into()),
                ("accept", "application/json".into()),
            ],
            Some(json!({"refresh_token":state.refresh_token})),
        )?;
        let tokens: Tokens = serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid renewal response; values withheld")?;
        if tokens.user.id != state.subject
            || tokens
                .refresh_token
                .as_deref()
                .is_none_or(|token| !credential(token))
        {
            return Err("Renewal identity or rotating credential mismatch; values withheld".into());
        }
        let verified = complete(
            &mut exchange,
            config,
            &state.scope.device_id,
            state.secret.clone(),
            &state.email,
            tokens,
            started,
        )?;
        if verified.scope != state.scope || !verified.is_administrator() {
            exchange.refused = true;
            return Err("Renewal scope or Administrator permission changed".into());
        }
        Ok(verified)
    })();
    result.map_err(|_: String| {
        if exchange.refused {
            RefreshFailure::Refused
        } else {
            RefreshFailure::Unavailable
        }
    })
}
impl VerifiedEnrollment {
    pub(super) fn valid_until(&self) -> DateTime<Utc> {
        self.expires_at
    }
    pub(super) fn renewal(&self) -> Option<RefreshState> {
        Some(RefreshState {
            scope: self.scope.clone(),
            subject: self.subject.clone(),
            email: self.email.clone(),
            secret: self.secret.clone(),
            refresh_token: self.refresh_token.clone()?,
            expires_at: self.expires_at,
        })
    }
    pub(super) fn offline_access(
        &self,
        store: &Store,
        auth_origin: &str,
    ) -> Result<super::offline_access::Grant> {
        super::offline_access::Grant::new(
            store,
            &self.scope,
            &self.subject,
            self.can_write,
            auth_origin,
        )
    }
    pub(super) fn is_administrator(&self) -> bool {
        matches!(self.role, Role::Administrator)
    }
    pub(crate) fn member_transport<C: HttpsExchange>(self, exchange: C) -> Result<MemberApi<C>> {
        MemberApi::new(
            exchange,
            self.scope,
            self.subject,
            self.token,
            self.secret,
            self.expires_at,
        )
    }
    pub(super) fn recovery_transport<C: HttpsExchange>(&self, exchange: C) -> Result<MemberApi<C>> {
        MemberApi::new(
            exchange,
            self.scope.clone(),
            self.subject.clone(),
            self.token.clone(),
            self.secret.clone(),
            self.expires_at,
        )
    }
}
impl Store {
    // Called with a private verified result, never a deserialized IPC payload.
    // Always lock first, so failure cannot retain a previous privileged session.
    pub(crate) fn enroll_native(&mut self, verified: &VerifiedEnrollment) -> Result<()> {
        self.removal_session = None;
        if verified.expires_at <= Utc::now() {
            return Err("Account session expired; sign in again".into());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        bind_scope_on(&tx, &verified.scope)?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM users WHERE subject=?1",
                [&verified.subject],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let user_id = existing.unwrap_or_else(|| verified.subject.clone());
        let collision: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM users WHERE (email=?1 COLLATE NOCASE OR id=?2) AND subject<>?3)", params![verified.email,user_id,verified.subject], |r| r.get(0)).map_err(db_error)?;
        if collision {
            return Err("Enrolled account conflicts with an existing local identity; reconciliation required".into());
        }
        tx.execute("INSERT INTO users(id,subject,email,display_name,active,version) VALUES(?1,?2,?3,?4,1,1) ON CONFLICT(subject) DO UPDATE SET email=excluded.email,display_name=excluded.display_name,active=1,version=users.version+1 WHERE users.email<>excluded.email OR users.display_name<>excluded.display_name OR users.active<>1", params![user_id,verified.subject,verified.email,verified.name]).map_err(db_error)?;
        tx.execute(
            "INSERT INTO roles(id,name) VALUES(?1,?2) ON CONFLICT(name) DO NOTHING",
            params![id(), verified.role.name()],
        )
        .map_err(db_error)?;
        // Server authority replaces stale local roles, including Administrator
        // after demotion. Historical actor rows/IDs and audit are never deleted.
        tx.execute("DELETE FROM user_roles WHERE user_id=?1", [&user_id])
            .map_err(db_error)?;
        tx.execute("INSERT INTO user_roles(user_id,role_id) SELECT ?1,id FROM roles WHERE name=?2 COLLATE NOCASE", params![user_id,verified.role.name()]).map_err(db_error)?;
        let after = json!({"subject":verified.subject,"role":verified.role.name(),"gymId":verified.scope.gym_id,"canWrite":verified.can_write});
        tx.execute("INSERT INTO audit(id,actor,device_id,action,entity_id,before_json,after_json,created_at,entity,actor_user_id) VALUES(?1,?2,?3,'Verified staff sign-in',?4,NULL,?5,?6,'user',?4)", params![id(),format!("{} ({})",verified.name,user_id),verified.scope.device_id,user_id,after.to_string(),Utc::now().to_rfc3339()]).map_err(db_error)?;
        if verified.expires_at <= Utc::now() {
            return Err("Account session expired before local enrollment; sign in again".into());
        }
        let native_nonce = id();
        tx.execute("INSERT INTO metadata VALUES('native_session_nonce',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [&native_nonce]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        self.removal_session = Some(removal::Session {
            user_id,
            expires_at: verified.expires_at,
            can_write: verified.can_write,
            native_nonce: Some(native_nonce),
        });
        Ok(())
    }
    pub(crate) fn lock_native_session(&mut self) {
        self.removal_session = None;
    }
}

#[cfg(test)]
#[path = "native_auth_tests.rs"]
mod tests;
