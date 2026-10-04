use super::*;
fn request() -> Request {
    Request {
        method: "POST",
        url: "https://auth.example/auth/v1/token?grant_type=password".into(),
        headers: vec![
            ("authorization", "Bearer private-token".into()),
            ("content-type", "application/json".into()),
        ],
        body: Some(br#"{"password":"quote\" slash\\ newline\n"}"#.to_vec()),
        response_limit: 1024,
    }
}
#[test]
fn private_config_quotes_data_and_pins_origin_tls_and_deadlines() {
    let client = NativeHttps::new("https://auth.example", "https://api.example");
    let bytes = config(&client, &request()).unwrap();
    let value = String::from_utf8(bytes).unwrap();
    for required in [
        "tlsv1.2\n",
        "max-time = 15\n",
        "proto = \"=https\"",
        "noproxy = \"*\"",
        "data-raw = \"",
    ] {
        assert!(value.contains(required));
    }
    assert!(
        !value.contains("insecure") && !value.contains("location =") && !value.contains("cacert")
    );
    assert_eq!(quoted("a\"\n\\b").unwrap(), "\"a\\\"\\n\\\\b\"");
    assert!(quoted("nul\0").is_err());
}
#[test]
fn unsafe_destinations_headers_and_bounds_are_rejected_before_io() {
    let client = NativeHttps::new("https://auth.example", "https://api.example");
    for url in [
        "http://auth.example/",
        "https://evil.example/",
        "https://user@auth.example/",
        "https://auth.example/#fragment",
    ] {
        let mut req = request();
        req.url = url.into();
        assert!(config(&client, &req).is_err());
    }
    let mut req = request();
    req.headers
        .push(("authorization", "token\r\nnew-header: injected".into()));
    assert!(config(&client, &req).is_err());
    let mut req = request();
    req.headers.push(("cookie", "unexpected".into()));
    assert!(config(&client, &req).is_err());
    let mut req = request();
    req.body = Some(vec![b'a'; 32769]);
    assert!(config(&client, &req).is_err());
    let mut req = request();
    req.response_limit = RESPONSE_LIMIT + 1;
    assert!(config(&client, &req).is_err());
}
#[test]
fn bounded_response_parser_preserves_errors_without_following_redirects() {
    let reply=parse(b"HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nRetry-After: 5\r\n\r\n{}".to_vec(),2).unwrap();
    assert_eq!(reply.status, 429);
    assert_eq!(reply.retry_after.as_deref(), Some("5"));
    assert_eq!(reply.body, b"{}");
    assert_eq!(
        parse(
            b"HTTP/1.1 302 Found\r\nLocation: https://evil.example/\r\n\r\n".to_vec(),
            2
        )
        .unwrap()
        .status,
        302
    );
    for invalid in [
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\ncontent-type: text/plain\r\n\r\n{}"
            .as_slice(),
        b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\n\r\n",
        b"HTTP/1.1 200 OK\r\n\r\nabc",
        b"garbage",
    ] {
        assert!(parse(invalid.to_vec(), 2).is_err());
    }
}
#[cfg(target_os = "linux")]
#[test]
#[ignore = "Real TLS probe requires permission to bind a loopback listener"]
fn real_installed_curl_refuses_plaintext_as_tls_and_honors_exact_origin() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("https://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut hello = [0; 4096];
        assert!(socket.read(&mut hello).unwrap() > 0);
        let _ = socket.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"token\":\"forged\"}",
        );
    });
    let mut client = NativeHttps::new(&origin, "https://api.example");
    let mut req = request();
    req.url = format!("{origin}/");
    assert!(matches!(client.send(req), Err(ExchangeError::Unavailable)));
    server.join().unwrap();
}
