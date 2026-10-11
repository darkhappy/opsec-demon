//! In-memory operating-system theater; never invokes WSL, serial, or a network.
use std::collections::BTreeSet;

pub const MAINFRAME: &str = "OPSECULAR97::LARP-CITY::MAINFRAME";

#[derive(Clone, Default)]
pub struct Desktop {
    booted: bool,
    bin: BTreeSet<String>,
    operator: String,
    spirit: u32,
    poutine: usize,
    explosions: u64,
}

impl Desktop {
    #[cfg(windows)]
    pub fn status(&self) -> String {
        self.clone().command(&[]).expect("status is infallible")
    }

    pub fn command(&mut self, args: &[String]) -> Result<String, String> {
        let words: Vec<_> = args.iter().map(String::as_str).collect();
        match words.as_slice() {
            [] | ["status"] | ["desktop"] => Ok(format!(
                "[SIM] OpSecULAR 97 / {}\n+-- Operator interface ----------------+\n| [Terminal] [Recycle Bin] [WSL 中文]  |\n| [Lost Signals] [Larp City] [Poutine] |\n+-------------------------------------+\noperator={} / spirit={} / recycle={} / poutine stage={} / explosions={}\nmainframe={MAINFRAME}\nopsecular boot | terminal | recycle ls|put NAME|restore NAME|empty | wsl | operator krillion|mo-krill | spirit N | support | poutine next|reset | weep N",
                if self.booted { "ONLINE" } else { "OFFLINE: use opsecular boot" },
                if self.operator.is_empty() { "mo-krill" } else { &self.operator },
                self.spirit, self.bin.len(), self.poutine, self.explosions
            )),
            ["boot"] => {
                self.booted = true;
                Ok(format!("[SIM] OpSecULAR 97 booted / {MAINFRAME}\n[SIM] Recycle Bin mounted in memory; WSL 中文 prop ready.\n[SIM] Larp City beacon ready; all events print to the console."))
            }
            _ if !self.booted => Err("OpSecULAR 97 is offline: opsecular boot".into()),
            ["terminal"] => Ok(format!("[SIM] Terminal prop / {MAINFRAME}\n[SIM] Tama-soft-inspired Lost Signals: use novel start, novel next, novel choose 1|2.\n[SIM] SERIAL BUS LOOPBACK -> ROBLOXIAN TERMINAL 🔥🔥🔥\n[SIM] Larp City connection acknowledged locally. No serial or network traffic.")),
            ["wsl"] => Ok("[SIM] WSL 中文 / 欢迎操作员 / 主机已连接 / 回收站就绪\n[SIM] Chinese console prop; real WSL is launched separately through shell.".into()),
            ["recycle", "ls"] => Ok(format!("[SIM] Recycle Bin: {}", self.bin.iter().cloned().collect::<Vec<_>>().join(", "))),
            ["recycle", "put", name] if !name.is_empty() && name.len() <= 80 && !name.chars().any(char::is_control) => {
                if self.bin.len() >= 32 { return Err("Recycle Bin is full (32 props)".into()); }
                self.bin.insert((*name).into());
                Ok(format!("[SIM] Recycled prop {name}; no host file changed."))
            }
            ["recycle", "restore", name] => {
                if !self.bin.remove(*name) { return Err("No such recycled prop".into()); }
                Ok(format!("[SIM] Restored prop {name}"))
            }
            ["recycle", "empty"] => {
                self.bin.clear();
                Ok("[SIM] Recycle Bin emptied; no host files changed.".into())
            }
            ["operator", name @ ("krillion" | "mo-krill")] => {
                self.operator = (*name).into();
                self.support()
            }
            ["spirit", value] => {
                self.spirit = value.parse::<u32>().ok().filter(|n| *n <= 10000)
                    .ok_or("spirit must be 0..10000")?;
                self.support()
            }
            ["support"] => self.support(),
            ["poutine", "reset"] => {
                self.poutine = 0;
                Ok("[SIM] Québécois kitchen reset; ingredients restocked.".into())
            }
            ["poutine", "next"] => {
                let steps = [
                    "Wash and cut potatoes; rinse and dry the fries.",
                    "Fry potatoes at 160 C, drain, then crisp at 190 C in the fictional fryer.",
                    "Whisk butter and flour into a roux; add stock and simmer the gravy.",
                    "Add fresh squeaky cheese curds to hot fries; ladle gravy over them.",
                    "Serve the poutine! Bon appétit, opérateur. 🔥",
                ];
                let Some(step) = steps.get(self.poutine) else { return Err("Poutine served: use poutine reset for another batch".into()); };
                self.poutine += 1;
                Ok(format!("[SIM] Poutine {}/5: {step}", self.poutine))
            }
            ["weep", count] => {
                let count = count.parse::<u64>().ok().filter(|n| (1..=16).contains(n))
                    .ok_or("weep requires 1..16 explosions")?;
                let mut output = String::new();
                for _ in 0..count {
                    self.explosions = self.explosions.saturating_add(1);
                    output.push_str(&format!("[SIM] Terminal weeps: ;_; -> KABOOM 🔥 #{} -> mainframe respawned\n", self.explosions));
                }
                Ok(output)
            }
            _ => Err("Unknown OpSecULAR command: opsecular desktop for usage".into()),
        }
    }

    fn support(&self) -> Result<String, String> {
        let krillion = self.operator == "krillion";
        let (lane, base, scale) = if krillion {
            (97, 120, 12)
        } else {
            (72, 100, 3)
        };
        Ok(format!(
            "[SIM / INVENTED BALANCE] {} / support\nlane pressure={lane} / heal={} / spirit={} / scaling={scale}x\nKrillion counterpicks Mo & Krill in this fictional lane: peel, spirit shields, absurd sustain. These are not live game stats.",
            if krillion { "KRILLION" } else { "MO & KRILL" },
            base + self.spirit * scale,
            self.spirit
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(d: &mut Desktop, args: &[&str]) -> Result<String, String> {
        d.command(&args.iter().map(|s| (*s).into()).collect::<Vec<_>>())
    }
    #[test]
    fn desktop_lifecycle_and_bounded_effects() {
        let mut d = Desktop::default();
        assert!(run(&mut d, &["terminal"]).is_err());
        run(&mut d, &["boot"]).unwrap();
        run(&mut d, &["recycle", "put", "novel.sav"]).unwrap();
        assert!(
            run(&mut d, &["recycle", "ls"])
                .unwrap()
                .contains("novel.sav")
        );
        run(&mut d, &["recycle", "restore", "novel.sav"]).unwrap();
        assert!(d.bin.is_empty());
        assert!(run(&mut d, &["weep", "17"]).is_err());
        assert_eq!(
            run(&mut d, &["weep", "3"])
                .unwrap()
                .matches("KABOOM")
                .count(),
            3
        );
        for _ in 0..5 {
            run(&mut d, &["poutine", "next"]).unwrap();
        }
        assert!(run(&mut d, &["poutine", "next"]).is_err());
        run(&mut d, &["operator", "krillion"]).unwrap();
        assert!(
            run(&mut d, &["spirit", "10000"])
                .unwrap()
                .contains("heal=120120")
        );
        assert!(run(&mut d, &["spirit", "10001"]).is_err());
    }
}
