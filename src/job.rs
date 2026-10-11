use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

pub struct Job {
    child: Child,
    output: Receiver<String>,
    status: Option<i32>,
}

fn reader(mut pipe: impl Read + Send + 'static, sender: SyncSender<String>, prefix: &'static str) {
    thread::spawn(move || {
        let mut chunk = [0; 512];
        let mut line = Vec::new();
        while let Ok(count) = pipe.read(&mut chunk) {
            if count == 0 {
                break;
            }
            for &byte in &chunk[..count] {
                if byte == b'\n' || line.len() == 2048 {
                    if sender
                        .send(format!("{prefix}{}", String::from_utf8_lossy(&line)))
                        .is_err()
                    {
                        return;
                    }
                    line.clear();
                }
                if byte != b'\n' {
                    line.push(byte);
                }
            }
        }
        if !line.is_empty() {
            let _ = sender.send(format!("{prefix}{}", String::from_utf8_lossy(&line)));
        }
    });
}

impl Job {
    pub fn start(mut command: Command) -> io::Result<Self> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()?;
        let (sender, output) = mpsc::sync_channel(128);
        reader(
            child.stdout.take().expect("piped stdout"),
            sender.clone(),
            "",
        );
        reader(
            child.stderr.take().expect("piped stderr"),
            sender,
            "[stderr] ",
        );
        Ok(Self {
            child,
            output,
            status: None,
        })
    }

    pub fn poll(&mut self) -> io::Result<(Vec<String>, Option<i32>)> {
        let mut lines = Vec::new();
        let mut disconnected = false;
        for _ in 0..64 {
            match self.output.try_recv() {
                Ok(line) => lines.push(line),
                Err(mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        if self.status.is_none() {
            self.status = self
                .child
                .try_wait()?
                .map(|status| status.code().unwrap_or(130));
        }
        Ok((lines, if disconnected { self.status } else { None }))
    }

    pub fn cancel(&mut self) {
        // Stop the entire pipeline, including descendants holding output pipes.
        let group = format!("-{}", self.child.id());
        let _ = Command::new("kill")
            .args(["-KILL", "--", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.child.kill();
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if self.status.is_none() {
            self.cancel();
        }
        let _ = self.child.wait();
    }
}
