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
struct Snapshot {
    scene: String,
    poopsec: bool,
    operations: u64,
}

struct State {
    id: Option<String>,
    snapshot: Snapshot,
    status: String,
    stop: bool,
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
                }),
                Condvar::new(),
            )),
            worker: None,
        }
    }

    pub fn update(&self, scene: &str, poopsec: bool, operations: u64) {
        self.shared.0.lock().unwrap().snapshot = Snapshot {
            scene: scene.into(),
            poopsec,
            operations,
        };
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
                    "usage: discord [status|on [APPLICATION_ID]|off]",
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
    json!({
        "details": TRANSMISSIONS[transmission % TRANSMISSIONS.len()],
        "state": format!("SIM | {clearance} // {} sector // {} rites sealed", snapshot.scene, snapshot.operations),
        "timestamps": { "start": started },
        "type": 0
    })
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
        let (id, snapshot, stop) = {
            let state = shared.0.lock().unwrap();
            (state.id.clone(), state.snapshot.clone(), state.stop)
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
                set_activity(
                    &mut connection.as_mut().unwrap().1,
                    activity(&snapshot, started, transmission),
                    nonce,
                )
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
