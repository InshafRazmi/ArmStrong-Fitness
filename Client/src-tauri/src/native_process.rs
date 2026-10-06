// Bounded native subprocess I/O. No shell, diagnostics, credential argv or files.
use std::{
    io::{Read, Write},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
pub(super) struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
// Categories only: OS messages and process diagnostics can contain secrets.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ProcessError {
    Start,
    Io,
    OutputLimit,
    Timeout,
}
fn read(mut stream: impl Read, limit: usize) -> std::result::Result<Vec<u8>, ProcessError> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let size = stream.read(&mut chunk).map_err(|_| ProcessError::Io)?;
        if size == 0 {
            return Ok(bytes);
        }
        if bytes.len() + size > limit {
            return Err(ProcessError::OutputLimit);
        }
        bytes.extend_from_slice(&chunk[..size]);
    }
}
pub(super) fn run(
    mut command: Command,
    input: Vec<u8>,
    limit: usize,
    timeout: Duration,
) -> std::result::Result<Output, ProcessError> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW, no credential console
    }
    let mut child = command.spawn().map_err(|_| ProcessError::Start)?;
    let stdin = child.stdin.take().ok_or(ProcessError::Io)?;
    let stdout = child.stdout.take().ok_or(ProcessError::Io)?;
    let stderr = child.stderr.take().ok_or(ProcessError::Io)?;
    let (sender, receiver) = mpsc::channel();
    let out_sender = sender.clone();
    let err_sender = sender.clone();
    let out = std::thread::spawn(move || {
        let _ = out_sender.send((0, read(stdout, limit)));
    });
    let err = std::thread::spawn(move || {
        let _ = err_sender.send((1, read(stderr, 4096)));
    });
    let writer = std::thread::spawn(move || {
        let mut stdin = stdin;
        let _ = sender.send((
            2,
            stdin
                .write_all(&input)
                .map(|_| Vec::new())
                .map_err(|_| ProcessError::Io),
        ));
    });
    let started = Instant::now();
    let mut streams: [Option<Vec<u8>>; 3] = [None, None, None];
    let mut failure = None;
    let result = loop {
        while let Ok((index, reply)) = receiver.try_recv() {
            match reply {
                Ok(bytes) => streams[index] = Some(bytes),
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        if let Some(error) = failure {
            break Err(error);
        }
        if started.elapsed() >= timeout {
            break Err(ProcessError::Timeout);
        }
        match child.try_wait() {
            Ok(Some(status)) if streams.iter().all(Option::is_some) => {
                break Ok(Output {
                    status,
                    stdout: streams[0].take().unwrap(),
                    stderr: streams[1].take().unwrap(),
                })
            }
            Ok(_) => (),
            Err(_) => break Err(ProcessError::Io),
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    let _ = writer.join();
    let _ = out.join();
    let _ = err.join();
    result
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn stdin_credentials_are_transferred_without_arguments_and_output_is_bounded() {
        let command = Command::new("/usr/bin/cat");
        assert_eq!(command.get_args().count(), 0);
        let data = vec![b'a'; 96 * 1024];
        let output = run(command, data.clone(), data.len(), Duration::from_secs(3)).unwrap();
        assert!(output.status.success() && output.stderr.is_empty());
        assert_eq!(output.stdout, data);
        let mut oversized = Command::new("/usr/bin/head");
        oversized.args(["-c", "8193", "/dev/zero"]);
        assert!(matches!(
            run(oversized, vec![], 8192, Duration::from_secs(3)),
            Err(ProcessError::OutputLimit)
        ));
    }
    #[test]
    fn timeout_kills_and_reaps_child_instead_of_waiting_for_service() {
        let mut command = Command::new("/usr/bin/sleep");
        command.arg("2");
        let started = Instant::now();
        assert!(matches!(
            run(command, vec![], 1, Duration::from_millis(30)),
            Err(ProcessError::Timeout)
        ));
        assert!(started.elapsed() < Duration::from_millis(1500));
    }
    #[test]
    fn missing_client_returns_a_category_without_os_diagnostics() {
        let path = std::env::temp_dir().join(format!("missing-client-{}", crate::id()));
        assert!(matches!(
            run(Command::new(path), vec![], 1, Duration::from_secs(1)),
            Err(ProcessError::Start)
        ));
    }
}
