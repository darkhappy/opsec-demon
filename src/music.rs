//! Local, read-only now-playing telemetry. Apple credentials are never displayed.
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
#[cfg(not(windows))]
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
static OUTPUT_ID: AtomicU64 = AtomicU64::new(0);

pub struct Music {
    updates: mpsc::Receiver<String>,
    stop: Arc<AtomicBool>,
    wake: mpsc::Sender<()>,
    worker: Option<JoinHandle<()>>,
    current: String,
    demo: bool,
}

impl Music {
    pub fn new() -> Self {
        let (sender, updates) = mpsc::sync_channel(1);
        let (wake, sleeping) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                let line = now_playing(&worker_stop);
                let _ = sender.try_send(line);
                if sleeping.recv_timeout(POLL_INTERVAL).is_ok() {
                    break;
                }
            }
        });
        Self {
            updates,
            stop,
            wake,
            worker: Some(worker),
            current: "LOCAL MUSIC | probing playback...".into(),
            demo: false,
        }
    }

    /// Drain completed updates without blocking rendering or keyboard input.
    pub fn poll(&mut self) {
        while let Ok(line) = self.updates.try_recv() {
            self.current = line;
        }
    }

    pub fn line(&self) -> String {
        if self.demo {
            "DEMO MUSIC | Playing | NIGHTWRAITH - Mainframe After Midnight".into()
        } else {
            self.current.clone()
        }
    }

    pub fn set_demo(&mut self, enabled: bool) {
        self.demo = enabled;
    }

    /// Presence is a configuration hint, never an authentication claim.
    pub fn credentials() -> &'static str {
        credential_status(
            token_present("APPLE_MUSIC_DEVELOPER_TOKEN"),
            token_present("APPLE_MUSIC_USER_TOKEN"),
        )
    }
}

impl Drop for Music {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.wake.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn token_present(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn credential_status(developer: bool, user: bool) -> &'static str {
    match (developer, user) {
        (true, true) => "APPLE TOKENS | present, unverified; local playback source",
        (false, false) => "APPLE TOKENS | absent; local playback needs no API key",
        _ => "APPLE TOKENS | incomplete, unverified; local playback source",
    }
}

fn clean(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control())
        .take(240)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn now_playing(stop: &AtomicBool) -> String {
    #[cfg(target_os = "macos")]
    let (program, arguments): (&str, &[&str]) = (
        "osascript",
        &[
            "-e",
            "if application \"Music\" is not running then return \"Stopped | Music is closed\"",
            "-e",
            "tell application \"Music\"",
            "-e",
            "if player state is stopped then return \"Stopped | no current track\"",
            "-e",
            "return (player state as text) & \" | \" & artist of current track & \" - \" & name of current track",
            "-e",
            "end tell",
        ],
    );
    #[cfg(all(not(target_os = "macos"), not(windows)))]
    let (program, arguments): (&str, &[&str]) = (
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{status}} | {{artist}} - {{title}}",
        ],
    );
    #[cfg(windows)]
    let script = format!(
        "& {{ {} }} -Once",
        include_str!("../scripts/windows-music.ps1")
    );
    #[cfg(windows)]
    let (program, arguments): (&str, &[&str]) = (
        "powershell.exe",
        &[
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ],
    );
    #[cfg(windows)]
    let timeout = Duration::from_secs(8);
    #[cfg(not(windows))]
    let timeout = COMMAND_TIMEOUT;
    match bounded_output(program, arguments, stop, timeout) {
        Ok(output) => {
            #[cfg(not(windows))]
            let text = clean(&output);
            #[cfg(windows)]
            let text = windows_metadata(&output);
            if text.is_empty() {
                "LOCAL MUSIC | no current track".into()
            } else {
                format!("LOCAL MUSIC | {text}")
            }
        }
        Err(reason) => format!("LOCAL MUSIC | {reason}"),
    }
}

