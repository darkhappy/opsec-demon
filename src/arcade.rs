//! Local rhythm controls, a paper coin ledger, and a fictional car gaming rig.
//! These commands do not contact exchanges, games, hardware, or storage services.
const SCORE_CAP: u64 = 999_999_999;
const COMBO_CAP: u32 = 1_000_000;
const PRICE_CAP_CENTS: u64 = 100_000_000;
const VIRTUAL_SLOT_GB: u64 = 512;

pub struct Arcade {
    left: u8,
    right: u8,
    combo: u32,
    score: u64,
    celebration: bool,
    balance_cents: u64,
    coins: u64,
    price_cents: u64,
}

impl Arcade {
    pub fn new() -> Self {
        Self {
            left: 50,
            right: 50,
            combo: 0,
            score: 0,
            celebration: false,
            balance_cents: 100_000,
            coins: 0,
            price_cents: 100,
        }
    }

    pub fn knobs(&self) -> (u8, u8) {
        (self.left, self.right)
    }

    pub fn combo(&self) -> u32 {
        self.combo
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn take_celebration(&mut self) -> bool {
        std::mem::take(&mut self.celebration)
    }

    pub fn status(&self) -> String {
        format!(
            "SIM ARCADE | knobs {:03}/{:03} | combo {} | score {} | {} paper coins",
            self.left, self.right, self.combo, self.score, self.coins
        )
    }

    /// Arguments include the namespace: `sdvx`, `coins`, or `focus`.
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args {
            [namespace] if namespace == "sdvx" => Ok(self.rhythm_status()),
            [namespace, action] if namespace == "sdvx" && action == "status" => {
                Ok(self.rhythm_status())
            }
            [namespace, action, side, delta] if namespace == "sdvx" && action == "knob" => {
                if !matches!(side.as_str(), "left" | "right") {
                    return Err("SIM | knob side must be left or right".into());
                }
                let delta = delta
                    .parse::<i16>()
                    .map_err(|_| "SIM | knob delta must be a signed whole number from -32768 to 32767")?;
                let knob = if side == "left" { &mut self.left } else { &mut self.right };
                *knob = (i32::from(*knob) + i32::from(delta)).clamp(0, 100) as u8;
                Ok(vec![format!("SIM SDVX | {side} laser knob {}% | phantom lane synchronized", *knob)])
            }
            [namespace, action, button] if namespace == "sdvx" && action == "hit" => {
                if !matches!(button.as_str(), "bt-a" | "bt-b" | "bt-c" | "bt-d" | "fx-l" | "fx-r") {
                    return Err("SIM | hit expects bt-a, bt-b, bt-c, bt-d, fx-l, or fx-r".into());
                }
                self.combo = self.combo.saturating_add(1).min(COMBO_CAP);
                self.score = self.score.saturating_add(1000).min(SCORE_CAP);
                self.celebration = true;
                Ok(vec![format!("SIM SDVX | {button} CRITICAL | combo {} | score {} | dancing explosion queued", self.combo, self.score)])
            }
            [namespace] if namespace == "coins" => Ok(self.coin_status()),
            [namespace, action] if namespace == "coins" && action == "generate" => {
                let mut lines = vec!["SIM $1000 GENERATOR | paper fixture funded once at startup; replay shows the same ledger, never creates money".into()];
                lines.extend(self.coin_status());
                self.celebration = true;
                Ok(lines)
            }
            [namespace, action] if namespace == "coins" && action == "status" => {
                Ok(self.coin_status())
            }
            [namespace, action] if namespace == "coins" && action == "mint" => {
                if self.balance_cents < self.price_cents {
                    return Err("SIM COINS | insufficient paper dollars; nothing was purchased".into());
                }
                self.balance_cents -= self.price_cents;
                self.coins += 1; // Maximum 1000 coins: fixed $1000 balance, price >= $1.
                self.celebration = true;
                let mut lines = vec!["SIM COINS | minted 1 $POOPSECDEMONLARP in the imaginary porcelain ledger".into()];
                lines.extend(self.coin_status());
                Ok(lines)
            }
            [namespace, action] if namespace == "coins" && action == "pump" => {
                if self.price_cents == PRICE_CAP_CENTS {
                    return Err("SIM COINS | cartoon price cap reached; ledger unchanged".into());
                }
                self.price_cents = (self.price_cents * 5).div_ceil(4);
                self.price_cents = self.price_cents.min(PRICE_CAP_CENTS);
                self.celebration = true;
                let mut lines = vec!["SIM COINS | fake pump: price increased by about 25%; no exchange or real transaction".into()];
                lines.extend(self.coin_status());
                Ok(lines)
            }
            [namespace] if namespace == "focus" => Ok(Self::focus_status()),
            [namespace, action] if namespace == "focus" && action == "status" => {
                Ok(Self::focus_status())
            }
            [namespace, action] if namespace == "focus" && action == "calculate" => {
                Ok(Self::focus_calculate(20, true))
            }
            [namespace, action, size] if namespace == "focus" && action == "calculate" => {
                if !size.bytes().all(|byte| byte.is_ascii_digit()) || size.is_empty() {
                    return Err("SIM FOCUS | virtual game size must be a whole number from 1 to 1000000 GB".into());
                }
                let size = size.parse::<u64>().ok().filter(|size| (1..=1_000_000).contains(size))
                    .ok_or("SIM FOCUS | virtual game size must be a whole number from 1 to 1000000 GB")?;
                Ok(Self::focus_calculate(size, false))
            }
            _ => Err("SIM | usage: sdvx status | knob left/right DELTA | hit BUTTON; coins status|generate|mint|pump; focus status|calculate [virtualGameGB]".into()),
        }
    }

