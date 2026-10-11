//! Real local Discord IPC carrying explicitly fictional operator theater.
#[cfg(windows)]
use std::fs::File as IpcStream;
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream as IpcStream;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::discord_profile::{Profile, Timer, bounded};
use serde_json::{Value, json};

const INTERVAL: Duration = Duration::from_secs(15);
#[cfg(unix)]
const TIMEOUT: Duration = Duration::from_millis(500);
const TRANSMISSIONS: &[&str] = &[
    "SIM | Exorcising the astral firewall",
    "SIM | BLACK CHOIR // midnight root ceremony",
    "SIM | Routing forbidden milk through the abyss",
    "SIM | GHOST KERNEL // soul cache armed",
    "SIM | Counterintel pigeons await the blood moon",
    "SIM | Negotiating clearance with the ethernet demon",
];

#[derive(Clone, Default)]
pub(crate) struct Snapshot {
    pub scene: String,
    pub poopsec: bool,
    pub operations: u64,
    pub mascot: String,
    pub mission: usize,
    pub yoggies: (usize, usize),
    pub lore: Vec<String>,
}

struct State {
    id: Option<String>,
    snapshot: Snapshot,
    status: String,
    stop: bool,
    profile: Profile,
}

pub struct Presence {
    shared: Arc<(Mutex<State>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}

impl Presence {
    pub fn new() -> Self {
        Self {
            shared: Arc::new((
                Mutex::new(State {
                    id: None,
                    snapshot: Snapshot::default(),
                    status: "OFF // the black choir observes radio silence".into(),
                    stop: false,
                    profile: Profile::default(),
                }),
                Condvar::new(),
            )),
            worker: None,
        }
    }

    #[cfg(test)]
    pub fn update(&self, scene: &str, poopsec: bool, operations: u64) {
        self.shared.0.lock().unwrap().snapshot = Snapshot {
            scene: scene.into(),
            poopsec,
            operations,
            ..Snapshot::default()
        };
    }

    pub fn update_snapshot(&self, snapshot: Snapshot) {
        self.shared.0.lock().unwrap().snapshot = snapshot;
    }

