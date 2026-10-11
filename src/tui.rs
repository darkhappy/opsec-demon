use crate::{
    Shell, Uplink, cwd,
    dsp::{Model, Response},
    is_builtin,
    job::Job,
    music::Music,
    theater,
};
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Terminal {
    saved: String,
    active: bool,
}

fn stty(arguments: &[&str]) -> io::Result<String> {
    let output = Command::new("stty")
        .args(arguments)
        .stdin(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("stty could not configure the terminal"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Apple's stty prints min/time in hexadecimal but reads them as decimal.
// Normalize only those BSD gfmt1 fields so the saved state round-trips.
fn restorable_stty(saved: String) -> String {
    if !saved.starts_with("gfmt1:") {
        return saved;
    }
    saved
        .split(':')
        .map(|field| {
            if let Some((name @ ("min" | "time"), value)) = field.split_once('=')
                && let Ok(value) = u8::from_str_radix(value, 16)
            {
                return format!("{name}={value}");
            }
            field.to_string()
        })
        .collect::<Vec<_>>()
        .join(":")
}

impl Terminal {
    fn new() -> io::Result<Self> {
        let mut terminal = Self {
            saved: restorable_stty(stty(&["-g"])?),
            active: false,
        };
        terminal.resume()?;
        Ok(terminal)
    }

    fn resume(&mut self) -> io::Result<()> {
        stty(&["raw", "-echo", "min", "0", "time", "0"])?;
        self.active = true;
        print!("\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H");
        io::stdout().flush()
    }

    fn suspend(&mut self) -> io::Result<()> {
        if self.active {
            print!("\x1b[?2026l\x1b[?7h\x1b[0m\x1b[?25h\x1b[?1049l");
            io::stdout().flush()?;
            stty(&[&self.saved])?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if self.active {
            // Restore input even when output has become unavailable.
            let _ = stty(&[&self.saved]);
            let _ = io::stdout().write_all(b"\x1b[?2026l\x1b[?7h\x1b[0m\x1b[?25h\x1b[?1049l");
            let _ = io::stdout().flush();
        }
    }
}

fn dimensions() -> (usize, usize) {
    let size = stty(&["size"]).unwrap_or_default();
    let mut parts = size
        .split_whitespace()
        .filter_map(|part| part.parse::<usize>().ok());
    let rows = parts
        .next()
        .filter(|size| *size > 0)
        .unwrap_or(24)
        .clamp(4, 1000);
    let columns = parts
        .next()
        .filter(|size| *size > 0)
        .unwrap_or(80)
        .clamp(8, 1000);
    (columns, rows)
}

fn safe_line(text: &str, width: usize) -> String {
    let mut characters = text.chars().peekable();
    let mut output = String::new();
    let mut count = 0;
    while let Some(character) = characters.next() {
        if character == '\x1b' {
            match characters.peek() {
                Some('[') => {
                    characters.next();
                    for next in characters.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    characters.next();
                    let mut escape = false;
                    for next in characters.by_ref() {
                        if next == '\x07' || (escape && next == '\\') {
                            break;
                        }
                        escape = next == '\x1b';
                    }
                }
                _ => {}
            }
        } else if !character.is_control() && count < width {
            output.push(character);
            count += 1;
        }
    }
    output
}

fn wrap_line(text: &str, width: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut line = String::new();
    let mut count = 0;
    for character in text.chars() {
        if count == width.max(1) {
            result.push(std::mem::take(&mut line));
            count = 0;
        }
        line.push(character);
        count += 1;
    }
    if !line.is_empty() || result.is_empty() {
        result.push(line);
    }
    result
}

struct Deck {
    input: String,
    events: VecDeque<String>,
    tick: u64,
    size: (usize, usize),
    history_index: usize,
    draft: String,
    escape: u8,
    escape_tick: u64,
    utf8: Vec<u8>,
    intro: bool,
    boot_started: Instant,
    transition: Option<Instant>,
    scripts: VecDeque<String>,
    seed: u64,
    burst: Option<u64>,
    job: Option<Job>,
    scroll: usize,
    boot_step: u64,
    cursor: usize,
    graph_index: usize,
    bido_active: bool,
    arcade_play: bool,
    celebration: Option<u64>,
    prompt: crate::prompt::Prompt,
    music: Music,
}

impl Deck {
    fn new(intro: bool) -> Self {
        Self {
            input: String::new(),
            events: VecDeque::new(),
            tick: 0,
            size: dimensions(),
            history_index: 0,
            draft: String::new(),
            escape: 0,
            escape_tick: 0,
            utf8: Vec::new(),
            intro,
            boot_started: Instant::now(),
            transition: None,
            scripts: VecDeque::new(),
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(1, |time| time.as_nanos() as u64),
            burst: None,
            job: None,
            scroll: 0,
            boot_step: 0,
            cursor: 0,
            graph_index: 0,
            bido_active: false,
            arcade_play: false,
            celebration: None,
            prompt: crate::prompt::Prompt::new(),
            music: Music::new(),
        }
    }

    fn log(&mut self, message: impl Into<String>) {
        if self.events.len() == 100 {
            self.events.pop_front();
        }
        self.events.push_back(safe_line(&message.into(), 4096));
        if self.scroll > 0 {
            self.scroll = (self.scroll + 1).min(self.events.len().saturating_sub(1));
        }
    }

    fn local(&mut self, shell: &mut Shell) {
        match Uplink::local().and_then(|mut uplink| uplink.ping().map(|latency| (uplink, latency)))
        {
            Ok((uplink, latency)) => {
                shell.uplink = Some(uplink);
                self.burst = Some(self.tick);
                self.log(format!(
                    "MAINFRAME CONNECTION ESTABLISHED // SIM ECHO {:.3} ms",
                    latency.as_secs_f64() * 1000.0
                ));
            }
            Err(error) => self.log(format!("UPLINK FAILED: {error}")),
        }
    }

    fn ping(&mut self, shell: &mut Shell) {
        match shell
            .uplink
            .as_mut()
            .ok_or_else(|| io::Error::other("offline: Ctrl-L establishes a local uplink"))
            .and_then(Uplink::ping)
        {
            Ok(latency) => self.log(format!(
                "ECHO VERIFIED // {:.3} ms // local simulator",
                latency.as_secs_f64() * 1000.0
            )),
            Err(error) => self.log(format!("UPLINK: {error}")),
        }
    }

    fn render(&self, shell: &Shell) -> String {
        let (columns, rows) = self.size;
        let width = columns.saturating_sub(1);
        let mut screen = Canvas::new(width, rows);
        if self.intro {
            self.boot_screen(&mut screen, shell, width, rows);
            return screen.render(shell);
        }
        let brand = if shell.poopsec { "POOPSEC" } else { "OPSEC" };
        screen.put(
            1,
            0,
            width.saturating_sub(2),
            &format!(
                "{} {brand} / NIGHTFALL   ▰▰▰   {} / {} / PEDAL {} LARP {}",
                if shell.nerd_font { "\u{f120}" } else { ">_" },
                shell.callsign,
                shell.countries.line(),
                if shell.stage.pedal_down() {
                    "DOWN"
                } else {
                    "UP"
                },
                shell.stage.larp_meter()
            ),
            1,
        );
        let status = if self.intro {
            self.boot_phase(shell.poopsec)
        } else {
            format!(
                "{}  /  {} ops  /  exit {}  /  {:.1}s",
                if shell.uplink.is_some() {
                    "UPLINK ONLINE"
                } else {
                    "NO CARRIER"
                },
                shell.operations,
                shell.last_status,
                shell.started.elapsed().as_secs_f64()
            )
        };
        screen.put(1, 1, width.saturating_sub(2), &status, 3);
        let music_w = if width >= 100 {
            width * 3 / 5
        } else {
            width.saturating_sub(2)
        };
        screen.put(1, 2, music_w, &self.music.line(), 2);
        if width >= 100 {
            screen.put(
                music_w + 2,
                2,
                width.saturating_sub(music_w + 3),
                &shell.ssd.line(),
                5,
            );
        }
        let command_y = rows.saturating_sub(4);
        if columns >= 80 && rows >= 24 {
            let body_y = 3;
            let body_h = (rows / 2 - 5).clamp(8, 22);
            let portrait_w = (width / 4).clamp(25, 36);
            let matrix_w = if columns >= 100 {
                (width / 4).max(24)
            } else {
                0
            };
            let stream_w = width - portrait_w - matrix_w;
            screen.panel(
                0,
                body_y,
                portrait_w,
                body_h,
                if shell.mascot == "mo-krill" && !shell.poopsec {
                    if shell.nerd_font {
                        "\u{f1b0} MO & KRILL "
                    } else {
                        " MO & KRILL "
                    }
                } else if shell.mascot == "tweeter" {
                    " TWEETER "
                } else if shell.mascot == "krillion" {
                    " KRILLION "
                } else if shell.nerd_font {
                    "\u{f007} OPERATOR "
                } else {
                    " OPERATOR "
                },
            );
            self.portrait(
                &mut screen,
                shell,
                2,
                body_y + 1,
                portrait_w - 4,
                body_h - 2,
            );
            screen.panel(
                portrait_w,
                body_y,
                stream_w,
                body_h,
                if shell.nerd_font {
                    "\u{f120} DAEMON SCRIPT / SIM "
                } else {
                    " DAEMON SCRIPT / SIM "
                },
            );
            let visible = body_h - 4;
            for (index, line) in self
                .scripts
                .iter()
                .skip(self.scripts.len().saturating_sub(visible))
                .enumerate()
            {
                screen.put(
                    portrait_w + 2,
                    body_y + 1 + index,
                    stream_w - 4,
                    line,
                    if line.starts_with("[SIM] $") { 2 } else { 4 },
                );
            }
            let ritual_status = shell.ritual.status();
            let ritual_phase = ritual_status.lines().nth(1).unwrap_or("AWAITING SIGNAL");
            screen.put(
                portrait_w + 2,
                body_y + body_h - 3,
                stream_w - 4,
                &format!("APPLE [SIM] {ritual_phase}"),
                5,
            );
            if self.intro {
                let elapsed = self.boot_started.elapsed().as_secs_f64().min(15.0);
                let bar_w = stream_w.saturating_sub(18);
                let fill = (elapsed / 15.0 * bar_w as f64) as usize;
                screen.put(
                    portrait_w + 2,
                    body_y + body_h - 2,
                    stream_w - 4,
                    &format!(
                        "{}{} {:04.1}/15s",
                        "━".repeat(fill),
                        "·".repeat(bar_w - fill),
                        elapsed
                    ),
                    1,
                );
            } else {
                screen.put(
                    portrait_w + 2,
                    body_y + body_h - 2,
                    stream_w - 4,
                    match &shell.uplink {
                        Some(Uplink::Local { .. }) => "I'M IN. // SIM:LINK // LOCAL SIMULATOR",
                        Some(Uplink::Remote(_)) => "TCP:LINK // no TLS // app unverified",
                        None => "OFFLINE // Ctrl-L reconnects simulator",
                    },
                    4,
                );
            }
            if matrix_w > 0 {
                let x = portrait_w + stream_w;
                screen.panel(
                    x,
                    body_y,
                    matrix_w,
                    body_h,
                    if shell.poopsec {
                        " SEWER RAIN "
                    } else {
                        " MATRIX "
                    },
                );
                if shell.scene == "prostreet" && !shell.poopsec {
                    self.race_lane(&mut screen, (x + 1, body_y + 1, matrix_w - 2, body_h - 2));
                } else {
                    self.digital_rain(
                        &mut screen,
                        x + 1,
                        body_y + 1,
                        matrix_w - 2,
                        body_h - 2,
                        shell.poopsec,
                    );
                }
            }
            if matrix_w > 0 && body_h >= 12 {
                let x = portrait_w + stream_w + 1;
                if body_h >= 16 && shell.stage.band_on() {
                    let y = body_y + 1;
                    for row in y..y + 14 {
                        screen.put(x, row, matrix_w - 2, &" ".repeat(matrix_w - 2), 0);
                    }
                    self.band_board(&mut screen, shell, (x, y, matrix_w - 2, 7));
                    self.sdvx_board(&mut screen, shell, (x, y + 7, matrix_w - 2, 7));
                } else {
                    let y = body_y + body_h - 8;
                    for row in y..y + 7 {
                        screen.put(x, row, matrix_w - 2, &" ".repeat(matrix_w - 2), 0);
                    }
                    if shell.stage.band_on()
                        && (self.tick / 240).is_multiple_of(2)
                        && !self.arcade_play
                    {
                        self.band_board(&mut screen, shell, (x, y, matrix_w - 2, 7));
                    } else {
                        self.sdvx_board(&mut screen, shell, (x, y, matrix_w - 2, 7));
                    }
                }
            }
            let lower_y = body_y + body_h;
            let lower_h = command_y - lower_y;
            let transcript_w = if columns >= 140 {
                width * 3 / 5
            } else {
                width / 2
            };
            screen.panel(
                0,
                lower_y,
                transcript_w,
                lower_h,
                if self.job.is_some() {
                    if shell.nerd_font {
                        "\u{f04b} COMMAND / RUNNING "
                    } else {
                        " COMMAND / RUNNING "
                    }
                } else if shell.nerd_font {
                    "\u{f120} COMMAND TRANSCRIPT "
                } else {
                    " COMMAND TRANSCRIPT "
                },
            );
            let capacity = lower_h.saturating_sub(2);
            let lines: Vec<String> = self
                .events
                .iter()
                .flat_map(|event| wrap_line(event, transcript_w - 4))
                .collect();
            let end = lines.len().saturating_sub(self.scroll);
            let start = end.saturating_sub(capacity);
            for (index, line) in lines.iter().skip(start).take(end - start).enumerate() {
                screen.put(
                    2,
                    lower_y + 1 + index,
                    transcript_w - 4,
                    line,
                    if line.contains("FAILED") || line.contains("stderr") {
                        5
                    } else if line.starts_with("LOST SIGNALS") || line.contains("ENDING:") {
                        1
                    } else if line.starts_with('"') {
                        7
                    } else {
                        2
                    },
                );
            }
            let graph_w = width - transcript_w;
            if lower_h >= 14 && graph_w >= 54 && self.graph_index < 4 {
                for (index, kind) in Response::ALL.iter().enumerate() {
                    let col = index % 2;
                    let row = index / 2;
                    let x = transcript_w + col * (graph_w / 2);
                    let y = lower_y + row * (lower_h / 2);
                    let w = if col == 0 {
                        graph_w / 2
                    } else {
                        graph_w - graph_w / 2
                    };
                    let h = if row == 0 {
                        lower_h / 2
                    } else {
                        lower_h - lower_h / 2
                    };
                    screen.panel(x, y, w, h, kind.label());
                    self.response_graph(&mut screen, (x + 1, y + 1, w - 2, h - 2), *kind);
                }
            } else if self.graph_index == 7 {
                screen.panel(
                    transcript_w,
                    lower_y,
                    graph_w,
                    lower_h,
                    " LOST SIGNALS / VISUAL NOVEL ",
                );
                let page = shell.novel.page();
                let lines: Vec<_> = page
                    .lines()
                    .flat_map(|line| wrap_line(line, graph_w.saturating_sub(4)))
                    .collect();
                for (row, line) in lines.iter().take(lower_h.saturating_sub(3)).enumerate() {
                    screen.put(
                        transcript_w + 2,
                        lower_y + 1 + row,
                        graph_w.saturating_sub(4),
                        line,
                        if row == 0 { 1 } else { 2 },
                    );
                }
                screen.put(
                    transcript_w + 2,
                    lower_y + lower_h.saturating_sub(2),
                    graph_w.saturating_sub(4),
                    "PgUp/PgDn: full text in transcript",
                    3,
                );
            } else if self.graph_index == 6 {
                screen.panel(
                    transcript_w,
                    lower_y,
                    graph_w,
                    lower_h,
                    " METRICS / ACTUAL SESSION ",
                );
                let uptime = shell.started.elapsed().as_secs_f64();
                let lines = [
                    format!("Uptime: {uptime:.1}s"),
                    format!("Render ticks: {}", self.tick),
                    format!(
                        "Average ticks/s: {:.1}",
                        self.tick as f64 / uptime.max(0.001)
                    ),
                    format!("Successful operations: {}", shell.operations),
                    format!("History entries: {}", shell.history.len()),
                    format!("Transcript events: {}", self.events.len()),
                    format!("Child command active: {}", self.job.is_some()),
                    format!("Terminal: {} x {}", self.size.0, self.size.1),
                    "Session values; no remote scraping".into(),
                ];
                for (row, line) in lines.iter().take(lower_h.saturating_sub(2)).enumerate() {
                    screen.put(
                        transcript_w + 2,
                        lower_y + 1 + row,
                        graph_w.saturating_sub(4),
                        line,
                        if row == 0 {
                            1
                        } else if row < 6 {
                            4
                        } else {
                            2
                        },
                    );
                }
            } else if self.graph_index == 5 {
                screen.panel(
                    transcript_w,
                    lower_y,
                    graph_w,
                    lower_h,
                    " HEADPHONE EQ / UNMEASURED / SIM ",
                );
                self.headphone_graph(
                    &mut screen,
                    shell,
                    (transcript_w + 1, lower_y + 1, graph_w - 2, lower_h - 2),
                );
            } else if self.graph_index == 4 {
                screen.panel(
                    transcript_w,
                    lower_y,
                    graph_w,
                    lower_h,
                    " RT60 / ROOM DECAY / SIM ",
                );
                self.room_graph(
                    &mut screen,
                    shell,
                    (transcript_w + 1, lower_y + 1, graph_w - 2, lower_h - 2),
                );
            } else {
                let kind = Response::ALL[self.graph_index];
                screen.panel(transcript_w, lower_y, graph_w, lower_h, kind.label());
                self.response_graph(
                    &mut screen,
                    (transcript_w + 1, lower_y + 1, graph_w - 2, lower_h - 2),
                    kind,
                );
            }
        } else {
            let panel_h = command_y.saturating_sub(2).max(2);
            screen.panel(0, 2, width, panel_h, " TRANSMISSIONS ");
            let capacity = panel_h.saturating_sub(2);
            let entries = if self.intro {
                &self.scripts
            } else {
                &self.events
            };
            for (index, line) in entries
                .iter()
                .skip(entries.len().saturating_sub(capacity))
                .enumerate()
            {
                screen.put(2, 3 + index, width.saturating_sub(4), line, 2);
            }
        }
        screen.panel(
            0,
            command_y,
            width,
            rows - command_y,
            if shell.nerd_font {
                "\u{f0e7} EXECUTE "
            } else {
                " EXECUTE "
            },
        );
        let directory = cwd();
        let directory_text = directory.display().to_string();
        let rendered_prompt = self.prompt.line();
        let prompt_line = if rendered_prompt.starts_with(&directory_text) {
            rendered_prompt
        } else {
            format!("{directory_text} | {rendered_prompt}")
        };
        screen.put(
            2,
            command_y + 1,
            width.saturating_sub(4),
            if self.intro {
                "BOOTLOCK / 15s SIM INFILTRATION / Ctrl-D abort"
            } else if self.arcade_play {
                "SDVX PLAY / ASDF BT / JK FX / Q E + U O knobs / Enter returns to shell"
            } else {
                &prompt_line
            },
            3,
        );
        if shell.bido.is_active() {
            screen.put(1, 1, width.saturating_sub(2), &shell.bido.status(), 5);
        }
        let input_width = width.saturating_sub(16);
        let start = self.cursor.saturating_sub(input_width.saturating_sub(1));
        let tail: String = self.input.chars().skip(start).take(input_width).collect();
        screen.put(
            2,
            command_y + 2,
            width.saturating_sub(4),
            &format!("EXECUTE > {tail}"),
            1,
        );
        if self.tick % 30 < 20 {
            let character = self
                .input
                .chars()
                .nth(self.cursor)
                .unwrap_or(if shell.color { ' ' } else { '▌' });
            screen.put(
                12 + self.cursor.saturating_sub(start),
                command_y + 2,
                1,
                &character.to_string(),
                8,
            );
        }
        self.dancing_blasts(&mut screen, width, rows, false);
        self.confetti(&mut screen, width, rows);
        if let Some(start) = self.transition {
            let progress = (start.elapsed().as_secs_f64() / 3.0).clamp(0.0, 1.0);
            if progress < 1.0 {
                let reveal = (rows as f64 * progress) as usize;
                for row in reveal..rows {
                    screen.put(0, row, width, &" ".repeat(width), 0);
                }
                self.digital_rain(&mut screen, 0, reveal, width, rows - reveal, shell.poopsec);
                let radius = (progress * width as f64 * 0.6) as isize;
                for particle in 0..48 {
                    let angle = particle as f64 * std::f64::consts::TAU / 48.0;
                    let px = width as isize / 2 + (angle.cos() * radius as f64) as isize;
                    let py = rows as isize / 2 + (angle.sin() * radius as f64 * 0.35) as isize;
                    if px >= 0 && py >= 0 && px < width as isize && py < rows as isize {
                        screen.put(px as usize, py as usize, 1, "*", 7);
                    }
                }
                screen.put(0, reveal.min(rows - 1), width, &"=".repeat(width), 7);
                if shell.color {
                    if progress < 0.06 {
                        for row in &mut screen.cells {
                            for cell in row {
                                *cell = (' ', 9);
                            }
                        }
                    } else {
                        let fade = if progress < 0.3 {
                            10
                        } else if progress < 0.55 {
                            11
                        } else if progress < 0.8 {
                            12
                        } else {
                            13
                        };
                        for row in &mut screen.cells {
                            for (_, style) in row {
                                if *style != 0 {
                                    *style = fade;
                                }
                            }
                        }
                    }
                }
                if progress < 0.55 && width >= 50 && rows >= 18 {
                    let cloud = [
                        "       _..---^^---.._",
                        "   _.-'              '-._",
                        "  <    *  N U K E  *     >",
                        "   \\._              _./",
                        "       '---.  .---'",
                        "           |  |",
                        "        .-=|  |=-.",
                        "      .-*#$%&%$#*-.",
                    ];
                    for (row, line) in cloud.iter().enumerate() {
                        screen.put(width / 2 - 14, rows / 2 - 8 + row, 30, line, 7);
                    }
                }
                screen.put(
                    width.saturating_sub(31) / 2,
                    rows / 2,
                    width,
                    "*** BOOM / CONSTRUCT BREACHED ***",
                    7,
                );
                if shell.uplink.is_some() {
                    screen.put(
                        2,
                        rows.saturating_sub(3),
                        width.saturating_sub(4),
                        "CONNECTION ESTABLISHED // LOCAL SIMULATOR",
                        7,
                    );
                    screen.put(
                        2,
                        rows.saturating_sub(2),
                        width.saturating_sub(4),
                        "SAY IT OUT LOUD: I'M IN.",
                        7,
                    );
                }
            }
        }
        screen.render(shell)
    }

    fn band_board(&self, screen: &mut Canvas, shell: &Shell, area: (usize, usize, usize, usize)) {
        let (x, y, width, height) = area;
        let compact = [
            "BEATLES / ASCII TRIBUTE",
            " o    o    o    o",
            "/|\\  /|\\  /|\\  /|\\",
            "/|=> <|=  /|=> (===)",
            "/ \\  / \\  / \\ |DRUM|",
            "JOHN PAUL GEORGE RINGO",
            "SIM / SILENT STAGE",
        ];
        let art = if width >= 44 {
            crate::stage::BAND_ART
        } else {
            &compact
        };
        for (row, line) in art.iter().take(height).enumerate() {
            let line = if (self.tick / 8).is_multiple_of(2) {
                line.replace("/|", "\\|")
            } else {
                line.to_string()
            };
            screen.put(x, y + row, width, &line, if row == 0 { 1 } else { 4 });
        }
        if shell.stage.drummer_blast() {
            let drummer_x = x + if width >= 44 { 34 } else { 15 };
            screen.put(
                drummer_x,
                y + 1,
                width.saturating_sub(drummer_x - x),
                "*^*",
                7,
            );
            screen.put(drummer_x.saturating_sub(1), y + 3, width, "BOOM!", 5);
            for particle in 0..8 {
                let ash_x = drummer_x + particle % 4;
                let ash_y = y + (particle + self.tick as usize / 3) % height.max(1);
                screen.put(ash_x, ash_y, 1, ".", 3);
            }
        }
    }

    fn headphone_graph(
        &self,
        screen: &mut Canvas,
        shell: &Shell,
        area: (usize, usize, usize, usize),
    ) {
        let (x, y, width, height) = area;
        if width < 14 || height < 4 {
            return;
        }
        screen.put(
            x,
            y,
            width,
            &format!("{} / invented dB curve", shell.stage.headphone_name()),
            3,
        );
        let plot_w = width - 7;
        let plot_h = height - 2;
        for row in 0..plot_h {
            screen.put(
                x,
                y + 1 + row,
                7,
                &format!("{:4.1} |", 6.0 - row as f64 / (plot_h - 1) as f64 * 12.0),
                3,
            );
        }
        let mut previous = 0;
        for col in 0..plot_w {
            let hz = 20.0 * 1000.0_f64.powf(col as f64 / (plot_w - 1) as f64);
            let value = shell.stage.frequency_gain(hz);
            let row =
                ((6.0 - value.clamp(-6.0, 6.0)) / 12.0 * (plot_h - 1) as f64).round() as usize;
            for join in previous.min(row)..=previous.max(row) {
                screen.put(x + 7 + col, y + 1 + join, 1, "|", 4);
            }
            screen.put(x + 7 + col, y + 1 + row, 1, "*", 7);
            previous = row;
        }
        screen.put(
            x,
            y + height - 1,
            width,
            &format!(
                "20Hz ---- 200 ---- 2k ---- 20k / {}",
                shell.stage.headphone_id()
            ),
            3,
        );
    }

    fn sdvx_board(&self, screen: &mut Canvas, shell: &Shell, area: (usize, usize, usize, usize)) {
        let (x, y, width, height) = area;
        let (left, right) = shell.arcade.knobs();
        let needle = |knob: u8| ["/", "|", "\\", "-"][knob as usize / 26];
        let lines = [
            "SOUND VOLTEX / SIM BOARD".to_string(),
            format!("  ({})             ({})", needle(left), needle(right)),
            format!(" LEFT {:03}%   RIGHT {:03}%", left, right),
            " [A] [S] [D] [F] / BT".into(),
            "   [J FX-L] [K FX-R]".into(),
            format!(
                "COMBO {:05} SCORE {}",
                shell.arcade.combo(),
                shell.arcade.score()
            ),
            if self.arcade_play {
                "PLAY / Q E : U O KNOBS".into()
            } else {
                "TYPE sdvx play TO ENTER".into()
            },
        ];
        for (row, line) in lines.iter().take(height).enumerate() {
            screen.put(x, y + row, width, line, if row == 0 { 1 } else { 7 });
        }
        if height > 7 {
            screen.put(x, y + 7, width, &shell.arcade.status(), 3);
        }
        for row in 8..height {
            let note = (row + self.tick as usize / 3) % 8;
            let lane = (self.tick as usize / 17 + row) % 4;
            let mut line = " |   |   |   | ".to_string();
            if note == 0 {
                line.replace_range(1 + lane * 4..2 + lane * 4, "#");
            }
            screen.put(x, y + row, width, &line, if note == 0 { 7 } else { 6 });
        }
    }

    fn room_graph(&self, screen: &mut Canvas, shell: &Shell, area: (usize, usize, usize, usize)) {
        let (x, y, width, height) = area;
        if width < 14 || height < 4 {
            return;
        }
        let rt60 = shell.room.rt60();
        screen.put(
            x,
            y,
            width,
            &format!("SABINE RT60 {:.3}s / idealized diffuse field", rt60),
            3,
        );
        let plot_w = width - 7;
        let plot_h = height - 2;
        for row in 0..plot_h {
            screen.put(
                x,
                y + 1 + row,
                7,
                &format!("{:4.0} |", -(row as f64) / (plot_h - 1) as f64 * 90.0),
                3,
            );
        }
        let mut previous = 0;
        for col in 0..plot_w {
            let time = col as f64 / (plot_w - 1) as f64 * rt60 * 1.5;
            let row = ((-shell.room.decay_db(time) / 90.0) * (plot_h - 1) as f64).round() as usize;
            for join in previous.min(row)..=previous.max(row) {
                screen.put(x + 7 + col, y + 1 + join, 1, "|", 4);
            }
            screen.put(x + 7 + col, y + 1 + row, 1, "*", 7);
            previous = row;
        }
        screen.put(
            x,
            y + height - 1,
            width,
            &format!("s 0 ------- {:.3} (-60dB) ---- {:.3}", rt60, rt60 * 1.5),
            3,
        );
    }

    fn confetti(&self, screen: &mut Canvas, width: usize, height: usize) {
        let Some(start) = self.celebration else {
            return;
        };
        let age = self.tick.saturating_sub(start);
        if age >= 75 || width < 8 || height < 8 {
            return;
        }
        for particle in 0..40u64 {
            let hash = noise(self.seed ^ particle);
            let px = (hash as usize + age as usize / 3) % width;
            let py = ((hash >> 32) as usize + age as usize / 2) % height;
            screen.put(
                px,
                py,
                1,
                ["*", "+", ":", "."][particle as usize % 4],
                [1, 5, 7, 4][particle as usize % 4],
            );
        }
        for firework in 0..3 {
            let radius = ((age + firework * 15) % 30) as f64 * 0.4;
            for particle in 0..20 {
                let angle = particle as f64 * std::f64::consts::TAU / 20.0;
                let px =
                    width as f64 * (0.25 + firework as f64 * 0.25) + angle.cos() * radius * 2.0;
                let py = height as f64 / 3.0 + angle.sin() * radius * 0.6;
                if px >= 0.0 && py >= 0.0 && px < width as f64 && py < height as f64 {
                    screen.put(px as usize, py as usize, 1, "*", 5);
                }
            }
        }
        screen.put(
            2,
            1,
            width.saturating_sub(4),
            "LARP CRITICAL / CONFETTI + FIREWORKS [SIM]",
            7,
        );
    }

    fn boot_screen(&self, screen: &mut Canvas, shell: &Shell, width: usize, rows: usize) {
        let elapsed = self.boot_started.elapsed().as_secs_f64().min(15.0);
        self.digital_rain(screen, 0, 0, width, rows, shell.poopsec);
        screen.panel(
            0,
            0,
            width,
            rows,
            if shell.poopsec {
                " POOPSEC MATRIX ENTRY / PORCELAIN INFILTRATION / SIM "
            } else {
                " MATRIX ENTRY / 15 SECOND INFILTRATION / SIM "
            },
        );
        screen.put(
            2,
            1,
            width.saturating_sub(4),
            &" ".repeat(width.saturating_sub(4)),
            0,
        );
        screen.put(
            2,
            2,
            width.saturating_sub(4),
            &" ".repeat(width.saturating_sub(4)),
            0,
        );
        screen.put(
            2,
            1,
            width.saturating_sub(4),
            &self.boot_phase(shell.poopsec),
            7,
        );
        let header = format!(
            "{} {} // TARA, PASOK! // DEMON GATE INITIALIZING",
            if shell.nerd_font { "\u{f0e7}" } else { ">>" },
            shell.callsign
        );
        screen.put(2, 2, width.saturating_sub(4), &header, 1);
        let stages = [
            (
                "LOADING TERMINAL DRIVER",
                "terminal size / raw input / palette",
            ),
            ("READING SIGIL ARCHIVE [SIM]", "/sim/nightfall/sigils.arc"),
            ("HACKING IN THE FIREWALL [SIM]", "/sim/firewall/ghost.rules"),
            (
                "LOADING OPERATOR CONSTRUCT [SIM]",
                "/sim/matrix/operator.mesh",
            ),
            (
                "DECODING FREQUENCY + PHASE [SIM]",
                "/sim/sensors/response.bank",
            ),
            ("LOADING DELAY TELEMETRY [SIM]", "/sim/sensors/delay.bank"),
            (
                "ASSEMBLING MAINFRAME ROUTE [SIM]",
                "/sim/mesh/tailscale-milk.route",
            ),
            (
                "ARMING LOCAL ECHO HANDSHAKE",
                "in-process worker / verification at 15s",
            ),
        ];
        let stage = ((elapsed / 2.0) as usize).min(7);
        let stage_duration = if stage == 7 { 1.0 } else { 2.0 };
        let fraction = ((elapsed - stage as f64 * 2.0) / stage_duration).clamp(0.0, 1.0);
        let stage_bar_w = width.saturating_sub(52).clamp(4, 32);
        let fill = (fraction * stage_bar_w as f64) as usize;
        screen.put(
            2,
            3,
            width.saturating_sub(4),
            &" ".repeat(width.saturating_sub(4)),
            0,
        );
        screen.put(
            2,
            4,
            width.saturating_sub(4),
            &" ".repeat(width.saturating_sub(4)),
            0,
        );
        screen.put(
            2,
            3,
            width.saturating_sub(4),
            &format!(
                "[{}{}] {:3.0}% / {}",
                "=".repeat(fill),
                ".".repeat(stage_bar_w - fill),
                fraction * 100.0,
                stages[stage].0
            ),
            7,
        );
        screen.put(
            2,
            4,
            width.saturating_sub(4),
            &format!(
                "SOURCE: {} / stage {}/8 / SIM blocks {}/512",
                stages[stage].1,
                stage + 1,
                (fraction * 512.0) as usize
            ),
            3,
        );
        if rows >= 18 && width >= 70 {
            let graph_y = rows * 3 / 5;
            let stream_w = width * 2 / 3;
            for row in 5..graph_y {
                screen.put(1, row, stream_w, &" ".repeat(stream_w), 0);
            }
            screen.panel(
                1,
                5,
                stream_w,
                graph_y - 5,
                " CONSTRUCT LOADER / SIM COMMAND STREAM ",
            );
            let capacity = graph_y.saturating_sub(7);
            for (index, line) in self
                .scripts
                .iter()
                .skip(self.scripts.len().saturating_sub(capacity))
                .enumerate()
            {
                screen.put(
                    3,
                    6 + index,
                    stream_w.saturating_sub(4),
                    line,
                    if line.starts_with("[SIM] $") { 7 } else { 4 },
                );
            }
            let portrait_x = stream_w + 2;
            self.portrait(
                screen,
                shell,
                portrait_x,
                5,
                width.saturating_sub(portrait_x + 2),
                graph_y.saturating_sub(7),
            );
            screen.put(
                portrait_x,
                4,
                width.saturating_sub(portrait_x + 2),
                "OPERATOR AVATAR / LOADING",
                1,
            );
            self.dancing_blasts(screen, width, rows, true);
            let graph_h = rows.saturating_sub(graph_y + 4);
            if graph_h >= 5 {
                for (index, kind) in Response::ALL.iter().enumerate() {
                    let x = index * (width / 4);
                    let w = if index == 3 { width - x } else { width / 4 };
                    let ready = elapsed >= 5.0 + index as f64 * 1.5;
                    for row in graph_y..graph_y + graph_h {
                        screen.put(x, row, w, &" ".repeat(w), 0);
                    }
                    screen.panel(
                        x,
                        graph_y,
                        w,
                        graph_h,
                        if ready {
                            kind.label()
                        } else {
                            " SENSOR / WAITING "
                        },
                    );
                    if ready {
                        self.response_graph(
                            screen,
                            (x + 1, graph_y + 1, w.saturating_sub(2), graph_h - 2),
                            *kind,
                        );
                    } else {
                        screen.put(
                            x + 2,
                            graph_y + 2,
                            w.saturating_sub(4),
                            "ALLOCATING SIGNAL...",
                            3,
                        );
                    }
                }
            }
        } else {
            for (index, line) in self
                .scripts
                .iter()
                .skip(self.scripts.len().saturating_sub(rows.saturating_sub(9)))
                .enumerate()
            {
                screen.put(2, 5 + index, width.saturating_sub(4), line, 7);
            }
        }
        let bar_w = width.saturating_sub(22);
        let filled = (elapsed / 15.0 * bar_w as f64) as usize;
        screen.put(
            2,
            rows.saturating_sub(3),
            width.saturating_sub(4),
            &format!(
                "[{}{}] {:04.1}/15s",
                "=".repeat(filled),
                ".".repeat(bar_w - filled),
                elapsed
            ),
            7,
        );
        screen.put(
            2,
            rows.saturating_sub(2),
            width.saturating_sub(4),
            "BOOTLOCK // GRAPHS COMING ONLINE // Ctrl-D abort",
            1,
        );
    }

    fn dancing_blasts(&self, screen: &mut Canvas, width: usize, height: usize, continuous: bool) {
        if width < 24 || height < 12 {
            return;
        }
        if !continuous
            && self
                .burst
                .is_none_or(|start| self.tick.saturating_sub(start) >= 45)
        {
            return;
        }
        let phase = self.tick as usize / 4;
        let dancer = if phase.is_multiple_of(2) {
            ["\\o/", " | ", "/ \\"]
        } else {
            [" o ", "/|\\", " > "]
        };
        let x = width * 5 / 6 + phase % 3;
        let y = height / 3;
        for (row, line) in dancer.iter().enumerate() {
            screen.put(x, y + row, width.saturating_sub(x), line, 7);
        }
        let ring = if phase.is_multiple_of(3) {
            "* . BOOM . *"
        } else {
            "+ * POW! * +"
        };
        screen.put(
            x.saturating_sub(4),
            y + 4,
            width.saturating_sub(x.saturating_sub(4)),
            ring,
            5,
        );
    }

    fn race_lane(&self, screen: &mut Canvas, area: (usize, usize, usize, usize)) {
        let (x, y, width, height) = area;
        if width < 4 || height == 0 {
            return;
        }
        for row in 0..height {
            let spread = 1 + row * (width / 2 - 1) / height;
            let center = width / 2;
            screen.put(x + center.saturating_sub(spread), y + row, 1, "/", 4);
            screen.put(x + (center + spread).min(width - 1), y + row, 1, "\\", 4);
            if (row + self.tick as usize / 2) % 4 < 2 {
                screen.put(x + center, y + row, 1, "|", 7);
            }
        }
        screen.put(
            x + 1,
            y,
            width.saturating_sub(2),
            &format!("SIM {:03} KM/H", 180 + self.tick % 120),
            1,
        );
    }

    fn portrait(
        &self,
        screen: &mut Canvas,
        shell: &Shell,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        if shell.scene == "sdvx" && !shell.poopsec {
            self.sdvx_board(screen, shell, (x, y, width, height));
            return;
        }
        let art: &[&str] = if shell.poopsec {
            &[
                r"        ~  ~  ~",
                r"          /\",
                r"         /__\",
                r"        / o o\",
                r"       /  ___ \",
                r"      /________\",
            ]
        } else if shell.scene == "prostreet" {
            crate::expansion::PROSTREET_ART
        } else if shell.scene == "vulcan" {
            crate::expansion::VULCAN_ART
        } else if shell.mascot == "tweeter" {
            &[
                r"    /\     /\",
                r"   (())   (())",
                r"    |  o o  |",
                r"  .-=========-.",
                r"  |  (( O ))  |",
                r"  |  (( o ))  |",
                r"  '-----------'",
                "TWEETER / SPEAKER DEMON",
            ]
        } else if shell.mascot == "krillion" {
            &[
                r"     \  |  //",
                r"      \ | //",
                r"   .---o-o---.",
                r"  <==[=======]=>",
                r"      /| |\",
                r"     / | | ",
                "KRILLION / SIGNAL SWARM",
            ]
        } else if shell.mascot == "chinese-demon" {
            &[
                r"   /\   /\   /\",
                r"  {  \_/  \_/  }",
                r"  /==\  /\  /==\",
                r" | (@)\/  \/(@) |",
                r"  \  / /\ \  /",
                r"   { /vvvvvv\ }",
                r"    \______//",
                "  DEMON GATE / ASCII MASK",
            ]
        } else if shell.mascot == "mo-krill" {
            &[
                r"       _/\_/\_  KRILL",
                r"      ( o  o )===[--",
                r"       /|__|\",
                r"    .-========-.",
                r"   /  O      O  \ MO",
                r"  |    \____/    |",
                r"  /|___/====\___|\",
                r" /_|  /|    |\  |_\",
            ]
        } else {
            &[
                r"   /\            /\",
                r"  /  \__________/  \",
                r"  \  [##]  [##]   /",
                r"   \     /\      /",
                r"    \ V V V V   /",
                r"     \_________/",
                r"  GHOST IN THE MACHINE",
            ]
        };
        for (index, line) in art.iter().take(height.saturating_sub(1)).enumerate() {
            let mut line = line.to_string();
            if self.tick % 90 < 5 {
                line = line.replace("[##]", "[--]").replace("o  o", "-  -");
            }
            if shell.poopsec && self.tick % 20 < 10 {
                line = line.replace('~', "^");
            }
            screen.put(x, y + index, width, &line, 1);
        }
        if height > art.len() + 1 {
            screen.put(
                x,
                y + art.len() + 1,
                width,
                if shell.poopsec {
                    "PORCELAIN ARCHDEMON"
                } else if shell.mascot == "mo-krill" {
                    "TUNNEL RATS / BURROW"
                } else {
                    theater::rank(shell.operations)
                },
                2,
            );
        }
        if height >= 17 {
            screen.put(x, y + height - 3, width, &shell.bucket.status(), 3);
            screen.put(x, y + height - 2, width, &shell.milk.status(), 3);
            screen.put(x, y + height - 1, width, &shell.yoggies.status(), 5);
        }
        if height > art.len() + 2 {
            let egg = shell.egg.status();
            for (index, line) in egg
                .lines()
                .skip(1)
                .take(height.saturating_sub(art.len() + if height >= 17 { 5 } else { 2 }))
                .enumerate()
            {
                screen.put(x, y + art.len() + 2 + index, width, line, 5);
            }
        }
    }

    fn digital_rain(
        &self,
        screen: &mut Canvas,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        poopsec: bool,
    ) {
        if height == 0 {
            return;
        }
        let alphabet = if poopsec {
            b"o@~.:/".as_slice()
        } else {
            b"01ABCDEF<>:+=/".as_slice()
        };
        for column in 0..width {
            let hash = noise(self.seed ^ (column as u64 * 7919));
            let cycle = height + 12;
            let head =
                (self.tick as usize / (2 + hash as usize % 3) + hash as usize % cycle) % cycle;
            for row in 0..height {
                let distance = (head + cycle - row) % cycle;
                if distance < 9 {
                    let character = alphabet[noise(hash ^ (row as u64 * 31) ^ (self.tick / 5))
                        as usize
                        % alphabet.len()] as char;
                    screen.put(
                        x + column,
                        y + row,
                        1,
                        &character.to_string(),
                        if distance == 0 {
                            7
                        } else if distance < 3 {
                            4
                        } else {
                            6
                        },
                    );
                }
            }
        }
        let missile_y = self.tick as usize / 10 % height;
        let missile_x = self.tick as usize * 2 % (width + 8);
        if missile_x < width {
            screen.put(
                x + missile_x,
                y + missile_y,
                width - missile_x,
                if poopsec { "o@o==>" } else { "---->" },
                7,
            );
        }
        if self.bido_active {
            let bx = (self.tick as usize * 3) % width.max(1);
            screen.put(
                x + bx,
                y + height / 2,
                width.saturating_sub(bx),
                "<bido>~*",
                7,
            );
        }
        if let Some(start) = self.burst {
            let age = self.tick.saturating_sub(start);
            if age < 30 {
                let radius = age as f64 * 0.48;
                for particle in 0..32 {
                    let angle = particle as f64 * std::f64::consts::TAU / 32.0;
                    let px = width as f64 / 2.0 + angle.cos() * radius * 1.6;
                    let py = height as f64 / 2.0 + angle.sin() * radius * 0.7;
                    if px >= 0.0 && py >= 0.0 && px < width as f64 && py < height as f64 {
                        screen.put(
                            x + px as usize,
                            y + py as usize,
                            1,
                            if poopsec {
                                "@"
                            } else if age < 7 {
                                "*"
                            } else {
                                "+"
                            },
                            if age < 10 { 7 } else { 5 },
                        );
                    }
                }
            }
        }
    }

    fn response_graph(
        &self,
        screen: &mut Canvas,
        area: (usize, usize, usize, usize),
        kind: Response,
    ) {
        let (x, y, width, height) = area;
        if width < 12 || height < 3 {
            return;
        }
        let model = Model {
            cutoff: 1500.0 + (self.tick as f64 / 180.0).sin() * 900.0,
            q: 1.2 + (self.tick as f64 / 129.0).sin().abs() * 1.8,
            transport_delay_s: 0.0003 + 0.0001 * (self.tick as f64 / 190.0).sin(),
        };
        let plot_w = width - 7;
        let values: Vec<f64> = (0..plot_w)
            .map(|column| {
                model.value(
                    kind,
                    20.0 * 1000.0_f64.powf(column as f64 / (plot_w - 1).max(1) as f64),
                )
            })
            .collect();
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let (low, high) = match kind {
            Response::Magnitude => (-48.0, 12.0),
            Response::Phase => ((min / 90.0).floor() * 90.0, 0.0),
            Response::GroupDelay => (0.0, (max * 1.2).max(1.0)),
            Response::ExcessDelay => (0.0, (max * 2.0).max(0.5)),
        };
        screen.put(
            x,
            y,
            width,
            &format!("{} / f_c {:.0}Hz", kind.unit(), model.cutoff),
            3,
        );
        let plot_y = y + 1;
        let plot_h = height - 2;
        for row in 0..plot_h {
            let value = high - row as f64 / (plot_h - 1).max(1) as f64 * (high - low);
            let label = if matches!(kind, Response::GroupDelay | Response::ExcessDelay) {
                format!("{value:4.2} ┊")
            } else {
                format!("{value:4.0} ┊")
            };
            screen.put(x, plot_y + row, 7, &label, 3);
            for column in 0..plot_w {
                if row % 2 == 0 && column % 4 == 0 {
                    screen.put(x + 7 + column, plot_y + row, 1, "·", 3);
                }
            }
        }
        let mut previous = 0;
        let loaded = if self.intro {
            let index = Response::ALL
                .iter()
                .position(|response| *response == kind)
                .unwrap_or(0);
            ((self.boot_started.elapsed().as_secs_f64() - 5.0 - index as f64 * 1.5) / 3.0)
                .clamp(0.0, 1.0)
        } else {
            1.0
        };
        for (column, value) in values
            .iter()
            .take((plot_w as f64 * loaded) as usize)
            .enumerate()
        {
            let row = ((high - value.clamp(low, high)) / (high - low) * (plot_h - 1) as f64).round()
                as usize;
            for fill in row + 1..plot_h {
                screen.put(x + 7 + column, plot_y + fill, 1, "░", 6);
            }
            if column > 0 {
                for join in previous.min(row)..=previous.max(row) {
                    screen.put(x + 7 + column, plot_y + join, 1, "│", 4);
                }
            }
            screen.put(x + 7 + column, plot_y + row, 1, "●", 7);
            previous = row;
        }
        screen.put(
            x,
            y + height - 1,
            width,
            " Hz  20   200   2k   20k / log",
            3,
        );
    }

    fn daemon(&mut self, shell: &Shell) {
        let token = noise(self.seed ^ self.tick);
        let commands: &[&str] = if shell.poopsec {
            &[
                "flushctl --fictional-firewall",
                "porcelain --throne acquire",
                "sewer-route --loopback",
                "waste-packet --decode",
                "clogscan --simulation",
            ]
        } else if shell.scene == "prostreet" {
            &[
                "prostreet --grid-stage",
                "nitrous --fictional-boost",
                "grip --telemetry",
                "drift --ascii-apex",
            ]
        } else if shell.scene == "vulcan" {
            &[
                "vulcan-z --virtual-bank",
                "t-force --scan-led",
                "mcp --sim-expand-512gb",
            ]
        } else if shell.mascot == "mo-krill" {
            &[
                "burrow --tunnel-rats",
                "krill --signal-lock",
                "mo --underground-route",
                "sandblast --simulated-noise",
                "combo --fictional-uplink",
            ]
        } else {
            &[
                "sigilctl --decode",
                "ghost-route --loopback",
                "matrix --synchronize",
                "spectral-lock --calibrate",
                "mainframe --sim-handshake",
                "cipher-lore --assemble",
                "trace-vector --fiction",
                "demon-gate --mask-awaken",
                "bayanihan --mesh-link-sim",
                "tara --matrix-entry",
                "salamat --signal-ack",
                "neon-exorcist --dance-burst",
                "construct --load-operator",
                "red-pill --fictional-sequence",
                "bido --decoy-sidestep",
                "tailscale-milk --churn-sim",
            ]
        };
        if self.scripts.len() > 80 {
            self.scripts.pop_front();
            self.scripts.pop_front();
        }
        self.scripts.push_back(format!(
            "[SIM] $ {} --seed {:04X}",
            commands[token as usize % commands.len()],
            token & 65535
        ));
        self.scripts.push_back(format!(
            "[SIM] {:04X} :: {}",
            (token >> 16) & 65535,
            if self.intro {
                "stage accepted / bootlock active"
            } else {
                "daemon cycle complete"
            }
        ));
        if self.intro || self.tick.is_multiple_of(20) {
            self.burst = Some(self.tick);
        }
    }

    fn poll_job(&mut self, shell: &mut Shell) {
        let Some(job) = self.job.as_mut() else {
            return;
        };
        match job.poll() {
            Ok((lines, status)) => {
                for line in lines {
                    self.log(line);
                }
                if let Some(status) = status {
                    shell.last_status = status;
                    if status == 0 {
                        shell.operations = shell.operations.saturating_add(1);
                    }
                    self.log(format!("[RETURN] exit {status} // command complete"));
                    self.burst = Some(self.tick);
                    self.job = None;
                }
            }
            Err(error) => {
                self.log(format!("[FAILED] {error}"));
                self.job = None;
                shell.last_status = 1;
            }
        }
    }

    fn submit(&mut self, shell: &mut Shell, terminal: &mut Terminal) -> io::Result<()> {
        let command = std::mem::take(&mut self.input);
        self.cursor = 0;
        if !command.starts_with(' ') {
            if shell.history.len() == 1000 {
                shell.history.remove(0);
            }
            shell.history.push(command.clone());
        }
        self.history_index = shell.history.len();
        self.draft.clear();
        // Space-prefixed commands also stay out of the visible session transcript.
        if !command.starts_with(' ') {
            self.log(format!("$ {command}"));
        }
        self.burst = Some(self.tick);
        if let Some(command) = if command.trim() == "shell" {
            Some("shell")
        } else {
            command.trim_start().strip_prefix("shell ")
        } {
            if self.job.is_some() {
                self.log("[BUSY] Cancel the running command with Ctrl-C first.");
                return Ok(());
            }
            terminal.suspend()?;
            shell.last_status = shell.execute(command);
            if shell.exit.is_some() {
                return Ok(());
            }
            println!(
                "\n[EXIT {}] Press Enter to return to NIGHTFALL.",
                shell.last_status
            );
            io::stdout().flush()?;
            let mut acknowledgement = String::new();
            io::stdin().read_line(&mut acknowledgement)?;
            terminal.resume()?;
            self.log(format!(
                "[RETURN] exit {} // terminal client closed",
                shell.last_status
            ));
        } else if command.trim() == "music demo" {
            self.music.set_demo(true);
            self.log("[MUSIC] demo enabled / fictional track");
        } else if command.trim() == "music live" {
            self.music.set_demo(false);
            self.log("[MUSIC] local playback adapter selected");
        } else if command.trim() == "music" || command.trim() == "music status" {
            self.log(self.music.line());
            self.log(Music::credentials());
        } else if is_builtin(&command)
            && (command.trim() == "soundtrack play"
                || command.trim_start().starts_with("soundtrack play "))
        {
            if self.job.is_some() {
                self.log("[BUSY] Cancel the running child with Ctrl-C before audio playback.");
            } else {
                match crate::words(&command)
                    .and_then(|args| shell.soundtrack_play_command(&args[1..]))
                    .and_then(Job::start)
                {
                    Ok(job) => {
                        self.job = Some(job);
                        self.log(
                            "[SOUNDTRACK] original score playing through the selected audio device",
                        );
                    }
                    Err(error) => {
                        shell.last_status = 1;
                        self.log(format!("[AUDIO FAILED] {error}"));
                    }
                }
            }
        } else if command.trim() == "sdvx play" {
            shell.scene = "sdvx".into();
            self.arcade_play = true;
            self.log("[SIM SDVX PLAY] ASDF/J K buttons; Q/E left knob; U/O right knob; Enter or ` returns to shell");
        } else if command.trim() == "graph novel" {
            self.graph_index = 7;
            self.log("[NOVEL] Lost Signals story tab selected");
        } else if command.trim() == "graph metrics" || command.trim() == "metrics" {
            self.graph_index = 6;
            self.log("[METRICS] actual session counters selected");
        } else if command.trim() == "graph headphones" {
            self.graph_index = 5;
            self.log("[HEADPHONES] invented display curve / not a measured headphone response");
        } else if command.trim() == "graph room" {
            self.graph_index = 4;
            self.log("[ROOM] RT60 / idealized Sabine decay displayed");
        } else if command.trim() == "graph next" {
            self.graph_index = (self.graph_index + 1) % 8;
        } else if command.trim() == "cinematic" {
            self.intro = true;
            self.boot_started = Instant::now();
            self.boot_step = 0;
            shell.uplink = None;
            self.scripts.clear();
            self.log("[SIM] Restarting the 15 second infiltration ceremony.");
        } else if command.trim() == "clear" {
            self.events.clear();
        } else if is_builtin(&command) {
            let color = shell.color;
            shell.color = false;
            *shell.transcript.borrow_mut() = Some(Vec::new());
            shell.last_status = shell.execute(&command);
            shell.color = color;
            let output = shell.transcript.borrow_mut().take().unwrap_or_default();
            if command.split_whitespace().next() == Some("novel") && shell.last_status == 0 {
                self.graph_index = 7;
            }
            for line in output {
                if line.contains("SIM SSD DETONATION") {
                    self.transition = Some(Instant::now());
                    self.burst = Some(self.tick);
                }
                self.log(line);
            }
        } else if self.job.is_some() {
            self.log("[BUSY] One command at a time. Ctrl-C cancels; built-ins remain available.");
        } else {
            match Job::start(shell.command(&command)) {
                Ok(job) => self.job = Some(job),
                Err(error) => {
                    self.log(format!("[FAILED] {error}"));
                    shell.last_status = 127;
                }
            }
        }
        Ok(())
    }

    fn boot_phase(&self, poopsec: bool) -> String {
        if !self.intro {
            return if poopsec {
                "PORCELAIN ARCHDEMON ONLINE"
            } else {
                "OPERATOR ONLINE"
            }
            .into();
        }
        let stages = if poopsec {
            [
                "SUMMONING PORCELAIN",
                "SEWER ROUTE ACQUIRED",
                "FLUSHING FICTIONAL FIREWALL",
                "ASCENDING THE THRONE",
            ]
        } else {
            [
                "SUMMONING CORE",
                "DECODING SIGIL",
                "INFILTRATING SIM MAINFRAME",
                "CALIBRATING PROTAGONIST",
            ]
        };
        format!(
            "{} {:.1}/15s",
            stages[((self.boot_started.elapsed().as_secs_f64() / 3.75) as usize).min(3)],
            self.boot_started.elapsed().as_secs_f64().min(15.0)
        )
    }

    fn history(&mut self, shell: &Shell, previous: bool) {
        if previous {
            if self.history_index == shell.history.len() {
                self.draft = self.input.clone();
            }
            self.history_index = self.history_index.saturating_sub(1);
        } else {
            self.history_index = (self.history_index + 1).min(shell.history.len());
        }
        self.input = shell
            .history
            .get(self.history_index)
            .cloned()
            .unwrap_or_else(|| self.draft.clone());
        self.cursor = self.input.chars().count();
    }
}

struct Canvas {
    cells: Vec<Vec<(char, u8)>>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![(' ', 0); width]; height],
        }
    }

    fn put(&mut self, x: usize, y: usize, width: usize, text: &str, style: u8) {
        if let Some(row) = self.cells.get_mut(y) {
            for (offset, character) in safe_line(text, width).chars().enumerate() {
                if let Some(cell) = row.get_mut(x + offset) {
                    *cell = (character, style);
                }
            }
        }
    }

    fn panel(&mut self, x: usize, y: usize, width: usize, height: usize, title: &str) {
        if width < 2 || height < 2 {
            return;
        }
        self.put(x, y, width, &format!("╭{}╮", "─".repeat(width - 2)), 3);
        self.put(
            x,
            y + height - 1,
            width,
            &format!("╰{}╯", "─".repeat(width - 2)),
            3,
        );
        self.put(x + 2, y, width.saturating_sub(4), &format!(" {title} "), 1);
        for row in y + 1..y + height - 1 {
            self.put(x, row, 1, "│", 3);
            self.put(x + width - 1, row, 1, "│", 3);
        }
    }

    fn render(&self, shell: &Shell) -> String {
        // Keep the terminal from displaying half a frame or wrapping wide glyphs.
        // Absolute row positioning avoids scrolling if the window shrinks between
        // size polls. Unsupported synchronized-output mode is ignored by terminals.
        let mut frame = String::from("\x1b[?2026h\x1b[?7l\x1b[H");
        let mut previous = u8::MAX;
        for (index, row) in self.cells.iter().enumerate() {
            frame.push_str(&format!("\x1b[{};1H", index + 1));
            for &(character, style) in row {
                if shell.color && style != previous {
                    frame.push_str("\x1b[0;");
                    frame.push_str(match style {
                        1 => shell.accent(),
                        2 => "38;5;252",
                        3 => "38;5;244",
                        4 => "38;5;82",
                        5 => "38;5;214",
                        6 => {
                            if shell.poopsec {
                                "38;5;94"
                            } else {
                                "38;5;22"
                            }
                        }
                        7 => "1;38;5;157",
                        8 => "48;5;157;38;5;16",
                        9 => "48;5;255;38;5;255",
                        10 => "38;5;22",
                        11 => "38;5;28",
                        12 => "38;5;34",
                        13 => "38;5;82",
                        _ => "0",
                    });
                    frame.push('m');
                    previous = style;
                }
                frame.push(character);
            }
            frame.push_str("\x1b[K");
        }
        if shell.color {
            frame.push_str("\x1b[0m");
        }
        frame.push_str("\x1b[?7h\x1b[?2026l");
        frame
    }
}

fn noise(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58476d1ce4e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

pub fn run(shell: &mut Shell, intro: bool) -> io::Result<u8> {
    let mut terminal = Terminal::new()?;
    let mut deck = Deck::new(intro);
    if intro {
        deck.log("15 SECOND INFILTRATION // theatrical simulator sequence");
    } else {
        deck.local(shell);
    }
    deck.log(if shell.poopsec {
        "ORACLE: THE THRONE HAS BEEN EXPECTING YOU."
    } else {
        "ORACLE: THE CITY HAS A MILLION TERMINALS. THIS ONE IS YOURS."
    });
    let stdin = io::stdin();
    let mut refresh = Instant::now();
    deck.daemon(shell);
    loop {
        let frame_started = Instant::now();
        deck.poll_job(shell);
        if let Some(event) = shell.stage.tick() {
            deck.log(event);
        }
        if shell.stage.take_celebration() {
            deck.celebration = Some(deck.tick);
            deck.burst = Some(deck.tick);
        }
        if shell.arcade.take_celebration() {
            deck.celebration = Some(deck.tick);
            deck.burst = Some(deck.tick);
        }
        deck.music.poll();
        deck.prompt
            .poll(&cwd(), shell.last_status, deck.size.0, &shell.environment);
        deck.bido_active = shell.bido.is_active();
        if let Some(event) = shell.countries.tick() {
            deck.log(event);
            deck.celebration = Some(deck.tick);
        }
        if let Some(event) = shell.ssd.tick() {
            deck.log(event);
        }
        if let Some(event) = shell.egg.tick() {
            deck.log(event);
            deck.burst = Some(deck.tick);
        }
        if deck.intro {
            let step = (deck.boot_started.elapsed().as_secs_f64() * 12.0) as u64;
            if step != deck.boot_step {
                deck.daemon(shell);
                deck.boot_step = step;
            }
        } else if deck.tick.is_multiple_of(60) {
            deck.daemon(shell);
        }
        if refresh.elapsed().as_secs() >= 1 {
            deck.size = dimensions();
            refresh = Instant::now();
        }
        if deck.intro && deck.boot_started.elapsed() >= theater::BOOT_DURATION {
            deck.intro = false;
            deck.transition = Some(Instant::now());
            deck.burst = Some(deck.tick);
            deck.local(shell);
            if shell.uplink.is_some() {
                deck.log("INFILTRATION COMPLETE // SAY IT OUT LOUD: I'M IN.");
                print!("\x07");
            }
        }
        {
            let frame = deck.render(shell);
            let mut output = io::stdout().lock();
            output.write_all(frame.as_bytes())?;
            output.flush()?;
        }
        let mut input = [0u8; 256];
        let count = stdin.lock().read(&mut input)?;
        if count == 0 && deck.tick.saturating_sub(deck.escape_tick) > 9 {
            deck.escape = 0;
        }
        if deck.intro {
            if input[..count].contains(&4) {
                return Ok(0);
            }
            deck.tick = deck.tick.wrapping_add(1);
            if let Some(delay) = Duration::from_millis(33).checked_sub(frame_started.elapsed()) {
                std::thread::sleep(delay);
            }
            continue;
        }
        for &byte in &input[..count] {
            if deck.arcade_play && (byte >= 32 || matches!(byte, b'\r' | b'\n')) {
                if matches!(byte, b'`' | b'\r' | b'\n') {
                    deck.arcade_play = false;
                    deck.log("[SIM SDVX] controller released // shell input restored");
                } else {
                    let action: Option<Vec<String>> = match byte.to_ascii_lowercase() {
                        b'a' => Some(vec!["sdvx".into(), "hit".into(), "bt-a".into()]),
                        b's' => Some(vec!["sdvx".into(), "hit".into(), "bt-b".into()]),
                        b'd' => Some(vec!["sdvx".into(), "hit".into(), "bt-c".into()]),
                        b'f' => Some(vec!["sdvx".into(), "hit".into(), "bt-d".into()]),
                        b'j' => Some(vec!["sdvx".into(), "hit".into(), "fx-l".into()]),
                        b'k' => Some(vec!["sdvx".into(), "hit".into(), "fx-r".into()]),
                        b'q' | b'e' => Some(vec![
                            "sdvx".into(),
                            "knob".into(),
                            "left".into(),
                            if byte.eq_ignore_ascii_case(&b'q') {
                                "-5"
                            } else {
                                "5"
                            }
                            .into(),
                        ]),
                        b'u' | b'o' => Some(vec![
                            "sdvx".into(),
                            "knob".into(),
                            "right".into(),
                            if byte.eq_ignore_ascii_case(&b'u') {
                                "-5"
                            } else {
                                "5"
                            }
                            .into(),
                        ]),
                        _ => None,
                    };
                    if let Some(args) = action {
                        match shell.arcade.command(&args) {
                            Ok(lines) => {
                                for line in lines {
                                    deck.log(line);
                                }
                            }
                            Err(error) => deck.log(error),
                        }
                    }
                }
                continue;
            }
            if deck.escape != 0 {
                match (deck.escape, byte) {
                    (1, b'[' | b'O') => deck.escape = 2,
                    (2, b'C') => {
                        deck.cursor = (deck.cursor + 1).min(deck.input.chars().count());
                        deck.escape = 0;
                    }
                    (2, b'D') => {
                        deck.cursor = deck.cursor.saturating_sub(1);
                        deck.escape = 0;
                    }
                    (2, b'H') => {
                        deck.cursor = 0;
                        deck.escape = 0;
                    }
                    (2, b'F') => {
                        deck.cursor = deck.input.chars().count();
                        deck.escape = 0;
                    }
                    (2, b'5') => deck.escape = 5,
                    (2, b'6') => deck.escape = 6,
                    (5, b'~') => {
                        deck.scroll = (deck.scroll + 5).min(deck.events.len().saturating_sub(1));
                        deck.escape = 0;
                    }
                    (6, b'~') => {
                        deck.scroll = deck.scroll.saturating_sub(5);
                        deck.escape = 0;
                    }
                    (2, b'A') => {
                        deck.history(shell, true);
                        deck.escape = 0;
                    }
                    (2, b'B') => {
                        deck.history(shell, false);
                        deck.escape = 0;
                    }
                    (_, b'@'..=b'~') => deck.escape = 0,
                    _ => {}
                }
                continue;
            }
            match byte {
                27 => {
                    deck.escape = 1;
                    deck.escape_tick = deck.tick;
                    deck.utf8.clear();
                }
                4 => return Ok(shell.last_status.clamp(0, 255) as u8),
                3 => {
                    if let Some(job) = deck.job.as_mut() {
                        job.cancel();
                    }
                    deck.input.clear();
                    deck.cursor = 0;
                    deck.utf8.clear();
                    deck.log("COMMAND BUFFER PURGED // the protagonist takes a breath");
                }
                12 => deck.local(shell),
                6 => {
                    deck.graph_index = (deck.graph_index + 1) % 8;
                    deck.log(format!(
                        "[PANEL] {}",
                        Response::ALL
                            .get(deck.graph_index)
                            .map(|kind| kind.label())
                            .unwrap_or(if deck.graph_index == 4 {
                                "RT60 / ROOM DECAY"
                            } else if deck.graph_index == 5 {
                                "HEADPHONE EQ / UNMEASURED"
                            } else if deck.graph_index == 6 {
                                "METRICS / ACTUAL SESSION"
                            } else {
                                "LOST SIGNALS / VISUAL NOVEL"
                            })
                    ));
                }
                7 => {
                    for line in shell
                        .stage
                        .command(&["pedal".into(), "stomp".into()])
                        .unwrap_or_default()
                    {
                        deck.log(line);
                    }
                }
                1 => deck.cursor = 0,
                5 => deck.cursor = deck.input.chars().count(),
                24 => {
                    deck.burst = Some(deck.tick);
                    deck.log("[FX] PARTICLE DETONATION // decorative");
                }
                20 => {
                    shell.mascot = if shell.mascot == "mo-krill" {
                        "demon"
                    } else {
                        "mo-krill"
                    }
                    .into();
                    deck.burst = Some(deck.tick);
                    deck.log(format!("[MASCOT] {} // crew deployed", shell.mascot));
                }
                2 => {
                    shell.poopsec = !shell.poopsec;
                    deck.log(if shell.poopsec {
                        "POOPSEC DEMON // PORCELAIN THRONE ACQUIRED"
                    } else {
                        "OPSEC DEMON // GHOST CONSOLE RESTORED"
                    });
                }
                16 => deck.ping(shell),
                14 => {
                    shell.mission = (shell.mission + 1).min(theater::MISSIONS.len() - 1);
                    deck.log(theater::MISSIONS[shell.mission].0);
                }
                9 => {
                    let completed = crate::completion::complete_with_environment(
                        &deck.input,
                        deck.cursor,
                        &shell.environment,
                    );
                    if completed.input.len() <= 4096 {
                        if completed.input == deck.input && completed.choices.len() > 1 {
                            deck.log(format!(
                                "[COMPLETE] {}",
                                completed
                                    .choices
                                    .iter()
                                    .take(32)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join("  ")
                            ));
                        }
                        deck.input = completed.input;
                        deck.cursor = completed.cursor;
                    }
                }
                127 | 8 => {
                    if deck.cursor > 0 {
                        let index = deck
                            .input
                            .char_indices()
                            .nth(deck.cursor - 1)
                            .map(|(index, _)| index)
                            .unwrap_or(0);
                        deck.input.remove(index);
                        deck.cursor -= 1;
                    }
                    deck.utf8.clear();
                }
                21 => {
                    deck.input.clear();
                    deck.cursor = 0;
                    deck.utf8.clear();
                }
                b'\r' | b'\n' if !deck.input.trim().is_empty() => {
                    deck.submit(shell, &mut terminal)?;
                    if let Some(code) = shell.exit {
                        return Ok(code);
                    }
                }
                32..=126 => {
                    if deck.input.len() < 4096 {
                        let index = deck
                            .input
                            .char_indices()
                            .nth(deck.cursor)
                            .map(|(index, _)| index)
                            .unwrap_or(deck.input.len());
                        deck.input.insert(index, byte as char);
                        deck.cursor += 1;
                    }
                }
                128..=255 => {
                    deck.utf8.push(byte);
                    if let Ok(text) = std::str::from_utf8(&deck.utf8) {
                        if deck.input.len() + text.len() <= 4096 {
                            let index = deck
                                .input
                                .char_indices()
                                .nth(deck.cursor)
                                .map(|(index, _)| index)
                                .unwrap_or(deck.input.len());
                            deck.input.insert_str(index, text);
                            deck.cursor += text.chars().count();
                        }
                        deck.utf8.clear();
                    } else if deck.utf8.len() >= 4 {
                        deck.utf8.clear();
                    }
                }
                _ => {}
            }
        }
        deck.tick = deck.tick.wrapping_add(1);
        if deck.tick.is_multiple_of(240) {
            let transmissions = if shell.poopsec {
                [
                    "ORACLE [FICTION]: THE PORCELAIN MAINFRAME KNOWS YOUR NAME.",
                    "ORACLE [FICTION]: FLUSH THE LORE. KEEP THE DREAM.",
                    "ORACLE [FICTION]: THE SEWER SIGNAL IS COMING FROM WITHIN.",
                    "ORACLE [FICTION]: I CAME. I SAW. I CLOGGED.",
                ]
            } else {
                [
                    "ORACLE [FICTION]: THE MACHINE DREAMS IN YOUR CALLSIGN.",
                    "ORACLE [FICTION]: THE CURSOR IS YOUR CROSSHAIR. BUILD SOMETHING.",
                    "ORACLE [FICTION]: EVERY LEGEND NEEDS A TERMINAL WINDOW.",
                    "ORACLE [FICTION]: THE SIGNAL WAS COMING FROM INSIDE THE MAINFRAME.",
                ]
            };
            deck.log(transmissions[(deck.tick / 240) as usize % transmissions.len()]);
        }
        if let Some(delay) = Duration::from_millis(33).checked_sub(frame_started.elapsed()) {
            std::thread::sleep(delay);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bsd_terminal_min_and_time_round_trip_as_decimal() {
        assert_eq!(
            restorable_stty("gfmt1:cflag=4b00:min=10:time=ff:eof=4:ispeed=9600".into()),
            "gfmt1:cflag=4b00:min=16:time=255:eof=4:ispeed=9600"
        );
        assert_eq!(
            restorable_stty("gfmt1:min=1:time=0".into()),
            "gfmt1:min=1:time=0"
        );
        assert_eq!(
            restorable_stty("500:5:bf:8a3b:3:1c:7f".into()),
            "500:5:bf:8a3b:3:1c:7f"
        );
    }

    use super::*;

    #[test]
    fn frames_fit_small_terminals_and_remove_control_sequences() {
        let shell = Shell::new(false);
        let mut deck = Deck::new(false);
        for size in [(20, 8), (80, 24), (120, 40)] {
            deck.size = size;
            let frame = deck.render(&shell);
            assert!(!frame.contains(['\r', '\n']));
            for row in 1..=size.1 {
                assert!(frame.contains(&format!("\x1b[{row};1H")));
            }
            assert!(frame.contains("EXECUTE >"));
        }
        assert_eq!(safe_line("evil\x1b\n\x07text", 30), "eviltext");
    }

    #[test]
    fn frames_bracket_redraws_and_wide_output_cannot_trigger_linefeed_scrolling() {
        let shell = Shell::new(false);
        let mut canvas = Canvas::new(79, 44);
        canvas.put(0, 0, 79, &"界🎵".repeat(40), 1);
        let frame = canvas.render(&shell);
        assert!(frame.starts_with("\x1b[?2026h\x1b[?7l\x1b[H"));
        assert!(frame.ends_with("\x1b[?7h\x1b[?2026l"));
        assert!(!frame.contains(['\r', '\n']));
        assert!(!frame.contains("\x1b[2J"));
        assert!(frame.contains("\x1b[44;1H"));
    }

    #[test]
    fn history_restores_draft_and_animation_changes() {
        let mut shell = Shell::new(false);
        shell.history = vec!["status".into(), "oracle".into()];
        let mut deck = Deck::new(false);
        deck.history_index = 2;
        deck.input = "draft".into();
        deck.history(&shell, true);
        assert_eq!(deck.input, "oracle");
        deck.history(&shell, false);
        assert_eq!(deck.input, "draft");
        assert_ne!(noise(13), noise(14));
        assert!(Model::default().value(Response::Magnitude, 20.0).abs() < 0.1);
        assert!(Model::default().value(Response::Magnitude, 20000.0) < -40.0);
    }

    #[test]
    fn all_named_palettes_color_panels_and_plain_mode_removes_sgr() {
        let mut shell = Shell::new(true);
        let mut deck = Deck::new(false);
        deck.size = (140, 44);
        deck.graph_index = 6;
        for (theme, color) in [
            ("crimson", "38;5;196"),
            ("ghost", "38;5;51"),
            ("venom", "38;5;82"),
        ] {
            shell.theme = theme.into();
            assert!(deck.render(&shell).contains(color));
        }
        shell.color = false;
        assert!(!deck.render(&shell).contains("38;5;"));
    }

    #[test]
    fn original_mascots_render_with_named_panels() {
        let mut shell = Shell::new(false);
        let mut deck = Deck::new(false);
        deck.size = (140, 44);
        for name in ["tweeter", "krillion"] {
            assert_eq!(shell.execute(&format!("mascot {name}")), 0);
            assert!(deck.render(&shell).contains(&name.to_uppercase()));
        }
    }

    #[test]
    fn novel_tab_shows_story_and_keeps_full_transcript_navigation() {
        let mut shell = Shell::new(false);
        shell.novel.command(&["start".into()]).unwrap();
        let mut deck = Deck::new(false);
        deck.graph_index = 7;
        deck.size = (140, 44);
        let frame = deck.render(&shell);
        assert!(frame.contains("VISUAL NOVEL"));
        assert!(frame.contains("STATION WITHOUT A NAME"));
        assert!(frame.contains("PgUp/PgDn"));
    }

    #[test]
    fn metrics_panel_uses_actual_counters_and_fits_resizes() {
        let mut shell = Shell::new(false);
        shell.operations = 17;
        let mut deck = Deck::new(false);
        deck.graph_index = 6;
        deck.tick = 90;
        for size in [(140, 44), (100, 30), (80, 24)] {
            deck.size = size;
            let frame = deck.render(&shell);
            assert!(frame.contains("METRICS / ACTUAL"));
            assert!(frame.contains("Successful operations: 17"));
            assert!(frame.contains("Render ticks: 90"));
        }
    }

    #[test]
    fn full_screen_boot_brings_graphs_online_in_order_then_reveals_dashboard() {
        let shell = Shell::new(false);
        let mut deck = Deck::new(true);
        deck.size = (140, 44);
        let first = deck.render(&shell);
        assert!(first.contains("MATRIX ENTRY"));
        assert!(first.contains("SENSOR / WAITING"));
        assert!(!first.contains("EXECUTE >"));
        assert!(!first.contains("FREQUENCY RESPONSE"));
        deck.boot_started = Instant::now() - Duration::from_secs(7);
        let mid = deck.render(&shell);
        assert!(mid.contains("FREQUENCY RESPONSE"));
        assert!(mid.contains("PHASE RESPONSE"));
        assert!(mid.contains("SENSOR / WAITING"));
        assert!(mid.contains('●'));
        deck.boot_started = Instant::now() - Duration::from_secs(14);
        let last = deck.render(&shell);
        for response in Response::ALL {
            assert!(last.contains(response.label()));
        }
        deck.intro = false;
        deck.transition = Some(Instant::now());
        assert!(deck.render(&shell).contains("CONSTRUCT BREACHED"));
        deck.transition = Some(Instant::now() - Duration::from_secs(4));
        assert!(deck.render(&shell).contains("EXECUTE >"));
    }

    #[test]
    fn palette_and_poopsec_frames_preserve_the_boot_contract() {
        let mut shell = Shell::new(true);
        let mut deck = Deck::new(true);
        deck.size = (110, 30);
        let frame = deck.render(&shell);
        assert!(frame.contains("38;5;196"));
        assert!(frame.contains("BOOTLOCK"));
        assert!(!frame.contains("I'M IN."));
        assert_eq!(theater::BOOT_DURATION.as_secs(), 15);
        shell.poopsec = true;
        let frame = deck.render(&shell);
        assert!(frame.contains("POOPSEC"));
        assert!(frame.contains("38;5;214"));
        assert!(frame.contains("PORCELAIN"));
        shell.color = false;
        assert!(!deck.render(&shell).contains("38;5;"));
    }
}
