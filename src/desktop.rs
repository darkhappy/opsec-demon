//! Native Windows concert deck. All pages render through Vulkan; the shell remains real.
use crate::{
    Shell, desktop_gpu,
    desktop_session::{Request, Session, Snapshot, Update},
    dsp::{Model, Response},
    soundtrack,
};
use eframe::{
    egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2},
    egui_wgpu::wgpu,
};
use std::{
    collections::VecDeque,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

const CYAN: Color32 = Color32::from_rgb(80, 226, 235);
const PINK: Color32 = Color32::from_rgb(245, 81, 158);
const GOLD: Color32 = Color32::from_rgb(255, 197, 102);
const MUTED: Color32 = Color32::from_rgb(139, 156, 182);
pub const PAGES: [&str; 8] = [
    "Command Deck",
    "Concert Stage",
    "Audio Lab",
    "Rhythm Arcade",
    "Story Archive",
    "Simulation Props",
    "Music + Discord",
    "Graphics + Audio",
];
const SUBTITLES: [&str; 8] = [
    "Your real shell, inside the fiction.",
    "Four silhouettes. One spectacularly unstable drummer.",
    "Follow the signal, from source to imaginary room.",
    "Six keys. Two lasers. Maximum theatrical pressure.",
    "Lost Signals / a local story you control.",
    "All the chaos, with the original simulation state.",
    "Local playback metadata and your Discord presence.",
    "Tune every pixel. Mix every sound. Save your setup.",
];

pub struct Deck {
    session: Session,
    state: Snapshot,
    page: usize,
    logs: VecDeque<String>,
    input: String,
    history_index: usize,
    music: String,
    player: soundtrack::LoopPlayer,
    started: Instant,
    last_frame: Instant,
    cadence: VecDeque<f32>,
    ui_ms: f32,
    adapter: String,
    burst_at: Option<Instant>,
    model: Model,
    fullscreen: bool,
    uncapped: bool,
    profile: String,
    room: [f64; 4],
    pcm: Vec<[f64; 2]>,
    spectrum: Vec<[f64; 2]>,
    analysis_at: Instant,
    command_focus: bool,
    closed: Option<u8>,
    exit_code: Arc<AtomicU8>,
    capture: bool,
    audio_source: bool,
    window_smoke_frames: Option<u32>,
    window_smoke_passed: Arc<AtomicBool>,
    settings: crate::desktop_settings::Settings,
    camera: Vec2,
    settings_status: String,
}

fn style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::TRANSPARENT;
    style.visuals.window_fill = Color32::from_rgb(13, 20, 37);
    style.visuals.override_text_color = Some(Color32::from_rgb(225, 232, 245));
    style.visuals.selection.bg_fill = Color32::from_rgb(32, 82, 98);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(23, 35, 53);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(37, 61, 78);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(45, 90, 109);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(23, 35, 53);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(37, 61, 78);
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(45, 90, 109);
    style.spacing.item_spacing = Vec2::new(12., 10.);
    style.spacing.button_padding = Vec2::new(15., 10.);
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(15.));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.));
    ctx.set_style(style);
}
fn card(ui: &mut egui::Ui, title: &str, render: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(12, 21, 37, 235))
        .stroke(Stroke::new(1., Color32::from_rgb(42, 65, 87)))
        .corner_radius(12.)
        .inner_margin(18.)
        .show(ui, |ui| {
            ui.label(RichText::new(title).color(CYAN).size(13.).strong());
            ui.add_space(8.);
            render(ui);
        });
}
impl Deck {
    pub fn new(shell: Shell, adapter: String, autoplay: bool) -> Self {
        let now = Instant::now();
        let mut player = soundtrack::LoopPlayer::default();
        let settings = if autoplay {
            crate::desktop_settings::Settings::load()
        } else {
            Default::default()
        };
        player.gains(settings.gains.clone());
        let mut logs = VecDeque::from(["[LOCAL] Vulkan deck ready. Commands execute on this Windows machine. SIM controls are fictional.".into()]);
        if autoplay && let Err(error) = player.set(true) {
            logs.push_back(format!("[AUDIO] {error}"));
        }
        Self {
            session: Session::start(shell),
            state: Snapshot::default(),
            page: 0,
            logs,
            input: String::new(),
            history_index: 0,
            music: "[LOCAL] Probing Windows media sessions…".into(),
            player,
            started: now,
            last_frame: now,
            cadence: VecDeque::new(),
            ui_ms: 0.,
            adapter,
            burst_at: None,
            model: Model::default(),
            fullscreen: false,
            uncapped: std::env::var("OPSEC_DEMON_UNCAPPED").is_ok_and(|value| value == "1")
                || !settings.vsync,
            profile: String::new(),
            room: [4., 5., 2.7, 0.22],
            pcm: Vec::new(),
            spectrum: Vec::new(),
            analysis_at: now - Duration::from_secs(1),
            command_focus: false,
            closed: None,
            exit_code: Arc::new(AtomicU8::new(0)),
            capture: !autoplay,
            audio_source: false,
            window_smoke_frames: None,
            window_smoke_passed: Arc::new(AtomicBool::new(false)),
            settings,
            camera: Vec2::ZERO,
            settings_status: String::new(),
        }
    }
    fn log(&mut self, line: String) {
        for line in line.lines() {
            self.logs.push_back(
                line.trim_end_matches('\r')
                    .chars()
                    .filter(|c| !c.is_control() || *c == '\t')
                    .take(4000)
                    .collect(),
            );
        }
        while self.logs.len() > 1500 {
            self.logs.pop_front();
        }
    }
    pub fn send(&mut self, command: &str) {
        let sound = if command.contains("pedal stomp") {
            3
        } else if command.starts_with("sdvx hit bt") {
            4
        } else if command.starts_with("sdvx hit fx") {
            5
        } else if command.starts_with("sdvx knob") {
            6
        } else if matches!(command.trim(), "burst" | "cinematic" | "intro") {
            10
        } else {
            8
        };
        self.player.cue(sound);
        match command.trim() {
            "graph next" | "graph room" | "graph headphones" => {
                self.page = 2;
                return;
            }
            "graph metrics" | "metrics" => {
                self.page = 0;
                return;
            }
            "clear" => {
                self.logs.clear();
                return;
            }
            _ => {}
        }
        if command.trim_start().starts_with("soundtrack play") {
            let _ = self.player.set(false);
        }
        if matches!(command.trim(), "intro" | "cinematic" | "burst") {
            self.burst_at = Some(Instant::now());
        }
        if self
            .session
            .requests
            .try_send(Request::Execute(command.into()))
            .is_err()
        {
            self.log("[LOCAL] Command queue full; retry shortly.".into());
        }
    }
    pub fn drain(&mut self) {
        while let Ok(update) = self.session.updates.try_recv() {
            match update {
                Update::Line(line) => {
                    if line.starts_with("[LOCAL / ERROR]") {
                        self.player.cue(9);
                    }
                    self.log(line);
                }
                Update::Busy(busy) => self.state.busy = busy,
                Update::Music(line) => self.music = line,
                Update::Snapshot(state) => {
                    if state.celebration != self.state.celebration {
                        self.burst_at = Some(Instant::now());
                        self.player.cue(10);
                    }
                    if state.drummer_blast && !self.state.drummer_blast {
                        self.player.cue(1);
                    }
                    if state.ssd_blast && !self.state.ssd_blast {
                        self.player.cue(2);
                    }
                    if state.history.len() != self.state.history.len() {
                        self.history_index = state.history.len();
                    }
                    self.state = *state;
                }
                Update::Completion(result) => {
                    self.input = result.input;
                    if result.choices.len() > 1 {
                        self.log(format!(
                            "[COMPLETE] {} / cursor {}",
                            result.choices.join("  "),
                            result.cursor
                        ));
                    }
                }
                Update::Closed(code) => {
                    self.closed = Some(code);
                    self.exit_code.store(code, Ordering::Relaxed);
                    self.log("[LOCAL] Session closed.".into());
                }
            }
        }
        let code = self.session.exit_code.load(Ordering::Acquire);
        if code <= 255 {
            self.closed = Some(code as u8);
            self.exit_code.store(code as u8, Ordering::Relaxed);
        }
        if self.analysis_at.elapsed() >= Duration::from_millis(100) {
            let samples = soundtrack::pcm_window(self.started.elapsed().as_secs_f64());
            self.pcm = samples
                .iter()
                .enumerate()
                .map(|(i, x)| [i as f64 / 44.1, *x])
                .collect();
            self.spectrum = (1..129)
                .map(|k| {
                    let (mut re, mut im) = (0., 0.);
                    for (i, sample) in samples.iter().enumerate() {
                        let window = 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / 511.).cos();
                        let angle = std::f64::consts::TAU * k as f64 * i as f64 / 512.;
                        re += sample * window * angle.cos();
                        im -= sample * window * angle.sin();
                    }
                    [
                        k as f64 * 44_100. / 512.,
                        (20. * ((re * re + im * im).sqrt() / 256.).max(1e-6).log10()).max(-120.),
                    ]
                })
                .collect();
            self.analysis_at = Instant::now();
        }
    }
    pub fn smoke_ready(&self) -> bool {
        self.logs.iter().any(|line| line == "VULKAN_NATIVE_OK")
            && self.state.drummer_blast
            && self.state.ssd_blast
            && self.state.pedal
            && self.state.combo == 1
            && self.state.headphone_curve.len() == 256
            && self.state.room_curve.len() == 256
            && self.state.novel.contains("LOST SIGNALS")
    }
    pub fn native_smoke_ready(&self) -> bool {
        self.logs.iter().any(|line| line == "VULKAN_NATIVE_OK")
            && self
                .logs
                .iter()
                .any(|line| line == "[LOCAL] process exited 0")
            && !self.state.busy
    }
    fn actions(&mut self, ui: &mut egui::Ui, buttons: &[(&str, &str)]) {
        ui.horizontal_wrapped(|ui| {
            for (label, command) in buttons {
                if ui.button(*label).clicked() {
                    self.send(command);
                }
            }
        });
    }
    pub fn smoke_diagnostics(&self) -> String {
        format!(
            "busy={} marker={} lines={:?}",
            self.state.busy,
            self.logs.iter().any(|line| line == "VULKAN_NATIVE_OK"),
            self.logs
                .iter()
                .filter(|line| line.contains("VULKAN_NATIVE_OK") || line.starts_with("[LOCAL"))
                .collect::<Vec<_>>()
        )
    }
    pub fn ui(&mut self, ctx: &egui::Context) {
        self.drain();
        if self.closed.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let now = Instant::now();
        self.cadence
            .push_back(now.duration_since(self.last_frame).as_secs_f32());
        self.last_frame = now;
        while self.cadence.len() > 120 {
            self.cadence.pop_front();
        }
        let screen = ctx.content_rect();
        if !self.capture
            && !self.settings.reduced_motion
            && ctx.input(|i| i.pointer.secondary_down())
        {
            let delta = ctx.input(|i| i.pointer.delta());
            self.camera += delta * (self.settings.look[1] * 0.001);
            self.camera.x = self.camera.x.clamp(-0.5, 0.5);
            self.camera.y = self.camera.y.clamp(-0.5, 0.5);
        }
        let burst = self
            .burst_at
            .map_or(0., |at| (1. - at.elapsed().as_secs_f32() / 2.5).max(0.));
        ctx.layer_painter(egui::LayerId::background()).add(
            eframe::egui_wgpu::Callback::new_paint_callback(
                screen,
                desktop_gpu::Backdrop {
                    time: self.started.elapsed().as_secs_f32(),
                    aspect: screen.width() / screen.height().max(1.),
                    burst,
                    page: self.page as f32,
                    settings: self.settings.clone(),
                    camera: [self.camera.x, self.camera.y],
                },
            ),
        );
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            let _ = self.session.requests.try_send(Request::Cancel);
        }
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G)) {
                self.send("pedal stomp");
            }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::X)) {
                self.send("band drummer explode");
            }
            if self.page == 3 {
                for (key, cmd) in [
                    (egui::Key::D, "sdvx hit bt-a"),
                    (egui::Key::F, "sdvx hit bt-b"),
                    (egui::Key::J, "sdvx hit bt-c"),
                    (egui::Key::K, "sdvx hit bt-d"),
                    (egui::Key::A, "sdvx hit fx-l"),
                    (egui::Key::L, "sdvx hit fx-r"),
                ] {
                    if ctx.input(|i| i.key_pressed(key)) {
                        self.send(cmd);
                    }
                }
            }
        }
        egui::SidePanel::left("navigation")
            .exact_width(218.)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(9, 15, 28, 248))
                    .inner_margin(20.),
            )
            .show(ctx, |ui| {
                ui.add_space(14.);
                ui.label(RichText::new("OPSEC").size(32.).strong().color(CYAN));
                ui.label(RichText::new("D E M O N").size(20.).color(PINK));
                ui.add_space(6.);
                ui.label(
                    RichText::new("VULKAN / NATIVE WINDOWS")
                        .size(10.)
                        .color(MUTED),
                );
                ui.add_space(34.);
                for (i, name) in PAGES.iter().enumerate() {
                    let text = RichText::new(format!("{:02}   {name}", i + 1))
                        .color(if self.page == i { CYAN } else { MUTED });
                    if ui
                        .add_sized(
                            [178., 42.],
                            egui::Button::new(text).selected(self.page == i),
                        )
                        .clicked()
                    {
                        self.page = i;
                        self.player.cue(7);
                    }
                }
                ui.add_space(28.);
                ui.separator();
                ui.add_space(12.);
                ui.label(RichText::new("NIGHTFALL").strong().color(GOLD));
                ui.label(
                    RichText::new("Original score · 148 BPM")
                        .size(12.)
                        .color(MUTED),
                );
                let mut playing = self.player.playing();
                if ui.checkbox(&mut playing, "Loop soundtrack").changed()
                    && let Err(error) = self.player.set(playing)
                {
                    self.log(format!("[AUDIO] {error}"));
                }
                ui.add_space(16.);
                let avg = self.cadence.iter().sum::<f32>() / self.cadence.len().max(1) as f32;
                if self.capture {
                    ui.label(RichText::new("4K VULKAN CAPTURE").color(CYAN));
                } else if self.settings.fps {
                    ui.label(
                        RichText::new(format!(
                            "{:.0} fps / {:.1} ms",
                            1. / avg.max(0.0001),
                            avg * 1000.
                        ))
                        .color(CYAN),
                    );
                }
                ui.label(
                    RichText::new(format!("UI CPU {:.2} ms", self.ui_ms))
                        .size(12.)
                        .color(MUTED),
                );
                ui.label(RichText::new(&self.adapter).size(11.).color(MUTED));
                ui.add_space(10.);
                ui.label(
                    RichText::new(if self.uncapped {
                        "UNCAPPED PRESENTATION"
                    } else {
                        "VSYNC PRESENTATION"
                    })
                    .size(11.)
                    .color(CYAN),
                );
                ui.label(
                    RichText::new("F11 fullscreen. Esc cancels a job.")
                        .size(11.)
                        .color(MUTED),
                );
            });
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(9, 17, 29))
                    .inner_margin(8.),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.state.busy {
                            "● RUNNING"
                        } else {
                            "● READY"
                        })
                        .color(CYAN),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}  /  exit {}  /  {} operations",
                            self.state.callsign, self.state.status, self.state.operations
                        ))
                        .size(12.),
                    );
                    ui.separator();
                    ui.label(RichText::new(&self.state.cwd).size(11.).color(MUTED));
                    if self.state.dropped_output > 0 {
                        ui.label(format!(
                            "{} dropped output lines",
                            self.state.dropped_output
                        ));
                    }
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(28.))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("THE OPERATOR'S CONCERT")
                            .size(11.)
                            .color(CYAN),
                    );
                    ui.label(
                        RichText::new(format!("/ {} / {}", self.state.theme, self.state.scene))
                            .size(11.)
                            .color(MUTED),
                    );
                });
                ui.add_space(8.);
                ui.label(RichText::new(PAGES[self.page]).size(38.).strong());
                ui.label(RichText::new(SUBTITLES[self.page]).color(MUTED));
                ui.add_space(20.);
                egui::ScrollArea::vertical()
                    .id_salt(("page", self.page))
                    .show(ui, |ui| match self.page {
                        0 => self.command(ui),
                        1 => self.concert(ui),
                        2 => self.audio(ui),
                        3 => self.arcade(ui),
                        4 => self.story(ui),
                        5 => self.props(ui),
                        6 => self.integrations(ui),
                        _ => self.settings_page(ui, ctx),
                    });
            });
        self.ui_ms = now.elapsed().as_secs_f32() * 1000.;
        if self.settings.fps && !self.capture {
            let avg = self.cadence.iter().sum::<f32>() / self.cadence.len().max(1) as f32;
            egui::Area::new("fps overlay".into())
                .anchor(egui::Align2::RIGHT_TOP, [-16., 12.])
                .interactable(false)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(6, 12, 24, 230))
                        .inner_margin(8.)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{:.0} FPS  /  {:.2} ms",
                                    1. / avg.max(0.0001),
                                    avg * 1000.
                                ))
                                .monospace()
                                .color(CYAN),
                            );
                        });
                });
        }
    }
    fn settings_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        use crate::{
            desktop_audio::SOUNDS,
            desktop_settings::{FSR, LOOK},
        };
        ui.horizontal_wrapped(|ui| {
            for (i, name) in ["Low", "Balanced", "High", "Ultra"].iter().enumerate() {
                if ui.button(*name).clicked() {
                    self.settings.preset(i);
                }
            }
            if ui.button("Reset defaults").clicked() {
                self.settings = Default::default();
                self.camera = Vec2::ZERO;
            }
            if ui.button("Save settings").clicked() {
                self.settings_status = match self.settings.save() {
                    Ok(()) => "Saved to LOCALAPPDATA/opsec-demon/graphics-audio.json".into(),
                    Err(e) => format!("Save failed: {e}"),
                };
            }
        });
        if !self.settings_status.is_empty() {
            ui.label(&self.settings_status);
        }
        ui.add_space(12.);
        ui.columns(2, |columns| {
            card(&mut columns[0], "DISPLAY / VULKAN / UPSCALING", |ui| {
                ui.checkbox(&mut self.settings.fps, "Show FPS / frame time counter");
                ui.checkbox(&mut self.settings.vsync, "VSync (save and restart to apply)");
                ui.label(RichText::new(if self.uncapped { "Active: uncapped presentation" } else { "Active: VSync presentation" }).color(MUTED));
                ui.add(egui::Slider::new(&mut self.settings.cap, 0..=1000).text("Frame limit (0 = unlimited)"));
                egui::ComboBox::from_label("Upscaler").selected_text(FSR[self.settings.fsr]).show_ui(ui, |ui| {
                    for (i,label) in FSR.iter().enumerate() { ui.selectable_value(&mut self.settings.fsr,i,*label); }
                });
                ui.add_enabled(self.settings.fsr == 0, egui::Slider::new(&mut self.settings.scale,0.5..=1.).text("Manual render scale"));
                ui.add_enabled(self.settings.fsr != 0, egui::Slider::new(&mut self.settings.sharpness,0.0..=1.).text("FSR RCAS sharpness"));
                ui.label(format!("Backdrop: {:.0}% resolution / UI + graphs: native", self.settings.render_scale()*100.));
                ui.label(RichText::new("FSR 1: EASU + RCAS spatial passes. Text stays crisp. Fullscreen uses your display's resolution.").color(MUTED));
                if ui.checkbox(&mut self.fullscreen, "Fullscreen / F11").changed() { ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen)); }
                ui.horizontal_wrapped(|ui| {
                    for (label, size) in [("1280×800", [1280.,800.]),("1600×1000",[1600.,1000.]),("1920×1080",[1920.,1080.]),("3840×2160",[3840.,2160.])] {
                        if ui.button(label).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::from(size))); }
                    }
                });
                if ui.add(egui::Slider::new(&mut self.settings.zoom,0.75..=1.5).text("Interface scale")).changed() { ctx.set_zoom_factor(self.settings.zoom); }
            });
            columns[0].add_space(12.);
            card(&mut columns[0], "CAMERA / LIGHT / THEATRICAL DETAIL", |ui| {
                ui.label(RichText::new("Hold right mouse and drag to move the backdrop camera.").color(MUTED));
                ui.checkbox(&mut self.settings.reduced_motion,"Reduced motion (freeze backdrop, disable camera shake)");
                for (i,(label,min,max,_)) in LOOK.iter().enumerate() { ui.add(egui::Slider::new(&mut self.settings.look[i],*min..=*max).text(*label)); }
                if ui.button("Center camera").clicked() { self.camera = Vec2::ZERO; }
                ui.label(RichText::new("FOV and mouse sensitivity control the procedural concert camera. SIM remains fictional. This renderer has no ray tracing, temporal frame generation, or HDR output.").color(MUTED));
            });
            card(&mut columns[1], "AUDIO / OUTPUT BUSES", |ui| {
                let mut playing = self.player.playing();
                if ui.checkbox(&mut playing,"Loop original soundtrack").changed() && let Err(e) = self.player.set(playing) { self.settings_status = e.to_string(); }
                for (label, gain) in [("Master", &mut self.settings.gains.master),("Music",&mut self.settings.gains.music),("Effects",&mut self.settings.gains.effects),("Interface",&mut self.settings.gains.interface)] {
                    ui.add(egui::Slider::new(gain,0.0..=1.).text(label));
                }
                ui.label(RichText::new("Default Windows stereo output · 44.1 kHz PCM · live gain changes").color(MUTED));
            });
            columns[1].add_space(12.);
            card(&mut columns[1], "EVERY SOUND / INDIVIDUAL MIX", |ui| {
                for (i,name) in SOUNDS.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut self.settings.gains.sounds[i],0.0..=1.).text(*name));
                        if i > 0 && ui.small_button("Test").clicked() { self.player.cue(i); }
                    });
                }
                ui.label(RichText::new("Cue tests need audio output started. Pause the music to hear effects alone. Apple Music and external file players use their own volume controls.").color(MUTED));
            });
        });
        self.player.gains(self.settings.gains.clone());
        if (ctx.zoom_factor() - self.settings.zoom).abs() > 0.001 {
            ctx.set_zoom_factor(self.settings.zoom);
        }
    }
    fn console(&self, ui: &mut egui::Ui, height: f32) {
        egui::ScrollArea::vertical()
            .id_salt("transcript")
            .max_height(height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for line in &self.logs {
                    ui.label(
                        RichText::new(line)
                            .monospace()
                            .color(if line.starts_with('>') {
                                CYAN
                            } else if line.contains("ERROR") {
                                PINK
                            } else {
                                Color32::from_rgb(183, 198, 218)
                            }),
                    );
                }
            });
    }
    fn command(&mut self, ui: &mut egui::Ui) {
        ui.columns(3, |columns| {
            card(&mut columns[0], "MISSION", |ui| {
                ui.label(RichText::new(format!("{:02}", self.state.mission + 1)).size(28.));
            });
            card(&mut columns[1], "LARP PRESSURE", |ui| {
                ui.label(
                    RichText::new(format!("{}%", self.state.larp))
                        .size(28.)
                        .color(PINK),
                );
            });
            card(&mut columns[2], "PERSONA", |ui| {
                ui.label(
                    RichText::new(if self.state.poopsec {
                        "PORCELAIN DEMON"
                    } else {
                        &self.state.mascot
                    })
                    .size(22.)
                    .color(GOLD),
                );
            });
        });
        ui.add_space(16.);
        self.actions(
            ui,
            &[
                ("Help", "help"),
                ("Mission next", "mission next"),
                ("Oracle", "oracle"),
                ("Drummer explosion", "band drummer explode"),
                ("Pedal stomp", "pedal stomp"),
            ],
        );
        card(ui, "COMMAND TRANSCRIPT / REAL LOCAL EXECUTION", |ui| {
            self.console(
                ui,
                ui.ctx().content_rect().height().mul_add(0.46, 0.).max(180.),
            );
        });
        ui.add_space(14.);
        ui.horizontal(|ui| {
            ui.label(RichText::new(">").size(24.).color(CYAN));
            let response = ui.add_sized(
                [ui.available_width() - 108., 42.],
                egui::TextEdit::singleline(&mut self.input)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("Enter a command…   Tab completes / ↑↓ history"),
            );
            if self.command_focus {
                response.request_focus();
                self.command_focus = false;
            }
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                let _ = self.session.requests.try_send(Request::Complete(
                    self.input.clone(),
                    self.input.chars().count(),
                ));
            }
            if response.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                && !self.state.history.is_empty()
            {
                self.history_index = self.history_index.saturating_sub(1);
                self.input = self.state.history
                    [self.history_index.min(self.state.history.len() - 1)]
                .clone();
            }
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.history_index = (self.history_index + 1).min(self.state.history.len());
                self.input = self
                    .state
                    .history
                    .get(self.history_index)
                    .cloned()
                    .unwrap_or_default();
            }
            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui.button("Execute").clicked()
            {
                let command = std::mem::take(&mut self.input);
                if !command.trim().is_empty() {
                    self.send(&command);
                }
                self.command_focus = true;
            }
        });
        if ui.button("Cancel running process").clicked() {
            let _ = self.session.requests.try_send(Request::Cancel);
        }
        ui.add_space(14.);
        let timings: Vec<_> = self
            .cadence
            .iter()
            .enumerate()
            .map(|(i, seconds)| [i as f64, f64::from(*seconds) * 1000.])
            .collect();
        chart(
            ui,
            "ACTUAL SESSION / FRAME CADENCE",
            &timings,
            false,
            ("frame", "ms"),
            CYAN,
            110.,
        );
        ui.label(RichText::new(format!("Uptime {:.1}s / {} history entries / {} transcript lines / {} successful operations",self.started.elapsed().as_secs_f64(),self.state.history.len(),self.logs.len(),self.state.operations)).size(12.).color(MUTED));
    }
    fn concert(&mut self, ui: &mut egui::Ui) {
        self.actions(
            ui,
            &[
                ("Band on", "band on"),
                ("Band off", "band off"),
                ("EXPLODE RINGO", "band drummer explode"),
                ("Stomp pedal", "pedal stomp"),
                ("SSD KABOOM", "ssd explode"),
            ],
        );
        stage_visual(ui, &self.state, self.started.elapsed().as_secs_f32(), 390.);
        ui.add_space(12.);
        ui.columns(2, |c| {
            card(&mut c[0],"THE BEATLES / SILENT VISUAL TRIBUTE",|ui| { ui.label("John · Paul · George · Ringo"); ui.label(RichText::new("Drummer bursts and reassembles on the original timer. The soundtrack is our original Nightfall score.").color(MUTED)); });
            card(&mut c[1],"PEDAL / HEADPHONE STAGE",|ui| { ui.label(format!("{} / pedal {} / LARP {}%",self.state.headphone,if self.state.pedal {"DOWN"} else {"UP"},self.state.larp)); });
        });
        ui.add_space(12.);
        companion_visual(ui, &self.state, self.started.elapsed().as_secs_f32());
        self.actions(
            ui,
            &[
                ("R70xa", "headphones select ath-r70xa"),
                ("HD58X", "headphones select hd58x"),
                ("Mo & Krill", "mascot mo-krill"),
                ("Tweeter", "mascot tweeter"),
                ("Krillion", "mascot krillion"),
                ("Chinese demon", "mascot chinese-demon"),
                ("Porcelain mode", "poopsec on"),
                ("Demon mode", "poopsec off"),
                ("Prostreet scene", "mode prostreet"),
                ("Vulcan scene", "mode vulcan"),
                ("Demon scene", "mode demon"),
            ],
        );
    }
    fn audio(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.audio_source, false, "Filter + room models");
            ui.selectable_value(&mut self.audio_source, true, "Original score signal");
        });
        ui.add_space(12.);
        if self.audio_source {
            self.source_graphs(ui);
            return;
        }
        card(ui, "FILTER LAB / ANALYTIC SIMULATION", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.model.cutoff, 20.0..=20_000.)
                        .logarithmic(true)
                        .text("Cutoff Hz"),
                );
                ui.add(egui::Slider::new(&mut self.model.q, 0.2..=4.).text("Q"));
                ui.add(
                    egui::Slider::new(&mut self.model.transport_delay_s, 0.0..=0.02)
                        .text("Delay s"),
                );
            });
            ui.label(RichText::new("Filter and headphone curves are invented models. Room decay uses Sabine's formula. These are not measured hardware responses.").size(12.).color(MUTED));
        });
        for responses in Response::ALL.chunks(2) {
            ui.add_space(12.);
            ui.columns(2, |columns| {
                for (column, response) in columns.iter_mut().zip(responses) {
                    let points: Vec<_> = (0..256)
                        .map(|i| {
                            let f = 20. * 1000f64.powf(f64::from(i) / 255.);
                            [f, self.model.value(*response, f)]
                        })
                        .collect();
                    chart(
                        column,
                        response.label(),
                        &points,
                        true,
                        ("Hz", response.unit()),
                        CYAN,
                        170.,
                    );
                }
            });
        }
        ui.add_space(12.);
        ui.columns(2, |c| {
            chart(
                &mut c[0],
                &format!("{} / INVENTED EQ", self.state.headphone),
                &self.state.headphone_curve,
                true,
                ("Hz", "dB"),
                PINK,
                170.,
            );
            chart(
                &mut c[1],
                &format!("ROOM DECAY / RT60 {:.2} s", self.state.rt60),
                &self.state.room_curve,
                false,
                ("s", "dB"),
                GOLD,
                170.,
            );
        });
        ui.add_space(12.);
        card(ui, "ROOM DIMENSIONS", |ui| {
            ui.horizontal_wrapped(|ui| {
                for (i, label) in ["Width m", "Length m", "Height m", "Absorption"]
                    .into_iter()
                    .enumerate()
                {
                    ui.label(label);
                    ui.add(
                        egui::DragValue::new(&mut self.room[i])
                            .speed(0.01)
                            .range(if i == 3 { 0.01..=0.99 } else { 0.1..=100. }),
                    );
                }
                if ui.button("Apply room").clicked() {
                    self.send(&format!(
                        "rt60 set {} {} {} {}",
                        self.room[0], self.room[1], self.room[2], self.room[3]
                    ));
                }
            });
        });
        ui.add_space(12.);
        self.source_graphs(ui);
    }
    fn source_graphs(&self, ui: &mut egui::Ui) {
        ui.columns(2, |c| {
            chart(
                &mut c[0],
                "NIGHTFALL / EMBEDDED PCM WINDOW",
                &self.pcm,
                false,
                ("ms", "amplitude"),
                CYAN,
                240.,
            );
            chart(
                &mut c[1],
                "NIGHTFALL / HANN-WINDOWED SPECTRUM",
                &self.spectrum,
                true,
                ("Hz", "dBFS"),
                PINK,
                240.,
            );
        });
        ui.label(RichText::new("Source analysis follows an estimated soundtrack timeline; it does not capture microphone, Apple Music audio, or the output device.").size(12.).color(MUTED));
    }
    fn arcade(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("SCORE {:08}", self.state.score))
                    .size(28.)
                    .color(CYAN),
            );
            ui.label(
                RichText::new(format!("{} COMBO", self.state.combo))
                    .size(28.)
                    .color(PINK),
            );
        });
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 340.), egui::Sense::hover());
        let p = ui.painter_at(rect);
        let center = rect.center();
        let t = self.started.elapsed().as_secs_f32();
        p.rect_filled(rect, 12., Color32::from_rgba_unmultiplied(6, 13, 29, 215));
        for lane in 0..6 {
            let x = center.x + (lane as f32 - 2.5) * rect.width() / 8.;
            let top = Pos2::new(center.x + (lane as f32 - 2.5) * 12., rect.top() + 24.);
            p.line_segment(
                [top, Pos2::new(x, rect.bottom() - 48.)],
                Stroke::new(1., MUTED.gamma_multiply(0.35)),
            );
            for note in 0..5 {
                let depth = (t * 0.6 + note as f32 * 0.21 + lane as f32 * 0.13).fract();
                let pos = top.lerp(Pos2::new(x, rect.bottom() - 54.), depth * depth);
                p.rect_filled(
                    Rect::from_center_size(pos, Vec2::new(8. + depth * 38., 3. + depth * 9.)),
                    2.,
                    if lane == 0 || lane == 5 { PINK } else { CYAN },
                );
            }
            p.text(
                Pos2::new(x, rect.bottom() - 25.),
                egui::Align2::CENTER_CENTER,
                [
                    "A / FX-L", "D / BT-A", "F / BT-B", "J / BT-C", "K / BT-D", "L / FX-R",
                ][lane],
                egui::FontId::monospace(13.),
                GOLD,
            );
        }
        ui.label(RichText::new("SIM rhythm lane: decorative notes, original command scoring. Keyboard works while no text input has focus.").color(MUTED).size(12.));
        self.actions(
            ui,
            &[
                ("BT-A", "sdvx hit bt-a"),
                ("BT-B", "sdvx hit bt-b"),
                ("BT-C", "sdvx hit bt-c"),
                ("BT-D", "sdvx hit bt-d"),
                ("FX-L", "sdvx hit fx-l"),
                ("FX-R", "sdvx hit fx-r"),
            ],
        );
        let (mut left, mut right) = self.state.knobs;
        ui.horizontal(|ui| {
            if ui
                .add(egui::Slider::new(&mut left, 0..=100).text("Left laser"))
                .changed()
            {
                self.send(&format!(
                    "sdvx knob left {}",
                    i16::from(left) - i16::from(self.state.knobs.0)
                ));
            }
            if ui
                .add(egui::Slider::new(&mut right, 0..=100).text("Right laser"))
                .changed()
            {
                self.send(&format!(
                    "sdvx knob right {}",
                    i16::from(right) - i16::from(self.state.knobs.1)
                ));
            }
        });
        ui.add_space(12.);
        self.actions(
            ui,
            &[
                ("Generate paper coins", "coins generate"),
                ("Mint paper coins", "coins mint"),
                ("Pump paper coins", "coins pump"),
                ("Coin status", "coins status"),
            ],
        );
    }
    fn story(&mut self, ui: &mut egui::Ui) {
        self.actions(
            ui,
            &[
                ("Start Lost Signals", "novel start"),
                ("Next page", "novel next"),
                ("Choose 1", "novel choose 1"),
                ("Choose 2", "novel choose 2"),
                ("Reset story", "novel reset"),
                ("Next mission", "mission next"),
            ],
        );
        card(ui, "LOST SIGNALS / ORIGINAL FICTION", |ui| {
            ui.set_min_height(280.);
            ui.label(RichText::new(&self.state.novel).size(20.));
        });
        ui.add_space(16.);
        card(ui, "STORY TRANSMISSIONS", |ui| self.console(ui, 220.));
    }
    fn props(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(
                "SIM / in-memory props. All commands remain available on the Command Deck.",
            )
            .color(GOLD),
        );
        let props = self.state.props.clone();
        for chunk in props.chunks(2) {
            ui.add_space(12.);
            ui.columns(2, |columns| {
                for (ui, (name, status)) in columns.iter_mut().zip(chunk) {
                    card(ui, name, |ui| {
                        ui.label(RichText::new(status).size(13.).color(MUTED));
                        let buttons: &[(&str, &str)] = match *name {
                            "GOLDEN GOOSE EGG" => {
                                &[("Start egg", "egg start"), ("Hatch", "egg hatch")]
                            }
                            "CURSED APPLE" => &[("Status", "ritual")],
                            "SSD DISASTER" => {
                                &[("Explode", "ssd explode"), ("Status", "ssd status")]
                            }
                            "MILK MESH" => {
                                &[("Routes", "tailscale routes"), ("Churn", "tailscale churn")]
                            }
                            "VIRTUAL EXPANSION" => &[
                                ("Expand", "expansion expand"),
                                ("Status", "expansion status"),
                            ],
                            "LARP CITY" => &[
                                ("Boot", "opsecular boot"),
                                ("Terminal", "opsecular terminal"),
                                ("Poutine next", "opsecular poutine next"),
                                ("Weep", "opsecular weep 3"),
                            ],
                            "YOGGIES" => &[
                                ("Use one", "yoggies use"),
                                ("Refill", "yoggies refill"),
                                ("Holidays", "yoggies holidays"),
                            ],
                            "COUNTRY DECK" => {
                                &[("Status", "hoi4 status"), ("Countries", "hoi4 list")]
                            }
                            _ => &[("Coins", "coins status"), ("Pump", "coins pump")],
                        };
                        self.actions(ui, buttons);
                    });
                }
            });
        }
        ui.add_space(12.);
        self.actions(
            ui,
            &[
                ("Bido technique", "bido"),
                ("B2 props", "bucket ls"),
                ("Car rig", "focus calculate"),
                ("Rips catalog", "rips list"),
                ("SiIva catalog", "rips siiva"),
            ],
        );
    }
    fn integrations(&mut self, ui: &mut egui::Ui) {
        card(ui, "APPLE MUSIC / LOCAL WINDOWS MEDIA SESSION", |ui| {
            ui.label(RichText::new(&self.music).size(20.).color(CYAN));
            ui.label(RichText::new("Read-only metadata from Windows media sessions. Open Apple Music on this PC and start a track. Nightfall's loop is controlled separately in the sidebar.").color(MUTED));
            self.actions(
                ui,
                &[
                    ("Live metadata", "music live"),
                    ("Demo metadata", "music demo"),
                    ("Music status", "music status"),
                ],
            );
        });
        ui.add_space(16.);
        card(ui, "DISCORD / NATIVE NAMED-PIPE RICH PRESENCE", |ui| {
            ui.label("Uses your running Discord desktop client and DISCORD_APPLICATION_ID. The existing profile, timer, asset and display commands are preserved.");
            self.actions(
                ui,
                &[
                    ("Connect", "discord on"),
                    ("Disconnect", "discord off"),
                    ("Status", "discord status"),
                    ("Preview", "discord preview"),
                    ("Reset profile", "discord reset"),
                ],
            );
            ui.horizontal(|ui| {
                ui.label("Profile path");
                ui.text_edit_singleline(&mut self.profile);
                if ui.button("Load profile").clicked() {
                    if self.profile.contains(['"', '\n', '\r']) {
                        self.log(
                            "[LOCAL] Profile path must not contain quotes or line breaks.".into(),
                        );
                    } else {
                        self.send(&format!("discord load \"{}\"", self.profile));
                    }
                }
            });
        });
        ui.add_space(16.);
        card(ui, "INTEGRATION OUTPUT", |ui| self.console(ui, 250.));
    }
}

