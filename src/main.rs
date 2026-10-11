mod advancedttk;
mod arcade;
mod completion;
mod countries;
#[cfg(windows)]
mod desktop;
#[cfg(windows)]
mod desktop_audio;
#[cfg(windows)]
mod desktop_gpu;
#[cfg(windows)]
mod desktop_session;
#[cfg(windows)]
mod desktop_settings;
mod discord;
mod discord_profile;
#[cfg(windows)]
mod discord_windows;
mod dsp;
mod expansion;
mod job;
mod metrics;
mod music;
mod novel;
mod opsecular;
mod prompt;
mod rips;
mod ritual;
mod room;
mod simulations;
mod soundtrack;
mod ssd;
mod stage;
mod theater;
mod tui;
mod yoggies;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

const HELP: &str = "OPSEC DEMON // command uplink
  opsecular [desktop|boot|terminal|wsl|recycle|operator|spirit|support|poutine|weep] SIM OS
  help                   show this console
  rips [list|siiva|render ID FILE.wav] original album and SiIvaGunner listening links
  discord [status|on [ID]|off|preview|load FILE|reset] local IPC, maximal SIM presence
  discord type playing|listening|watching|competing; display name|state|details
  discord timer elapsed|off|SECONDS  SIM ritual countdown (1..86400)
  novel [start|next|choose 1|2|status|reset] original Lost Signals story
  psyche                 fictional story-character analysis
  soundtrack [status|render FILE.wav|play [FILE.wav]] original demon score
  dac [status|list|serial] USB audio status and read-only serial discovery
  shell                  open your native interactive shell
  export NAME=value      persist environment for child commands
  unset NAME             remove a child environment variable
  ..advancedttk JSON      load a Conclave PvP combat calculation
  egg [start|status|hatch] manage a simulated Golden Goose Egg
  ritual                 show fictional Cursed Apple ritual status
  bido [status]          perform the invented Little Bido technique
  bucket [ls|put|get|rm|sync|status]  club-penguin B2 simulation
  tailscale [milk|routes|churn]     fictional dairy mesh
  yoggies [show|all|holidays|status|use [NUMBER]|refill] fictional seasonal berry tray
  expansion [status|expand|allocate GB|release GB]  512 GB virtual MCP bank
  mcp                    alias for the expansion simulator
  mode [demon|prostreet|vulcan|sdvx]    select the animated ASCII scene
  icons [on|off]         toggle Nerd Font icons
  doctor                 inspect terminal setup
  ssd [status|roll|explode] fictional IBM eBay SSD disaster prop
  rt60 [status|set W L H ABSORPTION] Sabine room model; room is an alias
  sdvx [status|knob SIDE DELTA|hit BUTTON] phantom rhythm controller
  sdvx play              TUI keyboard knobs/buttons; Enter returns
  graph room             show RT60 room-decay graph in the TUI
  coins [status|generate|mint|pump] $1000 paper $POOPSECDEMONLARP ledger
  focus [status|calculate GB] fictional RE5 car storage arithmetic
  hoi4 [list|select CODE|focus NAME|status] fictional country deck
  band [on|off|status|drummer explode] silent ASCII Beatles tribute
  pedal [stomp|release|status] Ctrl-G stomps the phantom larp pedal
  headphones [list|select ath-r70xa|select hd58x|status] unmeasured models
  graph metrics          show actual session counters in the TUI
  graph headphones       show the invented headphone EQ profile
  status                 session and uplink telemetry
  cinematic              replay the demon awakening
  callsign [NAME]        claim your operator identity
  dossier                view your fictional operator profile
  mascot [demon|mo-krill|chinese-demon|tweeter|krillion] select your ASCII companion
  poopsec [on|off]       toggle the porcelain demon persona
  mission [next]         read or advance the story briefing
  oracle                 receive a transmission from the machine
  matrix [1..64]         print decorative signal rain (default: 12 rows)
  theme [crimson|ghost|venom]
                         change your console palette
  mainframe local        connect to the built-in in-process simulator
  mainframe connect IP:PORT
                         open a real TCP connection (no TLS)
  mainframe ping         verify the local simulator with an echo round trip
  mainframe disconnect   close the uplink
  cd [PATH]              change directory (quotes and ~/ supported)
  pwd                    print working directory
  history                show this session's commands; never saved to disk
  clear                  clear the terminal
  exit [0..255]          leave the shell
  Other commands: /bin/sh on Unix; cmd.exe /D /S /C on Windows (real execution).
  TUI: music [status|live|demo], graph next, shell COMMAND for interactive apps.
  TUI keys: Ctrl-F graph, Ctrl-T Mo & Krill, Ctrl-X explosion, PgUp/PgDn output.
  This is a themed shell, not an anonymity or security boundary.";

enum Uplink {
    Local {
        requests: Sender<Vec<u8>>,
        replies: Receiver<Vec<u8>>,
    },
    Remote(TcpStream),
}

impl Uplink {
    fn connect(address: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        stream.set_nodelay(true)?;
        Ok(Self::Remote(stream))
    }

