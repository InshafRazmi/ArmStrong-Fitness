use super::*;
use std::{
    io::{Read, Write},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc,
    time::Instant,
};

// Absolute executable, no shell, no credential in argv/env/temp files. The
// Secret Service's default persistent collection is mandatory; no file fallback.
fn command(device: &str, store: bool) -> Command {
    item_command(device, "device-v1", if store { "store" } else { "lookup" })
}
fn item_command(device: &str, purpose: &str, action: &str) -> Command {
    let mut command = Command::new("/usr/bin/secret-tool");
    if action == "store" {
        command.args([
            "store",
            "--label=Armstrong native device",
            "--collection=default",
        ]);
    } else {
        command.arg(action);
    }
    command.args([
        "application",
        "lk.armstrong.fitness",
        "purpose",
        purpose,
        "device",
        device,
    ]);
    command
        .env("LC_ALL", "C")
        .env("SECRET_BACKEND", "service")
        .env_remove("SNAP_NAME")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}
struct Output {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
fn read_bounded(mut stream: impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 128];
    loop {
        let length = stream.read(&mut chunk).map_err(|_| VAULT_ERROR)?;
        if length == 0 {
            return Ok(bytes);
        }
        if bytes.len() + length > limit {
            wipe(&mut bytes);
            return Err(VAULT_ERROR.into());
        }
        bytes.extend_from_slice(&chunk[..length]);
    }
}
fn run(command: Command, input: &[u8], deadline: Duration) -> Result<Output> {
    run_bounded(command, input, deadline, 64)
}
fn run_bounded(
    mut command: Command,
    input: &[u8],
    deadline: Duration,
    limit: usize,
) -> Result<Output> {
    let mut child = command.spawn().map_err(|_| VAULT_ERROR)?;
    let stdout = child.stdout.take().ok_or(VAULT_ERROR)?;
    let stderr = child.stderr.take().ok_or(VAULT_ERROR)?;
    let (sender, receiver) = mpsc::channel();
    let out_sender = sender.clone();
    let out = std::thread::spawn(move || {
        let _ = out_sender.send((true, read_bounded(stdout, limit)));
    });
    let err = std::thread::spawn(move || {
        let _ = sender.send((false, read_bounded(stderr, 4096)));
    });
    let input_ok = child
        .stdin
        .take()
        .is_some_and(|mut pipe| pipe.write_all(input).is_ok());
    let started = Instant::now();
    let mut captured_out = None;
    let mut captured_err = None;
    let mut read_failed = false;
    let result = loop {
        while let Ok((is_stdout, bytes)) = receiver.try_recv() {
            match bytes {
                Ok(bytes) => {
                    if is_stdout {
                        captured_out = Some(bytes)
                    } else {
                        captured_err = Some(bytes)
                    }
                }
                Err(_) => {
                    read_failed = true;
                    break;
                }
            }
        }
        if !input_ok || read_failed || started.elapsed() >= deadline {
            break Err(VAULT_ERROR.into());
        }
        match child.try_wait() {
            Ok(Some(status)) if captured_out.is_some() && captured_err.is_some() => {
                break Ok(Output {
                    status,
                    stdout: captured_out.take().unwrap(),
                    stderr: captured_err.take().unwrap(),
                });
            }
            Ok(_) => (),
            Err(_) => break Err(VAULT_ERROR.into()),
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    let _ = out.join();
    let _ = err.join();
    if let Some(mut bytes) = captured_out {
        wipe(&mut bytes);
    }
    result
}
fn lookup(mut output: Output) -> Result<Option<String>> {
    if output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
        return Ok(None);
    }
    if !output.status.success() || !output.stderr.is_empty() {
        wipe(&mut output.stdout);
        return Err(VAULT_ERROR.into());
    }
    let secret = String::from_utf8(output.stdout).map_err(|_| RECONCILE)?;
    checked_secret(&secret)?;
    Ok(Some(secret))
}
impl CredentialVault for OsVault {
    fn read(&mut self, device: &str) -> Result<Option<String>> {
        checked_device(device)?;
        // libsecret selects a portal/file backend automatically in Flatpak.
        // This desktop path requires the persistent Secret Service instead.
        if Path::new("/.flatpak-info").exists() {
            return Err(VAULT_ERROR.into());
        }
        lookup(run(command(device, false), &[], Duration::from_secs(15))?)
    }
    fn create(&mut self, device: &str, secret: &str) -> Result<()> {
        checked_device(device)?;
        checked_secret(secret)?;
        if self.read(device)?.is_some() {
            return Err(RECONCILE.into());
        }
        let mut output = run(
            command(device, true),
            secret.as_bytes(),
            Duration::from_secs(15),
        )?;
        let ok = output.status.success() && output.stdout.is_empty() && output.stderr.is_empty();
        wipe(&mut output.stdout);
        if ok {
            Ok(())
        } else {
            Err(VAULT_ERROR.into())
        }
    }
}

impl OfflineVault for OsVault {
    fn read_access(&mut self, device: &str) -> Result<Option<String>> {
        checked_device(device)?;
        if Path::new("/.flatpak-info").exists() {
            return Err(VAULT_ERROR.into());
        }
        let mut output = run_bounded(
            item_command(device, "offline-access-v1", "lookup"),
            &[],
            Duration::from_secs(15),
            ACCESS_LIMIT,
        )?;
        if output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
            return Ok(None);
        }
        if !output.status.success() || !output.stderr.is_empty() {
            wipe(&mut output.stdout);
            return Err(VAULT_ERROR.into());
        }
        String::from_utf8(output.stdout)
            .map(Some)
            .map_err(|_| RECONCILE.into())
    }
    fn save_access(&mut self, device: &str, value: &str) -> Result<()> {
        checked_device(device)?;
        if value.is_empty() || value.len() > ACCESS_LIMIT || Path::new("/.flatpak-info").exists() {
            return Err(VAULT_ERROR.into());
        }
        let output = run_bounded(
            item_command(device, "offline-access-v1", "store"),
            value.as_bytes(),
            Duration::from_secs(15),
            ACCESS_LIMIT,
        )?;
        if output.status.success() && output.stdout.is_empty() && output.stderr.is_empty() {
            Ok(())
        } else {
            Err(VAULT_ERROR.into())
        }
    }
    fn clear_access(&mut self, device: &str) -> Result<()> {
        checked_device(device)?;
        let output = run_bounded(
            item_command(device, "offline-access-v1", "clear"),
            &[],
            Duration::from_secs(15),
            ACCESS_LIMIT,
        )?;
        if matches!(output.status.code(), Some(0 | 1))
            && output.stdout.is_empty()
            && output.stderr.is_empty()
        {
            Ok(())
        } else {
            Err(VAULT_ERROR.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    #[test]
    #[ignore = "Read-only installed Secret Service probe; requires an OS session"]
    fn installed_os_vault_read_only_probe() {
        match OsVault.read(&id()) {
            Ok(None) => eprintln!(
                "OS credential store read-only probe: available, no matching synthetic item"
            ),
            Ok(Some(_)) => panic!("Unexpected matching random device credential; values withheld"),
            Err(error) => {
                assert_eq!(error, VAULT_ERROR);
                eprintln!("OS credential store read-only probe: unavailable/locked; redacted error verified");
            }
        }
    }
    fn output(code: i32, stdout: &[u8], stderr: &[u8]) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }
    #[test]
    fn absent_is_distinct_from_locked_or_invalid_vault() {
        assert!(lookup(output(1, b"", b"")).unwrap().is_none());
        assert!(lookup(output(1, b"", b"private provider error"))
            .unwrap_err()
            .contains("OS credential storage"));
        assert!(lookup(output(0, b"", b"")).is_err());
        assert!(lookup(output(0, b"corrupt", b"")).is_err());
        assert!(lookup(output(0, "a".repeat(64).as_bytes(), b""))
            .unwrap()
            .is_some());
    }
    #[test]
    fn utility_limits_kill_and_reap_child_without_exposing_output() {
        let mut oversized = Command::new("/usr/bin/head");
        oversized.args(["-c", "65", "/dev/zero"]);
        oversized
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        assert!(run(oversized, &[], Duration::from_millis(100)).is_err());
        let mut hanging = Command::new("/usr/bin/sleep");
        hanging
            .arg("1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let started = Instant::now();
        assert!(run(hanging, &[], Duration::from_millis(30)).is_err());
        assert!(started.elapsed() < Duration::from_millis(900));
    }
    #[test]
    fn command_arguments_contain_only_public_device_identity() {
        let device = id();
        let cmd = command(&device, true);
        assert_eq!(cmd.get_program(), "/usr/bin/secret-tool");
        assert!(cmd.get_args().any(|arg| arg == "--collection=default"));
        assert!(cmd.get_args().any(|arg| arg == device.as_str()));
        assert!(cmd
            .get_envs()
            .any(|(key, value)| key == "SECRET_BACKEND"
                && value.is_some_and(|value| value == "service")));
        assert!(!cmd
            .get_args()
            .any(|arg| arg.to_string_lossy().contains(&"a".repeat(64))));
    }
}
