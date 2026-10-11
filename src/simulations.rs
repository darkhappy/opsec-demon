//! In-memory stage props. These never contact cloud services or send packets.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const MAX_OBJECTS: usize = 100;
const MAX_BYTES: usize = 4096;
const BIDO_DURATION: Duration = Duration::from_secs(3);

pub struct Bucket {
    objects: BTreeMap<String, String>,
    syncs: u64,
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            objects: BTreeMap::from([
                (
                    "igloo-manifest.txt".into(),
                    "SIM: igloo lights online; dance floor armed.".into(),
                ),
                (
                    "penguin-passport.txt".into(),
                    "SIM: callsign WADDLE-01; clearance snowflake.".into(),
                ),
            ]),
            syncs: 0,
        }
    }

    pub fn status(&self) -> String {
        format!(
            "SIM B2 / club-penguin / {} objects / {} bytes / {} fake syncs / memory only",
            self.objects.len(),
            self.objects.values().map(String::len).sum::<usize>(),
            self.syncs
        )
    }

    /// Args begin with the subcommand, e.g. ["put", "igloo.txt", "hello"].
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        let verb = args.first().map(String::as_str).unwrap_or("status");
        match verb {
            "status" if args.len() <= 1 => Ok(vec![self.status()]),
            "ls" if args.len() == 1 => {
                let mut lines = vec!["SIM B2 / club-penguin / object inventory".into()];
                lines.extend(
                    self.objects
                        .iter()
                        .map(|(name, body)| format!("SIM {:>4} B  {name}", body.len())),
                );
                if self.objects.is_empty() {
                    lines.push("SIM bucket is empty".into());
                }
                Ok(lines)
            }
            "put" if args.len() >= 3 => {
                validate_name(&args[1])?;
                let body = args[2..].join(" ");
                if body.len() > MAX_BYTES {
                    return Err(format!("SIM object exceeds {MAX_BYTES} bytes"));
                }
                if !self.objects.contains_key(&args[1]) && self.objects.len() >= MAX_OBJECTS {
                    return Err(format!("SIM bucket is full ({MAX_OBJECTS} objects)"));
                }
                let bytes = body.len();
                self.objects.insert(args[1].clone(), body);
                Ok(vec![format!(
                    "SIM stored club-penguin/{} ({bytes} bytes; memory only)",
                    args[1]
                )])
            }
            "get" if args.len() == 2 => {
                validate_name(&args[1])?;
                let body = self
                    .objects
                    .get(&args[1])
                    .ok_or_else(|| format!("SIM object not found: {}", args[1]))?;
                // Escape controls so mock content cannot inject terminal commands.
                Ok(vec![format!(
                    "SIM club-penguin/{}: {}",
                    args[1],
                    body.escape_debug()
                )])
            }
            "rm" if args.len() == 2 => {
                validate_name(&args[1])?;
                self.objects
                    .remove(&args[1])
                    .ok_or_else(|| format!("SIM object not found: {}", args[1]))?;
                Ok(vec![format!(
                    "SIM removed club-penguin/{} from memory",
                    args[1]
                )])
            }
            "sync" if args.len() == 1 => {
                self.syncs = self.syncs.saturating_add(1);
                Ok(vec![
                    "SIM snowflake handshake -> igloo relay -> imaginary B2 vault".into(),
                    format!(
                        "SIM fake sync #{} complete / {} mock objects / no network traffic",
                        self.syncs,
                        self.objects.len()
                    ),
                ])
            }
            _ => Err(
                "SIM usage: bucket status | ls | put NAME TEXT | get NAME | rm NAME | sync".into(),
            ),
        }
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 128
        || name == "."
        || name.contains("..")
        || name
            .bytes()
            .any(|byte| !(b' '..=b'~').contains(&byte) || matches!(byte, b'/' | b'\\'))
        || name.trim() != name
    {
        return Err("SIM object names must be 1-128 printable ASCII characters, without slashes or traversal".into());
    }
    Ok(())
}

pub struct Bido {
    started: Option<Instant>,
}

impl Bido {
    pub fn new() -> Self {
        Self { started: None }
    }

    pub fn activate(&mut self) -> String {
        self.started = Some(Instant::now());
        "SIM LITTLE BIDO TECHNIQUE / decoy shuffle / tiny packet sidestep / 3s fictional maneuver; no packets sent".into()
    }

    pub fn is_active(&self) -> bool {
        self.started
            .is_some_and(|start| start.elapsed() < BIDO_DURATION)
    }

    pub fn status(&self) -> String {
        match self
            .started
            .and_then(|start| BIDO_DURATION.checked_sub(start.elapsed()))
        {
            Some(remaining) => format!(
                "SIM LITTLE BIDO TECHNIQUE / decoy shuffle ACTIVE / {:.1}s / fictional only",
                remaining.as_secs_f64()
            ),
            None => "SIM LITTLE BIDO TECHNIQUE / IDLE / fictional maneuver ready".into(),
        }
    }
}

/// Fictional milk-powered mesh; no Tailscale software or network is queried.
pub struct MilkMesh {
    started: Option<Instant>,
}

impl MilkMesh {
    pub fn new() -> Self {
        Self { started: None }
    }

