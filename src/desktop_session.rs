//! Bounded bridge between the graphical deck and the existing shell simulations.
use crate::{Shell, completion, is_builtin, job::Job, music::Music};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub enum Request {
    Execute(String),
    Complete(String, usize),
    Cancel,
    Stop,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub callsign: String,
    pub theme: String,
    pub scene: String,
    pub mascot: String,
    pub poopsec: bool,
    pub operations: u64,
    pub history: Vec<String>,
    pub mission: usize,
    pub band: bool,
    pub drummer_blast: bool,
    pub ssd_blast: bool,
    pub celebration: u64,
    pub pedal: bool,
    pub larp: u16,
    pub knobs: (u8, u8),
    pub combo: u32,
    pub score: u64,
    pub headphone: String,
    pub headphone_curve: Vec<[f64; 2]>,
    pub room_curve: Vec<[f64; 2]>,
    pub rt60: f64,
    pub novel: String,
    pub props: Vec<(&'static str, String)>,
    pub cwd: String,
    pub status: i32,
    pub dropped_output: u64,
    pub busy: bool,
}

pub enum Update {
    Line(String),
    Busy(bool),
    Music(String),
    Snapshot(Box<Snapshot>),
    Completion(completion::Completion),
    Closed(u8),
}

pub struct Session {
    pub requests: SyncSender<Request>,
    pub updates: Receiver<Update>,
    worker: Option<JoinHandle<()>>,
    pub exit_code: Arc<AtomicU16>,
}
impl Session {
    pub fn start(shell: Shell) -> Self {
        let (requests, incoming) = mpsc::sync_channel(32);
        let (outgoing, updates) = mpsc::sync_channel(256);
        let exit_code = Arc::new(AtomicU16::new(256));
        let worker_code = Arc::clone(&exit_code);
        let worker = thread::spawn(move || run(shell, incoming, outgoing, worker_code));
        Self {
            requests,
            updates,
            worker: Some(worker),
            exit_code,
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        // Cancel/stop are retried while the worker drains its bounded input queue.
        let _ = self.requests.send(Request::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn snapshot(shell: &Shell, celebration: u64, dropped_output: u64) -> Snapshot {
    let headphone_curve = (0..256)
        .map(|i| {
            let f = 20.0 * 1000f64.powf(f64::from(i) / 255.0);
            [f, shell.stage.frequency_gain(f)]
        })
        .collect();
    let rt60 = shell.room.rt60();
    Snapshot {
        callsign: shell.callsign.clone(),
        theme: shell.theme.clone(),
        scene: shell.scene.clone(),
        mascot: shell.mascot.clone(),
        poopsec: shell.poopsec,
        operations: shell.operations,
        history: shell.history.clone(),
        mission: shell.mission,
        band: shell.stage.band_on(),
        drummer_blast: shell.stage.drummer_blast(),
        ssd_blast: shell.ssd.blasting(),
        celebration,
        pedal: shell.stage.pedal_down(),
        larp: shell.stage.larp_meter(),
        knobs: shell.arcade.knobs(),
        combo: shell.arcade.combo(),
        score: shell.arcade.score(),
        headphone: shell.stage.headphone_name().into(),
        headphone_curve,
        room_curve: (0..256)
            .map(|i| {
                let t = rt60 * 1.5 * f64::from(i) / 255.0;
                [t, shell.room.decay_db(t)]
            })
            .collect(),
        rt60,
        novel: shell.novel.page(),
        cwd: crate::cwd().display().to_string(),
        status: shell.last_status,
        props: vec![
            ("GOLDEN GOOSE EGG", shell.egg.status()),
            ("CURSED APPLE", shell.ritual.status()),
            ("SSD DISASTER", shell.ssd.status()),
            ("MILK MESH", shell.milk.status()),
            ("VIRTUAL EXPANSION", shell.expansion.status()),
            ("LARP CITY", shell.desktop.status()),
            ("YOGGIES", shell.yoggies.status()),
            ("COUNTRY DECK", shell.countries.status()),
            ("PAPER COINS / ARCADE", shell.arcade.status()),
        ],
        dropped_output,
        busy: false,
    }
}

fn run(
    mut shell: Shell,
    incoming: Receiver<Request>,
    outgoing: SyncSender<Update>,
    exit_code: Arc<AtomicU16>,
) {
    *shell.transcript.borrow_mut() = Some(Vec::new());
    shell.color = false;
    let _ = shell.local();
    if std::env::var_os("DISCORD_APPLICATION_ID").is_some() {
        shell.execute("discord on");
    }
    let mut music = Music::new();
    let mut last_music = String::new();
    let mut job: Option<Job> = None;
    let mut refresh = Instant::now() - Duration::from_secs(1);
    let mut celebration = 0;
    let mut was_ssd_blast = false;
    let mut dropped = 0;
    let emit = |text: String, dropped: &mut u64| {
        if outgoing.try_send(Update::Line(text)).is_err() {
            *dropped += 1;
        }
    };
    loop {
        match incoming.recv_timeout(Duration::from_millis(20)) {
            Ok(Request::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Request::Cancel) => {
                if let Some(active) = &mut job {
                    active.cancel();
                }
            }
            Ok(Request::Complete(input, cursor)) => {
                let _ = outgoing.try_send(Update::Completion(
                    completion::complete_with_environment(&input, cursor, &shell.environment),
                ));
            }
            Ok(Request::Execute(line)) => {
                if job.is_some() {
                    emit(
                        "[LOCAL] Command running: cancel it before submitting another.".into(),
                        &mut dropped,
                    );
                    continue;
                }
                if !line.starts_with(' ') {
                    emit(format!("> {line}"), &mut dropped);
                    if shell.history.len() == 1000 {
                        shell.history.remove(0);
                    }
                    shell.history.push(line.clone());
                }
                if matches!(line.trim(), "shell" | "sdvx play")
                    || line.trim_start().starts_with("shell ")
                {
                    emit("[LOCAL] Interactive terminal programs require --shell in a terminal. Use the Arcade page for SDVX.".into(), &mut dropped);
                } else if matches!(
                    line.trim(),
                    "music" | "music status" | "music demo" | "music live"
                ) {
                    if line.trim() == "music demo" {
                        music.set_demo(true);
                    }
                    if line.trim() == "music live" {
                        music.set_demo(false);
                    }
                    emit(music.line(), &mut dropped);
                    emit(Music::credentials().into(), &mut dropped);
                } else if is_builtin(&line) && !line.trim_start().starts_with("soundtrack play") {
                    shell.last_status = shell.execute(&line);
                    if shell.exit.is_some() {
                        break;
                    }
                } else {
                    let command = if line.trim_start().starts_with("soundtrack play") {
                        crate::words(&line)
                            .and_then(|args| shell.soundtrack_play_command(&args[1..]))
                    } else {
                        Ok(shell.command(&line))
                    };
                    match command.and_then(Job::start) {
                        Ok(active) => {
                            job = Some(active);
                            let _ = outgoing.try_send(Update::Busy(true));
                        }
                        Err(error) => emit(format!("[LOCAL / ERROR] {error}"), &mut dropped),
                    }
                }
                refresh = Instant::now() - Duration::from_secs(1);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(active) = &mut job {
            match active.poll() {
                Ok((lines, status)) => {
                    for line in lines {
                        emit(line, &mut dropped);
                    }
                    if let Some(status) = status {
                        shell.last_status = status;
                        if status == 0 {
                            shell.operations = shell.operations.saturating_add(1);
                        }
                        emit(format!("[LOCAL] process exited {status}"), &mut dropped);
                        job = None;
                        let _ = outgoing.try_send(Update::Busy(false));
                    }
                }
                Err(error) => {
                    emit(format!("[LOCAL / ERROR] {error}"), &mut dropped);
                    job = None;
                    let _ = outgoing.try_send(Update::Busy(false));
                }
            }
        }
        for event in [
            shell.stage.tick(),
            shell.countries.tick(),
            shell.ssd.tick(),
            shell.egg.tick(),
        ]
        .into_iter()
        .flatten()
        {
            celebration += 1;
            emit(event, &mut dropped);
        }
        let stage_burst = shell.stage.take_celebration();
        let arcade_burst = shell.arcade.take_celebration();
        let ssd_blast = shell.ssd.blasting();
        if stage_burst || arcade_burst || (ssd_blast && !was_ssd_blast) {
            celebration += 1;
        }
        was_ssd_blast = ssd_blast;
        if let Some(lines) = shell.transcript.borrow_mut().as_mut() {
            for line in lines.drain(..) {
                emit(line, &mut dropped);
            }
        }
        music.poll();
        let line = music.line();
        if line != last_music && outgoing.try_send(Update::Music(line.clone())).is_ok() {
            last_music = line;
        }
        if refresh.elapsed() >= Duration::from_millis(100) {
            shell.update_discord();
            let mut state = snapshot(&shell, celebration, dropped);
            state.busy = job.is_some();
            let _ = outgoing.try_send(Update::Snapshot(Box::new(state)));
            refresh = Instant::now();
        }
    }
    drop(job);
    let code = shell.exit.unwrap_or(0);
    exit_code.store(u16::from(code), Ordering::Release);
    let _ = outgoing.try_send(Update::Closed(code));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effects_and_audio_curves_are_snapshots_of_the_existing_models() {
        let mut shell = Shell::new(false);
        shell.execute("band drummer explode");
        shell.execute("pedal stomp");
        shell.execute("ssd explode");
        shell.execute("sdvx hit bt-a");
        let state = snapshot(&shell, 4, 0);
        assert!(state.drummer_blast && state.ssd_blast && state.pedal);
        assert_eq!(state.combo, 1);
        assert_eq!(state.score, 1000);
        assert_eq!(state.larp, 15);
        assert_eq!(state.headphone_curve.len(), 256);
        assert_eq!(state.room_curve[0], [0.0, 0.0]);
        assert!(state.room_curve.last().unwrap()[1] < -60.0);
        assert!(state.props.iter().all(|(_, text)| !text.is_empty()));
    }
}
