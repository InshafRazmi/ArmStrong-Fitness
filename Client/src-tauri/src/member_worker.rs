// Internal orchestration for the native HTTPS transport. No production scheduler
// or sync IPC is enabled before real API acceptance.
// Run with a dedicated Store/SQLite connection; do not hold the UI mutex over I/O.
#![allow(dead_code)]
use super::member_sync::{Page, Receipt};
use super::*;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SyncScope {
    pub(crate) server_origin: String,
    pub(crate) gym_id: String,
    pub(crate) device_id: String,
}
impl SyncScope {
    pub(crate) fn new(origin: &str, gym: &str, device: &str) -> Result<Self> {
        if origin.chars().any(char::is_whitespace) {
            return Err("Sync server must be a canonical HTTPS origin".into());
        }
        let url = Url::parse(origin).map_err(|_| "Invalid HTTPS server origin")?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err("Sync server must be an HTTPS origin without credentials or paths".into());
        }
        for value in [gym, device] {
            let parsed = Uuid::parse_str(value).map_err(|_| "Invalid scope UUID")?;
            if parsed.to_string() != value
                || !(1..=5).contains(&parsed.get_version_num())
                || parsed.get_variant() != uuid::Variant::RFC4122
            {
                return Err("Invalid canonical scope UUID".into());
            }
        }
        Ok(Self {
            server_origin: url.origin().ascii_serialization(),
            gym_id: gym.into(),
            device_id: device.into(),
        })
    }
    fn checked(&self) -> Result<Self> {
        let checked = Self::new(&self.server_origin, &self.gym_id, &self.device_id)?;
        if checked != *self {
            return Err("Noncanonical persisted sync scope".into());
        }
        Ok(checked)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Runtime {
    failures: u32,
    retry_on: Option<DateTime<Utc>>,
    last_error: Option<String>,
    last_success_on: Option<DateTime<Utc>>,
}
fn scope(conn: &Connection) -> Result<Option<SyncScope>> {
    let saved: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='member_sync_scope'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    saved
        .map(|value| {
            let scope: SyncScope = serde_json::from_str(&value)
                .map_err(|_| "Invalid saved sync scope; keep database for recovery")?;
            scope.checked()
        })
        .transpose()
}
fn runtime(conn: &Connection) -> Result<Runtime> {
    let saved: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='member_sync_runtime'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    match saved {
        Some(saved) => serde_json::from_str(&saved)
            .map_err(|_| "Invalid saved sync retry state; keep database for recovery".into()),
        None => Ok(Runtime::default()),
    }
}
fn save_runtime(store: &mut Store, state: &Runtime) -> Result<()> {
    put_runtime(&store.conn, state)
}
fn put_runtime(conn: &Connection, state: &Runtime) -> Result<()> {
    let value = serde_json::to_string(state).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO metadata VALUES('member_sync_runtime',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [value]).map_err(db_error)?;
    Ok(())
}
fn subject(store: &Store) -> Result<String> {
    subject_on(&store.conn, store.removal_session.as_ref())
}
fn subject_on(conn: &Connection, session: Option<&super::removal::Session>) -> Result<String> {
    let session = session.ok_or("Verified native staff sign-in is required for sync")?;
    if session.expires_at <= Utc::now() {
        return Err("Staff session expired; sign in again".into());
    }
    let subject: Option<String> = conn.query_row("SELECT subject FROM users u WHERE id=?1 AND active=1 AND EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=u.id AND r.name IN ('Administrator','Reception'))", [&session.user_id], |r| r.get(0)).optional().map_err(db_error)?;
    let subject = subject.ok_or("An active enrolled staff role is required for sync")?;
    let parsed = Uuid::parse_str(&subject)
        .map_err(|_| "Staff account is not mapped to a verified Supabase subject")?;
    if parsed.to_string() != subject
        || !(1..=5).contains(&parsed.get_version_num())
        || parsed.get_variant() != uuid::Variant::RFC4122
    {
        return Err("Staff account is not mapped to a canonical Supabase subject".into());
    }
    Ok(subject)
}
pub(super) fn authorize_reply(
    conn: &Connection,
    session: Option<&super::removal::Session>,
    expected_subject: &str,
) -> Result<()> {
    if subject_on(conn, session)? != expected_subject {
        return Err("Server reply identity does not match the current verified session".into());
    }
    Ok(())
}
fn restored(conn: &Connection) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='restore_requires_reconciliation')",
        [],
        |r| r.get(0),
    )
    .map_err(db_error)
}
// Conflict review uses only the saved scope and the current native session.
// Caller-controlled server, gym, device or subject values cannot grant access.
pub(super) fn review_scope(
    conn: &Connection,
    session: Option<&super::removal::Session>,
) -> Result<Value> {
    if restored(conn)? {
        return Err(
            "Restored database requires server reconciliation before conflict review".into(),
        );
    }
    let saved = scope(conn)?
        .ok_or("Verified server/gym/device enrollment is required for conflict review")?;
    let device: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if device != saved.device_id {
        return Err("Local device identity changed; reconciliation required".into());
    }
    let subject = subject_on(conn, session)?;
    Ok(json!({"scope":saved,"subject":subject}))
}
fn authorized(store: &Store, transport: &impl MemberTransport) -> Result<()> {
    if restored(&store.conn)? {
        return Err("Restored database requires server reconciliation before sync".into());
    }
    let saved =
        scope(&store.conn)?.ok_or("Verified server/gym/device enrollment is required for sync")?;
    if &saved != transport.scope() {
        return Err("Transport does not match the enrolled server/gym/device".into());
    }
    let device: String = store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if device != saved.device_id {
        return Err("Local device identity changed; reconciliation required".into());
    }
    if subject(store)? != transport.subject() {
        return Err("Transport identity does not match the verified staff session".into());
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) enum Rejection {
    Revision,
    Card,
    Archived,
    OperationReuse,
    InvalidOperation,
}
impl Rejection {
    fn reason(&self) -> &'static str {
        match self {
            Self::Revision => "Server member revision conflicts with this operation",
            Self::Card => "Server card assignment conflicts with this member",
            Self::Archived => "Server member is already archived",
            Self::OperationReuse => "Server operation identity conflicts with this request",
            Self::InvalidOperation => "Server rejected the member operation",
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum RemoteFailure {
    Transient { retry_after_seconds: Option<u32> },
    Authentication,
    Authorization,
    Rejected { kind: Rejection, member: Value },
    InvalidResponse,
}
// Implementations must verify HTTPS, scope and access-token identity. They must
// never return synthetic success or trust caller-supplied webview identity fields.
pub(crate) trait MemberTransport {
    fn scope(&self) -> &SyncScope;
    fn subject(&self) -> &str;
    fn push(&mut self, request: &Value) -> std::result::Result<Receipt, RemoteFailure>;
    fn pull(&mut self, after: i64) -> std::result::Result<Page, RemoteFailure>;
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Run {
    Complete { pushed: usize, pages: usize },
    Yielded { pushed: usize, pages: usize },
    Deferred { retry_on: DateTime<Utc> },
    Failed,
    Blocked,
}
#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub(crate) pushes: usize,
    pub(crate) pages: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            pushes: 100,
            pages: 100,
        }
    }
}
fn failed(
    store: &mut Store,
    now: DateTime<Utc>,
    failure: &RemoteFailure,
    operation: Option<&str>,
) -> Result<Run> {
    if matches!(
        failure,
        RemoteFailure::Authentication | RemoteFailure::Authorization
    ) {
        store.removal_session = None;
    }
    let tx = store
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    if restored(&tx)? {
        return Err("Restored database requires reconciliation before recording sync state".into());
    }
    let mut state = runtime(&tx)?;
    state.failures = state.failures.saturating_add(1).min(16);
    if matches!(
        failure,
        RemoteFailure::Authentication | RemoteFailure::Authorization
    ) {
        state.retry_on = None;
        state.last_error =
            Some("Server denied staff/device access; verified sign-in is required".into());
    } else {
        let requested = match failure {
            RemoteFailure::Transient {
                retry_after_seconds,
            } => retry_after_seconds.unwrap_or(0).min(3600),
            _ => 0,
        };
        let delay = (1u32 << state.failures.min(8)).min(300).max(requested);
        state.retry_on = Some(now + chrono::Duration::seconds(i64::from(delay)));
        state.last_error = Some(
            if matches!(failure, RemoteFailure::InvalidResponse) {
                "Invalid server reply; local operations are retained"
            } else {
                "Server request failed; local operations are retained"
            }
            .into(),
        );
    }
    if let Some(operation) = operation {
        let changed=tx.execute("UPDATE member_deliveries SET last_error=?2 WHERE operation_id=?1 AND state='pending'", params![operation,state.last_error]).map_err(db_error)?;
        if changed != 1 {
            return Err(
                "Member delivery changed during the failed request; refresh sync state".into(),
            );
        }
    }
    put_runtime(&tx, &state)?;
    tx.commit().map_err(db_error)?;
    Ok(Run::Failed)
}
impl Store {
    // Called only after verified native enrollment, never from caller identity IDs.
    pub(crate) fn bind_member_scope(&mut self, binding: SyncScope) -> Result<()> {
        let binding = binding.checked()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        subject_on(&tx, self.removal_session.as_ref())?;
        bind_scope_on(&tx, &binding)?;
        tx.commit().map_err(db_error)
    }
    pub(crate) fn run_member_sync(
        &mut self,
        transport: &mut impl MemberTransport,
        now: DateTime<Utc>,
        limits: Limits,
    ) -> Result<Run> {
        authorized(self, transport)?;
        if limits.pushes == 0 || limits.pages == 0 || limits.pushes > 1000 || limits.pages > 1000 {
            return Err("Invalid bounded sync run limits".into());
        }
        let state = runtime(&self.conn)?;
        if let Some(retry_on) = state.retry_on {
            if now < retry_on {
                return Ok(Run::Deferred { retry_on });
            }
        }
        let mut pushed = 0;
        let mut processed = 0;
        // No database transaction is held during transport calls. Preserve edits
        // made by other local connections while a request is in flight.
        while processed < limits.pushes {
            authorized(self, transport)?;
            let Some(request) = self.prepare_member_push()? else {
                break;
            };
            if !self
                .removal_session
                .as_ref()
                .is_some_and(|session| session.can_write)
            {
                return Err(
                    "This enrolled device is read-only; pending member changes are retained".into(),
                );
            }
            if request["deviceId"].as_str() != Some(transport.scope().device_id.as_str()) {
                return Err(
                    "Queued operation belongs to another device; reconciliation required".into(),
                );
            }
            if request["action"] == "archive" {
                if super::removal::authorization(&self.conn, self.removal_session.as_ref())
                    ["allowed"]
                    != true
                {
                    return Err(
                        "An active Administrator session is required to transmit an archive".into(),
                    );
                }
                let origin: String = self.conn.query_row("SELECT u.subject FROM outbox o JOIN users u ON u.id=json_extract(o.payload_json,'$.actorUserId') WHERE o.id=?1", [request["operationId"].as_str().ok_or("Invalid operation")?], |r|r.get(0)).map_err(db_error)?;
                if origin != transport.subject() {
                    return Err("Sign in as the original enrolled archive actor before transmitting this operation".into());
                }
            }
            match transport.push(&request) {
                Ok(receipt) => {
                    authorized(self, transport)?;
                    if self
                        .acknowledge_member_for_worker(receipt, transport.subject())
                        .is_err()
                    {
                        return failed(
                            self,
                            now,
                            &RemoteFailure::InvalidResponse,
                            request["operationId"].as_str(),
                        );
                    }
                    pushed += 1;
                }
                Err(RemoteFailure::Rejected { kind, member }) => {
                    authorized(self, transport)?;
                    self.reject_member_push(
                        request["operationId"].as_str().ok_or("Invalid operation")?,
                        kind.reason(),
                        &member,
                        true,
                    )?;
                }
                Err(failure) => {
                    return failed(self, now, &failure, request["operationId"].as_str());
                }
            }
            processed += 1;
        }
        if processed == limits.pushes {
            return Ok(Run::Yielded { pushed, pages: 0 });
        }
        let pending: bool = self.conn.query_row("SELECT EXISTS(SELECT 1 FROM outbox o WHERE o.entity='member' AND NOT EXISTS(SELECT 1 FROM member_deliveries d WHERE d.operation_id=o.id AND d.state='acknowledged') AND NOT EXISTS(SELECT 1 FROM member_resolved_operations r WHERE r.operation_id=o.id))", [], |r|r.get(0)).map_err(db_error)?;
        if pending {
            let mut state = runtime(&self.conn)?;
            state.last_error = Some(
                "Pending member changes require reconciliation before pull can continue".into(),
            );
            save_runtime(self, &state)?;
            return Ok(Run::Blocked);
        }
        for pages in 1..=limits.pages {
            authorized(self, transport)?;
            let cursor: i64 = self
                .conn
                .query_row(
                    "SELECT sequence FROM member_sync_cursor WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .map_err(db_error)?;
            match transport.pull(cursor) {
                Ok(page) => {
                    authorized(self, transport)?;
                    let encoded = serde_json::to_value(&page).map_err(|e| e.to_string())?;
                    let more = encoded["hasMore"].as_bool().ok_or("Invalid server page")?;
                    if let Err(error) =
                        self.apply_member_page_after_pushes(page, transport.subject())
                    {
                        if error == super::member_sync::PENDING_DURING_PULL {
                            return Ok(Run::Yielded {
                                pushed,
                                pages: pages - 1,
                            });
                        }
                        return failed(self, now, &RemoteFailure::InvalidResponse, None);
                    }
                    if !more {
                        let mut state = runtime(&self.conn)?;
                        state.failures = 0;
                        state.retry_on = None;
                        state.last_error = None;
                        state.last_success_on = Some(now);
                        save_runtime(self, &state)?;
                        return Ok(Run::Complete { pushed, pages });
                    }
                }
                Err(failure) => return failed(self, now, &failure, None),
            }
        }
        Ok(Run::Yielded {
            pushed,
            pages: limits.pages,
        })
    }
}

// Shared by the verified native enrollment transaction. No transaction/commit
// here: identity/roles and scope must either all save or all roll back together.
pub(super) fn bind_scope_on(conn: &Connection, binding: &SyncScope) -> Result<()> {
    binding.checked()?;
    if restored(conn)? {
        return Err("Restored database requires reconciliation before enrollment".into());
    }
    let device: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if device != binding.device_id {
        return Err("Enrollment must register this existing local device".into());
    }
    if let Some(saved) = scope(conn)? {
        if saved != *binding {
            return Err(
                "Database is already bound to another server/gym/device; reconciliation required"
                    .into(),
            );
        }
    } else {
        let unscoped: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM member_deliveries) OR EXISTS(SELECT 1 FROM member_remote_heads) OR EXISTS(SELECT 1 FROM member_sync_conflicts) OR EXISTS(SELECT 1 FROM member_sync_cursor WHERE sequence<>0)", [], |r| r.get(0)).map_err(db_error)?;
        if unscoped {
            return Err(
                "Existing remote state has no verified scope; reconciliation required".into(),
            );
        }
        conn.execute(
            "INSERT INTO metadata VALUES('member_sync_scope',?1)",
            [serde_json::to_string(binding).map_err(|e| e.to_string())?],
        )
        .map_err(db_error)?;
    }
    Ok(())
}
pub(super) fn append_status(conn: &Connection, snapshot: &mut Value) -> Result<()> {
    let state = runtime(conn)?;
    snapshot["memberSync"]["scope"] =
        serde_json::to_value(scope(conn)?).map_err(|e| e.to_string())?;
    snapshot["memberSync"]["retryOn"] = json!(state.retry_on);
    snapshot["memberSync"]["lastError"] = json!(state.last_error);
    snapshot["memberSync"]["lastSuccessOn"] = json!(state.last_success_on);
    Ok(())
}