    pub fn status(&self) -> String {
        let mode = if self
            .started
            .is_some_and(|start| start.elapsed() < BIDO_DURATION)
        {
            "CHURNING"
        } else {
            "PASTEURIZED"
        };
        format!("SIM TAILSCALE MILK / {mode} / 3 imaginary peers / dairy mesh")
    }

    /// Args follow the tailscale command prefix.
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args.first().map(String::as_str).unwrap_or("milk") {
            "milk" | "status" if args.len() <= 1 => Ok(vec![
                self.status(),
                "SIM milk tunnel is fictional; no real Tailscale connection".into(),
            ]),
            "routes" if args.len() == 1 => Ok(vec![
                "SIM imaginary routes / cow-node -> mainframe -> club-penguin".into(),
                "SIM cow-node: froth relay / mainframe: dairy core / club-penguin: chilled igloo"
                    .into(),
            ]),
            "churn" if args.len() == 1 => {
                self.started = Some(Instant::now());
                Ok(vec![
                    "SIM TAILSCALE MILK / CHURN initiated / 3s butterstorm / no network traffic"
                        .into(),
                ])
            }
            _ => Err("SIM usage: tailscale milk | status | routes | churn".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).into()).collect()
    }

    #[test]
    fn mock_objects_roundtrip_replace_and_remove() {
        let mut bucket = Bucket::new();
        bucket
            .command(&args(&["put", "note.txt", "ice", "cold"]))
            .unwrap();
        assert_eq!(
            bucket.command(&args(&["get", "note.txt"])).unwrap(),
            ["SIM club-penguin/note.txt: ice cold"]
        );
        bucket.command(&args(&["put", "note.txt", "warm"])).unwrap();
        assert_eq!(bucket.objects["note.txt"], "warm");
        bucket.command(&args(&["rm", "note.txt"])).unwrap();
        assert!(bucket.command(&args(&["get", "note.txt"])).is_err());
    }

    #[test]
    fn rejects_unsafe_names_and_does_not_mutate() {
        let mut bucket = Bucket::new();
        for name in [
            "", ".", "..", "../key", "a/b", "a\\b", "line\n", "café", "\x1b[2J", " leading",
        ] {
            assert!(
                bucket.command(&args(&["put", name, "text"])).is_err(),
                "{name:?}"
            );
        }
        assert_eq!(bucket.objects.len(), 2);
        bucket
            .command(&args(&["put", "controls.txt", "\x1b[2J\n"]))
            .unwrap();
        let output = bucket.command(&args(&["get", "controls.txt"])).unwrap();
        assert!(!output[0].contains('\x1b'));
        assert!(!output[0].contains('\n'));
    }

    #[test]
    fn enforces_object_and_byte_caps_but_allows_replacement() {
        let mut bucket = Bucket::new();
        for n in 2..MAX_OBJECTS {
            bucket
                .command(&args(&["put", &format!("mock-{n}"), "x"]))
                .unwrap();
        }
        assert!(bucket.command(&args(&["put", "overflow", "x"])).is_err());
        bucket
            .command(&args(&["put", "mock-2", &"x".repeat(MAX_BYTES)]))
            .unwrap();
        assert!(
            bucket
                .command(&args(&["put", "mock-2", &"x".repeat(MAX_BYTES + 1)]))
                .is_err()
        );
        assert_eq!(bucket.objects["mock-2"].len(), MAX_BYTES);
        assert!(
            bucket
                .command(&args(&["put", "mock-2", &"é".repeat(MAX_BYTES / 2 + 1)]))
                .is_err()
        );
    }

    #[test]
    fn sync_is_only_simulated_and_inventory_is_stable() {
        let mut bucket = Bucket::new();
        let before = bucket.objects.clone();
        let output = bucket.command(&args(&["sync"])).unwrap();
        assert!(output.iter().all(|line| line.starts_with("SIM")));
        assert_eq!(bucket.objects, before);
        assert_eq!(bucket.syncs, 1);
        assert_eq!(
            bucket.command(&args(&["ls"])).unwrap(),
            Bucket::new().command(&args(&["ls"])).unwrap()
        );
        assert!(bucket.command(&args(&["sync", "extra"])).is_err());
    }

    #[test]
    fn bido_expires_and_can_be_reactivated() {
        let mut bido = Bido::new();
        assert!(bido.status().contains("IDLE"));
        assert!(bido.activate().starts_with("SIM"));
        assert!(bido.status().contains("ACTIVE"));
        assert!(bido.is_active());
        bido.started = Some(Instant::now() - Duration::from_secs(4));
        assert!(bido.status().contains("IDLE"));
        assert!(!bido.is_active());
        bido.activate();
        assert!(bido.status().contains("ACTIVE"));
    }

    #[test]
    fn milk_mesh_is_labeled_sim_and_churn_expires() {
        let mut mesh = MilkMesh::new();
        for command in ["milk", "status", "routes", "churn"] {
            assert!(
                mesh.command(&args(&[command]))
                    .unwrap()
                    .iter()
                    .all(|line| line.starts_with("SIM"))
            );
        }
        assert!(mesh.status().contains("CHURNING"));
        mesh.started = Some(Instant::now() - Duration::from_secs(4));
        assert!(mesh.status().contains("PASTEURIZED"));
        assert!(mesh.command(&args(&["login"])).is_err());
        assert!(mesh.command(&args(&["milk", "extra"])).is_err());
    }
}
