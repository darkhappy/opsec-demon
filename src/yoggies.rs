//! Fictional, session-only yogurt berry bites for the mainframe command deck.

const COUNT: usize = 5;
const USAGE: &str = "SIM usage: yoggies [show|all|status|use [1..5]|refill]";

pub struct Yoggies {
    ready: [bool; COUNT],
}

impl Yoggies {
    pub fn new() -> Self {
        Self {
            ready: [true; COUNT],
        }
    }

    pub fn status(&self) -> String {
        let tray: String = self
            .ready
            .iter()
            .map(|ready| if *ready { 'O' } else { '.' })
            .collect();
        format!(
            "SIM YOGGIES [{tray}] {}/{COUNT} ready",
            self.ready.iter().filter(|ready| **ready).count()
        )
    }

    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
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
            "use" if args.len() <= 2 => {
                let index = if let Some(value) = args.get(1) {
                    value
                        .parse::<usize>()
                        .ok()
                        .filter(|value| (1..=COUNT).contains(value))
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
                    "SIM all five yoggies restored / memory only".into(),
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
    fn every_slot_remains_visible_after_use_and_refill() {
        let mut tray = Yoggies::new();
        tray.command(&args(&["use", "3"])).unwrap();
        let display = tray.command(&[]).unwrap();
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
        tray.command(&args(&["refill"])).unwrap();
        assert_eq!(tray.ready, [true; COUNT]);
    }

    #[test]
    fn use_selects_next_ready_bite_and_empty_tray_is_recoverable() {
        let mut tray = Yoggies::new();
        for index in 1..=COUNT {
            let output = tray.command(&args(&["use"])).unwrap();
            assert!(output[0].contains(&format!("YOGGIE {index} / NOM")));
        }
        assert!(tray.command(&args(&["use"])).is_err());
        assert!(tray.status().contains("0/5 ready"));
        tray.command(&args(&["refill"])).unwrap();
        assert!(tray.command(&args(&["use"])).is_ok());
    }

    #[test]
    fn invalid_arguments_and_reusing_a_bite_do_not_mutate_tray() {
        let mut tray = Yoggies::new();
        tray.command(&args(&["use", "2"])).unwrap();
        let before = tray.ready;
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
            assert!(tray.command(&args(&command)).is_err(), "{command:?}");
            assert_eq!(tray.ready, before);
        }
    }
}