    fn local() -> io::Result<Self> {
        let (requests, incoming) = mpsc::channel::<Vec<u8>>();
        let (outgoing, replies) = mpsc::channel();
        thread::Builder::new()
            .name("mainframe-simulator".into())
            .spawn(move || {
                while let Ok(payload) = incoming.recv() {
                    if outgoing.send(payload).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self::Local { requests, replies })
    }

    fn ping(&mut self) -> io::Result<Duration> {
        let Self::Local { requests, replies } = self else {
            return Err(io::Error::other(
                "ping is only supported by the local simulator",
            ));
        };
        let started = Instant::now();
        let payload = b"DEMON::UPLINK::CHECK";
        requests
            .send(payload.to_vec())
            .map_err(|_| io::Error::other("simulator disconnected"))?;
        let response = replies
            .recv_timeout(Duration::from_secs(2))
            .map_err(io::Error::other)?;
        if response != payload {
            return Err(io::Error::other("mainframe echo mismatch"));
        }
        Ok(started.elapsed())
    }

    fn description(&self) -> String {
        match self {
            Self::Local { .. } => "local simulator · in-process echo worker".into(),
            Self::Remote(stream) => format!(
                "{} · TCP socket open · application unverified · no TLS",
                stream
                    .peer_addr()
                    .map_or_else(|_| "unknown peer".into(), |a| a.to_string())
            ),
        }
    }
}

struct Shell {
    color: bool,
    uplink: Option<Uplink>,
    history: Vec<String>,
    started: Instant,
    last_status: i32,
    exit: Option<u8>,
    callsign: String,
    theme: String,
    operations: u64,
    mission: usize,
    poopsec: bool,
    mascot: String,
    transcript: RefCell<Option<Vec<String>>>,
    egg: ritual::Egg,
    ritual: ritual::Ritual,
    bucket: simulations::Bucket,
    bido: simulations::Bido,
    milk: simulations::MilkMesh,
    yoggies: yoggies::Yoggies,
    nerd_font: bool,
    expansion: expansion::Expansion,
    scene: String,
    ssd: ssd::Ssd,
    room: room::Room,
    arcade: arcade::Arcade,
    environment: BTreeMap<String, Option<String>>,
    previous_dir: Option<PathBuf>,
    countries: countries::Countries,
    stage: stage::Stage,
    novel: novel::Novel,
    discord: discord::Presence,
    desktop: opsecular::Desktop,
}

macro_rules! out {
    ($shell:expr, $($arg:tt)*) => {{ let message = format!($($arg)*); $shell.emit(message) }};
}

impl Shell {
    fn emit(&self, message: String) {
        if let Some(lines) = self.transcript.borrow_mut().as_mut() {
            lines.extend(message.lines().take(1000).map(String::from));
        } else {
            println!("{message}");
        }
    }

    fn error(&self, message: &str) {
        if self.transcript.borrow().is_some() {
            self.emit(message.into());
        } else {
            eprintln!("{message}");
        }
    }

    fn new(color: bool) -> Self {
        Self {
            color,
            uplink: None,
            history: Vec::new(),
            started: Instant::now(),
            last_status: 0,
            exit: None,
            callsign: "ZERO-DAY-SAINT".into(),
            theme: "crimson".into(),
            operations: 0,
            mission: 0,
            poopsec: false,
            mascot: "demon".into(),
            transcript: RefCell::new(None),
            egg: ritual::Egg::new(),
            ritual: ritual::Ritual::new(),
            bucket: simulations::Bucket::new(),
            bido: simulations::Bido::new(),
            milk: simulations::MilkMesh::new(),
            yoggies: yoggies::Yoggies::new(),
            nerd_font: false,
            expansion: expansion::Expansion::new(),
            scene: "demon".into(),
            ssd: ssd::Ssd::new(),
            room: room::Room::new(),
            arcade: arcade::Arcade::new(),
            environment: BTreeMap::new(),
            previous_dir: None,
            countries: countries::Countries::new(),
            stage: stage::Stage::new(),
            novel: novel::Novel::default(),
            discord: discord::Presence::new(),
            desktop: opsecular::Desktop::default(),
        }
    }

    fn paint(&self, code: &str, message: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{message}\x1b[0m")
        } else {
            message.to_string()
        }
    }

    fn accent(&self) -> &'static str {
        if self.poopsec {
            return "1;38;5;214";
        }
        match self.theme.as_str() {
            "ghost" => "1;38;5;51",
            "venom" => "1;38;5;82",
            _ => "1;38;5;196",
        }
    }

    fn banner(&self) {
        out!(
            self,
            "{}",
            self.paint(
                self.accent(),
                if self.poopsec {
                    theater::POOP_SIGIL
                } else {
                    theater::SIGIL
                }
            )
        );
        out!(
            self,
            "{}",
            self.paint(
                "2",
                "  ── FICTIONAL OPERATOR CONSOLE // REAL COMMAND EXECUTION ──"
            )
        );
    }

    fn cinematic(&self, animated: bool) -> io::Result<()> {
        self.banner();
        theater::boot(animated, self.color, self.accent(), self.poopsec)?;
        out!(
            self,
            "\n{}",
            self.paint(
                self.accent(),
                &format!("  WELCOME BACK, {}.", self.callsign)
            )
        );
        out!(self, "  The cursor blinks. The city holds its breath.");
        out!(self, "  Somewhere in the wires, a machine remembers you.");
        out!(
            self,
            "  You are the operator they wrote the warning about.\n"
        );
        Ok(())
    }

    fn dossier(&self) {
        out!(
            self,
            "{}",
            self.paint(self.accent(), "╔══ OPERATOR DOSSIER // FICTION ══╗")
        );
        out!(self, "  CALLSIGN    {}", self.callsign);
        out!(
            self,
            "  CLASS       {}",
            if self.poopsec {
                "PORCELAIN ARCHDEMON"
            } else {
                theater::rank(self.operations)
            }
        );
        out!(
            self,
            "  MODE        {}",
            if self.poopsec {
                "POOPSEC // SEWER MAINFRAME DIVISION"
            } else {
                "OPSEC // GHOST CONSOLE"
            }
        );
        out!(self, "  ALIGNMENT   CHAOTIC TERMINAL");
        out!(self, "  SIGNATURE   NATIVE RUST / NO CRATE DEPENDENCIES");
        out!(
            self,
            "  OPERATIONS  {} successful commands",
            self.operations
        );
        out!(
            self,
            "  STORY ARC   {} / {}",
            self.mission + 1,
            theater::MISSIONS.len()
        );
        out!(self, "  WANTED FOR  EXCESSIVE PROTAGONIST ENERGY");
        out!(self, "  THREAT      CINEMATICALLY IMMEASURABLE");
        out!(
            self,
            "{}",
            self.paint(
                self.accent(),
                "╚══ THE LEGEND IS LOCAL. THE COMMANDS ARE REAL. ══╝"
            )
        );
    }

    fn briefing(&self) {
        let (title, objective) = theater::MISSIONS[self.mission];
        out!(self, "{}", self.paint(self.accent(), title));
        out!(self, "  {objective}");
        out!(self, "  [STORY MODE] Advance manually with mission next.");
    }

    fn local(&mut self) -> io::Result<()> {
        let mut uplink = Uplink::local()?;
        let latency = uplink.ping()?;
        out!(
            self,
            "{} {:.3} ms · local simulator",
            self.paint(
                "1;92",
                "[MAINFRAME] CONNECTION ESTABLISHED // ECHO VERIFIED"
            ),
            latency.as_secs_f64() * 1000.0
        );
        self.uplink = Some(uplink);
        Ok(())
    }

    fn status(&self) {
        out!(
            self,
            "{}",
            self.paint(self.accent(), "[BLACKBOX] DEMON CORE // LIVE SESSION")
        );
        out!(
            self,
            "  operator   {} · {}",
            self.callsign,
            theater::rank(self.operations)
        );
        out!(
            self,
            "  session    {} · uptime {:.1}s",
            std::process::id(),
            self.started.elapsed().as_secs_f64()
        );
        out!(self, "  directory  {}", cwd().display());
        out!(self, "  last exit  {}", self.last_status);
        out!(
            self,
            "  history    {} commands · memory only",
            self.history.len()
        );
        if let Some(uplink) = &self.uplink {
            out!(self, "  mainframe  {}", uplink.description());
        } else {
            out!(self, "  mainframe  OFFLINE");
        }
    }

    fn prompt(&self) -> io::Result<()> {
        let directory = cwd();
        let directory = directory
            .file_name()
            .unwrap_or(directory.as_os_str())
            .to_string_lossy();
        let link = match self.uplink {
            Some(Uplink::Local { .. }) => "SIM:LINK",
            Some(Uplink::Remote(_)) => "TCP:LINK",
            None => "OFFLINE",
        };
        print!(
            "\n{} {} {}\n{} ",
            self.paint(
                self.accent(),
                &format!(
                    "╭─[{} // {}]",
                    self.callsign,
                    theater::rank(self.operations)
                )
            ),
            self.paint("36", &directory),
            self.paint(
                if self.last_status == 0 { "92" } else { "91" },
                &format!("[{link} | {}]", self.last_status)
            ),
            self.paint(self.accent(), "╰─[EXECUTE] ❯")
        );
        io::stdout().flush()
    }

    fn apply_environment(&self, command: &mut Command) {
        for (key, value) in &self.environment {
            if let Some(value) = value {
                command.env(key, value);
            } else {
                command.env_remove(key);
            }
        }
        command.env("PWD", cwd());
    }

    fn command(&self, line: &str) -> Command {
        #[cfg(unix)]
        let mut command = Command::new("/bin/sh");
        #[cfg(unix)]
        command.args(["-c", line]);
        #[cfg(windows)]
        let mut command = Command::new("cmd.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.raw_arg(format!("/D /S /C \"{line}\""));
        }
        self.apply_environment(&mut command);
        command
    }

    fn interactive_shell(&self) -> io::Result<i32> {
        #[cfg(unix)]
        let program = self
            .environment
            .get("SHELL")
            .and_then(Clone::clone)
            .or_else(|| env::var("SHELL").ok())
            .unwrap_or_else(|| "/bin/sh".into());
        #[cfg(unix)]
        let mut command = Command::new(program);
        #[cfg(unix)]
        command.arg("-i");
        #[cfg(windows)]
        let mut command = Command::new("cmd.exe");
        #[cfg(windows)]
        command.arg("/D");
        self.apply_environment(&mut command);
        Ok(command.status()?.code().unwrap_or(130))
    }

    fn soundtrack_play_command(&self, args: &[String]) -> io::Result<Command> {
        let path = match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["play"] => "assets/audio/nightfall.wav",
            ["play", path] => path,
            _ => return Err(io::Error::other("usage: soundtrack play [FILE.wav]")),
        };
        let device = match self.environment.get("OPSEC_DEMON_AUDIO_DEVICE") {
            Some(value) => value.clone(),
            None => env::var("OPSEC_DEMON_AUDIO_DEVICE").ok(),
        };
        #[cfg(windows)]
        let mut command = if args.len() == 1 {
            soundtrack::bundled_player(device.as_deref())?
        } else {
            soundtrack::player(path, device.as_deref())?
        };
        #[cfg(not(windows))]
        let mut command = soundtrack::player(path, device.as_deref())?;
        self.apply_environment(&mut command);
        Ok(command)
    }

    fn execute(&mut self, line: &str) -> i32 {
        let status = self.execute_inner(line);
        if status == 0 && !line.trim().is_empty() {
            self.operations = self.operations.saturating_add(1);
        }
        self.update_discord();
        status
    }

    fn update_discord(&self) {
        self.discord.update_snapshot(discord::Snapshot {
            scene: self.scene.clone(),
            poopsec: self.poopsec,
            operations: self.operations,
            mascot: self.mascot.clone(),
            mission: self.mission,
            yoggies: self.yoggies.presence_counts(),
            lore: vec![
                self.yoggies.status(),
                self.egg.status(),
                self.arcade.status(),
                self.stage.status(),
                self.expansion.status(),
                self.ritual.status(),
                self.ssd.line(),
                format!(
                    "MISSION {} // {}",
                    self.mission + 1,
                    theater::MISSIONS[self.mission].0
                ),
            ],
        });
    }

    fn execute_inner(&mut self, line: &str) -> i32 {
        if line.trim().is_empty() {
            return self.last_status;
        }
        if line.split_whitespace().next() == Some("..advancedttk") {
            let payload = line
                .trim_start()
                .strip_prefix("..advancedttk")
                .unwrap_or("")
                .trim();
            return match advancedttk::run(payload) {
                Ok(lines) => {
                    for message in lines {
                        out!(self, "{message}");
                    }
                    0
                }
                Err(error) => {
                    self.error(&format!("[TTK] {error}"));
                    1
                }
            };
        }
        if line.trim() == "shell" {
            return self.interactive_shell().unwrap_or_else(|error| {
                self.error(&format!("[SHELL] {error}"));
                127
            });
        }
        if !is_builtin(line) {
            return match self.command(line).status() {
                Ok(status) => status.code().unwrap_or(128),
                Err(error) => {
                    self.error(&format!("[EXEC] {error}"));
                    127
                }
            };
        }
        match words(line).and_then(|args| self.builtin(&args)) {
            Ok(()) => 0,
            Err(error) => {
                self.error(&format!("[DEMON] {error}"));
                1
            }
        }
    }

    fn builtin(&mut self, args: &[String]) -> io::Result<()> {
        if let Some(event) = self.egg.tick() {
            out!(self, "{event}");
        }
        match args[0].as_str() {
            "rips" => match args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["rips"] | ["rips", "list"] => out!(self, "{}", rips::catalog()),
                ["rips", "siiva"] => out!(self, "{}", rips::siiva_catalog()),
                ["rips", "render", id, path] => {
                    let id = id.parse::<usize>().map_err(io::Error::other)?;
                    rips::render(id, path)?;
                    out!(self, "[SYNTH AUDIO] Rendered track {id} to {path}");
                }
                _ => {
                    return Err(io::Error::other(
                        "usage: rips [list|siiva|render ID FILE.wav]",
                    ));
                }
            },
            "opsecular" => out!(
                self,
                "{}",
                self.desktop.command(&args[1..]).map_err(io::Error::other)?
            ),
            "help" if args.len() == 1 => out!(self, "{HELP}"),
            "discord" => {
                self.update_discord();
                let id = self
                    .environment
                    .get("DISCORD_APPLICATION_ID")
                    .cloned()
                    .unwrap_or_else(|| env::var("DISCORD_APPLICATION_ID").ok());
                out!(self, "{}", self.discord.command(&args[1..], id)?);
            }
            "soundtrack" => match args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["soundtrack"] | ["soundtrack", "status"] => out!(self, "{}", soundtrack::info()),
                ["soundtrack", "render", path] => {
                    soundtrack::render(path)?;
                    out!(self, "[SOUNDTRACK] wrote original PCM score to {path}");
                }
                ["soundtrack", "play"] | ["soundtrack", "play", _] => {
                    let status = self.soundtrack_play_command(&args[1..])?.status()?;
                    if !status.success() {
                        return Err(io::Error::other("audio player exited unsuccessfully"));
                    }
                }
                _ => {
                    return Err(io::Error::other(
                        "usage: soundtrack [status|render FILE.wav|play [FILE.wav]]",
                    ));
                }
            },
            "dac" if args.len() <= 2 => match args.get(1).map(String::as_str) {
                None | Some("status" | "list") => out!(self, "{}", soundtrack::dac_status(false)),
                Some("serial") => out!(self, "{}", soundtrack::dac_status(true)),
                _ => return Err(io::Error::other("usage: dac [status|list|serial]")),
            },
            "novel" => out!(
                self,
                "{}",
                self.novel.command(&args[1..]).map_err(io::Error::other)?
            ),
            "psyche" if args.len() == 1 => out!(self, "{}", self.novel.psyche()),
            "status" if args.len() == 1 => {
                self.status();
                out!(self, "{}", self.stage.status());
            }
            "doctor" if args.len() == 1 => {
                out!(
                    self,
                    "[TERMINAL] {} / {}",
                    env::var("TERM_PROGRAM").unwrap_or_else(|_| "unknown emulator".into()),
                    env::var("TERM").unwrap_or_else(|_| "TERM unset".into())
                );
                out!(
                    self,
                    "[DISPLAY] 120x40+ recommended for all four response plots; ANSI 256 colors; ~30fps target"
                );
                out!(
                    self,
                    "[FONT] Nerd Font Mono icons {} / select JetBrainsMono Nerd Font Mono in terminal settings",
                    if self.nerd_font {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                out!(self, "[MUSIC] {}", music::Music::credentials());
                out!(self, "[HOST] {} / {}", env::consts::OS, env::consts::ARCH);
            }
            "band" | "pedal" | "headphones" => {
                for line in self.stage.command(args).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
            }
            "hoi4" => {
                self.countries.tick();
                for line in self
                    .countries
                    .command(&args[1..])
                    .map_err(io::Error::other)?
                {
                    out!(self, "{line}");
                }
            }
            "rt60" | "room" => {
                for line in self.room.command(&args[1..]).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
            }
            "sdvx" | "coins" | "focus" => {
                for line in self.arcade.command(args).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
                if args[0] == "sdvx" {
                    self.scene = "sdvx".into();
                }
            }
            "ssd" => {
                self.ssd.tick();
                for line in self.ssd.command(&args[1..]).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
            }
            "icons" if args.len() <= 2 => {
                self.nerd_font = match args.get(1).map(String::as_str) {
                    None => !self.nerd_font,
                    Some("on") => true,
                    Some("off") => false,
                    _ => return Err(io::Error::other("usage: icons [on|off]")),
                };
                out!(
                    self,
                    "[ICONS] {} / requires a Nerd Font Mono in your terminal",
                    if self.nerd_font { "on" } else { "off" }
                );
            }
            "bido" if args.len() <= 2 => {
                let message = match args.get(1).map(String::as_str) {
                    None => self.bido.activate(),
                    Some("status") => self.bido.status(),
                    _ => return Err(io::Error::other("usage: bido [status]")),
                };
                out!(self, "{message}");
            }
            "bucket" => {
                for line in self.bucket.command(&args[1..]).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
            }
            "yoggies" => {
                for line in self.yoggies.command(&args[1..]).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
            }
            "tailscale" => {
                for line in self.milk.command(&args[1..]).map_err(io::Error::other)? {
                    out!(self, "{line}");
                }
            }
            "expansion" | "mcp" => {
                for line in self
                    .expansion
                    .command(&args[1..])
                    .map_err(io::Error::other)?
                {
                    out!(self, "{line}");
                }
            }
            "mode" if args.len() <= 2 => {
                if let Some(scene) = args.get(1) {
                    if !matches!(scene.as_str(), "demon" | "prostreet" | "vulcan" | "sdvx") {
                        return Err(io::Error::other(
                            "usage: mode [demon|prostreet|vulcan|sdvx]",
                        ));
                    }
                    self.scene = scene.clone();
                }
                out!(self, "[SIM SCENE] {} // ASCII theater online", self.scene);
            }
            "ritual" if args.len() == 1 => out!(self, "{}", self.ritual.status()),
            "egg" if args.len() <= 2 => match args.get(1).map(String::as_str) {
                None | Some("status") => out!(self, "{}", self.egg.status()),
                Some("start") => out!(self, "{}", self.egg.start().map_err(io::Error::other)?),
                Some("hatch") => out!(self, "{}", self.egg.hatch().map_err(io::Error::other)?),
                _ => return Err(io::Error::other("usage: egg [start|status|hatch]")),
            },
            "cinematic" if args.len() == 1 => self.cinematic(self.color)?,
            "dossier" if args.len() == 1 => self.dossier(),
            "mascot" if args.len() <= 2 => {
                if let Some(name) = args.get(1) {
                    if !matches!(
                        name.as_str(),
                        "demon" | "mo-krill" | "chinese-demon" | "tweeter" | "krillion"
                    ) {
                        return Err(io::Error::other(
                            "mascot expects demon, mo-krill, chinese-demon, tweeter, or krillion",
                        ));
                    }
                    self.mascot = name.clone();
                }
                out!(self, "[MASCOT] {} // tunnel crew on standby", self.mascot);
                if self.mascot == "mo-krill" && self.yoggies.is_christmas() {
                    out!(
                        self,
                        "[SIM MASCOT] Santa Mo carries elf Krill and Christmas yoggies"
                    );
                    for line in theater::CHRISTMAS_MO_KRILL {
                        out!(self, "{line}");
                    }
                }
            }
            "poopsec" if args.len() <= 2 => {
                self.poopsec = match args.get(1).map(String::as_str) {
                    None => !self.poopsec,
                    Some("on") => true,
                    Some("off") => false,
                    _ => return Err(io::Error::other("usage: poopsec [on|off]")),
                };
                out!(
                    self,
                    "{}",
                    self.paint(
                        self.accent(),
                        if self.poopsec {
                            "[POOPSEC DEMON] PORCELAIN THRONE ACQUIRED // sewer lore online"
                        } else {
                            "[OPSEC DEMON] ghost console restored"
                        }
                    )
                );
            }
            "callsign" if args.len() <= 2 => {
                if let Some(name) = args.get(1) {
                    theater::validate_callsign(name)?;
                    self.callsign = name.to_ascii_uppercase();
                }
                out!(
                    self,
                    "[IDENTITY] {} // the mainframe knows your name",
                    self.callsign
                );
            }
            "theme" if args.len() <= 2 => {
                if let Some(theme) = args.get(1) {
                    if !matches!(theme.as_str(), "crimson" | "ghost" | "venom") {
                        return Err(io::Error::other("theme expects crimson, ghost, or venom"));
                    }
                    self.theme = theme.clone();
                }
                out!(
                    self,
                    "{}",
                    self.paint(self.accent(), &format!("[PALETTE] {}", self.theme))
                );
            }
            "mission" if args.len() == 1 || (args.len() == 2 && args[1] == "next") => {
                if args.len() == 2 {
                    self.mission = (self.mission + 1).min(theater::MISSIONS.len() - 1);
                }
                self.briefing();
            }
            "oracle" if args.len() == 1 => {
                let transmissions = if self.poopsec {
                    [
                        "I'M IN. THE SEWER MAINFRAME.",
                        "THE FIREWALL HAS BEEN FLUSHED. FICTIONALLY.",
                        "YOUR THREAT MODEL IS A PORCELAIN THRONE.",
                        "ROOT ACCESS? YOU MEAN SEWER ACCESS.",
                        "THE PACKETS ARE BROWN. THE LORE IS GOLD.",
                        "NO LOGS. ONLY LORE. AND A QUESTIONABLE SMELL.",
                    ]
                } else {
                    [
                        "THEY BUILT A FIREWALL. YOU BUILT A CHARACTER ARC.",
                        "THE MAINFRAME DOES NOT DREAM. UNTIL YOU LOG IN.",
                        "EVERY CURSOR IS A LOADED CHEKHOV'S GUN.",
                        "YOUR ORIGIN STORY COMPILES WITHOUT WARNINGS.",
                        "THE CITY HAS A MILLION TERMINALS. THIS ONE IS YOURS.",
                        "PROPHECY IS JUST A TODO WITH BETTER TYPOGRAPHY.",
                    ]
                };
                out!(
                    self,
                    "{}",
                    self.paint(self.accent(), "[INCOMING TRANSMISSION // FICTION]")
                );
                out!(
                    self,
                    "  {}",
                    transmissions[self.operations as usize % transmissions.len()]
                );
            }
            "matrix" if args.len() <= 2 => {
                let rows = args.get(1).map_or(Ok(12), |value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| io::Error::other("matrix expects 1–64 rows"))
                })?;
                if !(1..=64).contains(&rows) {
                    return Err(io::Error::other("matrix expects 1–64 rows"));
                }
                out!(self, "[SIGNAL RAIN // DECORATIVE]");
                out!(self, "{}", self.paint("92", &theater::matrix(rows)));
                io::stdout().flush()?;
            }
            "export" => {
                if args.len() == 1 {
                    for (key, value) in &self.environment {
                        if let Some(value) = value {
                            out!(self, "{key}={value}");
                        }
                    }
                } else {
                    let mut pending = Vec::new();
                    for entry in &args[1..] {
                        let (key, value) = entry.split_once('=').unwrap_or((entry, ""));
                        if !valid_variable(key) {
                            return Err(io::Error::other(
                                "export expects NAME=value with a valid variable name",
                            ));
                        }
                        let value = if entry.contains('=') {
                            value.to_string()
                        } else {
                            self.environment
                                .get(key)
                                .and_then(Clone::clone)
                                .or_else(|| env::var(key).ok())
                                .unwrap_or_default()
                        };
                        pending.push((key.to_string(), Some(value)));
                    }
                    self.environment.extend(pending);
                }
            }
            "unset" if args.len() >= 2 => {
                if args[1..].iter().any(|key| !valid_variable(key)) {
                    return Err(io::Error::other("unset expects valid variable names"));
                }
                for key in &args[1..] {
                    self.environment.insert(key.clone(), None);
                }
            }
            "pwd" if args.len() == 1 => out!(self, "{}", env::current_dir()?.display()),
            "history" if args.len() == 1 => {
                for (index, command) in self.history.iter().enumerate() {
                    out!(self, "{:>4}  {command}", index + 1);
                }
            }
            "clear" if args.len() == 1 => {
                if self.color {
                    print!("\x1b[2J\x1b[H");
                    io::stdout().flush()?;
                }
            }
            "cd" if args.len() <= 2 => {
                let home = || {
                    self.environment
                        .get("HOME")
                        .and_then(Clone::clone)
                        .map(PathBuf::from)
                        .or_else(|| env::var_os("HOME").map(PathBuf::from))
                        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
                        .ok_or_else(|| io::Error::other("HOME is not set"))
                };
                let path = match args.get(1).map(String::as_str) {
                    None | Some("~") => home()?,
                    Some("-") => self
                        .previous_dir
                        .clone()
                        .ok_or_else(|| io::Error::other("no previous directory"))?,
                    Some(path) if path.starts_with("~/") => home()?.join(&path[2..]),
                    Some(path) => PathBuf::from(path),
                };
                let previous = env::current_dir()?;
                env::set_current_dir(path)?;
                self.previous_dir = Some(previous);
                self.environment.insert(
                    "OLDPWD".into(),
                    self.previous_dir
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                );
            }
            "exit" if args.len() <= 2 => {
                self.exit = Some(match args.get(1) {
                    Some(code) => code
                        .parse::<u8>()
                        .map_err(|_| io::Error::other("exit expects a code between 0 and 255"))?,
                    None => self.last_status.clamp(0, 255) as u8,
                });
            }
            "mainframe" => match args.get(1).map(String::as_str) {
                None | Some("status") if args.len() <= 2 => self.status(),
                Some("local") if args.len() == 2 => self.local()?,
                Some("disconnect") if args.len() == 2 => {
                    self.uplink = None;
                    out!(self, "[MAINFRAME] disconnected");
                }
                Some("connect") if args.len() == 3 => {
                    let address = args[2]
                        .parse::<SocketAddr>()
                        .map_err(|_| io::Error::other("use a numeric IP:PORT, or [IPv6]:PORT"))?;
                    let uplink = Uplink::connect(address)?;
                    self.uplink = Some(uplink);
                    out!(
                        self,
                        "[MAINFRAME] TCP connection established to {address} · application unverified · no TLS"
                    );
                }
                Some("ping") if args.len() == 2 => {
                    let uplink = self.uplink.as_mut().ok_or_else(|| {
                        io::Error::other("mainframe offline; run mainframe local")
                    })?;
                    let latency = uplink.ping()?;
                    out!(
                        self,
                        "[MAINFRAME] echo verified · {:.3} ms",
                        latency.as_secs_f64() * 1000.0
                    );
                }
                _ => {
                    return Err(io::Error::other(
                        "usage: mainframe [status|local|ping|disconnect|connect IP:PORT]",
                    ));
                }
            },
            _ => return Err(io::Error::other("invalid arguments; run help")),
        }
        Ok(())
    }
}

