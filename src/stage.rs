//! An original ASCII band tribute, cartoon percussion, and unmeasured headphone lore.
//! No playback or audio hardware is controlled by this module.
use std::time::{Duration, Instant};

pub const BAND_ART: &[&str] = &[
    "    THE FAB FOUR // ORIGINAL ASCII TRIBUTE",
    "      o         o         o         o",
    "     /|\\       /|\\       /|\\       /|\\",
    "    / |==>    <==|      / |==>   .-(===)-.",
    "     / \\       / \\      / \\     | DRUM |",
    "    JOHN       PAUL     GEORGE     RINGO",
    "      SIM SILENT STAGE // NO AUDIO OUTPUT",
];

const BLAST_INTERVAL: Duration = Duration::from_secs(25);
const BLAST_DURATION: Duration = Duration::from_secs(3);
const LARP_CAP: u16 = 999;

#[derive(Clone, Copy)]
enum Headphones {
    R70xa,
    Hd58x,
}

pub struct Stage {
    band: bool,
    blast_until: Option<Instant>,
    next_blast: Instant,
    pedal: bool,
    larp: u16,
    celebration: bool,
    headphones: Headphones,
}

impl Stage {
    pub fn new() -> Self {
        Self {
            band: true,
            blast_until: None,
            next_blast: Instant::now() + BLAST_INTERVAL,
            pedal: false,
            larp: 0,
            celebration: false,
            headphones: Headphones::R70xa,
        }
    }

    pub fn status(&self) -> String {
        format!(
            "SIM STAGE | band {} | pedal {} | larp {}/{} | {} / synthetic",
            if self.band { "on" } else { "off" },
            if self.pedal { "DOWN" } else { "up" },
            self.larp,
            LARP_CAP,
            self.headphone_name(),
        )
    }

    pub fn band_on(&self) -> bool {
        self.band
    }

    pub fn drummer_blast(&self) -> bool {
        self.band && self.blast_until.is_some_and(|until| Instant::now() < until)
    }

    pub fn pedal_down(&self) -> bool {
        self.pedal
    }

    pub fn larp_meter(&self) -> u16 {
        self.larp
    }

    pub fn headphone_name(&self) -> &str {
        match self.headphones {
            Headphones::R70xa => "Audio-Technica ATH-R70xa",
            Headphones::Hd58x => "Sennheiser x Massdrop HD58X Jubilee",
        }
    }

    pub fn headphone_id(&self) -> &str {
        match self.headphones {
            Headphones::R70xa => "ath-r70xa",
            Headphones::Hd58x => "hd58x",
        }
    }

    pub fn take_celebration(&mut self) -> bool {
        std::mem::take(&mut self.celebration)
    }

    /// Invented display gain in dB; these curves are not headphone measurements.
    pub fn frequency_gain(&self, frequency: f64) -> f64 {
        if !frequency.is_finite() || frequency <= 0.0 {
            return 0.0;
        }
        let log_frequency = frequency.clamp(20.0, 20_000.0).log10();
        match self.headphones {
            Headphones::R70xa => {
                0.6 * (log_frequency - 3.0) + 1.2 * (-((log_frequency - 3.5) / 0.28).powi(2)).exp()
            }
            Headphones::Hd58x => {
                2.0 / (1.0 + ((log_frequency - 2.5) * 5.0).exp())
                    - 1.1 * (-((log_frequency - 3.7) / 0.32).powi(2)).exp()
            }
        }
    }

    pub fn tick(&mut self) -> Option<String> {
        self.tick_at(Instant::now())
    }

    fn tick_at(&mut self, now: Instant) -> Option<String> {
        if !self.band {
            return None;
        }
        if self.blast_until.is_some_and(|until| now >= until) {
            self.blast_until = None;
            return Some(
                "SIM BAND | drummer reassembled from cartoon ashes; back on the silent beat".into(),
            );
        }
        if now >= self.next_blast {
            self.explode_at(now);
            return Some(
                "SIM BAND | scheduled drum solo detonated into dancing ASCII confetti".into(),
            );
        }
        None
    }

    fn explode_at(&mut self, now: Instant) {
        self.blast_until = Some(now + BLAST_DURATION);
        self.next_blast = now + BLAST_INTERVAL;
        self.celebration = true;
    }

