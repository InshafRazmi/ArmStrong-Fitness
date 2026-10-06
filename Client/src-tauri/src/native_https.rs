// OS-provided curl is invoked directly. TLS certificate/hostname verification
// uses system trust; no redirects, proxy credentials, curlrc or env overrides.
use super::member_http::{ExchangeError, HttpsExchange, Request, Response, RESPONSE_LIMIT};
use super::*;
use std::process::Command;
use url::Url;
const HEADER_LIMIT: usize = 16 * 1024;
pub(super) struct NativeHttps {
    origins: [String; 2],
}
impl NativeHttps {
    pub(super) fn new(auth_origin: &str, api_origin: &str) -> Self {
        Self {
            origins: [auth_origin.into(), api_origin.into()],
        }
    }
}
fn quoted(value: &str) -> std::result::Result<String, ExchangeError> {
    if value.contains('\0') {
        return Err(ExchangeError::InvalidResponse);
    }
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    Ok(out)
}
fn config(client: &NativeHttps, request: &Request) -> std::result::Result<Vec<u8>, ExchangeError> {
    let url = Url::parse(&request.url).map_err(|_| ExchangeError::InvalidResponse)?;
    let business = url.path().starts_with("/v2/business/");
    let response_limit = if business {
        super::business_sync::REQUEST_LIMIT + 8192
    } else {
        RESPONSE_LIMIT
    };
    let body_limit = if business {
        super::business_sync::REQUEST_LIMIT
    } else {
        32 * 1024
    };
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !client.origins.contains(&url.origin().ascii_serialization())
        || !matches!(request.method, "GET" | "POST")
        || request.response_limit == 0
        || request.response_limit > response_limit
        || request
            .body
            .as_ref()
            .is_some_and(|body| body.len() > body_limit)
    {
        return Err(ExchangeError::InvalidResponse);
    }
    let mut config = format!("silent\nhttp1.1\ninclude\nproto = \"=https\"\nproto-redir = \"=https\"\nmax-redirs = 0\nmax-time = 15\nconnect-timeout = 5\ntlsv1.2\nnoproxy = \"*\"\nurl = {}\nrequest = {}\nmax-filesize = {}\nheader = \"Expect:\"\n",quoted(&request.url)?,quoted(request.method)?,request.response_limit);
    for (name, value) in &request.headers {
        if !matches!(
            *name,
            "apikey"
                | "authorization"
                | "content-type"
                | "accept"
                | "x-gym-id"
                | "x-device-id"
                | "x-device-secret"
        ) || value.bytes().any(|b| b < 32 || b == 127)
        {
            return Err(ExchangeError::InvalidResponse);
        }
        config.push_str(&format!(
            "header = {}\n",
            quoted(&format!("{name}: {value}"))?
        ));
    }
    if let Some(body) = &request.body {
        let body = std::str::from_utf8(body).map_err(|_| ExchangeError::InvalidResponse)?;
        // data-raw prevents a leading @ from opening a file.
        config.push_str(&format!("data-raw = {}\n", quoted(body)?));
    }
    Ok(config.into_bytes())
}
fn parse(bytes: Vec<u8>, limit: usize) -> std::result::Result<Response, ExchangeError> {
    let end = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .ok_or(ExchangeError::InvalidResponse)?;
    if end > HEADER_LIMIT || bytes.len() - end - 4 > limit {
        return Err(ExchangeError::InvalidResponse);
    }
    let headers = std::str::from_utf8(&bytes[..end]).map_err(|_| ExchangeError::InvalidResponse)?;
    let mut lines = headers.split("\r\n");
    let status = lines.next().ok_or(ExchangeError::InvalidResponse)?;
    let mut status = status.splitn(3, ' ');
    if !matches!(status.next(), Some("HTTP/1.1" | "HTTP/1.0")) {
        return Err(ExchangeError::InvalidResponse);
    }
    let number = status.next().ok_or(ExchangeError::InvalidResponse)?;
    if number.len() != 3 {
        return Err(ExchangeError::InvalidResponse);
    }
    let status: u16 = number.parse().map_err(|_| ExchangeError::InvalidResponse)?;
    if !(200..600).contains(&status) {
        return Err(ExchangeError::InvalidResponse);
    }
    let mut content_type = None;
    let mut retry_after = None;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or(ExchangeError::InvalidResponse)?;
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(ExchangeError::InvalidResponse);
        }
        if name.eq_ignore_ascii_case("content-type") {
            if content_type.is_some() {
                return Err(ExchangeError::InvalidResponse);
            }
            content_type = Some(value.trim().into());
        }
        if name.eq_ignore_ascii_case("retry-after") {
            if retry_after.is_some() {
                return Err(ExchangeError::InvalidResponse);
            }
            retry_after = Some(value.trim().into());
        }
    }
    Ok(Response {
        status,
        content_type,
        retry_after,
        body: bytes[end + 4..].into(),
    })
}
#[cfg(target_os = "linux")]
fn executable() -> std::result::Result<std::path::PathBuf, ExchangeError> {
    Ok("/usr/bin/curl".into())
}
#[cfg(windows)]
mod windows_system {
    use super::*;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
        fn GetSystemWindowsDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    fn directory(
        get: unsafe extern "system" fn(*mut u16, u32) -> u32,
    ) -> std::result::Result<std::path::PathBuf, ExchangeError> {
        let mut directory = vec![0u16; 32768];
        // SAFETY: both SDK functions write at most the supplied UTF-16 capacity.
        let size = unsafe { get(directory.as_mut_ptr(), directory.len() as u32) } as usize;
        if size == 0 || size >= directory.len() {
            return Err(ExchangeError::ClientUnavailable);
        }
        use std::os::windows::ffi::OsStringExt;
        Ok(std::path::PathBuf::from(std::ffi::OsString::from_wide(
            &directory[..size],
        )))
    }
    pub(super) fn executable() -> std::result::Result<std::path::PathBuf, ExchangeError> {
        Ok(directory(GetSystemDirectoryW)?.join("curl.exe"))
    }
    pub(super) fn environment(command: &mut Command) -> std::result::Result<(), ExchangeError> {
        // Windows loader/security providers need the system Windows root.
        // Obtain it from the OS, never an inherited/user-controlled env value.
        let root = directory(GetSystemWindowsDirectoryW)?;
        command.env("SystemRoot", &root).env("WINDIR", root);
        Ok(())
    }
}
#[cfg(windows)]
use windows_system::executable;
#[cfg(not(any(windows, target_os = "linux")))]
fn executable() -> std::result::Result<std::path::PathBuf, ExchangeError> {
    Err(ExchangeError::ClientUnavailable)
}
fn command() -> std::result::Result<Command, ExchangeError> {
    let mut command = Command::new(executable()?);
    // --disable must be first to ignore personal/global curlrc. Credentials
    // travel through stdin config, never argv. No shell interprets the text.
    command
        .args(["--disable", "--config", "-"])
        .env_clear()
        .env("LC_ALL", "C");
    #[cfg(windows)]
    windows_system::environment(&mut command)?;
    Ok(command)
}
fn exit_error(code: Option<i32>) -> ExchangeError {
    // Only curl's stable numeric categories cross the transport boundary.
    // Never expose stderr, response bodies or process/OS error strings.
    match code {
        Some(1 | 2 | 4 | 48) => ExchangeError::ClientUnsupported,
        Some(5 | 6) => ExchangeError::Dns,
        Some(7) => ExchangeError::Connection,
        Some(28) => ExchangeError::Timeout,
        Some(35 | 51 | 53 | 54 | 58 | 59 | 60 | 66 | 77 | 80 | 82 | 83 | 90 | 91 | 98) => {
            ExchangeError::Tls
        }
        Some(3 | 8 | 63 | 100) => ExchangeError::InvalidResponse,
        _ => ExchangeError::Unavailable,
    }
}
impl HttpsExchange for NativeHttps {
    fn send(&mut self, request: Request) -> std::result::Result<Response, ExchangeError> {
        let input = config(self, &request)?;
        let output = super::native_process::run(
            command()?,
            input,
            request.response_limit + HEADER_LIMIT + 4,
            Duration::from_secs(18),
        )
        .map_err(|error| match error {
            super::native_process::ProcessError::Start => ExchangeError::ClientUnavailable,
            super::native_process::ProcessError::Timeout => ExchangeError::Timeout,
            super::native_process::ProcessError::OutputLimit => ExchangeError::InvalidResponse,
            super::native_process::ProcessError::Io => ExchangeError::Unavailable,
        })?;
        if !output.status.success() {
            return Err(exit_error(output.status.code()));
        }
        if !output.stderr.is_empty() {
            return Err(ExchangeError::Unavailable);
        }
        parse(output.stdout, request.response_limit)
    }
}
#[cfg(test)]
#[path = "native_https_tests.rs"]
mod tests;