fn chart(
    ui: &mut egui::Ui,
    title: &str,
    points: &[[f64; 2]],
    log_x: bool,
    units: (&str, &str),
    color: Color32,
    height: f32,
) {
    let (x_unit, y_unit) = units;
    card(ui, title, |ui| {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );
        let p = ui.painter_at(rect);
        let plot = Rect::from_min_max(
            rect.min + Vec2::new(70., 20.),
            rect.max - Vec2::new(14., 20.),
        );
        if points.is_empty() {
            p.text(
                plot.center(),
                egui::Align2::CENTER_CENTER,
                "Waiting for model…",
                egui::FontId::proportional(14.),
                MUTED,
            );
            return;
        }
        let min_x = points[0][0];
        let max_x = points.last().unwrap()[0];
        let min_y = points.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min);
        let max_y = points
            .iter()
            .map(|v| v[1])
            .fold(f64::NEG_INFINITY, f64::max);
        let pad = ((max_y - min_y) * 0.08).max(if y_unit == "ms" { 0.1 } else { 0.5 });
        let (min_y, max_y) = (min_y - pad, max_y + pad);
        let x = |v: f64| {
            if log_x {
                (v / min_x).ln() / (max_x / min_x).ln()
            } else {
                (v - min_x) / (max_x - min_x).max(1e-9)
            }
        };
        for i in 0..=4 {
            let fraction = i as f32 / 4.;
            let yy = plot.bottom() - plot.height() * fraction;
            let xx = plot.left() + plot.width() * fraction;
            p.line_segment(
                [Pos2::new(plot.left(), yy), Pos2::new(plot.right(), yy)],
                Stroke::new(1., MUTED.gamma_multiply(0.15)),
            );
            p.line_segment(
                [Pos2::new(xx, plot.top()), Pos2::new(xx, plot.bottom())],
                Stroke::new(1., MUTED.gamma_multiply(0.15)),
            );
            p.text(
                Pos2::new(plot.left() - 5., yy),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", min_y + (max_y - min_y) * f64::from(fraction)),
                egui::FontId::monospace(10.),
                MUTED,
            );
        }
        let line: Vec<_> = points
            .iter()
            .map(|v| {
                Pos2::new(
                    plot.left() + x(v[0]) as f32 * plot.width(),
                    plot.bottom() - ((v[1] - min_y) / (max_y - min_y)) as f32 * plot.height(),
                )
            })
            .collect();
        p.add(egui::Shape::line(
            line.clone(),
            Stroke::new(5., color.gamma_multiply(0.15)),
        ));
        p.add(egui::Shape::line(line, Stroke::new(1.8, color)));
        p.text(
            Pos2::new(plot.left(), plot.bottom() + 14.),
            egui::Align2::LEFT_CENTER,
            format!("{min_x:.0} {x_unit}"),
            egui::FontId::monospace(10.),
            MUTED,
        );
        p.text(
            Pos2::new(plot.right(), plot.bottom() + 14.),
            egui::Align2::RIGHT_CENTER,
            format!("{max_x:.0} {x_unit} / {y_unit}"),
            egui::FontId::monospace(10.),
            MUTED,
        );
        if response.hovered()
            && let Some(pos) = response.hover_pos()
        {
            let fraction = ((pos.x - plot.left()) / plot.width()).clamp(0., 1.) as f64;
            let target = if log_x {
                min_x * (max_x / min_x).powf(fraction)
            } else {
                min_x + (max_x - min_x) * fraction
            };
            if let Some(nearest) = points
                .iter()
                .min_by(|a, b| (a[0] - target).abs().total_cmp(&(b[0] - target).abs()))
            {
                response.on_hover_text(format!(
                    "{:.2} {x_unit} → {:.3} {y_unit}",
                    nearest[0], nearest[1]
                ));
            }
        }
    });
}

