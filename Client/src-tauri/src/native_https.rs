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
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !client.origins.contains(&url.origin().ascii_serialization())
        || !matches!(request.method, "GET" | "POST")
        || request.response_limit == 0
        || request.response_limit > RESPONSE_LIMIT
        || request
            .body
            .as_ref()
            .is_some_and(|body| body.len() > 32 * 1024)
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
fn executable() -> std::result::Result<std::path::PathBuf, ExchangeError> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    let mut directory = vec![0u16; 32768];
    // SAFETY: buffer has size writable UTF-16 slots; SDK returns length.
    let size =
        unsafe { GetSystemDirectoryW(directory.as_mut_ptr(), directory.len() as u32) } as usize;
    if size == 0 || size >= directory.len() {
        return Err(ExchangeError::Unavailable);
    }
    use std::os::windows::ffi::OsStringExt;
    Ok(
        std::path::PathBuf::from(std::ffi::OsString::from_wide(&directory[..size]))
            .join("curl.exe"),
    )
}
#[cfg(not(any(windows, target_os = "linux")))]
fn executable() -> std::result::Result<std::path::PathBuf, ExchangeError> {
    Err(ExchangeError::Unavailable)
}
impl HttpsExchange for NativeHttps {
    fn send(&mut self, request: Request) -> std::result::Result<Response, ExchangeError> {
        let input = config(self, &request)?;
        let mut command = Command::new(executable()?);
        // --disable must be first to ignore personal/global curlrc. Credentials
        // travel through stdin config, never argv. No shell interprets the text.
        command
            .args(["--disable", "--config", "-"])
            .env_clear()
            .env("LC_ALL", "C");
        let output = super::native_process::run(
            command,
            input,
            request.response_limit + HEADER_LIMIT + 4,
            Duration::from_secs(18),
        )
        .map_err(|_| ExchangeError::Unavailable)?;
        if !output.status.success() || !output.stderr.is_empty() {
            return Err(ExchangeError::Unavailable);
        }
        parse(output.stdout, request.response_limit)
    }
}
#[cfg(test)]
#[path = "native_https_tests.rs"]
mod tests;