fn is_builtin(line: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(mark) = quote {
            if character == mark {
                quote = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if matches!(character, '|' | '&' | ';' | '<' | '>' | '\n') {
            return false;
        }
    }
    matches!(
        line.split_whitespace().next().unwrap_or(""),
        "help"
            | "rips"
            | "discord"
            | "opsecular"
            | "soundtrack"
            | "dac"
            | "novel"
            | "psyche"
            | "..advancedttk"
            | "status"
            | "mainframe"
            | "cd"
            | "pwd"
            | "history"
            | "clear"
            | "exit"
            | "cinematic"
            | "callsign"
            | "dossier"
            | "mission"
            | "oracle"
            | "matrix"
            | "theme"
            | "poopsec"
            | "mascot"
            | "egg"
            | "ritual"
            | "bido"
            | "bucket"
            | "yoggies"
            | "tailscale"
            | "icons"
            | "doctor"
            | "expansion"
            | "mcp"
            | "mode"
            | "ssd"
            | "rt60"
            | "room"
            | "sdvx"
            | "coins"
            | "focus"
            | "export"
            | "unset"
            | "shell"
            | "hoi4"
            | "band"
            | "pedal"
            | "headphones"
    )
}

fn valid_variable(name: &str) -> bool {
    let mut chars = name.bytes();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == b'_')
}