fn stage_visual(ui: &mut egui::Ui, state: &Snapshot, time: f32, height: f32) {
    let (r, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let p = ui.painter_at(r);
    p.rect_filled(r, 14., Color32::from_rgba_unmultiplied(5, 10, 26, 200));
    let floor = r.bottom() - 62.;
    for i in 0..8 {
        let x = r.left() + r.width() * i as f32 / 7.;
        let target = r.left() + r.width() * (0.5 + 0.4 * (time * 0.4 + i as f32).sin());
        p.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(x, r.top()),
                Pos2::new(target - 25., floor),
                Pos2::new(target + 25., floor),
            ],
            CYAN.gamma_multiply(0.055),
            Stroke::NONE,
        ));
    }
    p.line_segment(
        [
            Pos2::new(r.left() + 20., floor),
            Pos2::new(r.right() - 20., floor),
        ],
        Stroke::new(2., CYAN),
    );
    if !state.band {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            "BAND STANDBY",
            egui::FontId::monospace(28.),
            MUTED,
        );
        return;
    }
    for (i, name) in ["JOHN", "PAUL", "GEORGE", "RINGO"].into_iter().enumerate() {
        let x = r.left() + r.width() * (0.14 + i as f32 * 0.24);
        let bounce = (time * std::f32::consts::TAU * 148. / 60. + i as f32).sin() * 4.;
        let y = floor - 105. + bounce;
        let color = if i == 3 {
            GOLD
        } else if i == 1 {
            PINK
        } else {
            CYAN
        };
        if i == 3 && state.drummer_blast {
            for n in 0..48 {
                let a = n as f32 * 2.399 + time * 0.4;
                let distance = 25. + (n as f32 * 17. + time * 100.) % 140.;
                let point = Pos2::new(x + a.cos() * distance, y + a.sin() * distance * 0.7);
                p.rect_filled(
                    Rect::from_center_size(point, Vec2::splat(3. + n as f32 % 6.)),
                    1.,
                    color,
                );
            }
            p.text(
                Pos2::new(x, y),
                egui::Align2::CENTER_CENTER,
                "KABOOM",
                egui::FontId::monospace(28.),
                PINK,
            );
        } else {
            p.circle_filled(Pos2::new(x, y - 53.), 19., Color32::from_rgb(182, 166, 153));
            p.rect_filled(
                Rect::from_min_max(Pos2::new(x - 21., y - 74.), Pos2::new(x + 21., y - 55.)),
                7.,
                Color32::from_rgb(20, 22, 33),
            );
            p.rect_filled(
                Rect::from_center_size(Pos2::new(x, y), Vec2::new(42., 62.)),
                7.,
                Color32::from_rgb(24, 34, 52),
            );
            for side in [-1., 1.] {
                p.line_segment(
                    [
                        Pos2::new(x + side * 12., y + 27.),
                        Pos2::new(x + side * 20., floor - 4.),
                    ],
                    Stroke::new(9., Color32::from_rgb(23, 31, 43)),
                );
            }
            if i == 3 {
                p.circle_filled(
                    Pos2::new(x, floor - 27.),
                    31.,
                    Color32::from_rgb(57, 35, 38),
                );
                p.circle_stroke(Pos2::new(x, floor - 27.), 31., Stroke::new(2., GOLD));
                for side in [-1., 1.] {
                    let cymbal = Pos2::new(x + side * 51., y - 3.);
                    p.line_segment([cymbal, Pos2::new(cymbal.x, floor)], Stroke::new(2., MUTED));
                    p.rect_filled(Rect::from_center_size(cymbal, Vec2::new(47., 5.)), 3., GOLD);
                    p.line_segment(
                        [
                            Pos2::new(x + side * 18., y),
                            Pos2::new(x + side * 40., y - 24. + bounce * 3.),
                        ],
                        Stroke::new(3., GOLD),
                    );
                }
            } else {
                p.circle_filled(Pos2::new(x + 5., y + 15.), 20., color.gamma_multiply(0.65));
                p.line_segment(
                    [Pos2::new(x, y + 20.), Pos2::new(x + 57., y - 40.)],
                    Stroke::new(6., GOLD),
                );
                p.line_segment(
                    [
                        Pos2::new(x - 20., y - 14.),
                        Pos2::new(x + 8., y + 12. + bounce),
                    ],
                    Stroke::new(5., color),
                );
            }
        }
        p.text(
            Pos2::new(x, r.bottom() - 27.),
            egui::Align2::CENTER_CENTER,
            name,
            egui::FontId::monospace(16.),
            color,
        );
    }
    if state.ssd_blast {
        p.text(
            Pos2::new(r.center().x, r.top() + 30.),
            egui::Align2::CENTER_CENTER,
            "SSD DISASTER / SIM EXPLOSION",
            egui::FontId::monospace(22.),
            PINK,
        );
    }
}