    fn rhythm_status(&self) -> Vec<String> {
        vec![
            "SIM SDVX | SOUND VOLTEX inspired phantom controller; no game connection or real timing judgment".into(),
            format!("SIM SDVX | left {}% | right {}% | combo {} | score {}", self.left, self.right, self.combo, self.score),
            "SIM SDVX | each valid button press earns a pretend CRITICAL and 1000 points; knobs stay within 0..100".into(),
        ]
    }

    fn coin_status(&self) -> Vec<String> {
        vec![
            format!(
                "SIM COINS | $POOPSECDEMONLARP | paper balance {} | holdings {} | cartoon price {}",
                dollars(self.balance_cents),
                self.coins,
                dollars(self.price_cents)
            ),
            format!(
                "SIM COINS | imaginary holdings value {} | pump.fun-style fictional ledger; no financial API",
                dollars(self.coins * self.price_cents)
            ),
        ]
    }

    fn focus_status() -> Vec<String> {
        vec![
            "SIM FOCUS | Ford Focus ST 2014 / black / Track Attack Pack / RE5 gaming fantasy".into(),
            "SIM FOCUS | imaginary 512 GB M.2 slots; a car name does not establish actual slots, compatibility, or game requirements".into(),
        ]
    }

    fn focus_calculate(size: u64, assumed: bool) -> Vec<String> {
        let mut lines = Self::focus_status();
        lines.push(format!("SIM FOCUS | virtual RE5 installation = {size} GB ({}) | capacity = {VIRTUAL_SLOT_GB} GB per virtual M.2 slot", if assumed { "explicit fantasy assumption, not a verified game requirement" } else { "user-supplied hypothetical size" }));
        lines.push(format!("SIM FOCUS | slots = ceil({size}/{VIRTUAL_SLOT_GB}) = {} | bookkeeping only; no allocation or hardware detection", size.div_ceil(VIRTUAL_SLOT_GB)));
        lines
    }
}

fn dollars(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn knobs_clamp_and_invalid_commands_preserve_state() {
        let mut arcade = Arcade::new();
        arcade
            .command(&args(&["sdvx", "knob", "left", "32767"]))
            .unwrap();
        arcade
            .command(&args(&["sdvx", "knob", "right", "-32768"]))
            .unwrap();
        assert_eq!(arcade.knobs(), (100, 0));
        let before = arcade.status();
        for command in [
            vec!["sdvx", "knob", "left", "oops"],
            vec!["sdvx", "knob", "middle", "5"],
            vec!["sdvx", "hit", "unknown"],
            vec!["sdvx", "hit", "bt-a", "extra"],
        ] {
            assert!(arcade.command(&args(&command)).is_err());
            assert_eq!(arcade.status(), before);
        }
        assert!(!arcade.take_celebration());
    }

    #[test]
    fn rhythm_scores_are_bounded_and_celebration_is_consumed_once() {
        let mut arcade = Arcade::new();
        arcade.combo = COMBO_CAP;
        arcade.score = SCORE_CAP - 10;
        arcade.command(&args(&["sdvx", "hit", "fx-r"])).unwrap();
        assert_eq!(arcade.combo(), COMBO_CAP);
        assert_eq!(arcade.score(), SCORE_CAP);
        assert!(arcade.take_celebration());
        assert!(!arcade.take_celebration());
    }

    #[test]
    fn ledger_conserves_paper_dollars_and_never_overflows() {
        let mut arcade = Arcade::new();
        for _ in 0..1000 {
            arcade.command(&args(&["coins", "mint"])).unwrap();
        }
        assert_eq!(arcade.balance_cents, 0);
        assert_eq!(arcade.coins, 1000);
        assert!(arcade.command(&args(&["coins", "mint"])).is_err());
        while arcade.price_cents < PRICE_CAP_CENTS {
            arcade.command(&args(&["coins", "pump"])).unwrap();
        }
        assert!(arcade.command(&args(&["coins", "pump"])).is_err());
        assert!(
            arcade
                .coin_status()
                .iter()
                .all(|line| line.starts_with("SIM"))
        );
        assert_eq!(arcade.coins * arcade.price_cents, 100_000_000_000);
        let state = (arcade.coins, arcade.balance_cents, arcade.price_cents);
        assert!(arcade.command(&args(&["coins", "mint", "100"])).is_err());
        assert_eq!(
            (arcade.coins, arcade.balance_cents, arcade.price_cents),
            state
        );
    }

    #[test]
    fn focus_calculation_uses_hypothetical_sizes_and_ceiling_division() {
        let mut arcade = Arcade::new();
        let default = arcade
            .command(&args(&["focus", "calculate"]))
            .unwrap()
            .join("\n");
        assert!(default.contains("explicit fantasy assumption"));
        assert!(default.contains("ceil(20/512) = 1"));
        let two = arcade
            .command(&args(&["focus", "calculate", "513"]))
            .unwrap()
            .join("\n");
        assert!(two.contains("ceil(513/512) = 2"));
        for value in ["0", "-1", "1000001", "1.5", "18446744073709551616"] {
            assert!(
                arcade
                    .command(&args(&["focus", "calculate", value]))
                    .is_err()
            );
        }
    }
}
