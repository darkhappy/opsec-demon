//! An entirely fictional SSD disaster prop. Never inspects hardware or files.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BLAST_DURATION: Duration = Duration::from_secs(3);

pub struct Ssd {
    started: Instant,
    blast_started: Option<Instant>,
    rolls: u64,
    detonations: u64,
    seed: u64,
}

impl Ssd {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            blast_started: None,
            rolls: 0,
            detonations: 0,
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0x0480_5a7a, |time| time.as_nanos() as u64),
        }
    }

    fn risk_basis_points(&self) -> u64 {
        // Oscillating theater odds, with no relation to any product's reliability.
        let phase = self.started.elapsed().as_secs() % 24;
        400 + phase.min(24 - phase) * 50
    }

    pub fn line(&self) -> String {
        format!(
            "SIM SSD 480GB / {:.1}% per roll / FICTION{}",
            self.risk_basis_points() as f64 / 100.0,
            if self.blast_started.is_some() {
                " / BOOM"
            } else {
                ""
            }
        )
    }

    pub fn status(&self) -> String {
        let phase = if self.blast_started.is_some() {
            "ASCII DETONATION / dancing debris"
        } else {
            "ARMED / imaginary eBay delivery"
        };
        let odds = self.risk_basis_points() as f64 / 100.0;
        let temperature = 30 + self.started.elapsed().as_secs() % 19;
        format!(
            "SIM SSD | IBM 1.8-inch 480GB SATA / eBay prop | {phase}\nSIM FICTIONAL ODDS {odds:.2}% per roll / invented, not a reliability estimate\nSIM temp {temperature}C / {} rolls / {} cartoon blasts / no hardware access",
            self.rolls, self.detonations
        )
    }

    /// Arguments exclude the `ssd` command prefix.
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args {
            [] => Ok(vec![self.status()]),
            [action] if action == "status" => Ok(vec![self.status()]),
            [action] if action == "roll" => {
                self.seed ^= self.seed << 13;
                self.seed ^= self.seed >> 7;
                self.seed ^= self.seed << 17;
                self.roll(self.seed % 10_000)
            }
            [action] if action == "explode" => Ok(vec![self.detonate()]),
            _ => Err("SIM | usage: ssd status | roll | explode".into()),
        }
    }

    fn roll(&mut self, sample: u64) -> Result<Vec<String>, String> {
        if self.blast_started.is_some() {
            return Err("SIM | cartoon blast active; let the imaginary debris settle".into());
        }
        self.rolls = self.rolls.saturating_add(1);
        let threshold = self.risk_basis_points();
        let result = if sample < threshold {
            self.detonate()
        } else {
            "SIM SSD | SURVIVED / sticker shield held / no real device was tested".into()
        };
        Ok(vec![
            format!(
                "SIM FICTIONAL SSD ROLL #{} / invented odds {:.2}% / not product reliability",
                self.rolls,
                threshold as f64 / 100.0
            ),
            result,
        ])
    }

    fn detonate(&mut self) -> String {
        self.blast_started = Some(Instant::now());
        self.detonations = self.detonations.saturating_add(1);
        "SIM SSD DETONATION / dancing ASCII fireworks / fictional IBM eBay prop; no hardware or files touched".into()
    }

    pub fn tick(&mut self) -> Option<String> {
        if self
            .blast_started
            .is_some_and(|started| started.elapsed() >= BLAST_DURATION)
        {
            self.blast_started = None;
            Some(
                "SIM SSD | imaginary debris settled / replacement spawned / cartoon bank rearmed"
                    .into(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_statuses_mark_the_odds_as_fictional() {
        let mut ssd = Ssd::new();
        assert!(ssd.status().contains("not a reliability estimate"));
        ssd.detonate();
        assert!(ssd.status().contains("ASCII DETONATION"));
        assert!(ssd.status().contains("no hardware access"));
    }

    #[test]
    fn roll_threshold_and_active_blast_are_enforced() {
        let mut ssd = Ssd::new();
        assert!(ssd.roll(9_999).unwrap()[1].contains("SURVIVED"));
        assert_eq!(ssd.rolls, 1);
        assert!(ssd.roll(0).unwrap()[1].contains("DETONATION"));
        assert_eq!(ssd.rolls, 2);
        assert_eq!(ssd.detonations, 1);
        assert!(ssd.roll(0).is_err());
        assert_eq!(ssd.rolls, 2);
    }

    #[test]
    fn settling_emits_once_and_does_not_touch_rolls() {
        let mut ssd = Ssd::new();
        ssd.detonate();
        assert!(ssd.tick().is_none());
        ssd.blast_started = Some(Instant::now() - BLAST_DURATION);
        assert!(ssd.tick().unwrap().contains("rearmed"));
        assert!(ssd.tick().is_none());
        assert_eq!(ssd.rolls, 0);
        assert!(!ssd.status().contains("ASCII DETONATION"));
    }

    #[test]
    fn command_arity_is_bounded_and_invalid_input_does_not_mutate() {
        let mut ssd = Ssd::new();
        assert!(ssd.command(&[]).is_ok());
        assert!(ssd.command(&["explode".into(), "disk".into()]).is_err());
        assert!(ssd.command(&["erase".into()]).is_err());
        assert_eq!(ssd.detonations, 0);
    }
}
