//! Native Windows stage. GDI artwork is procedural illustration, never telemetry.
//! All Win32 calls and their resource lifetimes stay on the window thread.
#![allow(clippy::upper_case_acronyms)]
use crate::{Shell, is_builtin, job::Job, music::Music};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::io;
use std::ptr::null_mut;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

type Handle = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Class {
    style: u32,
    procedure: unsafe extern "system" fn(Handle, u32, usize, isize) -> isize,
    extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
#[derive(Default)]
struct Message {
    window: Handle,
    message: u32,
    w: usize,
    l: isize,
    time: u32,
    point: Point,
    private: u32,
}
#[repr(C)]
struct Paint {
    dc: Handle,
    erase: i32,
    rect: Rect,
    restore: i32,
    update: i32,
    reserved: [u8; 32],
}
#[repr(C)]
struct Create {
    data: Handle,
    instance: Handle,
    menu: Handle,
    parent: Handle,
    height: i32,
    width: i32,
    y: i32,
    x: i32,
    style: i32,
    name: *const u16,
    class: *const u16,
    extended: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const Class) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        data: Handle,
    ) -> Handle;
    fn DefWindowProcW(window: Handle, message: u32, w: usize, l: isize) -> isize;
    fn GetMessageW(message: *mut Message, window: Handle, min: u32, max: u32) -> i32;
    fn TranslateMessage(message: *const Message) -> i32;
    fn DispatchMessageW(message: *const Message) -> isize;
    fn SetWindowLongPtrW(window: Handle, index: i32, value: isize) -> isize;
    fn GetWindowLongPtrW(window: Handle, index: i32) -> isize;
    fn ShowWindow(window: Handle, command: i32) -> i32;
    fn DestroyWindow(window: Handle) -> i32;
    fn PostQuitMessage(code: i32);
    fn GetClientRect(window: Handle, rect: *mut Rect) -> i32;
    fn SetTimer(window: Handle, id: usize, interval: u32, callback: Handle) -> usize;
    fn KillTimer(window: Handle, id: usize) -> i32;
    fn InvalidateRect(window: Handle, rect: *const Rect, erase: i32) -> i32;
    fn BeginPaint(window: Handle, paint: *mut Paint) -> Handle;
    fn EndPaint(window: Handle, paint: *const Paint) -> i32;
    fn FillRect(dc: Handle, rect: *const Rect, brush: Handle) -> i32;
    fn DrawTextW(dc: Handle, text: *const u16, len: i32, rect: *mut Rect, format: u32) -> i32;
    fn MoveWindow(window: Handle, x: i32, y: i32, w: i32, h: i32, repaint: i32) -> i32;
    fn SetFocus(window: Handle) -> Handle;
    fn GetWindowTextW(window: Handle, text: *mut u16, count: i32) -> i32;
    fn SetWindowTextW(window: Handle, text: *const u16) -> i32;
    fn SendMessageW(window: Handle, message: u32, w: usize, l: isize) -> isize;
    fn LoadCursorW(instance: Handle, name: *const u16) -> Handle;
    fn SetProcessDpiAwarenessContext(context: isize) -> i32;
    fn GetDC(window: Handle) -> Handle;
    fn ReleaseDC(window: Handle, dc: Handle) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn FreeConsole() -> i32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: u32) -> Handle;
    fn DeleteObject(object: Handle) -> i32;
    fn SelectObject(dc: Handle, object: Handle) -> Handle;
    fn CreatePen(style: i32, width: i32, color: u32) -> Handle;
    fn MoveToEx(dc: Handle, x: i32, y: i32, old: *mut Point) -> i32;
    fn LineTo(dc: Handle, x: i32, y: i32) -> i32;
    fn Ellipse(dc: Handle, l: i32, t: i32, r: i32, b: i32) -> i32;
    fn SetTextColor(dc: Handle, color: u32) -> u32;
    fn SetBkMode(dc: Handle, mode: i32) -> i32;
    fn SetBkColor(dc: Handle, color: u32) -> u32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike: u32,
        charset: u32,
        out: u32,
        clip: u32,
        quality: u32,
        pitch: u32,
        face: *const u16,
    ) -> Handle;
    fn CreateCompatibleDC(dc: Handle) -> Handle;
    fn CreateCompatibleBitmap(dc: Handle, w: i32, h: i32) -> Handle;
    fn DeleteDC(dc: Handle) -> i32;
    fn BitBlt(
        dest: Handle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        source: Handle,
        sx: i32,
        sy: i32,
        op: u32,
    ) -> i32;
    fn GetDIBits(
        dc: Handle,
        bitmap: Handle,
        start: u32,
        lines: u32,
        bits: Handle,
        info: *mut BitmapInfo,
        usage: u32,
    ) -> i32;
}
#[repr(C)]
struct BitmapInfo {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits: u16,
    compression: u32,
    image: u32,
    xp: i32,
    yp: i32,
    used: u32,
    important: u32,
    colors: [u32; 3],
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
fn rgb(r: u32, g: u32, b: u32) -> u32 {
    r | g << 8 | b << 16
}
const CYAN: u32 = 0xefd95a;
const RED: u32 = 0x7154ff;
const WHITE: u32 = 0xefe7df;
const MUTED: u32 = 0xa38a76;
const BUTTONS: [(&str, &str); 8] = [
    ("01  UPLINK", "mainframe ping"),
    ("02  MISSION", "mission next"),
    ("03  DOSSIER", "dossier"),
    ("04  ORACLE", "oracle"),
    ("05  GOOSE EGG", "egg start"),
    ("06  SOUL BANK", "mcp expand"),
    ("07  DISCORD", "discord status"),
    ("08  VENOM", "theme venom"),
];
enum Request {
    Execute(String),
    Cancel,
    Stop,
}
enum Update {
    Line(String),
    Busy(bool),
    Music(String),
    Closed,
    Theme(String),
}

fn worker(mut shell: Shell, input: Receiver<Request>, output: Sender<Update>) {
    *shell.transcript.borrow_mut() = Some(Vec::new());
    let _ = shell.local();
    if std::env::var_os("DISCORD_APPLICATION_ID").is_some() {
        let _ = shell.execute("discord on");
    }
    let mut music = Music::new();
    let mut job: Option<Job> = None;
    loop {
        music.poll();
        let _ = output.send(Update::Music(music.line()));
        match input.recv_timeout(Duration::from_millis(30)) {
            Ok(Request::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Request::Cancel) => {
                if let Some(job) = &mut job {
                    job.cancel();
                }
            }
            Ok(Request::Execute(line)) => {
                if job.is_some() {
                    let _ = output.send(Update::Line(
                        "[LOCAL] A command is running. Use STOP before submitting another.".into(),
                    ));
                    continue;
                }
                let _ = output.send(Update::Line(format!("> {line}")));
                if !line.starts_with(' ') {
                    if shell.history.len() == 1000 {
                        shell.history.remove(0);
                    }
                    shell.history.push(line.clone());
                }
                if line.trim() == "shell" || line.trim() == "sdvx play" {
                    let _ = output.send(Update::Line(
                        "[LOCAL] Interactive terminal apps require --shell in a terminal.".into(),
                    ));
                } else if matches!(
                    line.trim(),
                    "music status" | "music live" | "music demo" | "music"
                ) {
                    if line.trim() == "music demo" {
                        music.set_demo(true);
                    }
                    if line.trim() == "music live" {
                        music.set_demo(false);
                    }
                    let _ = output.send(Update::Line(music.line()));
                    let _ = output.send(Update::Line(Music::credentials().into()));
                } else if is_builtin(&line) && !line.starts_with("soundtrack play") {
                    shell.last_status = shell.execute(&line);
                    let _ = output.send(Update::Theme(shell.theme.clone()));
                    if let Some(lines) = shell.transcript.borrow_mut().as_mut() {
                        for line in lines.drain(..) {
                            let _ = output.send(Update::Line(line));
                        }
                    }
                    if shell.exit.is_some() {
                        break;
                    }
                } else {
                    let command = if line.starts_with("soundtrack play") {
                        crate::words(&line)
                            .and_then(|args| shell.soundtrack_play_command(&args[1..]))
                    } else {
                        Ok(shell.command(&line))
                    };
                    match command.and_then(Job::start) {
                        Ok(next) => {
                            job = Some(next);
                            let _ = output.send(Update::Busy(true));
                        }
                        Err(error) => {
                            let _ = output.send(Update::Line(format!("[LOCAL / ERROR] {error}")));
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(active) = &mut job {
            match active.poll() {
                Ok((lines, status)) => {
                    for line in lines {
                        let _ = output.send(Update::Line(line));
                    }
                    if let Some(status) = status {
                        shell.last_status = status;
                        if status == 0 {
                            shell.operations = shell.operations.saturating_add(1);
                        }
                        shell.update_discord();
                        let _ =
                            output.send(Update::Line(format!("[LOCAL] process exited {status}")));
                        let _ = output.send(Update::Busy(false));
                        job = None;
                    }
                }
                Err(error) => {
                    let _ = output.send(Update::Line(format!("[LOCAL / ERROR] {error}")));
                    job = None;
                    let _ = output.send(Update::Busy(false));
                }
            }
        }
    }
    let _ = output.send(Update::Closed);
}

struct Deck {
    requests: Sender<Request>,
    updates: Receiver<Update>,
    lines: VecDeque<String>,
    edit: Handle,
    started: Instant,
    music: String,
    busy: bool,
    paused: bool,
    scroll: usize,
    smoke: bool,
    smoke_ok: bool,
    closed: bool,
    theme: String,
    edit_brush: Handle,
    edit_font: Handle,
}
impl Deck {
    fn submit(&mut self, line: String) {
        if !line.trim().is_empty() {
            let _ = self.requests.send(Request::Execute(line));
            self.scroll = 0;
        }
    }
    fn collect(&mut self) {
        for update in self.updates.try_iter().take(256) {
            match update {
                Update::Line(line) => {
                    if line.contains("WINDOWS_GUI_SMOKE") && !line.starts_with('>') {
                        self.smoke_ok = true;
                    }
                    if self.lines.len() == 1500 {
                        self.lines.pop_front();
                    }
                    self.lines.push_back(line);
                }
                Update::Busy(busy) => self.busy = busy,
                Update::Music(line) => self.music = line,
                Update::Closed => self.closed = true,
                Update::Theme(theme) => self.theme = theme,
            }
        }
    }
}

// Canvas coordinates use a 1600 x 900 design space; all artwork/text is redrawn at
// the physical client resolution, including 3840 x 2160 and per-monitor DPI changes.
struct Canvas {
    dc: Handle,
    sx: f64,
    sy: f64,
}
impl Canvas {
    fn x(&self, x: f64) -> i32 {
        (x * self.sx) as i32
    }
    fn y(&self, y: f64) -> i32 {
        (y * self.sy) as i32
    }
    fn rect(&self, x: f64, y: f64, w: f64, h: f64, color: u32) {
        unsafe {
            let brush = CreateSolidBrush(color);
            FillRect(
                self.dc,
                &Rect {
                    left: self.x(x),
                    top: self.y(y),
                    right: self.x(x + w),
                    bottom: self.y(y + h),
                },
                brush,
            );
            DeleteObject(brush);
        }
    }
    fn line(&self, a: (f64, f64), b: (f64, f64), color: u32, width: i32) {
        unsafe {
            let pen = CreatePen(
                0,
                (f64::from(width) * self.sx.min(self.sy)).max(1.0) as i32,
                color,
            );
            let old = SelectObject(self.dc, pen);
            MoveToEx(self.dc, self.x(a.0), self.y(a.1), null_mut());
            LineTo(self.dc, self.x(b.0), self.y(b.1));
            SelectObject(self.dc, old);
            DeleteObject(pen);
        }
    }
    fn ring(&self, x: f64, y: f64, r: f64, color: u32) {
        unsafe {
            let pen = CreatePen(0, 1, color);
            let old = SelectObject(self.dc, pen);
            // NULL_BRUSH stock object avoids filling the radar's concentric rings.
            let brush = GetStockObject(5);
            let previous = SelectObject(self.dc, brush);
            Ellipse(
                self.dc,
                self.x(x - r),
                self.y(y - r),
                self.x(x + r),
                self.y(y + r),
            );
            SelectObject(self.dc, previous);
            SelectObject(self.dc, old);
            DeleteObject(pen);
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn text(&self, x: f64, y: f64, w: f64, h: f64, size: i32, color: u32, text: &str) {
        unsafe {
            let font = CreateFontW(
                -(f64::from(size) * self.sy) as i32,
                0,
                0,
                0,
                400,
                0,
                0,
                0,
                1,
                0,
                0,
                5,
                0,
                wide("Consolas").as_ptr(),
            );
            let old = SelectObject(self.dc, font);
            SetTextColor(self.dc, color);
            SetBkMode(self.dc, 1);
            let text = wide(text);
            let mut rect = Rect {
                left: self.x(x),
                top: self.y(y),
                right: self.x(x + w),
                bottom: self.y(y + h),
            };
            DrawTextW(
                self.dc,
                text.as_ptr(),
                (text.len() - 1) as i32,
                &mut rect,
                0x800 | 0x20,
            ); // ellipsis, single line
            SelectObject(self.dc, old);
            DeleteObject(font);
        }
    }
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetStockObject(index: i32) -> Handle;
}

fn render(canvas: &Canvas, deck: &Deck) {
    let c = canvas;
    let (accent, secondary) = match deck.theme.as_str() {
        "venom" => (rgb(103, 255, 164), rgb(90, 217, 239)),
        "ghost" => (rgb(223, 231, 239), rgb(173, 168, 244)),
        _ => (RED, CYAN),
    };
    let t = if deck.paused {
        0.0
    } else {
        deck.started.elapsed().as_secs_f64()
    };
    for row in 0..90 {
        c.rect(
            0.0,
            f64::from(row) * 10.0,
            1600.0,
            10.0,
            rgb(6 + row / 20, 10 + row / 12, 20 + row / 5),
        );
    }
    for x in (0..1600).step_by(40) {
        c.line(
            (f64::from(x), 120.0),
            (f64::from(x), 900.0),
            rgb(15, 29, 42),
            1,
        );
    }
    for y in (120..900).step_by(40) {
        c.line(
            (0.0, f64::from(y)),
            (1600.0, f64::from(y)),
            rgb(15, 29, 42),
            1,
        );
    }
    c.rect(28.0, 26.0, 5.0, 62.0, accent);
    c.text(
        48.0,
        20.0,
        1040.0,
        48.0,
        38,
        WHITE,
        "OPSEC DEMON / NIGHTFALL",
    );
    c.text(
        50.0,
        74.0,
        900.0,
        26.0,
        14,
        MUTED,
        "PORCELAIN DIRECTORATE     //     CLEARANCE ABYSS-999     //     SIM THEATER",
    );
    c.text(
        1200.0,
        30.0,
        370.0,
        28.0,
        17,
        secondary,
        "NATIVE WINDOWS COMMAND DECK",
    );
    c.text(
        1240.0,
        63.0,
        330.0,
        28.0,
        14,
        MUTED,
        &format!("LOCAL SESSION  {:02}:{:02}", t as u64 / 60, t as u64 % 60),
    );
    c.line((28.0, 112.0), (1572.0, 112.0), rgb(40, 73, 90), 1);
    // The left-hand controls map directly to existing shell builtins.
    c.text(
        32.0,
        140.0,
        250.0,
        25.0,
        14,
        secondary,
        "OPERATOR / ACTIONS",
    );
    for (index, (label, _)) in BUTTONS.iter().enumerate() {
        let y = 190.0 + index as f64 * 62.0;
        c.rect(28.0, y, 252.0, 46.0, rgb(14, 27, 39));
        c.rect(
            28.0,
            y,
            3.0,
            46.0,
            if index == 6 { secondary } else { accent },
        );
        c.text(44.0, y + 13.0, 222.0, 25.0, 16, WHITE, label);
    }
    c.text(32.0, 705.0, 250.0, 25.0, 14, accent, "FICTIONAL AUTHORITY");
    c.text(
        32.0,
        739.0,
        250.0,
        25.0,
        13,
        MUTED,
        "REAL COMMANDS / cmd.exe",
    );
    c.text(
        32.0,
        767.0,
        250.0,
        25.0,
        13,
        MUTED,
        "No anonymity is provided.",
    );
    // Procedural demon illustration with orbital rings and an animated heartbeat.
    let cx = 660.0;
    let cy = 385.0;
    for r in [110.0, 180.0, 224.0] {
        c.ring(cx, cy, r, rgb(51, 42, 63));
    }
    for i in 0..72 {
        let a = i as f64 * std::f64::consts::TAU / 72.0;
        c.line(
            (cx + a.cos() * 224.0, cy + a.sin() * 224.0),
            (
                cx + a.cos() * if i % 6 == 0 { 239.0 } else { 230.0 },
                cy + a.sin() * if i % 6 == 0 { 239.0 } else { 230.0 },
            ),
            accent,
            1,
        );
    }
    let outline = [
        (-135.0, -25.0),
        (-178.0, -160.0),
        (-75.0, -85.0),
        (0.0, -112.0),
        (75.0, -85.0),
        (178.0, -160.0),
        (135.0, -25.0),
        (104.0, 75.0),
        (0.0, 150.0),
        (-104.0, 75.0),
        (-135.0, -25.0),
    ];
    for width in [9, 4, 1] {
        let color = match width {
            9 => rgb(48, 23, 42),
            4 => rgb(115, 39, 64),
            _ => accent,
        };
        for pair in outline.windows(2) {
            c.line(
                (cx + pair[0].0, cy + pair[0].1),
                (cx + pair[1].0, cy + pair[1].1),
                color,
                width,
            );
        }
    }
    for side in [-1.0, 1.0] {
        c.line(
            (cx + side * 20.0, cy - 8.0),
            (cx + side * 90.0, cy - 35.0),
            secondary,
            5,
        );
        c.line(
            (cx + side * 20.0, cy - 8.0),
            (cx + side * 66.0, cy + 4.0),
            secondary,
            2,
        );
        c.line(
            (cx + side * 5.0, cy + 65.0),
            (cx + side * 46.0, cy + 52.0),
            accent,
            2,
        );
        c.line(
            (cx + side * 46.0, cy + 52.0),
            (cx + side * 20.0, cy + 99.0),
            accent,
            2,
        );
    }
    c.text(
        340.0,
        136.0,
        590.0,
        26.0,
        14,
        accent,
        "CONSTRUCT 01 / THE NIGHTWRAITH / ILLUSTRATION",
    );
    c.text(462.0, 633.0, 460.0, 35.0, 22, WHITE, "ROOT OF ALL LORE");
    c.text(
        432.0,
        671.0,
        530.0,
        26.0,
        13,
        MUTED,
        "SOUL REACTOR / DECORATIVE SIGNAL / SIM",
    );
    let mut last = (332.0, 738.0);
    for i in 1..190 {
        let x = 332.0 + i as f64 * 3.2;
        let y = 738.0 + ((i as f64 * 0.15 + t * 2.0).sin() * 12.0) * (i as f64 * 0.035 + t).sin();
        c.line(last, (x, y), secondary, 1);
        last = (x, y);
    }
    // Synthetic radar: explicitly labelled, never derived from network activity.
    c.rect(1002.0, 136.0, 570.0, 245.0, rgb(9, 18, 28));
    c.text(
        1020.0,
        149.0,
        530.0,
        25.0,
        14,
        secondary,
        "ABYSS RELAY / SIMULATED SIGNAL MAP",
    );
    for radius in [35.0, 65.0, 95.0] {
        c.ring(1140.0, 273.0, radius, rgb(24, 74, 80));
    }
    let a = t * 0.8;
    c.line(
        (1140.0, 273.0),
        (1140.0 + a.cos() * 95.0, 273.0 + a.sin() * 95.0),
        secondary,
        2,
    );
    for i in 0..7 {
        let angle = i as f64 * 2.1;
        let r = 24.0 + i as f64 * 10.0;
        c.rect(
            1138.0 + angle.cos() * r,
            271.0 + angle.sin() * r,
            4.0,
            4.0,
            accent,
        );
    }
    c.text(1260.0, 204.0, 294.0, 30.0, 23, WHITE, "UPLINK / FICTION");
    c.text(1260.0, 249.0, 294.0, 26.0, 14, MUTED, "NO NETWORK SCAN");
    c.text(
        1260.0,
        284.0,
        294.0,
        26.0,
        14,
        secondary,
        "IN-PROCESS RELAY AVAILABLE",
    );
    c.text(
        1260.0,
        321.0,
        294.0,
        26.0,
        14,
        accent,
        "REAL EXECUTION BELOW",
    );
    c.rect(1002.0, 399.0, 570.0, 351.0, rgb(5, 11, 18));
    c.text(
        1020.0,
        410.0,
        510.0,
        28.0,
        14,
        secondary,
        if deck.busy {
            "COMMAND TRANSCRIPT / LOCAL PROCESS RUNNING"
        } else {
            "COMMAND TRANSCRIPT / LOCAL + SIM LABELS"
        },
    );
    let end = deck.lines.len().saturating_sub(deck.scroll);
    let start = end.saturating_sub(14);
    for (row, line) in deck.lines.iter().skip(start).take(end - start).enumerate() {
        c.text(
            1020.0,
            444.0 + row as f64 * 20.0,
            530.0,
            22.0,
            13,
            if line.starts_with('>') {
                secondary
            } else {
                WHITE
            },
            line,
        );
    }
    c.text(1005.0, 764.0, 570.0, 26.0, 13, secondary, &deck.music);
    c.rect(28.0, 812.0, 1544.0, 62.0, rgb(14, 26, 39));
    c.text(40.0, 831.0, 120.0, 24.0, 16, secondary, "LOCAL >");
    c.rect(1300.0, 824.0, 92.0, 38.0, rgb(22, 69, 75));
    c.text(1315.0, 835.0, 76.0, 22.0, 14, WHITE, "EXECUTE");
    c.rect(1402.0, 824.0, 72.0, 38.0, rgb(88, 28, 43));
    c.text(1416.0, 835.0, 60.0, 22.0, 14, WHITE, "STOP");
    c.text(
        1486.0,
        835.0,
        80.0,
        22.0,
        13,
        MUTED,
        if deck.paused { "RESUME" } else { "PAUSE" },
    );
    c.text(30.0,882.0,1500.0,18.0,11,MUTED,"ENTER execute   /   Mouse wheel transcript   /   F11 fullscreen   /   ESC stop   /   SIM = fictional effects   /   Music = local playback metadata");
}

unsafe fn execute_edit(deck: &mut Deck) {
    unsafe {
        let mut input = [0u16; 4097];
        let length = GetWindowTextW(deck.edit, input.as_mut_ptr(), input.len() as i32);
        deck.submit(String::from_utf16_lossy(&input[..length.max(0) as usize]));
        SetWindowTextW(deck.edit, wide("").as_ptr());
        SetFocus(deck.edit);
    }
}

unsafe extern "system" fn procedure(window: Handle, message: u32, w: usize, l: isize) -> isize {
    unsafe {
        if message == 0x81 {
            let create = &*(l as *const Create);
            SetWindowLongPtrW(window, -21, create.data as isize);
        }
        let pointer = GetWindowLongPtrW(window, -21) as *mut Deck;
        if pointer.is_null() {
            return DefWindowProcW(window, message, w, l);
        }
        let deck = &mut *pointer;
        match message {
            0x1 => {
                deck.edit = CreateWindowExW(
                    0x200,
                    wide("EDIT").as_ptr(),
                    wide("").as_ptr(),
                    0x50010080,
                    0,
                    0,
                    100,
                    30,
                    window,
                    1usize as Handle,
                    GetModuleHandleW(null_mut()),
                    null_mut(),
                );
                SendMessageW(deck.edit, 0xC5, 4096, 0); // EM_LIMITTEXT
                SetTimer(window, 1, 33, null_mut());
                0
            }
            0x5 => {
                let width = (l as u32 & 65535) as f64;
                let height = ((l as u32 >> 16) & 65535) as f64;
                MoveWindow(
                    deck.edit,
                    (155.0 * width / 1600.0) as i32,
                    (826.0 * height / 900.0) as i32,
                    (1125.0 * width / 1600.0) as i32,
                    (34.0 * height / 900.0) as i32,
                    1,
                );
                let font = CreateFontW(
                    -(17.0 * height / 900.0).max(12.0) as i32,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    5,
                    0,
                    wide("Consolas").as_ptr(),
                );
                SendMessageW(deck.edit, 0x30, font as usize, 1);
                if !deck.edit_font.is_null() {
                    DeleteObject(deck.edit_font);
                }
                deck.edit_font = font;
                0
            }
            0x133 => {
                SetTextColor(w as Handle, CYAN);
                SetBkColor(w as Handle, rgb(14, 26, 39));
                deck.edit_brush as isize
            }
            0x2E0 => {
                let rect = &*(l as *const Rect);
                SetWindowPos(
                    window,
                    null_mut(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    0x14,
                );
                0
            }
            0x14 => 1, // WM_ERASEBKGND: double-buffered paint owns the background
            0x113 => {
                deck.collect();
                if deck.closed || (deck.smoke && deck.started.elapsed() > Duration::from_secs(5)) {
                    DestroyWindow(window);
                } else {
                    InvalidateRect(window, null_mut(), 0);
                }
                0
            }
            0x20A => {
                let delta = (w >> 16) as i16;
                if delta > 0 {
                    deck.scroll = (deck.scroll + 3).min(deck.lines.len().saturating_sub(1));
                } else {
                    deck.scroll = deck.scroll.saturating_sub(3);
                }
                0
            }
            0x202 => {
                let mut rect = Rect::default();
                GetClientRect(window, &mut rect);
                let x = (l as u32 & 65535) as f64 * 1600.0 / f64::from(rect.right.max(1));
                let y = ((l as u32 >> 16) & 65535) as f64 * 900.0 / f64::from(rect.bottom.max(1));
                if (28.0..280.0).contains(&x) && (190.0..686.0).contains(&y) {
                    let index = ((y - 190.0) / 62.0) as usize;
                    if (y - 190.0) % 62.0 < 46.0 {
                        deck.submit(BUTTONS[index].1.into());
                    }
                } else if (824.0..862.0).contains(&y) {
                    if (1300.0..1392.0).contains(&x) {
                        execute_edit(deck);
                    } else if (1402.0..1474.0).contains(&x) {
                        let _ = deck.requests.send(Request::Cancel);
                    } else if (1486.0..1572.0).contains(&x) {
                        deck.paused = !deck.paused;
                    }
                }
                0
            }
            0xF => {
                let mut paint: Paint = std::mem::zeroed();
                let target = BeginPaint(window, &mut paint);
                let mut rect = Rect::default();
                GetClientRect(window, &mut rect);
                if rect.right > 0 && rect.bottom > 0 {
                    let dc = CreateCompatibleDC(target);
                    let bitmap = CreateCompatibleBitmap(target, rect.right, rect.bottom);
                    let old = SelectObject(dc, bitmap);
                    render(
                        &Canvas {
                            dc,
                            sx: f64::from(rect.right) / 1600.0,
                            sy: f64::from(rect.bottom) / 900.0,
                        },
                        deck,
                    );
                    BitBlt(target, 0, 0, rect.right, rect.bottom, dc, 0, 0, 0xCC0020);
                    SelectObject(dc, old);
                    DeleteObject(bitmap);
                    DeleteDC(dc);
                }
                EndPaint(window, &paint);
                0
            }
            0x2 => {
                KillTimer(window, 1);
                let _ = deck.requests.send(Request::Stop);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(window, message, w, l),
        }
    }
}

fn preview(window: Handle, deck: &Deck) -> io::Result<()> {
    unsafe {
        let target = GetDC(window);
        let dc = CreateCompatibleDC(target);
        let bitmap = CreateCompatibleBitmap(target, 3840, 2160);
        let old = SelectObject(dc, bitmap);
        render(
            &Canvas {
                dc,
                sx: 2.4,
                sy: 2.4,
            },
            deck,
        );
        SelectObject(dc, old);
        let mut info = BitmapInfo {
            size: 40,
            width: 3840,
            height: -2160,
            planes: 1,
            bits: 32,
            compression: 0,
            image: 0,
            xp: 0,
            yp: 0,
            used: 0,
            important: 0,
            colors: [0; 3],
        };
        let mut pixels = vec![0u8; 3840 * 2160 * 4];
        let copied = GetDIBits(
            dc,
            bitmap,
            0,
            2160,
            pixels.as_mut_ptr().cast(),
            &mut info,
            0,
        );
        DeleteObject(bitmap);
        DeleteDC(dc);
        ReleaseDC(window, target);
        if copied != 2160 {
            return Err(io::Error::last_os_error());
        }
        let mut bytes = Vec::new();
        bytes.extend(b"BM");
        bytes.extend(((54 + pixels.len()) as u32).to_le_bytes());
        bytes.extend([0u8; 4]);
        bytes.extend(54u32.to_le_bytes());
        bytes.extend(40u32.to_le_bytes());
        bytes.extend(3840i32.to_le_bytes());
        bytes.extend((-2160i32).to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(32u16.to_le_bytes());
        bytes.extend([0u8; 24]);
        bytes.extend(pixels);
        std::fs::create_dir_all("target")?;
        std::fs::write("target/desktop-4k.bmp", bytes)
    }
}

pub fn run(shell: Shell, smoke: bool) -> io::Result<u8> {
    unsafe {
        SetProcessDpiAwarenessContext(-4); // Per-monitor aware V2, Windows 10+
        let (requests, input) = mpsc::channel();
        let (output, updates) = mpsc::channel();
        let worker = thread::spawn(move || worker(shell, input, output));
        let mut deck = Box::new(Deck {
            requests,
            updates,
            lines: VecDeque::from([
                "[SIM] Welcome to Operation Porcelain Nightfall.".into(),
                "[LOCAL] Commands execute on this Windows machine.".into(),
                "Try help, mission next, discord status, or echo hello.".into(),
            ]),
            edit: null_mut(),
            started: Instant::now(),
            music: "LOCAL MUSIC / connecting to Windows playback".into(),
            busy: false,
            paused: false,
            scroll: 0,
            smoke,
            smoke_ok: false,
            closed: false,
            theme: "crimson".into(),
            edit_brush: CreateSolidBrush(rgb(14, 26, 39)),
            edit_font: null_mut(),
        });
        let instance = GetModuleHandleW(null_mut());
        let name = wide("OpsecDemonNativeDeck");
        let class = Class {
            style: 3,
            procedure,
            extra: 0,
            window_extra: 0,
            instance,
            icon: null_mut(),
            cursor: LoadCursorW(null_mut(), 32512usize as *const u16),
            background: null_mut(),
            menu: null_mut(),
            name: name.as_ptr(),
        };
        if RegisterClassW(&class) == 0 {
            let _ = deck.requests.send(Request::Stop);
            let _ = worker.join();
            DeleteObject(deck.edit_brush);
            if !deck.edit_font.is_null() {
                DeleteObject(deck.edit_font);
            }
            return Err(io::Error::last_os_error());
        }
        let window = CreateWindowExW(
            0,
            name.as_ptr(),
            wide("OPSEC DEMON / NIGHTFALL COMMAND DECK").as_ptr(),
            0x00CF0000,
            0x80000000u32 as i32,
            0x80000000u32 as i32,
            1600,
            960,
            null_mut(),
            null_mut(),
            instance,
            (&mut *deck as *mut Deck).cast(),
        );
        if window.is_null() {
            let _ = deck.requests.send(Request::Stop);
            let _ = worker.join();
            return Err(io::Error::last_os_error());
        }
        if smoke {
            let mut client = Rect::default();
            GetClientRect(window, &mut client);
            let x = 50 * client.right / 1600;
            let y = 270 * client.bottom / 900;
            SendMessageW(window, 0x202, 0, ((y as u32) << 16 | x as u32) as isize);
            SetWindowTextW(deck.edit, wide("echo WINDOWS_GUI_SMOKE").as_ptr());
            execute_edit(&mut deck);
        } else {
            FreeConsole();
            ShowWindow(window, 3);
            SetFocus(deck.edit);
        }
        let mut message = Message::default();
        let mut fullscreen = false;
        let mut previous = Rect::default();
        loop {
            let result = GetMessageW(&mut message, null_mut(), 0, 0);
            if result <= 0 {
                break;
            }
            if message.message == 0x100 && message.w == 13 && message.window == deck.edit {
                execute_edit(&mut deck);
                continue;
            }
            if message.message == 0x100 && message.w == 27 {
                let _ = deck.requests.send(Request::Cancel);
                continue;
            }
            if message.message == 0x100 && message.w == 122 {
                if !fullscreen {
                    GetWindowRect(window, &mut previous);
                    let mut monitor: Monitor = std::mem::zeroed();
                    monitor.size = std::mem::size_of::<Monitor>() as u32;
                    GetMonitorInfoW(MonitorFromWindow(window, 2), &mut monitor);
                    SetWindowLongPtrW(window, -16, 0x90000000u32 as isize);
                    SetWindowPos(
                        window,
                        null_mut(),
                        monitor.monitor.left,
                        monitor.monitor.top,
                        monitor.monitor.right - monitor.monitor.left,
                        monitor.monitor.bottom - monitor.monitor.top,
                        0x24,
                    );
                } else {
                    SetWindowLongPtrW(window, -16, 0x10CF0000);
                    SetWindowPos(
                        window,
                        null_mut(),
                        previous.left,
                        previous.top,
                        previous.right - previous.left,
                        previous.bottom - previous.top,
                        0x24,
                    );
                }
                fullscreen = !fullscreen;
                continue;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
            if deck.smoke && deck.smoke_ok && !deck.music.contains("probing playback") {
                preview(window, &deck)?;
                DestroyWindow(window);
            }
        }
        let _ = deck.requests.send(Request::Stop);
        let _ = worker.join();
        if smoke && !deck.smoke_ok {
            return Err(io::Error::other(
                "GUI smoke did not receive native command output",
            ));
        }
        Ok(0)
    }
}

#[repr(C)]
struct Monitor {
    size: u32,
    monitor: Rect,
    work: Rect,
    flags: u32,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn GetWindowRect(window: Handle, rect: *mut Rect) -> i32;
    fn MonitorFromWindow(window: Handle, flags: u32) -> Handle;
    fn GetMonitorInfoW(monitor: Handle, info: *mut Monitor) -> i32;
    fn SetWindowPos(
        window: Handle,
        after: Handle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        flags: u32,
    ) -> i32;
}
