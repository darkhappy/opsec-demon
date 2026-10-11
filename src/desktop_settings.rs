//! Local, versioned desktop preferences. Unknown fields are harmless; values are bounded.
use crate::desktop_audio::Gains;
use std::{io, path::PathBuf};
pub const LOOK: [(&str, f32, f32, f32); 14] = [
    ("Camera FOV (degrees)", 30., 130., 75.),
    ("Mouse sensitivity / backdrop parallax", 0., 5., 1.),
    ("Exposure", 0.1, 3., 1.),
    ("Neon glow", 0., 3., 1.),
    ("Saturation", 0., 2., 1.),
    ("Vignette", 0., 1., 0.2),
    ("Film grain", 0., 0.15, 0.),
    ("Chromatic split", 0., 0.02, 0.),
    ("Scanlines", 0., 1., 0.),
    ("Floor grid", 0., 2., 1.),
    ("Laser beams", 0., 2., 1.),
    ("Orbit particles", 0., 128., 28.),
    ("Animation speed", 0., 3., 1.),
    ("Explosion camera shake", 0., 1., 0.2),
];
pub const FSR: [&str; 5] = [
    "Off / native or bilinear",
    "FSR 1 Ultra Quality",
    "FSR 1 Quality",
    "FSR 1 Balanced",
    "FSR 1 Performance",
];
#[derive(Clone)]
pub struct Settings {
    pub fsr: usize,
    pub scale: f32,
    pub sharpness: f32,
    pub look: [f32; 14],
    pub vsync: bool,
    pub fps: bool,
    pub cap: u32,
    pub reduced_motion: bool,
    pub zoom: f32,
    pub gains: Gains,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            fsr: 0,
            scale: 1.,
            sharpness: 0.7,
            look: LOOK.map(|x| x.3),
            vsync: true,
            fps: true,
            cap: 0,
            reduced_motion: false,
            zoom: 1.,
            gains: Gains::default(),
        }
    }
}
impl Settings {
    pub fn render_scale(&self) -> f32 {
        match self.fsr {
            1 => 1. / 1.3,
            2 => 1. / 1.5,
            3 => 1. / 1.7,
            4 => 0.5,
            _ => self.scale,
        }
    }
    pub fn preset(&mut self, preset: usize) {
        self.look = LOOK.map(|x| x.3);
        self.fsr = [4, 2, 1, 0][preset];
        self.scale = 1.;
        self.look[11] = [8., 28., 64., 128.][preset];
        self.look[3] = [0.5, 1., 1.4, 2.][preset];
    }
    fn path() -> io::Result<PathBuf> {
        std::env::var_os("LOCALAPPDATA")
            .map(|p| PathBuf::from(p).join("opsec-demon/graphics-audio.json"))
            .ok_or_else(|| io::Error::other("LOCALAPPDATA is unavailable"))
    }
    pub fn load() -> Self {
        Self::path()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|data| serde_json::from_slice(&data).ok())
            .map(|v| Self::decode(&v))
            .unwrap_or_default()
    }
    fn decode(v: &serde_json::Value) -> Self {
        let mut s = Self::default();
        let bounded = |v: &serde_json::Value, default: f32, min: f32, max: f32| {
            v.as_f64()
                .filter(|v| v.is_finite())
                .map_or(default, |v| (v as f32).clamp(min, max))
        };
        s.fsr = v["fsr"].as_u64().unwrap_or(0).min(4) as usize;
        s.scale = bounded(&v["scale"], 1., 0.5, 1.);
        s.sharpness = bounded(&v["sharpness"], 0.7, 0., 1.);
        s.zoom = bounded(&v["zoom"], 1., 0.75, 1.5);
        s.cap = v["cap"].as_u64().unwrap_or(0).min(1000) as u32;
        s.vsync = v["vsync"].as_bool().unwrap_or(true);
        s.fps = v["fps"].as_bool().unwrap_or(true);
        s.reduced_motion = v["reduced_motion"].as_bool().unwrap_or(false);
        for (i, (_, min, max, default)) in LOOK.iter().enumerate() {
            s.look[i] = bounded(&v["look"][i], *default, *min, *max);
        }
        s.gains.master = bounded(&v["master"], s.gains.master, 0., 1.);
        s.gains.music = bounded(&v["music"], s.gains.music, 0., 1.);
        s.gains.effects = bounded(&v["effects"], s.gains.effects, 0., 1.);
        s.gains.interface = bounded(&v["interface"], s.gains.interface, 0., 1.);
        for i in 0..s.gains.sounds.len() {
            s.gains.sounds[i] = bounded(&v["sounds"][i], 1., 0., 1.);
        }
        s
    }
    pub fn save(&self) -> io::Result<()> {
        let path = Self::path()?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        let data = serde_json::json!({"version":1,"fsr":self.fsr,"scale":self.scale,"sharpness":self.sharpness,"look":self.look,"vsync":self.vsync,"fps":self.fps,"cap":self.cap,"reduced_motion":self.reduced_motion,"zoom":self.zoom,"master":self.gains.master,"music":self.gains.music,"effects":self.gains.effects,"interface":self.gains.interface,"sounds":self.gains.sounds});
        // Preferences contain no credentials and are only written on the explicit Save action.
        std::fs::write(path, serde_json::to_vec_pretty(&data)?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_and_out_of_range_preferences_cannot_allocate_unbounded_targets() {
        let s = Settings::decode(
            &serde_json::json!({"fsr":999,"scale":-500,"cap":999999,"look":[999,-1],"sounds":[-1,50]}),
        );
        assert_eq!(s.fsr, 4);
        assert_eq!(s.render_scale(), 0.5);
        assert_eq!(s.cap, 1000);
        assert_eq!(s.look[0], 130.);
        assert_eq!(s.look[1], 0.);
        assert_eq!(s.gains.sounds[0], 0.);
        assert_eq!(s.gains.sounds[1], 1.);
    }
}