    pub fn command(
        &mut self,
        args: &[String],
        configured_id: Option<String>,
    ) -> io::Result<String> {
        match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] | ["status"] => {}
            ["preview"] => {
                let state = self.shared.0.lock().unwrap();
                let mut payload = activity(&state.snapshot, now(), 0);
                state.profile.apply(&mut payload, 0);
                if payload.get("secrets").is_some() {
                    payload["secrets"] = json!("[REDACTED session join/spectate/match values]");
                }
                return Ok(format!(
                    "[DISCORD / SIM PREVIEW]\n{}",
                    serde_json::to_string_pretty(&payload)?
                ));
            }
            ["load", path] => {
                let file = std::fs::File::open(path)?;
                let mut text = String::new();
                file.take(32 * 1024 + 1).read_to_string(&mut text)?;
                if text.len() > 32 * 1024 {
                    return Err(io::Error::other("Discord profile exceeds 32 KiB"));
                }
                let profile = Profile::parse(&text)?;
                self.shared.0.lock().unwrap().profile = profile;
                return Ok(
                    "[DISCORD] SIM arsenal loaded; next broadcast uses the validated profile"
                        .into(),
                );
            }
            ["art", base] => {
                let base = base.trim_end_matches('/');
                let profile = Profile::parse(&json!({
                    "large_images": [format!("{base}/nightfall.gif"), format!("{base}/black_choir.gif"), format!("{base}/yoggies.gif")],
                    "small_images": [format!("{base}/yoggies.gif"), format!("{base}/porcelain.gif"), format!("{base}/ghost_kernel.gif")]
                }).to_string())?;
                let mut state = self.shared.0.lock().unwrap();
                state.profile.large_images = profile.large_images;
                state.profile.small_images = profile.small_images;
                if let Some(assets) = state.profile.overrides["assets"].as_object_mut() {
                    assets.remove("large_image");
                    assets.remove("small_image");
                    assets.insert(
                        "invite_cover_image".into(),
                        json!(format!("{base}/black_choir.gif")),
                    );
                }
                return Ok("[DISCORD] Hosted animated illustration pack armed; Discord fetches the public URLs".into());
            }
            ["reset"] => {
                self.shared.0.lock().unwrap().profile = Profile::default();
                return Ok("[DISCORD] BLACK CHOIR defaults restored".into());
            }
            ["type", kind] => {
                let kind = match *kind {
                    "playing" => 0,
                    "listening" => 2,
                    "watching" => 3,
                    "competing" => 5,
                    _ => {
                        return Err(io::Error::other(
                            "type: playing|listening|watching|competing",
                        ));
                    }
                };
                self.shared.0.lock().unwrap().profile.overrides["type"] = json!(kind);
                return Ok("[DISCORD] SIM activity mode armed".into());
            }
            ["display", field] => {
                let field = match *field {
                    "name" => 0,
                    "state" => 1,
                    "details" => 2,
                    _ => return Err(io::Error::other("display: name|state|details")),
                };
                self.shared.0.lock().unwrap().profile.overrides["status_display_type"] =
                    json!(field);
                return Ok("[DISCORD] Member-list display selected".into());
            }
            ["timer", duration] => {
                let timer = match *duration {
                    "off" => Timer::Off,
                    "elapsed" => Timer::Elapsed,
                    value => {
                        let seconds = value
                            .parse::<u64>()
                            .ok()
                            .filter(|v| (1..=86400).contains(v))
                            .ok_or_else(|| {
                                io::Error::other("timer: elapsed|off|SECONDS (1..86400)")
                            })?;
                        let start = now();
                        Timer::Countdown {
                            start,
                            end: start + seconds,
                        }
                    }
                };
                let mut state = self.shared.0.lock().unwrap();
                state
                    .profile
                    .overrides
                    .as_object_mut()
                    .unwrap()
                    .remove("timestamps");
                state.profile.timer = timer;
                return Ok("[DISCORD] SIM ritual clock configured".into());
            }
            ["off"] => {
                let mut state = self.shared.0.lock().unwrap();
                state.id = None;
                state.status = "OFF // banishment ordered; local activity will be cleared".into();
                self.shared.1.notify_one();
            }
            ["on"] | ["on", _] => {
                let id = args.get(1).cloned().or(configured_id).ok_or_else(|| {
                    io::Error::other(
                        "usage: discord on APPLICATION_ID (or set DISCORD_APPLICATION_ID)",
                    )
                })?;
                validate_id(&id)?;
                {
                    let mut state = self.shared.0.lock().unwrap();
                    state.id = Some(id);
                    state.status = "CONNECTING // summoning the local Discord familiar".into();
                }
                if self.worker.is_none() {
                    let shared = Arc::clone(&self.shared);
                    self.worker = Some(thread::spawn(move || worker(shared)));
                }
                self.shared.1.notify_one();
            }
            _ => {
                return Err(io::Error::other(
                    "usage: discord status|on [ID]|off|preview|load FILE|art HTTPS_BASE|reset|type MODE|display FIELD|timer SECONDS|elapsed|off",
                ));
            }
        }
        Ok(format!(
            "[DISCORD / LOCAL IPC] {}",
            self.shared.0.lock().unwrap().status
        ))
    }
}