#[cfg(windows)]
fn windows_metadata(output: &str) -> String {
    output
        .lines()
        .filter_map(|line| {
            serde_json::from_str::<serde_json::Value>(line.trim_start_matches('\u{feff}')).ok()
        })
        .next_back()
        .map_or_else(
            || "no Windows media session available".into(),
            |state| {
                format!(
                    "{} | {} - {}",
                    clean(state["status"].as_str().unwrap_or("Stopped")),
                    clean(state["artist"].as_str().unwrap_or("")),
                    clean(state["title"].as_str().unwrap_or(""))
                )
            },
        )
}

// An unlinked temporary file avoids pipe deadlocks and unbounded output buffers.
// Subprocesses are checked every 20ms and killed on timeout or UI shutdown.
fn capture_file() -> Result<File, &'static str> {
    let id = OUTPUT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("opsec-demon-music-{}-{id}", std::process::id()));
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x04000000).share_mode(1 | 2 | 4); // delete on last close
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path).map_err(|_| "capture unavailable")?;
    #[cfg(unix)]
    std::fs::remove_file(path).map_err(|_| "capture unavailable")?;
    Ok(file)
}

fn bounded_output(
    program: &str,
    arguments: &[&str],
    stop: &AtomicBool,
    timeout: Duration,
) -> Result<String, &'static str> {
    let mut capture = capture_file()?;
    let output = capture.try_clone().map_err(|_| "capture unavailable")?;
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .args(arguments)
        .env_remove("APPLE_MUSIC_DEVELOPER_TOKEN")
        .env_remove("APPLE_MUSIC_USER_TOKEN")
        .stdin(Stdio::null())
        .stdout(Stdio::from(output))
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "local playback adapter unavailable")?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err("no active player or playback access unavailable");
                }
                capture
                    .seek(SeekFrom::Start(0))
                    .map_err(|_| "capture unavailable")?;
                let mut bytes = Vec::new();
                capture
                    .take(8192)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "capture unavailable")?;
                return Ok(String::from_utf8_lossy(&bytes).into_owned());
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("playback adapter failed");
            }
            Ok(None) => {}
        }
        if stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("playback adapter timed out or stopped");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn windows_metadata_and_capture_preserve_unicode_and_hide_control_bytes() {
        assert!(
            windows_metadata(r#"{"status":"Paused","artist":"猫","title":"Song"}"#)
                .contains("Paused | 猫 - Song")
        );
        let stop = AtomicBool::new(false);
        assert_eq!(
            bounded_output(
                "cmd.exe",
                &["/D", "/C", "echo Playing"],
                &stop,
                Duration::from_secs(2)
            )
            .unwrap()
            .trim(),
            "Playing"
        );
    }

    #[test]
    fn credentials_do_not_claim_authentication() {
        assert!(credential_status(true, true).contains("unverified"));
        assert!(credential_status(true, false).contains("incomplete"));
        assert!(credential_status(false, false).contains("absent"));
    }

    #[test]
    fn terminal_control_bytes_are_removed_and_metadata_is_bounded() {
        assert_eq!(clean("\n Artist\x1b[2J\t - Title \r"), "Artist[2J - Title");
        assert_eq!(clean(&"a".repeat(1000)).len(), 240);
    }

    #[test]
    #[cfg(unix)]
    fn bounded_adapter_captures_output_and_times_out() {
        let stop = AtomicBool::new(false);
        assert_eq!(
            bounded_output(
                "/bin/sh",
                &["-c", "printf 'Playing | Artist - Song'"],
                &stop,
                Duration::from_secs(1)
            )
            .unwrap(),
            "Playing | Artist - Song"
        );
        let start = Instant::now();
        assert!(bounded_output("/bin/sleep", &["5"], &stop, Duration::from_millis(60)).is_err());
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn explicit_demo_and_shutdown_remain_responsive() {
        let mut music = Music::new();
        music.set_demo(true);
        assert!(music.line().starts_with("DEMO MUSIC"));
        music.set_demo(false);
        assert!(music.line().starts_with("LOCAL MUSIC"));
        let start = Instant::now();
        drop(music);
        assert!(start.elapsed() < Duration::from_secs(1));
    }
}