fn cwd() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("?"))
}

fn words(line: &str) -> io::Result<Vec<String>> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut active = false;
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if escaped {
            word.push(character);
            escaped = false;
        } else if character == '\\' && quote != Some('\'') {
            if cfg!(windows)
                && characters
                    .peek()
                    .is_some_and(|next| !matches!(next, ' ' | '\'' | '"' | '\\'))
            {
                word.push(character);
            } else {
                escaped = true;
            }
            active = true;
        } else if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            } else {
                word.push(character);
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            active = true;
        } else if character.is_whitespace() {
            if active {
                result.push(std::mem::take(&mut word));
                active = false;
            }
        } else {
            word.push(character);
            active = true;
        }
    }
    if escaped || quote.is_some() {
        return Err(io::Error::other("unfinished quote or escape"));
    }
    if active {
        result.push(word);
    }
    Ok(result)
}

fn run() -> io::Result<u8> {
    let launch: Vec<String> = env::args().skip(1).collect();
    if launch.first().is_some_and(|arg| arg == "--metrics-server") {
        if launch.len() != 2 {
            return Err(io::Error::other(
                "usage: opsec-demon --metrics-server IP:PORT",
            ));
        }
        metrics::serve(&launch[1])?;
        return Ok(0);
    }
    let mut plain = false;
    let mut banner = true;
    let mut instant = false;
    let mut force_tui = false;
    let mut gui = false;
    let mut gui_smoke = false;
    let mut line_shell = false;
    let mut poopsec = false;
    let mut mo_krill = false;
    let mut start_egg = false;
    let mut nerd_font = false;
    let mut scene = "demon";
    let mut expand_mcp = false;
    let mut chinese_demon = false;
    let mut command = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--plain" => plain = true,
            "--no-banner" => banner = false,
            "--instant" => instant = true,
            "--tui" => force_tui = true,
            "--gui" => gui = true,
            "--gui-smoke" => {
                gui = true;
                gui_smoke = true;
            }
            "--shell" => line_shell = true,
            "--poopsec" => poopsec = true,
            "--mo-krill" => mo_krill = true,
            "--egg" => start_egg = true,
            "--nerd-font" => nerd_font = true,
            "--prostreet" => scene = "prostreet",
            "--vulcan" => scene = "vulcan",
            "--mcp-expansion" => expand_mcp = true,
            "--chinese-demon" => chinese_demon = true,
            "-c" => {
                command = Some(
                    args.next()
                        .ok_or_else(|| io::Error::other("-c requires a command"))?,
                );
            }
            "--help" | "-h" => {
                println!(
                    "Usage: opsec-demon [--plain] [--no-banner] [--instant] [--gui|--tui|--shell] [--poopsec] [--mo-krill|--chinese-demon] [--egg] [--nerd-font] [--prostreet|--vulcan] [--mcp-expansion] [-c COMMAND]\n       opsec-demon --metrics-server IP:PORT\n\n{HELP}"
                );
                return Ok(0);
            }
            "--version" | "-V" => {
                println!("opsec-demon {}", env!("CARGO_PKG_VERSION"));
                return Ok(0);
            }
            _ => return Err(io::Error::other(format!("unknown option: {argument}"))),
        }
    }
    if gui && (line_shell || force_tui || command.is_some()) {
        return Err(io::Error::other(
            "--gui cannot be combined with --shell, --tui, or -c",
        ));
    }
    if gui && !cfg!(windows) {
        return Err(io::Error::other("the native desktop requires Windows"));
    }
    if cfg!(windows) && force_tui {
        return Err(io::Error::other(
            "use --gui or --shell on Windows; the legacy TUI requires Unix",
        ));
    }
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal() && command.is_none();
    let color = interactive
        && !plain
        && env::var_os("NO_COLOR").is_none()
        && env::var("TERM").is_ok_and(|term| term != "dumb");
    if force_tui && (!interactive || line_shell) {
        return Err(io::Error::other(
            "--tui requires an interactive terminal and cannot be combined with --shell or -c",
        ));
    }
    let mut shell = Shell::new(color);
    shell.poopsec = poopsec;
    shell.nerd_font = nerd_font;
    shell.scene = scene.into();
    if expand_mcp {
        shell
            .expansion
            .command(&["expand".into()])
            .map_err(io::Error::other)?;
    }
    if start_egg {
        shell.egg.start().map_err(io::Error::other)?;
    }
    if mo_krill {
        shell.mascot = "mo-krill".into();
    }
    if chinese_demon {
        shell.mascot = "chinese-demon".into();
    }
    if let Some(command) = command {
        let status = shell.execute(&command);
        return Ok(shell.exit.unwrap_or(status.clamp(0, 255) as u8));
    }
    #[cfg(windows)]
    if gui || (!line_shell && !force_tui) {
        return desktop::run(shell, gui_smoke);
    }
    #[cfg(not(windows))]
    let _ = gui_smoke;
    shell.update_discord();
    if interactive
        && env::var_os("DISCORD_APPLICATION_ID").is_some()
        && let Err(error) = shell
            .discord
            .command(&["on".into()], env::var("DISCORD_APPLICATION_ID").ok())
    {
        shell.error(&format!("[DISCORD] {error}"));
    }
    if interactive
        && !line_shell
        && cfg!(unix)
        && (force_tui || env::var("TERM").is_ok_and(|term| term != "dumb"))
    {
        return tui::run(&mut shell, banner && !instant);
    }
    if interactive {
        if banner {
            shell.cinematic(!instant)?;
        }
        if let Err(error) = shell.local() {
            eprintln!("[MAINFRAME] local simulator unavailable: {error}");
        }
        if shell.uplink.is_some() {
            println!("\x07[INFILTRATION COMPLETE // SIMULATOR] SAY IT OUT LOUD: I'M IN.");
        }
        if banner {
            shell.briefing();
            println!("\n  dossier · oracle · matrix · theme venom · help");
        }
    }
    let stdin = io::stdin();
    loop {
        if interactive {
            shell.prompt()?;
        }
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if interactive && !line.trim().is_empty() {
            // Bound memory, and exclude space-prefixed commands from history.
            if !line.starts_with(' ') {
                if shell.history.len() == 1000 {
                    shell.history.remove(0);
                }
                shell.history.push(line.to_string());
            }
        }
        shell.last_status = shell.execute(line);
        if let Some(code) = shell.exit {
            return Ok(code);
        }
    }
    Ok(shell.last_status.clamp(0, 255) as u8)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("[DEMON] {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn windows_commands_preserve_quotes_pipes_status_and_session_environment() {
        let mut shell = Shell::new(false);
        shell.discord.update("demon", false, 0);
        assert_eq!(shell.execute("export OPSEC_WIN_TEST='hello twin'"), 0);
        let output = shell
            .command("echo %OPSEC_WIN_TEST%|findstr twin")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            "hello twin"
        );
        assert_eq!(shell.command("exit /B 7").status().unwrap().code(), Some(7));
        assert_eq!(
            words(r#"cd "C:\Users\operator\space folder""#).unwrap(),
            vec!["cd", r"C:\Users\operator\space folder"]
        );
        let output = shell.command(r#"echo "quoted text""#).output().unwrap();
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            r#""quoted text""#
        );
        shell.execute("unset OPSEC_WIN_TEST");
        let output = shell
            .command("if defined OPSEC_WIN_TEST (exit /B 1) else (exit /B 0)")
            .status()
            .unwrap();
        assert!(output.success());
    }

    #[test]
    fn builtin_arguments_preserve_quotes_escapes_and_empty_strings() {
        assert_eq!(
            words(r#"cd "space path" '' escaped\ space"#).unwrap(),
            vec!["cd", "space path", "", "escaped space"]
        );
        assert!(words("cd 'unfinished").is_err());
        assert!(words("cd trailing\\").is_err());
    }

    #[test]
    fn local_mainframe_verifies_worker_echo() {
        let mut uplink = Uplink::local().unwrap();
        uplink.ping().unwrap();
        uplink.ping().unwrap();
        assert!(uplink.description().contains("in-process"));
    }

    #[test]
    fn broken_simulator_reports_disconnect() {
        let (requests, incoming) = mpsc::channel();
        let (_, replies) = mpsc::channel();
        drop(incoming);
        let mut uplink = Uplink::Local { requests, replies };
        assert!(uplink.ping().is_err());
    }

    #[test]
    #[ignore = "requires permission to create TCP sockets; run cargo test -- --ignored on a network-enabled host"]
    fn external_uplink_does_not_send_simulator_protocol() {
        use std::io::Read;
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let mut uplink = Uplink::connect(listener.local_addr().unwrap()).unwrap();
        let (mut peer, _) = listener.accept().unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        assert!(uplink.ping().is_err());
        let error = peer.read(&mut [0; 1]).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ));
    }

    #[test]
    #[cfg(unix)]
    fn command_status_and_exit_are_preserved() {
        let mut shell = Shell::new(false);
        assert_eq!(shell.execute("false"), 1);
        assert_eq!(shell.execute("printf hello | cat > /dev/null"), 0);
        assert_eq!(shell.execute("exit 256"), 1);
        shell.last_status = 7;
        assert_eq!(shell.execute("exit"), 0);
        assert_eq!(shell.exit, Some(7));
    }

    #[test]
    fn persona_changes_validate_input_and_preserve_state_on_errors() {
        let mut shell = Shell::new(false);
        assert_eq!(shell.execute("callsign nightwraith"), 0);
        assert_eq!(shell.callsign, "NIGHTWRAITH");
        assert_eq!(shell.execute("callsign 'bad name'"), 1);
        assert_eq!(shell.execute("callsign '\u{1b}[31m'"), 1);
        assert_eq!(shell.callsign, "NIGHTWRAITH");
        assert_eq!(shell.execute("theme venom"), 0);
        assert_eq!(shell.execute("theme unknown"), 1);
        assert_eq!(shell.theme, "venom");
        assert_eq!(shell.operations, 2);
        assert_eq!(shell.execute("poopsec on"), 0);
        assert!(shell.poopsec);
        assert_eq!(shell.execute("poopsec invalid"), 1);
        assert!(shell.poopsec);
        assert_eq!(shell.execute("poopsec off"), 0);
        assert!(!shell.poopsec);
    }

    #[test]
    fn theater_output_and_story_progress_are_bounded() {
        let rain = theater::matrix(3);
        assert_eq!(rain.lines().count(), 3);
        assert!(rain.lines().all(|line| line.len() == 64));
        let mut shell = Shell::new(false);
        assert_eq!(shell.execute("matrix 0"), 1);
        assert_eq!(shell.execute("matrix 65"), 1);
        assert_eq!(shell.execute("matrix nope"), 1);
        for _ in 0..10 {
            assert_eq!(shell.execute("mission next"), 0);
        }
        assert_eq!(shell.mission, theater::MISSIONS.len() - 1);
        assert_eq!(theater::rank(0), "GHOST IN THE WIRES");
        assert_eq!(theater::rank(5), "MAINFRAME REVENANT");
        assert_eq!(theater::rank(15), "NEON ARCHDEMON");
        assert_eq!(theater::rank(30), "ROOT OF ALL LORE");
    }

    #[test]
    #[cfg(unix)]
    fn compound_builtins_follow_shell_syntax_and_quoted_operators_remain_literal() {
        assert!(!is_builtin("pwd | cat"));
        assert!(!is_builtin("cd /tmp && pwd"));
        assert!(is_builtin("export LABEL='ghost;firewall'"));
        assert!(is_builtin("callsign DEMON"));
        let shell = Shell::new(false);
        let output = shell.command("pwd | cat").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            cwd().to_string_lossy()
        );
    }

    #[test]
    #[cfg(unix)]
    fn exported_environment_persists_for_children_and_unset_removes_it() {
        let mut shell = Shell::new(false);
        assert_eq!(shell.execute("export OPSEC_DEMON_TEST='hello twin'"), 0);
        let output = shell
            .command("printf '%s' \"$OPSEC_DEMON_TEST\"")
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "hello twin");
        let before = shell.environment.clone();
        assert_eq!(shell.execute("export OTHER=ok 1INVALID=no"), 1);
        assert_eq!(shell.environment, before);
        assert_eq!(shell.execute("unset OPSEC_DEMON_TEST"), 0);
        let output = shell
            .command("printf '%s' \"${OPSEC_DEMON_TEST+set}\"")
            .output()
            .unwrap();
        assert!(output.stdout.is_empty());
        assert!(valid_variable("KUBECONFIG"));
        assert!(!valid_variable("bad-name"));
    }

    #[test]
    fn scene_and_expansion_commands_validate_before_mutating() {
        let mut shell = Shell::new(false);
        assert_eq!(shell.execute("mode prostreet"), 0);
        assert_eq!(shell.scene, "prostreet");
        assert_eq!(shell.execute("mode invalid"), 1);
        assert_eq!(shell.scene, "prostreet");
        assert_eq!(shell.execute("mode vulcan"), 0);
        assert_eq!(shell.execute("mascot chinese-demon"), 0);
        assert_eq!(shell.mascot, "chinese-demon");
        assert_eq!(shell.execute("expansion allocate 512"), 1);
        assert_eq!(shell.execute("mcp expand"), 0);
        assert_eq!(shell.execute("expansion allocate 512"), 0);
        assert_eq!(shell.execute("mcp allocate 1"), 1);
        assert!(shell.expansion.status().contains("512 / 512"));
        assert_eq!(shell.execute("mcp release 512"), 0);
        assert!(shell.expansion.status().contains("0 / 512"));
    }
}
