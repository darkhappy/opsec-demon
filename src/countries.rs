//! Local strategy-game flavor; no game integration or historical rules engine.
use std::time::{Duration, Instant};

const FOCUS_SECONDS: u64 = 20;
const COUNTRIES: &[(&str, &str, &str)] = &[
    ("GER", "Germany", "INDUSTRIAL CIRCUIT"),
    ("SOV", "USSR", "SNOWBOUND SIGNAL"),
    ("USA", "USA", "ARSENAL OF NEON"),
    ("ENG", "UK", "ISLAND FREQUENCY"),
    ("FRA", "France", "MIDNIGHT RELAY"),
    ("ITA", "Italy", "ALPINE AMPLIFIER"),
    ("JAP", "Japan", "PACIFIC PULSE"),
    ("POL", "Poland", "AMBER UPLINK"),
    ("CHI", "China", "JADE MAINFRAME"),
    ("PHI", "Philippines", "ARCHIPELAGO OVERDRIVE"),
];

pub struct Countries {
    selected: usize,
    focus: Option<(String, Instant)>,
    completed: u64,
}

impl Default for Countries {
    fn default() -> Self {
        Self::new()
    }
}

impl Countries {
    pub fn new() -> Self {
        Self {
            selected: 9,
            focus: None,
            completed: 0,
        }
    }

    pub fn line(&self) -> String {
        let (code, _, _) = COUNTRIES[self.selected];
        let progress = self.focus.as_ref().map_or("idle".into(), |(_, start)| {
            format!("focus {:3}%", progress(start.elapsed()))
        });
        format!("SIM HOI4 {code} | {progress} | {} complete", self.completed)
    }

    pub fn status(&self) -> String {
        let (code, name, doctrine) = COUNTRIES[self.selected];
        let focus = self.focus.as_ref().map_or("none".into(), |(name, start)| {
            format!("{name} / {}%", progress(start.elapsed()))
        });
        format!(
            "+--- SIM COUNTRY CARD ---+\n{code} | {name}\n{doctrine}\nFocus: {focus}\nCompleted: {}\nFictional 20-second focus timer; no real HOI4 rules or game effects.",
            self.completed
        )
    }

    /// Arguments exclude the `hoi4` command prefix.
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args {
            [] => Ok(vec![self.status()]),
            [action] if action == "status" => Ok(vec![self.status()]),
            [action] if action == "list" => Ok(COUNTRIES
                .iter()
                .map(|(code, name, _)| format!("SIM | {code} {name}"))
                .collect()),
            [action, code] if action == "select" => {
                let code = code.to_ascii_uppercase();
                let code = match code.as_str() {
                    "UK" => "ENG",
                    "USSR" => "SOV",
                    other => other,
                };
                let selected = COUNTRIES
                    .iter()
                    .position(|(candidate, _, _)| *candidate == code)
                    .ok_or("SIM | unknown country code; use hoi4 list")?;
                self.selected = selected;
                self.focus = None;
                self.completed = 0;
                Ok(vec![self.status()])
            }
            [action, rest @ ..] if action == "focus" => {
                let name = if rest.is_empty() {
                    COUNTRIES[self.selected].2.to_string()
                } else {
                    rest.join(" ")
                };
                if name.len() > 64
                    || name.is_empty()
                    || !name
                        .bytes()
                        .all(|byte| byte.is_ascii_graphic() || byte == b' ')
                {
                    return Err("SIM | focus name must be 1..64 printable ASCII characters".into());
                }
                self.focus = Some((name, Instant::now()));
                Ok(vec![self.status()])
            }
            _ => Err("SIM | usage: hoi4 list | select CODE | focus [NAME] | status".into()),
        }
    }

    pub fn tick(&mut self) -> Option<String> {
        self.tick_at(Instant::now())
    }

    fn tick_at(&mut self, now: Instant) -> Option<String> {
        let (_, started) = self.focus.as_ref()?;
        if now.saturating_duration_since(*started) < Duration::from_secs(FOCUS_SECONDS) {
            return None;
        }
        let (name, _) = self.focus.take()?;
        self.completed = self.completed.saturating_add(1);
        Some(format!(
            "SIM HOI4 | {} focus COMPLETE: {name} | imaginary doctrine unlocked",
            COUNTRIES[self.selected].0
        ))
    }
}

fn progress(elapsed: Duration) -> u64 {
    (elapsed.as_millis().saturating_mul(100) / (u128::from(FOCUS_SECONDS) * 1000)).min(100) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn all_countries_available_and_bad_selection_preserves_state() {
        let mut countries = Countries::new();
        assert_eq!(countries.command(&args(&["list"])).unwrap().len(), 10);
        countries.command(&args(&["select", "GER"])).unwrap();
        countries
            .command(&args(&["focus", "NEON", "INDUSTRY"]))
            .unwrap();
        assert!(countries.command(&args(&["select", "BAD"])).is_err());
        assert_eq!(countries.selected, 0);
        assert_eq!(countries.focus.as_ref().unwrap().0, "NEON INDUSTRY");
        countries.command(&args(&["select", "UK"])).unwrap();
        assert_eq!(countries.selected, 3);
        assert!(countries.focus.is_none());
    }

    #[test]
    fn synthetic_clock_completes_once_without_waiting() {
        let mut countries = Countries::new();
        countries.command(&args(&["focus"])).unwrap();
        let start = countries.focus.as_ref().unwrap().1;
        assert_eq!(progress(Duration::from_secs(10)), 50);
        assert!(countries.tick_at(start + Duration::from_secs(19)).is_none());
        assert!(
            countries
                .tick_at(start + Duration::from_secs(20))
                .unwrap()
                .contains("COMPLETE")
        );
        assert_eq!(countries.completed, 1);
        assert!(
            countries
                .tick_at(start + Duration::from_secs(100))
                .is_none()
        );
        assert!(countries.tick().is_none());
        assert!(countries.line().contains("idle"));
    }

    #[test]
    fn names_are_bounded_ascii_and_failure_preserves_active_focus() {
        let mut countries = Countries::new();
        countries.command(&args(&["focus", "SAFE"])).unwrap();
        for name in [
            "x".repeat(65),
            "猫".into(),
            "line\nbreak".into(),
            String::new(),
        ] {
            assert!(countries.command(&["focus".into(), name]).is_err());
            assert_eq!(countries.focus.as_ref().unwrap().0, "SAFE");
        }
        assert!(countries.status().contains("no real HOI4 rules"));
    }
}
