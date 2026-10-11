//! Optional local Starship prompt, refreshed off the rendering thread.
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const REFRESH: Duration = Duration::from_secs(2);
const TIMEOUT: Duration = Duration::from_secs(2);
const OUTPUT_LIMIT: u64 = 8192;
static CAPTURE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, PartialEq)]
struct Request {
    path: PathBuf,
    status: i32,
    width: usize,
    environment: BTreeMap<String, Option<String>>,
}

pub struct Prompt {
    requests: mpsc::SyncSender<Request>,
    updates: mpsc::Receiver<(Request, String)>,
    requested: Option<Request>,
    last_refresh: Instant,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    current: String,
}

impl Prompt {
    pub fn new() -> Self {
        let (requests, incoming) = mpsc::sync_channel::<Request>(1);
        let (sender, updates) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                let request = match incoming.recv_timeout(Duration::from_millis(20)) {
                    Ok(request) => request,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                let line = render(&request, &worker_stop);
                let _ = sender.try_send((request, line));
            }
        });
        Self {
            requests,
            updates,
            requested: None,
            last_refresh: Instant::now(),
            stop,
            worker: Some(worker),
            current: "LOCAL PROMPT | loading...".into(),
        }
    }

    /// Nonblocking refresh: changing cwd/status/width requests a new prompt.
    pub fn poll(
        &mut self,
        path: &Path,
        status: i32,
        width: usize,
        environment: &BTreeMap<String, Option<String>>,
    ) {
        let request = Request {
            path: path.to_path_buf(),
            status,
            width: width.clamp(20, 1000),
            environment: environment.clone(),
        };
        while let Ok((finished, line)) = self.updates.try_recv() {
            if finished == request {
                self.current = line;
            }
        }
        if self.requested.as_ref() != Some(&request) || self.last_refresh.elapsed() >= REFRESH {
            if self
                .requested
                .as_ref()
                .is_none_or(|old| old.path != request.path)
            {
                // A new directory is visible immediately, while the worker probes tools.
                self.current = fallback(&request.path, None);
            }
            if self.requests.try_send(request.clone()).is_ok() {
                self.requested = Some(request);
                self.last_refresh = Instant::now();
            }
        }
    }

    pub fn line(&self) -> String {
        self.current.clone()
    }
}

impl Drop for Prompt {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn render(request: &Request, stop: &AtomicBool) -> String {
    let status = request.status.to_string();
    let width = request.width.to_string();
    let path = request.path.to_string_lossy();
    let starship = bounded_output(
        "starship",
        &[
            "prompt",
            "--path",
            &path,
            "--logical-path",
            &path,
            "--status",
            &status,
            "--terminal-width",
            &width,
        ],
        &request.path,
        stop,
        TIMEOUT,
        &request.environment,
    )
    .map(|line| clean(&line))
    .filter(|line| !line.is_empty());
    let mut line = match starship {
        Some(line) => line,
        None => {
            let branch = bounded_output(
                "git",
                &["symbolic-ref", "--quiet", "--short", "HEAD"],
                &request.path,
                stop,
                TIMEOUT,
                &request.environment,
            )
            .map(|branch| clean(&branch));
            fallback(&request.path, branch.as_deref())
        }
    };
    // `config current-context` reads local kubeconfig; no cluster request is made.
    let context = bounded_output(
        "kubectl",
        &["config", "current-context"],
        &request.path,
        stop,
        TIMEOUT,
        &request.environment,
    )
    .map(|context| clean(&context))
    .filter(|context| !context.is_empty());
    line.push_str(&format!(" [k8s: {}]", context.as_deref().unwrap_or("-")));
    line
}

fn fallback(path: &Path, branch: Option<&str>) -> String {
    let directory = clean(&path.to_string_lossy());
    match branch.filter(|branch| !branch.is_empty()) {
        Some(branch) => format!("{directory} [git: {branch}] >"),
        None => format!("{directory} >"),
    }
}

/// Strip complete CSI/OSC escape sequences and flatten text to one safe line.
fn clean(text: &str) -> String {
    let mut characters = text.chars().peekable();
    let mut result = String::new();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            match characters.next() {
                Some('[') => {
                    for next in characters.by_ref() {
                        if ('\u{40}'..='\u{7e}').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    while let Some(next) = characters.next() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' && characters.peek() == Some(&'\\') {
                            characters.next();
                            break;
                        }
                    }
                }
                Some('P' | 'X' | '^' | '_') => {
                    while let Some(next) = characters.next() {
                        if next == '\u{1b}' && characters.peek() == Some(&'\\') {
                            characters.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else if character == '\u{9b}' {
            for next in characters.by_ref() {
                if ('\u{40}'..='\u{7e}').contains(&next) {
                    break;
                }
            }
        } else if character.is_whitespace() {
            result.push(' ');
        } else if !character.is_control() {
            result.push(character);
        }
    }
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(800)
        .collect()
}

fn capture_file() -> Option<File> {
    let id = CAPTURE_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("opsec-demon-prompt-{}-{id}", std::process::id()));
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x04000000).share_mode(1 | 2 | 4);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path).ok()?;
    #[cfg(unix)]
    std::fs::remove_file(path).ok()?;
    Some(file)
}

