//! In-session Deadlock-inspired theater. No game connection or API is used.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Ritual {
    started: Instant,
}

impl Ritual {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    pub fn status(&self) -> String {
        let elapsed = self.started.elapsed().as_secs();
        let phases = [
            "CHALK SIGILS / tracing subway ward",
            "SOUL CONDUIT / feeding the cursed apple",
            "PATRON WHISPERS / receiving forbidden terms",
            "VEIL FRACTURE / the city hears your name",
            "SUMMONING / keeping the breach contained",
            "WARD RESET / another cycle begins",
        ];
        let phase = (elapsed / 20) as usize % phases.len();
        let progress = elapsed % 20 * 5;
        format!(
            "CURSED APPLE [SIM / FICTIONAL]\n{} [{progress:02}%]\nCycle {} | veil pressure {}%\nMo & Krill: tunnel perimeter secure",
            phases[phase],
            elapsed / 120 + 1,
            40 + (elapsed * 7 % 57)
        )
    }
}

const COST: u64 = 800;
const INITIAL_VALUE: u64 = 400;
const SOULS_PER_MINUTE: u64 = 80;
const HATCH_CHANNEL: Duration = Duration::from_secs(2);
const BUFF_NAMES: [&str; 10] = [
    "fire rate",
    "max ammo",
    "cooldown reduction",
    "weapon damage",
    "max health",
    "spirit power",
    "bullet resist",
    "spirit resist",
    "move speed",
    "ability range",
];
const BUFF_AMOUNTS: [[f64; 10]; 3] = [
    [1.5, 3.0, 0.5, 3.0, 15.0, 2.0, 0.5, 0.5, 0.1, 0.25],
    [2.0, 5.0, 0.75, 4.0, 20.0, 3.0, 0.75, 0.75, 0.2, 0.5],
    [2.5, 7.0, 1.0, 6.0, 30.0, 4.0, 1.0, 1.0, 0.3, 0.75],
];

pub struct Egg {
    session_started: Instant,
    held_since: Option<Instant>,
    hatch_started: Option<Instant>,
    souls: u64,
    hatched: u64,
    buff_stacks: [u64; 10],
    buff_totals: [f64; 10],
    random_state: u64,
}

impl Egg {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |time| time.as_nanos() as u64);
        Self {
            session_started: Instant::now(),
            held_since: None,
            hatch_started: None,
            souls: 2400,
            hatched: 0,
            buff_stacks: [0; 10],
            buff_totals: [0.0; 10],
            random_state: seed.max(1),
        }
    }

    pub fn start(&mut self) -> Result<String, String> {
        if self.held_since.is_some() {
            return Err("You already carry a Golden Goose Egg. Try egg hatch.".into());
        }
        if self.souls < COST {
            return Err(format!(
                "SIM wallet: {} souls; an egg costs {COST}.",
                self.souls
            ));
        }
        self.souls -= COST;
        self.held_since = Some(Instant::now());
        Ok("GOLDEN GOOSE EGG ACQUIRED [SIM] | -15% damage | +1m sprint | +1 out-of-combat regen | 400 starting souls +80/min | egg hatch to cash out".into())
    }

    fn accrued_at(&self, now: Instant) -> u64 {
        self.held_since.map_or(0, |started| {
            let nanos = now.saturating_duration_since(started).as_nanos();
            (nanos.saturating_mul(u128::from(SOULS_PER_MINUTE)) / 60_000_000_000)
                .min(u128::from(u64::MAX)) as u64
        })
    }

    pub fn status(&self) -> String {
        let now = Instant::now();
        let held = if let Some(started) = self.held_since {
            let accrued = self.accrued_at(now);
            let state = if let Some(channel) = self.hatch_started {
                format!(
                    "HATCH CHANNEL {:.1}/2.0s",
                    now.duration_since(channel).as_secs_f64().min(2.0)
                )
            } else {
                format!(
                    "INCUBATING {:02}:{:02}",
                    started.elapsed().as_secs() / 60,
                    started.elapsed().as_secs() % 60
                )
            };
            format!(
                "{state} | {} souls / {} buffs\n-15% damage | +1m sprint | +1 OOC regen",
                INITIAL_VALUE + accrued,
                accrued / 80
            )
        } else {
            "EMPTY NEST | egg start to purchase\nCarrying penalties: none".into()
        };
        let buffs: Vec<String> = self
            .buff_totals
            .iter()
            .enumerate()
            .filter(|(index, _)| self.buff_stacks[*index] > 0)
            .map(|(index, total)| {
                format!(
                    "{} +{total:.2}{} ({} stacks)",
                    BUFF_NAMES[index],
                    buff_unit(index),
                    self.buff_stacks[index]
                )
            })
            .collect();
        format!(
            "GOLDEN GOOSE [SIM]\n{held}\nWallet {} souls | {} hatched\nPermanent: {}",
            self.souls,
            self.hatched,
            if buffs.is_empty() {
                "none".into()
            } else {
                buffs.join(", ")
            }
        )
    }

    pub fn hatch(&mut self) -> Result<String, String> {
        if self.held_since.is_none() {
            return Err("No egg to hatch. Use egg start first.".into());
        }
        if self.hatch_started.is_some() {
            return Err("Egg hatch channel is already running.".into());
        }
        self.hatch_started = Some(Instant::now());
        Ok("HATCH CHANNEL [SIM] | 2 seconds | the gilded shell begins to fracture...".into())
    }

    /// Poll once per UI tick; emits a completion event exactly once.
    pub fn tick(&mut self) -> Option<String> {
        self.tick_at(Instant::now())
    }

    fn tick_at(&mut self, now: Instant) -> Option<String> {
        let channel = self.hatch_started?;
        if now.saturating_duration_since(channel) < HATCH_CHANNEL {
            return None;
        }
        // Fix payout to channel completion even if a busy UI polls later.
        let completed = channel + HATCH_CHANNEL;
        let accrued = self.accrued_at(completed);
        let payout = INITIAL_VALUE.saturating_add(accrued);
        let rolls = accrued / 80;
        let age = completed
            .saturating_duration_since(self.session_started)
            .as_secs();
        let tier = if age >= 1800 {
            2
        } else if age >= 600 {
            1
        } else {
            0
        };
        let mut gained = [0_u64; 10];
        for _ in 0..rolls {
            let index = self.roll_buff();
            gained[index] += 1;
            self.buff_stacks[index] += 1;
            self.buff_totals[index] += BUFF_AMOUNTS[tier][index];
        }
        self.souls = self.souls.saturating_add(payout);
        self.hatched += 1;
        self.held_since = None;
        self.hatch_started = None;
        let rewards: Vec<String> = gained
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(index, count)| {
                format!(
                    "{} +{:.2}{}",
                    BUFF_NAMES[index],
                    *count as f64 * BUFF_AMOUNTS[tier][index],
                    buff_unit(index)
                )
            })
            .collect();
        Some(format!(
            "EGG HATCHED [SIM] | +{payout} souls | {rolls} permanent buffs: {} | carrying penalties cleared",
            if rewards.is_empty() {
                "none (hold >=60 seconds for the first buff)".into()
            } else {
                rewards.join(", ")
            }
        ))
    }

    fn roll_buff(&mut self) -> usize {
        // Simple weighted draws for theater; not Deadlock's hidden PRD system.
        self.random_state ^= self.random_state << 13;
        self.random_state ^= self.random_state >> 7;
        self.random_state ^= self.random_state << 17;
        let draw = (self.random_state % 20) as usize;
        let weights = [2, 2, 2, 2, 4, 2, 2, 2, 1, 1];
        let mut cumulative = 0;
        for (index, weight) in weights.iter().enumerate() {
            cumulative += weight;
            if draw < cumulative {
                return index;
            }
        }
        9
    }
}