fn companion_visual(ui: &mut egui::Ui, state: &Snapshot, time: f32) {
    let (r, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 180.), egui::Sense::hover());
    let p = ui.painter_at(r);
    let c = r.center();
    let bounce = (time * 3.).sin() * 4.;
    p.rect_filled(r, 12., Color32::from_rgba_unmultiplied(7, 16, 30, 220));
    let color = match state.theme.as_str() {
        "venom" => Color32::from_rgb(139, 233, 109),
        "ghost" => Color32::from_rgb(185, 209, 241),
        _ => PINK,
    };
    if state.scene == "prostreet" {
        let body = Rect::from_center_size(c + Vec2::new(0., 10.), Vec2::new(250., 40.));
        p.rect_filled(body, 15., color.gamma_multiply(0.65));
        p.add(egui::Shape::convex_polygon(
            vec![
                c + Vec2::new(-70., -10.),
                c + Vec2::new(-43., -45.),
                c + Vec2::new(48., -45.),
                c + Vec2::new(88., -10.),
            ],
            CYAN.gamma_multiply(0.25),
            Stroke::new(2., CYAN),
        ));
        for dx in [-78., 78.] {
            let wheel = c + Vec2::new(dx, 30.);
            p.circle_filled(wheel, 22., Color32::BLACK);
            p.circle_stroke(wheel, 18., Stroke::new(2., CYAN));
            for i in 0..5 {
                let a = time * 6. + i as f32 * 1.257;
                p.line_segment(
                    [wheel, wheel + Vec2::angled(a) * 15.],
                    Stroke::new(2., MUTED),
                );
            }
        }
        for i in 0..12 {
            let x = r.left() + (i as f32 * 140. - time * 180.).rem_euclid(r.width());
            p.line_segment(
                [
                    Pos2::new(x, r.bottom() - 18.),
                    Pos2::new(x + 60., r.bottom() - 18.),
                ],
                Stroke::new(2., GOLD.gamma_multiply(0.6)),
            );
        }
    } else if state.scene == "vulcan" {
        for n in 0..4 {
            let radius = 20. + n as f32 * 17.;
            p.circle_stroke(
                c,
                radius,
                Stroke::new(2., if n % 2 == 0 { CYAN } else { color }),
            );
            for i in 0..8 {
                let a = time * (0.2 + n as f32 * 0.2) + i as f32 * 0.785;
                p.circle_filled(c + Vec2::angled(a) * radius, 3., GOLD);
            }
        }
    } else {
        let c = c + Vec2::new(0., bounce);
        if state.poopsec {
            p.rect_filled(
                Rect::from_center_size(c + Vec2::new(0., 30.), Vec2::new(55., 60.)),
                12.,
                Color32::from_rgb(218, 227, 233),
            );
            p.add(egui::Shape::ellipse_filled(
                c,
                Vec2::new(68., 35.),
                Color32::from_rgb(236, 236, 225),
            ));
            p.add(egui::Shape::ellipse_filled(
                c,
                Vec2::new(48., 19.),
                color.gamma_multiply(0.7),
            ));
        } else if state.mascot == "tweeter" {
            p.circle_filled(c, 48., CYAN.gamma_multiply(0.7));
            p.add(egui::Shape::convex_polygon(
                vec![
                    c + Vec2::new(45., -7.),
                    c + Vec2::new(76., 3.),
                    c + Vec2::new(43., 12.),
                ],
                GOLD,
                Stroke::NONE,
            ));
            p.circle_filled(c + Vec2::new(18., -16.), 5., Color32::BLACK);
            p.line_segment(
                [c + Vec2::new(-8., 47.), c + Vec2::new(-8., 64.)],
                Stroke::new(4., GOLD),
            );
        } else {
            let crab = matches!(state.mascot.as_str(), "mo-krill" | "krillion");
            p.circle_filled(c, 44., color.gamma_multiply(0.5));
            for side in [-1., 1.] {
                if crab {
                    let claw = c + Vec2::new(side * 75., -10.);
                    p.line_segment(
                        [c + Vec2::new(side * 35., 12.), claw],
                        Stroke::new(8., color),
                    );
                    p.circle_stroke(claw, 18., Stroke::new(6., color));
                } else {
                    p.add(egui::Shape::convex_polygon(
                        vec![
                            c + Vec2::new(side * 35., -18.),
                            c + Vec2::new(side * 54., -69.),
                            c + Vec2::new(side * 9., -38.),
                        ],
                        color,
                        Stroke::NONE,
                    ));
                }
                let eye = c + Vec2::new(side * 18., -8.);
                p.line_segment(
                    [eye + Vec2::new(-9., -4.), eye + Vec2::new(9., 4.)],
                    Stroke::new(4., CYAN),
                );
            }
            p.line_segment(
                [c + Vec2::new(-20., 18.), c + Vec2::new(20., 18.)],
                Stroke::new(3., GOLD),
            );
        }
    }
    p.text(
        r.left_top() + Vec2::new(18., 18.),
        egui::Align2::LEFT_TOP,
        format!(
            "SIM ILLUSTRATION / {} / {}",
            state.scene,
            if state.poopsec {
                "porcelain demon"
            } else {
                &state.mascot
            }
        ),
        egui::FontId::monospace(12.),
        MUTED,
    );
}

