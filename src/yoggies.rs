//! Fictional, session-only yogurt berry bites for the mainframe command deck.

const COUNT: usize = 5;
const CHRISTMAS_COUNT: usize = 10;
const USAGE: &str = "SIM usage: yoggies [show|all|holidays|status|use [NUMBER]|refill] (choose a slot shown in yoggies all)";

fn local_date() -> Option<String> {
    #[cfg(windows)]
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-Date).ToString('yyyy-MM-dd')",
        ])
        .output()
        .ok()?;
    #[cfg(not(windows))]
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let date = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    if date.len() != 10
        || date.as_bytes()[4] != b'-'
        || date.as_bytes()[7] != b'-'
        || !date
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return None;
    }
    Some(date)
}

pub struct Yoggies {
    ready: Vec<bool>,
}

impl Yoggies {
    pub fn new() -> Self {
        let mut tray = Self {
            ready: vec![true; COUNT],
        };
        tray.refresh(local_date().as_deref());
        tray
    }

    fn refresh(&mut self, date: Option<&str>) {
        // A missing clock reading preserves the existing tray; no guessed holiday.
        if let Some(date) = date {
            let count = if date.ends_with("-12-25") {
                CHRISTMAS_COUNT
            } else {
                COUNT
            };
            self.ready.resize(count, true);
        }
    }

    pub fn is_christmas(&self) -> bool {
        self.ready.len() == CHRISTMAS_COUNT
    }