fn bounded_output(
    program: &str,
    arguments: &[&str],
    path: &Path,
    stop: &AtomicBool,
    timeout: Duration,
    environment: &BTreeMap<String, Option<String>>,
) -> Option<String> {
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    let mut capture = capture_file()?;
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    for (name, value) in environment {
        if let Some(value) = value {
            command.env(name, value);
        } else {
            command.env_remove(name);
        }
    }
    let mut child = command
        .args(arguments)
        .current_dir(path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(capture.try_clone().ok()?))
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                capture.seek(SeekFrom::Start(0)).ok()?;
                let mut bytes = Vec::new();
                capture.take(OUTPUT_LIMIT).read_to_end(&mut bytes).ok()?;
                return Some(String::from_utf8_lossy(&bytes).into_owned());
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => {}
        }
        if stop.load(Ordering::Relaxed)
            || Instant::now() >= deadline
            || capture
                .metadata()
                .is_ok_and(|metadata| metadata.len() > OUTPUT_LIMIT)
        {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn native_adapter_capture_applies_child_environment() {
        let environment =
            BTreeMap::from([("OPSEC_PROMPT_TEST".into(), Some("session-value".into()))]);
        let result = bounded_output(
            "cmd.exe",
            &["/D", "/C", "echo %OPSEC_PROMPT_TEST%"],
            &std::env::current_dir().unwrap(),
            &AtomicBool::new(false),
            TIMEOUT,
            &environment,
        );
        assert_eq!(result.as_deref().map(str::trim), Some("session-value"));
    }

    #[test]
    fn prompt_escape_sequences_and_linebreaks_are_removed() {
        assert_eq!(
            clean(
                "\u{1b}[31mrepo\u{1b}[0m\n\u{1b}]8;;https://example.test\u{7}main\u{1b}]8;;\u{1b}\\\t>"
            ),
            "repo main >"
        );
        assert_eq!(clean("x\u{1b}Ppayload\u{1b}\\y\u{9b}2Jz"), "xyz");
        assert_eq!(clean(&"é".repeat(1000)).chars().count(), 800);
        assert_eq!(clean("safe\u{1b}]unterminated"), "safe");
    }

    #[test]
    #[cfg(unix)]
    fn local_adapter_output_and_timeout_are_bounded() {
        let stop = AtomicBool::new(false);
        assert_eq!(
            bounded_output(
                "/bin/sh",
                &["-c", "printf branch"],
                Path::new("/tmp"),
                &stop,
                TIMEOUT,
                &BTreeMap::new()
            ),
            Some("branch".into())
        );
        let started = Instant::now();
        assert!(
            bounded_output(
                "/bin/sleep",
                &["5"],
                Path::new("/tmp"),
                &stop,
                Duration::from_millis(60),
                &BTreeMap::new()
            )
            .is_none()
        );
        assert!(started.elapsed() < Duration::from_secs(1));
        let output = bounded_output(
            "/bin/sh",
            &["-c", "while :; do printf '12345678901234567890'; done"],
            Path::new("/tmp"),
            &stop,
            TIMEOUT,
            &BTreeMap::new(),
        );
        assert!(output.is_none());
    }

    #[test]
    fn refresh_never_blocks_rendering_and_shutdown_wakes_worker() {
        let mut prompt = Prompt::new();
        let started = Instant::now();
        for status in 0..100 {
            prompt.poll(Path::new("/tmp"), status, 120, &BTreeMap::new());
        }
        assert!(started.elapsed() < Duration::from_millis(100));
        assert!(prompt.line().contains("/tmp"));
        let started = Instant::now();
        drop(prompt);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    #[cfg(unix)]
    fn child_environment_overrides_are_local_and_removals_are_applied() {
        let stop = AtomicBool::new(false);
        let mut environment = BTreeMap::new();
        environment.insert("OPSEC_PROMPT_TEST".into(), Some("session-value".into()));
        environment.insert("OPSEC_PROMPT_REMOVED".into(), None);
        assert_eq!(
            bounded_output(
                "/bin/sh",
                &[
                    "-c",
                    "printf '%s|%s' \"$OPSEC_PROMPT_TEST\" \"${OPSEC_PROMPT_REMOVED-unset}\""
                ],
                Path::new("/tmp"),
                &stop,
                TIMEOUT,
                &environment,
            ),
            Some("session-value|unset".into())
        );
        // This checks the worker request comparison includes environment changes.
        let first = Request {
            path: PathBuf::from("/tmp"),
            status: 0,
            width: 120,
            environment: environment.clone(),
        };
        environment.insert("KUBECONFIG".into(), Some("/tmp/local-kubeconfig".into()));
        let second = Request {
            environment,
            ..first.clone()
        };
        assert!(first != second);
    }
}