impl Drop for Presence {
    fn drop(&mut self) {
        self.shared.0.lock().unwrap().stop = true;
        self.shared.1.notify_one();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn validate_id(id: &str) -> io::Result<()> {
    if id.is_empty()
        || !id.bytes().all(|byte| byte.is_ascii_digit())
        || !id.parse::<u64>().is_ok_and(|value| value > 0)
    {
        return Err(io::Error::other(
            "Discord application ID must be a positive decimal u64; no token needed",
        ));
    }
    Ok(())
}

fn activity(snapshot: &Snapshot, started: u64, transmission: usize) -> Value {
    let clearance = if snapshot.poopsec {
        "PORCELAIN ARCHDEMON"
    } else {
        "NIGHTFALL ARCHON"
    };
    let details = if transmission % 2 == 1 && !snapshot.lore.is_empty() {
        format!(
            "SIM | {}",
            snapshot.lore[(transmission / 2) % snapshot.lore.len()]
        )
    } else {
        TRANSMISSIONS[(transmission / 2) % TRANSMISSIONS.len()].into()
    };
    json!({
        "details": bounded(&details, 128),
        "state": bounded(&format!("SIM | {clearance} // {} sector // {} rites sealed // {}/{} Yoggies", snapshot.scene, snapshot.operations, snapshot.yoggies.0, snapshot.yoggies.1), 128),
        "timestamps": { "start": started },
        "type": 0,
        "assets": {
            "large_text": bounded(&format!("SIM MISSION {} // {} // {clearance}", snapshot.mission + 1, snapshot.scene), 128),
            "small_text": bounded(&format!("SIM {} familiar // Yoggies {}/{} // no actual clearance", snapshot.mascot, snapshot.yoggies.0, snapshot.yoggies.1), 128)
        },
        "party": { "id": format!("SIM-YOGGIES-{}", std::process::id()), "size": [snapshot.yoggies.0, snapshot.yoggies.1.max(1)] }
    })
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn worker(shared: Arc<(Mutex<State>, Condvar)>) {
    worker_with_connector(shared, INTERVAL, connect);
}

fn worker_with_connector(
    shared: Arc<(Mutex<State>, Condvar)>,
    interval: Duration,
    mut connector: impl FnMut(&str) -> io::Result<IpcStream>,
) {
    let mut connection: Option<(String, IpcStream)> = None;
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut transmission = 0;
    let mut nonce = 0u64;
    loop {
        let (id, snapshot, profile, stop) = {
            let state = shared.0.lock().unwrap();
            (
                state.id.clone(),
                state.snapshot.clone(),
                state.profile.clone(),
                state.stop,
            )
        };
        if (stop
            || connection
                .as_ref()
                .is_some_and(|(active, _)| Some(active) != id.as_ref()))
            && let Some((_, mut stream)) = connection.take()
        {
            nonce += 1;
            let _ = set_activity(&mut stream, Value::Null, nonce);
        }
        if stop {
            break;
        }
        if let Some(id) = id.as_ref() {
            let result = (|| -> io::Result<()> {
                if connection.is_none() {
                    connection = Some((id.clone(), connector(id)?));
                }
                nonce += 1;
                let mut payload = activity(&snapshot, started, transmission);
                profile.apply(&mut payload, transmission);
                set_activity(&mut connection.as_mut().unwrap().1, payload, nonce)
            })();
            let mut state = shared.0.lock().unwrap();
            if state.id.as_ref() == Some(id) {
                state.status = match result {
                    Ok(()) => {
                        "TRANSMITTING // Discord acknowledged the SIM black-choir broadcast".into()
                    }
                    Err(_) => {
                        connection = None;
                        "WAITING // local Discord unavailable or rejected activity; retry in 15s"
                            .into()
                    }
                };
            }
            transmission += 1;
        }
        let state = shared.0.lock().unwrap();
        // A changed configuration must be handled immediately, even if its wake arrived during I/O.
        if state.stop || state.id != id {
            continue;
        }
        let _ = shared.1.wait_timeout(state, interval).unwrap();
    }
}

#[cfg(unix)]
fn socket_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"] {
        if let Some(root) = std::env::var_os(key).filter(|value| !value.is_empty()) {
            let root = PathBuf::from(root);
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
    }
    if !roots.contains(&PathBuf::from("/tmp")) {
        roots.push(PathBuf::from("/tmp"));
    }
    roots
}

#[cfg(windows)]
fn connect(_id: &str) -> io::Result<IpcStream> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Discord named-pipe transport is not implemented",
    ))
}