    pub fn status(&self) -> String {
        let tray: String = self
            .ready
            .iter()
            .map(|ready| if *ready { 'O' } else { '.' })
            .collect();
        format!(
            "SIM YOGGIES [{tray}] {}/{} ready{}",
            self.ready.iter().filter(|ready| **ready).count(),
            self.ready.len(),
            if self.ready.len() == CHRISTMAS_COUNT {
                " / CHRISTMAS BONUS"
            } else {
                ""
            }
        )
    }

    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        self.refresh(local_date().as_deref());
        self.execute(args)
    }

    fn execute(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args.first().map(String::as_str).unwrap_or("show") {
            "show" | "all" if args.len() <= 1 => {
                let mut lines = vec![
                    self.status(),
                    "SIM purple sparkle tray / yogurt-coated berry bites / ASCII illustration"
                        .into(),
                ];
                // Keep every slot visible, including yoggies already used.
                for (index, ready) in self.ready.iter().enumerate() {
                    lines.push(if *ready {
                        format!(
                            "SIM * . (o*o) . *  YOGGIE {} / creamy shell, berry core / READY",
                            index + 1
                        )
                    } else {
                        format!(
                            "SIM * . (   ) . *  YOGGIE {} / USED / refill to return",
                            index + 1
                        )
                    });
                }
                Ok(lines)
            }
            "status" if args.len() == 1 => Ok(vec![self.status()]),
            "holidays" if args.len() == 1 => Ok(vec![
                "SIM MAINFRAME / 2026 CHRISTMAS AND NEW YEAR HOLIDAY DATES".into(),
                "SIM Christmas Eve   / December 24, 2026".into(),
                "SIM Christmas Day   / December 25, 2026 / ten yoggies!".into(),
                "SIM New Year's Eve  / December 31, 2026".into(),
                "SIM New Year's Day  / January 1, 2027".into(),
                "SIM Christmas bonus every December 25 / host local date checked at startup and on yoggies commands".into(),
            ]),
            "use" if args.len() <= 2 => {
                let index = if let Some(value) = args.get(1) {
                    value
                        .parse::<usize>()
                        .ok()
                        .filter(|value| (1..=self.ready.len()).contains(value))
                        .map(|value| value - 1)
                        .ok_or_else(|| USAGE.to_string())?
                } else {
                    self.ready
                        .iter()
                        .position(|ready| *ready)
                        .ok_or("SIM yoggies tray empty; run yoggies refill")?
                };
                if !self.ready[index] {
                    return Err(format!(
                        "SIM yoggie {} already used; choose another or run yoggies refill",
                        index + 1
                    ));
                }
                self.ready[index] = false;
                Ok(vec![
                    format!(
                        "SIM YOGGIE {} / NOM / berry sparkle delivered to the fictional mainframe",
                        index + 1
                    ),
                    self.status(),
                ])
            }
            "refill" if args.len() == 1 => {
                self.ready.fill(true);
                Ok(vec![
                    if self.ready.len() == CHRISTMAS_COUNT {
                        "SIM all ten Christmas yoggies restored / memory only".into()
                    } else {
                        "SIM all five yoggies restored / memory only".into()
                    },
                    self.status(),
                ])
            }
            _ => Err(USAGE.into()),
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
    fn christmas_doubles_tray_without_restoring_used_bites_until_refill() {
        let mut tray = Yoggies {
            ready: vec![true; COUNT],
        };
        tray.refresh(Some("2026-12-24"));
        assert!(!tray.is_christmas());
        tray.execute(&args(&["use", "1"])).unwrap();
        tray.refresh(Some("2026-12-25"));
        assert!(tray.is_christmas());
        assert_eq!(tray.ready.len(), CHRISTMAS_COUNT);
        assert!(!tray.ready[0]);
        assert!(tray.status().contains("9/10 ready / CHRISTMAS BONUS"));
        tray.execute(&args(&["use", "10"])).unwrap();
        tray.refresh(Some("2026-12-25"));
        assert!(!tray.ready[9]);
        let display = tray.execute(&args(&["all"])).unwrap();
        assert_eq!(
            display
                .iter()
                .filter(|line| line.contains("YOGGIE "))
                .count(),
            CHRISTMAS_COUNT
        );
        assert!(tray.execute(&args(&["use", "11"])).is_err());
        let restored = tray.execute(&args(&["refill"])).unwrap();
        assert!(restored[0].contains("all ten Christmas yoggies restored"));
        assert!(tray.ready.iter().all(|ready| *ready));
        tray.refresh(None);
        assert_eq!(tray.ready.len(), CHRISTMAS_COUNT);
        for date in ["2026-12-26", "2026-12-31", "2027-01-01"] {
            tray.refresh(Some(date));
            assert!(!tray.is_christmas());
            assert_eq!(tray.ready.len(), COUNT);
            assert!(tray.execute(&args(&["use", "10"])).is_err());
        }
    }

    #[test]
    fn holiday_schedule_crosses_the_year_and_does_not_consume_bites() {
        let mut tray = Yoggies {
            ready: vec![true; COUNT],
        };
        let output = tray.execute(&args(&["holidays"])).unwrap();
        for date in [
            "December 24, 2026",
            "December 25, 2026",
            "December 31, 2026",
            "January 1, 2027",
        ] {
            assert!(output.iter().any(|line| line.contains(date)));
        }
        assert!(output.iter().all(|line| line.starts_with("SIM")));
        assert_eq!(tray.ready, vec![true; COUNT]);
        assert!(tray.execute(&args(&["holidays", "extra"])).is_err());
    }

    #[test]
    fn every_slot_remains_visible_after_use_and_refill() {
        let mut tray = Yoggies {
            ready: vec![true; COUNT],
        };
        tray.execute(&args(&["use", "3"])).unwrap();
        let display = tray.execute(&[]).unwrap();
        assert_eq!(
            display
                .iter()
                .filter(|line| line.contains("YOGGIE "))
                .count(),
            COUNT
        );
        assert!(display.iter().any(|line| line.contains("YOGGIE 3 / USED")));
        assert!(display.iter().all(|line| line.starts_with("SIM")));
        assert!(tray.status().contains("[OO.OO] 4/5"));
        tray.execute(&args(&["refill"])).unwrap();
        assert_eq!(tray.ready, [true; COUNT]);
    }

    #[test]
    fn use_selects_next_ready_bite_and_empty_tray_is_recoverable() {
        let mut tray = Yoggies {
            ready: vec![true; COUNT],
        };
        for index in 1..=COUNT {
            let output = tray.execute(&args(&["use"])).unwrap();
            assert!(output[0].contains(&format!("YOGGIE {index} / NOM")));
        }
        assert!(tray.execute(&args(&["use"])).is_err());
        assert!(tray.status().contains("0/5 ready"));
        tray.execute(&args(&["refill"])).unwrap();
        assert!(tray.execute(&args(&["use"])).is_ok());
    }

    #[test]
    fn invalid_arguments_and_reusing_a_bite_do_not_mutate_tray() {
        let mut tray = Yoggies {
            ready: vec![true; COUNT],
        };
        tray.execute(&args(&["use", "2"])).unwrap();
        let before = tray.ready.clone();
        for command in [
            vec!["use", "0"],
            vec!["use", "6"],
            vec!["use", "-1"],
            vec!["use", "berry"],
            vec!["use", "2"],
            vec!["use", "1", "extra"],
            vec!["refill", "extra"],
            vec!["all", "extra"],
            vec!["status", "extra"],
            vec!["unknown"],
        ] {
            assert!(tray.execute(&args(&command)).is_err(), "{command:?}");
            assert_eq!(tray.ready, before);
        }
    }
}