impl eframe::App for Deck {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.ui(ctx);
        if let Some(frames) = &mut self.window_smoke_frames {
            *frames += 1;
            if *frames >= 120 {
                self.window_smoke_passed.store(true, Ordering::Release);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.settings.cap == 0 {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_secs_f64(1. / f64::from(self.settings.cap)));
        }
    }
}
pub fn run(shell: Shell, smoke: bool) -> io::Result<u8> {
    let shell = if smoke {
        crate::desktop_audio::smoke()?;
        desktop_gpu::smoke(shell)?;
        Shell::new(false)
    } else {
        shell
    };
    if !smoke {
        hide_own_console();
    }
    let uncapped = std::env::var("OPSEC_DEMON_UNCAPPED").is_ok_and(|value| value == "1")
        || (!smoke && !crate::desktop_settings::Settings::load().vsync);
    let exit_code = Arc::new(AtomicU8::new(0));
    let app_exit = Arc::clone(&exit_code);
    let smoke_passed = Arc::new(AtomicBool::new(false));
    let app_smoke_passed = Arc::clone(&smoke_passed);
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: desktop_gpu::configuration(uncapped),
        viewport: egui::ViewportBuilder::default()
            .with_title("OPSEC DEMON / Vulkan Concert Deck")
            .with_inner_size([1600., 1000.])
            .with_min_inner_size([1000., 700.]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "OPSEC DEMON",
        options,
        Box::new(move |cc| {
            style(&cc.egui_ctx);
            let state = cc
                .wgpu_render_state
                .as_ref()
                .ok_or("Vulkan render state unavailable")?;
            let info = state.adapter.get_info();
            if info.backend != wgpu::Backend::Vulkan {
                return Err("A Vulkan adapter is required".into());
            }
            desktop_gpu::install(
                &state.device,
                state.target_format,
                &mut state.renderer.write(),
            );
            let mut deck = Deck::new(shell, format!("Vulkan / {}", info.name), !smoke);
            cc.egui_ctx.set_zoom_factor(deck.settings.zoom);
            deck.exit_code = app_exit;
            if smoke {
                deck.window_smoke_frames = Some(0);
                deck.window_smoke_passed = app_smoke_passed;
            }
            Ok(Box::new(deck))
        }),
    );
    if let Err(error) = result {
        if smoke {
            return Err(io::Error::other(error.to_string()));
        }
        let message = format!(
            "The Vulkan concert deck could not open.\n\n{error}\n\nInstall/update your graphics manufacturer's Vulkan-capable driver. You can still run --shell from a terminal."
        );
        let message: Vec<_> = message.encode_utf16().chain([0]).collect();
        let title: Vec<_> = "OPSEC DEMON / graphics initialization"
            .encode_utf16()
            .chain([0])
            .collect();
        unsafe {
            MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), 0x10);
        }
        return Err(io::Error::other(error.to_string()));
    }
    if smoke {
        if !smoke_passed.load(Ordering::Acquire) {
            return Err(io::Error::other(
                "Native Vulkan window did not complete its presentation smoke",
            ));
        }
        println!("[WINDOW] Native Vulkan presentation and 120 GUI updates passed (audio muted).");
        let mut report: serde_json::Value =
            serde_json::from_slice(&std::fs::read("target/vulkan-smoke.json")?)?;
        report["native_window"] = "passed".into();
        report["audio_graphs"] = 8.into();
        report["windows_pcm_output"] = "passed (master muted)".into();
        std::fs::write(
            "target/vulkan-smoke.json",
            serde_json::to_vec_pretty(&report)?,
        )?;
    }
    Ok(exit_code.load(Ordering::Relaxed))
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
    fn GetConsoleProcessList(processes: *mut u32, count: u32) -> u32;
}
#[link(name = "user32")]
unsafe extern "system" {
    fn ShowWindow(window: *mut std::ffi::c_void, command: i32) -> i32;
    fn MessageBoxW(
        window: *mut std::ffi::c_void,
        message: *const u16,
        title: *const u16,
        flags: u32,
    ) -> i32;
}
fn hide_own_console() {
    // Double-click creates a console owned by this app. Preserve a shared terminal.
    let mut processes = [0u32; 2];
    unsafe {
        if GetConsoleProcessList(processes.as_mut_ptr(), 2) == 1 {
            let window = GetConsoleWindow();
            if !window.is_null() {
                ShowWindow(window, 0);
            }
        }
    }
}

pub fn prepare_smoke(ctx: &egui::Context) {
    style(ctx);
}
pub fn set_page(deck: &mut Deck, page: usize) {
    deck.page = page;
}
pub fn set_source_view(deck: &mut Deck, source: bool) {
    deck.audio_source = source;
}
pub fn set_smoke_upscaler(deck: &mut Deck, fsr: usize, scale: f32) {
    deck.settings.fsr = fsr;
    deck.settings.scale = scale;
}
