//! Fictional memory bookkeeping for the hacker dashboard, never real allocation.
pub const VULCAN_ART: &[&str] = &[
    "+--------------------------+",
    "| TEAMGROUP                |",
    r"| T-FORCE VULCAN Z   /\/\/ |",
    "+==========================+",
    " |||||||||||||||||||||||||| ",
    "       SIM MEMORY BANK      ",
];

pub const PROSTREET_ART: &[&str] = &[
    "   NEED FOR SPEED",
    "      PROSTREET",
    "         ____",
    "    ____/ __ \\___",
    "   /___/_||_\\____\\",
    "  =--(o)------(o)--=",
    "      SIM RACE FEED",
];

const CAPACITY_GB: u16 = 512;

pub struct Expansion {
    enabled: bool,
    used_gb: u16,
}

impl Expansion {
    pub fn new() -> Self {
        Self {
            enabled: false,
            used_gb: 0,
        }
    }

    pub fn status(&self) -> String {
        if self.enabled {
            format!(
                "SIM MCP EXPANSION | {} / {CAPACITY_GB} GB virtual | {} GB free",
                self.used_gb,
                CAPACITY_GB - self.used_gb
            )
        } else {
            "SIM MCP EXPANSION | offline | 512 GB virtual bank available".into()
        }
    }

    /// Receives subcommand arguments, excluding the dashboard command name.
    pub fn command(&mut self, arguments: &[String]) -> Result<Vec<String>, String> {
        match arguments {
            [] => Ok(vec![self.status()]),
            [action] if action == "status" => Ok(vec![self.status()]),
            [action] if action == "expand" => {
                self.enabled = true;
                Ok(vec![
                    "SIM | T-FORCE VULCAN Z expansion sequence complete".into(),
                    "SIM | 512 GB virtual capacity; no real memory or MCP changes".into(),
                    self.status(),
                ])
            }
            [action, quantity] if action == "allocate" || action == "release" => {
                if !self.enabled {
                    return Err("SIM | bank offline; run expansion expand first".into());
                }
                let amount = parse_amount(quantity)?;
                if action == "allocate" {
                    if amount > CAPACITY_GB - self.used_gb {
                        return Err(format!(
                            "SIM | allocation exceeds remaining {} GB virtual capacity",
                            CAPACITY_GB - self.used_gb
                        ));
                    }
                    self.used_gb += amount;
                } else {
                    if amount > self.used_gb {
                        return Err(format!(
                            "SIM | cannot release {amount} GB; only {} GB virtually allocated",
                            self.used_gb
                        ));
                    }
                    self.used_gb -= amount;
                }
                Ok(vec![self.status()])
            }
            _ => Err("SIM | usage: expansion status | expand | allocate GB | release GB".into()),
        }
    }
}

fn parse_amount(text: &str) -> Result<u16, String> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("SIM | GB must be a whole number from 1 to 512".into());
    }
    let amount = text
        .parse::<u16>()
        .ok()
        .filter(|amount| (1..=CAPACITY_GB).contains(amount))
        .ok_or_else(|| "SIM | GB must be a whole number from 1 to 512".to_string())?;
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).into()).collect()
    }

    #[test]
    fn bank_is_opt_in_and_all_outputs_identify_simulation() {
        let mut bank = Expansion::new();
        assert!(bank.status().contains("offline"));
        assert!(bank.command(&args(&["allocate", "1"])).is_err());
        let messages = bank.command(&args(&["expand"])).unwrap();
        assert!(messages.iter().all(|line| line.starts_with("SIM")));
        assert!(bank.status().contains("0 / 512"));
    }

    #[test]
    fn full_bank_and_release_enforce_bookkeeping_limits() {
        let mut bank = Expansion::new();
        bank.command(&args(&["expand"])).unwrap();
        bank.command(&args(&["allocate", "512"])).unwrap();
        assert!(bank.command(&args(&["allocate", "1"])).is_err());
        bank.command(&args(&["release", "200"])).unwrap();
        assert!(bank.status().contains("312 / 512"));
        assert!(bank.command(&args(&["release", "313"])).is_err());
        assert!(bank.status().contains("312 / 512"));
        bank.command(&args(&["expand"])).unwrap();
        assert!(bank.status().contains("312 / 512"));
        bank.command(&args(&["release", "312"])).unwrap();
        assert!(bank.status().contains("0 / 512"));
    }

    #[test]
    fn invalid_quantities_and_arity_do_not_mutate_bank() {
        let mut bank = Expansion::new();
        bank.command(&args(&["expand"])).unwrap();
        for value in ["0", "513", "65536", "-1", "+1", "1.5", "", "abc"] {
            assert!(bank.command(&args(&["allocate", value])).is_err());
        }
        assert!(bank.command(&args(&["allocate"])).is_err());
        assert!(bank.command(&args(&["expand", "512"])).is_err());
        assert!(bank.status().contains("0 / 512"));
    }

    #[test]
    fn racing_art_fits_small_panel_and_is_ascii() {
        assert!(PROSTREET_ART.len() <= 8);
        assert!(
            PROSTREET_ART
                .iter()
                .all(|line| line.is_ascii() && line.len() <= 24)
        );
        assert!(VULCAN_ART.iter().all(|line| line.is_ascii()));
    }
}