#[cfg(unix)]
fn connect(id: &str) -> io::Result<IpcStream> {
    for root in socket_roots() {
        for subpath in [
            "",
            "app/com.discordapp.Discord",
            "snap.discord",
            "snap.discord-canary",
        ] {
            for index in 0..10 {
                if let Ok(mut stream) =
                    IpcStream::connect(root.join(subpath).join(format!("discord-ipc-{index}")))
                {
                    stream.set_read_timeout(Some(TIMEOUT))?;
                    stream.set_write_timeout(Some(TIMEOUT))?;
                    send(&mut stream, 0, &json!({ "v": 1, "client_id": id }))?;
                    let (opcode, ready) = receive(&mut stream)?;
                    if opcode != 1 || ready["evt"] != "READY" {
                        return Err(io::Error::other("Discord handshake rejected"));
                    }
                    return Ok(stream);
                }
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no local Discord IPC socket",
    ))
}

fn send(stream: &mut IpcStream, opcode: u32, payload: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(payload)?;
    stream.write_all(&opcode.to_le_bytes())?;
    stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
    stream.write_all(&bytes)
}

fn receive(stream: &mut IpcStream) -> io::Result<(u32, Value)> {
    let mut header = [0; 8];
    stream.read_exact(&mut header)?;
    let opcode = u32::from_le_bytes(header[..4].try_into().unwrap());
    let length = u32::from_le_bytes(header[4..].try_into().unwrap()) as usize;
    if length > 64 * 1024 {
        return Err(io::Error::other("oversized Discord frame"));
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes)?;
    Ok((opcode, serde_json::from_slice(&bytes)?))
}

fn set_activity(stream: &mut IpcStream, activity: Value, nonce: u64) -> io::Result<()> {
    let nonce = nonce.to_string();
    send(
        stream,
        1,
        &json!({
            "cmd": "SET_ACTIVITY", "args": { "pid": std::process::id(), "activity": activity }, "nonce": nonce
        }),
    )?;
    for _ in 0..8 {
        let (opcode, response) = receive(stream)?;
        if opcode == 3 {
            send(stream, 4, &response)?;
            continue;
        }
        if opcode != 1 || response["evt"] == "ERROR" {
            return Err(io::Error::other("Discord rejected activity"));
        }
        if response["nonce"] == nonce && response["cmd"] == "SET_ACTIVITY" {
            return Ok(());
        }
    }
    Err(io::Error::other("Discord did not acknowledge activity"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_is_fictional_bounded_and_has_no_operator_data() {
        let snapshot = Snapshot {
            scene: "vulcan".into(),
            poopsec: true,
            operations: 42,
            ..Snapshot::default()
        };
        for index in 0..TRANSMISSIONS.len() {
            let payload = activity(&snapshot, 123, index);
            assert!(payload["details"].as_str().unwrap().starts_with("SIM |"));
            assert!(
                payload["state"]
                    .as_str()
                    .unwrap()
                    .contains("PORCELAIN ARCHDEMON")
            );
            assert!(payload["state"].as_str().unwrap().len() <= 128);
            assert!(payload["details"].as_str().unwrap().len() <= 128);
            assert_eq!(payload["timestamps"]["start"], 123);
            assert!(payload.get("secrets").is_none());
        }
    }

    #[test]
    fn commands_validate_configuration_without_starting_ipc() {
        let mut presence = Presence::new();
        assert!(presence.command(&[], None).unwrap().contains("OFF"));
        for id in ["", "0", "-1", "token", "18446744073709551616"] {
            assert!(presence.command(&["on".into(), id.into()], None).is_err());
        }
        assert!(presence.command(&["on".into()], None).is_err());
        assert!(presence.command(&["wat".into()], None).is_err());
        assert!(
            presence
                .command(&["off".into()], None)
                .unwrap()
                .contains("OFF")
        );
        assert!(validate_id("123456789012345678").is_ok());
        assert!(presence.worker.is_none());
    }

    #[test]
    fn clocks_are_fixed_and_preview_redacts_session_identifiers() {
        let mut presence = Presence::new();
        presence.update_snapshot(Snapshot {
            scene: "sdvx".into(),
            yoggies: (3, 5),
            lore: vec!["YOGGIES // 3 of 5 imaginary rites remain".into()],
            ..Snapshot::default()
        });
        presence
            .command(&["timer".into(), "120".into()], None)
            .unwrap();
        let state = presence.shared.0.lock().unwrap();
        let mut first = activity(&state.snapshot, 123, 0);
        let mut later = activity(&state.snapshot, 999, 9);
        state.profile.apply(&mut first, 0);
        state.profile.apply(&mut later, 9);
        assert_eq!(first["timestamps"], later["timestamps"]);
        assert_eq!(first["party"]["size"], json!([3, 5]));
        assert!(later["details"].as_str().unwrap().contains("YOGGIES"));
        drop(state);
        presence.shared.0.lock().unwrap().profile.overrides["secrets"] =
            json!({"join": "must-not-appear"});
        let preview = presence.command(&["preview".into()], None).unwrap();
        assert!(!preview.contains("must-not-appear"));
        assert!(preview.contains("REDACTED"));
        presence
            .command(&["timer".into(), "off".into()], None)
            .unwrap();
        assert!(
            !presence
                .command(&["preview".into()], None)
                .unwrap()
                .contains("timestamps")
        );
        assert!(
            presence
                .command(&["timer".into(), "0".into()], None)
                .is_err()
        );
        assert!(
            presence
                .command(&["type".into(), "streaming".into()], None)
                .is_err()
        );
        assert!(presence.worker.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn ipc_handles_ping_acknowledgement_and_clear() {
        let (mut client, mut server) = IpcStream::pair().unwrap();
        client.set_read_timeout(Some(TIMEOUT)).unwrap();
        let peer = thread::spawn(move || {
            let (opcode, payload) = receive(&mut server).unwrap();
            assert_eq!(opcode, 1);
            assert_eq!(payload["args"]["activity"]["details"], TRANSMISSIONS[0]);
            send(&mut server, 3, &json!({"probe": true})).unwrap();
            assert_eq!(receive(&mut server).unwrap(), (4, json!({"probe": true})));
            send(
                &mut server,
                1,
                &json!({"cmd": "SET_ACTIVITY", "nonce": "1", "evt": null}),
            )
            .unwrap();
            let (_, payload) = receive(&mut server).unwrap();
            assert!(payload["args"]["activity"].is_null());
            send(
                &mut server,
                1,
                &json!({"cmd": "SET_ACTIVITY", "nonce": "2", "evt": null}),
            )
            .unwrap();
        });
        set_activity(&mut client, activity(&Snapshot::default(), 123, 0), 1).unwrap();
        set_activity(&mut client, Value::Null, 2).unwrap();
        peer.join().unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn worker_retries_then_clears_on_disable_and_shutdown() {
        for shutdown in [false, true] {
            let presence = Presence::new();
            presence.update("sdvx", true, 12);
            presence.shared.0.lock().unwrap().id = Some("123".into());
            let shared = Arc::clone(&presence.shared);
            let (client, mut server) = IpcStream::pair().unwrap();
            client.set_read_timeout(Some(TIMEOUT)).unwrap();
            server
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let handle = thread::spawn(move || {
                let mut client = Some(client);
                let mut attempts = 0;
                worker_with_connector(shared, Duration::from_millis(10), |_| {
                    attempts += 1;
                    if attempts == 1 {
                        return Err(io::Error::other("Discord is closed"));
                    }
                    Ok(client.take().unwrap())
                });
                attempts
            });
            let (_, payload) = receive(&mut server).unwrap();
            assert!(
                payload["args"]["activity"]["state"]
                    .as_str()
                    .unwrap()
                    .contains("PORCELAIN ARCHDEMON // sdvx sector // 12 rites")
            );
            // Change configuration during I/O, exercising the lost-wake guard.
            {
                let mut state = presence.shared.0.lock().unwrap();
                if shutdown {
                    state.stop = true;
                } else {
                    state.id = None;
                }
            }
            presence.shared.1.notify_one();
            send(
                &mut server,
                1,
                &json!({"cmd": "SET_ACTIVITY", "nonce": payload["nonce"], "evt": null}),
            )
            .unwrap();
            let (_, clear) = receive(&mut server).unwrap();
            assert!(clear["args"]["activity"].is_null());
            send(
                &mut server,
                1,
                &json!({"cmd": "SET_ACTIVITY", "nonce": clear["nonce"], "evt": null}),
            )
            .unwrap();
            presence.shared.0.lock().unwrap().stop = true;
            presence.shared.1.notify_one();
            assert_eq!(handle.join().unwrap(), 2);
        }
    }

    #[test]
    #[cfg(unix)]
    fn ipc_rejects_errors_and_oversized_frames() {
        let (mut client, mut server) = IpcStream::pair().unwrap();
        send(&mut server, 1, &json!({"evt": "ERROR", "nonce": "1"})).unwrap();
        assert!(set_activity(&mut client, Value::Null, 1).is_err());
        server.write_all(&1u32.to_le_bytes()).unwrap();
        server.write_all(&65537u32.to_le_bytes()).unwrap();
        assert!(receive(&mut client).is_err());
    }
}