    /// Namespace (`band`, `pedal`, or `headphones`) is included in arguments.
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args {
            [namespace] if namespace == "band" => Ok(vec![self.band_status()]),
            [namespace, action] if namespace == "band" && action == "status" => Ok(vec![self.band_status()]),
            [namespace, action] if namespace == "band" && matches!(action.as_str(), "on" | "off") => {
                self.band = action == "on";
                self.blast_until = None;
                self.next_blast = Instant::now() + BLAST_INTERVAL;
                Ok(vec![self.band_status()])
            }
            [namespace, role, action] if namespace == "band" && role == "drummer" && action == "explode" => {
                if !self.band { return Err("SIM BAND | band is off; run band on before the cartoon drum solo".into()); }
                self.explode_at(Instant::now());
                Ok(vec!["SIM BAND | drummer exploded into dancing ASCII confetti; cartoon respawn in 3 seconds".into()])
            }
            [namespace] if namespace == "pedal" => Ok(vec![self.pedal_status()]),
            [namespace, action] if namespace == "pedal" && action == "status" => Ok(vec![self.pedal_status()]),
            [namespace, action] if namespace == "pedal" && action == "stomp" => {
                self.pedal = true;
                self.larp = self.larp.saturating_add(15).min(LARP_CAP);
                self.celebration = true;
                Ok(vec!["SIM PEDAL | stomp accepted; phantom distortion and dancing explosion queued".into(), self.pedal_status()])
            }
            [namespace, action] if namespace == "pedal" && action == "release" => {
                self.pedal = false;
                Ok(vec![self.pedal_status()])
            }
            [namespace] if namespace == "headphones" => Ok(self.headphone_status()),
            [namespace, action] if namespace == "headphones" && action == "status" => Ok(self.headphone_status()),
            [namespace, action] if namespace == "headphones" && action == "list" => Ok(vec![
                "SIM HEADPHONES | ath-r70xa = Audio-Technica ATH-R70xa".into(),
                "SIM HEADPHONES | hd58x = Sennheiser x Massdrop HD58X Jubilee".into(),
                "SIM HEADPHONES | decorative, unmeasured curves; selecting a profile does not configure audio".into(),
            ]),
            [namespace, action, profile] if namespace == "headphones" && action == "select" => {
                self.headphones = match profile.as_str() {
                    "ath-r70xa" => Headphones::R70xa,
                    "hd58x" => Headphones::Hd58x,
                    _ => return Err("SIM HEADPHONES | select ath-r70xa or hd58x".into()),
                };
                Ok(self.headphone_status())
            }
            _ => Err("SIM | usage: band on|off|status|drummer explode; pedal stomp|release|status; headphones list|select ath-r70xa|select hd58x|status".into()),
        }
    }

    fn band_status(&self) -> String {
        format!(
            "SIM BAND | original Beatles ASCII tribute {} | cartoon drummer blasts every 25 seconds | silent animation",
            if self.band { "on" } else { "off" }
        )
    }

    fn pedal_status(&self) -> String {
        format!(
            "SIM PEDAL | {} | fictional larp meter {}/{} | no audio output",
            if self.pedal { "DOWN" } else { "up" },
            self.larp,
            LARP_CAP
        )
    }

    fn headphone_status(&self) -> Vec<String> {
        vec![
            format!("SIM HEADPHONES | {} ({})", self.headphone_name(), self.headphone_id()),
            "SIM HEADPHONES | frequency gain is an invented example curve, not measured response, EQ advice, or hardware detection".into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn periodic_blasts_expire_without_repeating_and_off_stops_the_clock() {
        let mut stage = Stage::new();
        let due = stage.next_blast;
        assert!(stage.tick_at(due - Duration::from_millis(1)).is_none());
        assert!(stage.tick_at(due).unwrap().contains("confetti"));
        assert!(stage.take_celebration());
        assert!(!stage.take_celebration());
        assert!(stage.tick_at(due + Duration::from_secs(2)).is_none());
        assert!(
            stage
                .tick_at(due + BLAST_DURATION)
                .unwrap()
                .contains("reassembled")
        );
        assert!(stage.tick_at(due + BLAST_DURATION).is_none());
        stage.command(&args(&["band", "off"])).unwrap();
        assert!(stage.tick_at(due + BLAST_INTERVAL * 10).is_none());
        assert!(!stage.band_on());
        assert!(
            stage
                .command(&args(&["band", "drummer", "explode"]))
                .is_err()
        );
    }

    #[test]
    fn pedal_caps_larp_and_invalid_commands_preserve_state() {
        let mut stage = Stage::new();
        for _ in 0..100 {
            stage.command(&args(&["pedal", "stomp"])).unwrap();
        }
        assert!(stage.pedal_down());
        assert_eq!(stage.larp_meter(), LARP_CAP);
        let before = stage.status();
        for command in [
            vec!["headphones", "select", "unknown"],
            vec!["pedal", "stomp", "extra"],
            vec!["band", "invalid"],
        ] {
            assert!(stage.command(&args(&command)).is_err());
            assert_eq!(stage.status(), before);
        }
        stage.command(&args(&["pedal", "release"])).unwrap();
        assert!(!stage.pedal_down());
        assert_eq!(stage.larp_meter(), LARP_CAP);
    }

    #[test]
    fn headphone_profiles_are_distinct_finite_and_explicitly_synthetic() {
        let mut stage = Stage::new();
        let first = stage.frequency_gain(100.0);
        stage
            .command(&args(&["headphones", "select", "hd58x"]))
            .unwrap();
        assert_ne!(first, stage.frequency_gain(100.0));
        assert!(stage.headphone_name().contains("HD58X Jubilee"));
        for frequency in [
            f64::NAN,
            f64::INFINITY,
            -1.0,
            0.0,
            1.0,
            20.0,
            1000.0,
            20000.0,
            f64::MAX,
        ] {
            let gain = stage.frequency_gain(frequency);
            assert!(gain.is_finite() && gain.abs() < 5.0);
        }
        assert!(
            stage
                .headphone_status()
                .iter()
                .all(|line| line.starts_with("SIM"))
        );
        assert!(stage.headphone_status()[1].contains("not measured"));
        assert!(BAND_ART.iter().all(|line| line.is_ascii()));
    }

    #[test]
    fn manual_blast_is_visible_and_consumes_one_celebration() {
        let mut stage = Stage::new();
        stage
            .command(&args(&["band", "drummer", "explode"]))
            .unwrap();
        assert!(stage.drummer_blast());
        assert!(stage.take_celebration());
        assert!(!stage.take_celebration());
        stage.command(&args(&["band", "off"])).unwrap();
        assert!(!stage.drummer_blast());
    }
}