fn buff_unit(index: usize) -> &'static str {
    match index {
        4 | 5 => "",
        8 => "m/s",
        _ => "%",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purchase_charges_wallet_and_prevents_duplicate_egg() {
        let mut egg = Egg::new();
        assert!(egg.hatch().is_err());
        assert!(egg.start().is_ok());
        assert_eq!(egg.souls, 1600);
        assert!(egg.start().is_err());
        assert_eq!(egg.souls, 1600);
    }

    #[test]
    fn initial_souls_do_not_grant_buffs_and_hatch_is_nonblocking() {
        let mut egg = Egg::new();
        egg.start().unwrap();
        egg.hatch().unwrap();
        let channel = egg.hatch_started.unwrap();
        assert!(egg.tick_at(channel + Duration::from_millis(1999)).is_none());
        assert!(egg.hatch().is_err());
        assert!(egg.tick_at(channel + HATCH_CHANNEL).is_some());
        assert_eq!(egg.buff_stacks.iter().sum::<u64>(), 0);
        assert_eq!(egg.souls, 2002);
        assert!(egg.held_since.is_none());
        assert!(egg.tick().is_none());
    }

    #[test]
    fn held_five_minutes_breaks_even_and_rewards_five_buffs() {
        let mut egg = Egg::new();
        egg.start().unwrap();
        let completed = Instant::now();
        egg.held_since = Some(completed - Duration::from_secs(300));
        egg.hatch_started = Some(completed - HATCH_CHANNEL);
        let event = egg.tick_at(completed + Duration::from_secs(60)).unwrap();
        assert!(event.contains("+800 souls"));
        assert_eq!(egg.souls, 2400);
        assert_eq!(egg.buff_stacks.iter().sum::<u64>(), 5);
        assert!(egg.buff_totals.iter().any(|total| *total > 0.0));
        assert!(egg.start().is_ok());
        assert_eq!(egg.buff_stacks.iter().sum::<u64>(), 5);
    }

    #[test]
    fn accrued_soul_and_buff_boundaries_are_exact() {
        let mut egg = Egg::new();
        egg.start().unwrap();
        let started = egg.held_since.unwrap();
        assert_eq!(egg.accrued_at(started + Duration::from_secs(59)), 78);
        assert_eq!(egg.accrued_at(started + Duration::from_secs(60)), 80);
        assert_eq!(egg.accrued_at(started + Duration::from_secs(300)), 400);
    }

    #[test]
    fn ritual_is_labeled_as_fiction() {
        let ritual = Ritual::new();
        assert!(ritual.status().contains("SIM / FICTIONAL"));
    }
}
