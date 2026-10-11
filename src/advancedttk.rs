//! A bounded JSON command reader for combat setups awaiting explicit game rules.
pub fn run(payload: &str) -> Result<Vec<String>, String> {
    if payload.len() > 4096 {
        return Err("SIM | advancedttk payload exceeds 4096 bytes".into());
    }
    let mut parser = Parser {
        text: payload,
        at: 0,
    };
    parser.expect(b'{')?;
    let (mut events, mut modes, mut target) = (None, None, None);
    let mut calculation = None;
    let mut swap_ms = None;
    loop {
        parser.space();
        if parser.take(b'}') {
            break;
        }
        let key = parser.string()?;
        parser.expect(b':')?;
        match key.as_str() {
            "events" if events.is_none() => events = Some(parser.strings()?),
            "mode" if modes.is_none() => modes = Some(parser.numbers()?),
            "target" if target.is_none() => target = Some(parser.string()?),
            "calculation" if calculation.is_none() => calculation = Some(parser.string()?),
            "swap_ms" if swap_ms.is_none() => swap_ms = Some(parser.swap_ms()?),
            _ => {
                return Err(
                    "SIM | advancedttk expects unique events, mode, target, and optional calculation/swap_ms fields".into(),
                );
            }
        }
        parser.space();
        if parser.take(b'}') {
            break;
        }
        parser.expect(b',')?;
        parser.space();
        if parser.peek() == Some(b'}') {
            return Err("SIM | trailing JSON commas are not supported".into());
        }
    }
    parser.space();
    if parser.at != payload.len() {
        return Err("SIM | unexpected text after advancedttk object".into());
    }
    let events = events.ok_or("SIM | missing events array")?;
    let modes = modes.ok_or("SIM | missing mode array")?;
    let target = target.ok_or("SIM | missing target string")?;
    if events.is_empty() || events.len() > 16 || modes.is_empty() || modes.len() > 16 {
        return Err("SIM | events and mode arrays require 1 to 16 entries".into());
    }
    let target_words: Vec<_> = target.split_whitespace().collect();
    if target_words.len() != 4 || target_words[0] != "-ehp" {
        return Err("SIM | target expects -ehp VALUE VALUE VALUE".into());
    }
    let mut target_values = Vec::new();
    for value in &target_words[1..] {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("SIM | target values must be whole numbers from 0 to 1000000000".into());
        }
        let value = value
            .parse::<u64>()
            .ok()
            .filter(|value| *value <= 1_000_000_000)
            .ok_or("SIM | target values must be whole numbers from 0 to 1000000000")?;
        target_values.push(value);
    }
    let hit = match calculation.as_deref() {
        None => None,
        Some("individual-body") => Some(false),
        Some("individual-head" | "sequence-head") => Some(true),
        Some("sequence-body") => Some(false),
        _ => return Err(
            "calculation expects individual-body, individual-head, sequence-body, or sequence-head"
                .into(),
        ),
    };
    let sequence = calculation
        .as_deref()
        .is_some_and(|value| value.starts_with("sequence-"));
    let mut output = vec![if hit.is_some() {
        "ADVANCED TTK // historical Conclave preset / IDEALIZED MODEL ESTIMATE".into()
    } else {
        "ADVANCED TTK // Conclave combat setup / UNCOMPUTED".into()
    }];
    let mut parsed_events = Vec::new();
    for event in events {
        let parts: Vec<_> = event.split_whitespace().collect();
        let Some(index) = parts.iter().position(|part| *part == "-mod") else {
            return Err("SIM | event expects WEAPON NAME -mod VALUE DAMAGE-TYPE".into());
        };
        if index == 0 || parts.len() != index + 3 {
            return Err("SIM | event expects WEAPON NAME -mod VALUE DAMAGE-TYPE".into());
        }
        let modifier = parts[index + 1]
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && (-1000.0..=1000.0).contains(value))
            .ok_or("SIM | modifier must be a finite number from -1000 to 1000")?;
        let damage = parts[index + 2];
        if !damage
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic() || byte == b'-')
        {
            return Err("SIM | damage type must contain ASCII letters or hyphens".into());
        }
        output.push(format!(
            "SETUP | weapon={} | modifier={} | type={damage}",
            parts[..index].join(" "),
            modifier
        ));
        parsed_events.push((parts[..index].join(" "), modifier, damage.to_string()));
    }
    for index in &modes {
        if *index == 0 || usize::from(*index) > parsed_events.len() {
            return Err(format!(
                "mode entries are one-based weapon indices from 1 to {}",
                parsed_events.len()
            ));
        }
    }
    output.push(format!(
        "SETUP | mode sequence: {} (one-based weapon indices)",
        modes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(" -> ")
    ));
    output.push(format!(
        "SEQUENCE | {}",
        modes
            .iter()
            .map(|index| parsed_events[usize::from(*index) - 1].0.as_str())
            .collect::<Vec<_>>()
            .join(" -> ")
    ));
    output.push(format!(
        "SETUP | target -ehp {} {} {} (supplied values; interpretation unspecified)",
        target_values[0], target_values[1], target_values[2]
    ));
    if let Some(head) = hit {
        if target_values[0] == 0 {
            return Err("calculation requires target health greater than zero".into());
        }
        output.push(
            "MODEL | override interprets target as HEALTH SHIELD ARMOR, Flesh/Ferrite/Shield."
                .into(),
        );
        output.push(if sequence {
            "MODEL | each mode entry fires one shot from its indexed event weapon; finite sequence, no automatic repetition."
        } else {
            "MODEL | individual calculation evaluates each weapon independently; use sequence-body/head to execute mode order."
        }.into());
        output.push("MODEL | historical wiki presets, not verified current game values; perfect accuracy, no regeneration, falloff, scoped bonus, status, or damage quantization.".into());
        output.push("MODEL | assumed conversion: D'=D*(1-c), then add c*total IPS to chosen type; assumed shield overflow uses remaining shot fraction.".into());
        output.push("MODEL | first shot t=0; empty-mag reload starts immediately, next shot waits max(reload, fire interval). No weapon-swap timing.".into());
        let mut sequence_weapons = Vec::new();
        for (name, conversion, damage_type) in parsed_events {
            let weapon = Weapon::preset(&name).ok_or(
                "calculation supports only Vectis Prime and Lex Prime historical Conclave presets",
            )?;
            if !(0.0..=20.0).contains(&conversion) {
                return Err("Conclave model conversion must be from 0 to 20 percent".into());
            }
            let index = match damage_type.to_ascii_lowercase().as_str() {
                "impact" => 0,
                "puncture" => 1,
                "slash" => 2,
                _ => {
                    return Err(
                        "Conclave model conversion type must be impact, puncture, or slash".into(),
                    );
                }
            };
            let mut ips = convert(weapon.ips, conversion / 100.0, index);
            if head {
                for component in &mut ips {
                    *component *= weapon.head;
                }
            }
            let shield_damage = ips[0] * 1.5 + ips[1] * 0.8 + ips[2];
            let health_damage = health_damage(ips, target_values[2] as f64);
            if sequence {
                sequence_weapons.push(ShotWeapon {
                    name,
                    preset: weapon,
                    health_damage,
                    shield_damage,
                });
                continue;
            }
            let shots = shots_required(
                target_values[0] as f64,
                target_values[1] as f64,
                health_damage,
                shield_damage,
            );
            let reloads = (shots - 1) / weapon.magazine;
            let time = shot_time(shots, weapon.rate, weapon.magazine, weapon.reload);
            output.push(format!("ESTIMATE | {name} | {} | IPS {:.4}/{:.4}/{:.4} | shield/shot {:.4} | health/shot {:.4} | {} shots | {} reloads | TTK {:.4}s", if head { "unscoped head" } else { "body" }, ips[0], ips[1], ips[2], shield_damage, health_damage, shots, reloads, time));
        }
        if sequence {
            output.extend(ordered_sequence(
                &sequence_weapons,
                &modes,
                target_values[0] as f64,
                target_values[1] as f64,
                swap_ms,
            ));
        }
        output.push("SOURCE | https://wiki.warframe.com/w/Conclave | https://warframe.fandom.com/wiki/Conclave:Vectis_Prime | https://warframe.fandom.com/wiki/Conclave:Lex_Prime".into());
        output.push("SOURCE | https://warframe.fandom.com/wiki/Damage_2.0/Overview_Table | https://www.warframe.com/en/patch-notes/pc/17-0-0".into());
    } else {
        output.push("TTK pending: target interpretation and hit placement must be selected. Weapon order is resolved. Use calculation=sequence-body or sequence-head to interpret target as HEALTH SHIELD ARMOR in the disclosed historical model; swap_ms supplies switch timing.".into());
    }
    Ok(output)
}

struct ShotWeapon {
    name: String,
    preset: Weapon,
    health_damage: f64,
    shield_damage: f64,
}

fn ordered_sequence(
    weapons: &[ShotWeapon],
    modes: &[u8],
    mut health: f64,
    mut shield: f64,
    swap_ms: Option<u32>,
) -> Vec<String> {
    let mut output = vec!["MODEL | each weapon has its own cooldown and magazine; idealized immediate automatic reload can complete while holstered. Swap delay is supplied, not inferred from Conclave.".into()];
    let mut ready = vec![0.0_f64; weapons.len()];
    let mut ammo: Vec<_> = weapons
        .iter()
        .map(|weapon| weapon.preset.magazine)
        .collect();
    let mut elapsed = 0.0_f64;
    let mut previous = None;
    for (shot, index) in modes.iter().enumerate() {
        let index = usize::from(*index) - 1;
        let weapon = &weapons[index];
        if previous.is_some_and(|previous| previous != index) {
            elapsed += f64::from(swap_ms.unwrap_or(0)) / 1000.0;
        }
        elapsed = elapsed.max(ready[index]);
        let shield_before = shield;
        let health_before = health;
        let consumed = (shield / weapon.shield_damage).clamp(0.0, 1.0);
        shield = (shield - weapon.shield_damage).max(0.0);
        health = (health - weapon.health_damage * (1.0 - consumed)).max(0.0);
        ammo[index] -= 1;
        ready[index] = elapsed
            + if ammo[index] == 0 {
                ammo[index] = weapon.preset.magazine;
                weapon.preset.reload.max(1.0 / weapon.preset.rate)
            } else {
                1.0 / weapon.preset.rate
            };
        output.push(format!("SHOT {} | weapon {} {} | t={elapsed:.4}s{} | shield {shield_before:.4}->{shield:.4} | health {health_before:.4}->{health:.4}", shot + 1, index + 1, weapon.name, if swap_ms.is_none() { " excluding swap time" } else { "" }));
        if health <= 0.0 {
            output.push(if swap_ms.is_some() {
                format!("ESTIMATE SEQUENCE | target defeated on shot {} | TTK {elapsed:.4}s / supplied swap delay", shot + 1)
            } else {
                format!("RESULT SEQUENCE | target defeated on shot {} | firing-only lower bound {elapsed:.4}s; TTK requires swap_ms", shot + 1)
            });
            return output;
        }
        previous = Some(index);
    }
    output.push(format!("RESULT SEQUENCE | target survives all {} shots | health {health:.4} | shield {shield:.4} | no kill, no TTK", modes.len()));
    output
}

struct Weapon {
    ips: [f64; 3],
    rate: f64,
    magazine: u64,
    reload: f64,
    head: f64,
}

impl Weapon {
    fn preset(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "vectis prime" => Some(Self {
                ips: [43.12, 48.51, 16.17],
                rate: 2.67,
                magazine: 2,
                reload: 0.90,
                head: 1.4,
            }),
            "lex prime" => Some(Self {
                ips: [7.8, 62.4, 7.8],
                rate: 2.08,
                magazine: 8,
                reload: 2.30,
                head: 1.5,
            }),
            _ => None,
        }
    }
}

fn convert(ips: [f64; 3], fraction: f64, index: usize) -> [f64; 3] {
    let mut converted = ips.map(|value| value * (1.0 - fraction));
    converted[index] += ips.iter().sum::<f64>() * fraction;
    converted
}

fn health_damage(ips: [f64; 3], armor: f64) -> f64 {
    let flesh = [0.75, 1.0, 1.25];
    let ferrite = [0.0, 0.5, -0.15];
    (0..3)
        .map(|index| {
            let armor_modifier = if armor > 0.0 { ferrite[index] } else { 0.0 };
            ips[index] * flesh[index] * (1.0 + armor_modifier) * 300.0
                / (300.0 + armor * (1.0 - armor_modifier))
        })
        .sum()
}

fn ceil_shots(value: f64) -> u64 {
    (value - 1e-10).ceil().max(0.0) as u64
}

fn shots_required(health: f64, shield: f64, health_damage: f64, shield_damage: f64) -> u64 {
    let shield_shots = ceil_shots(shield / shield_damage);
    let overflow_fraction = if shield_shots > 0 {
        ((shield_shots as f64 * shield_damage - shield) / shield_damage).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let remaining_health = (health - health_damage * overflow_fraction).max(0.0);
    shield_shots + ceil_shots(remaining_health / health_damage)
}

fn shot_time(shots: u64, rate: f64, magazine: u64, reload: f64) -> f64 {
    let reloads = (shots - 1) / magazine;
    let ordinary_intervals = shots - 1 - reloads;
    ordinary_intervals as f64 / rate + reloads as f64 * reload.max(1.0 / rate)
}

struct Parser<'a> {
    text: &'a str,
    at: usize,
}

impl Parser<'_> {
    fn swap_ms(&mut self) -> Result<u32, String> {
        self.space();
        let start = self.at;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.at += 1;
        }
        let value = &self.text[start..self.at];
        if value.len() > 1 && value.starts_with('0') {
            return Err("swap_ms requires a JSON whole number from 0 to 60000".into());
        }
        value
            .parse::<u32>()
            .ok()
            .filter(|value| *value <= 60_000)
            .ok_or_else(|| "swap_ms requires a JSON whole number from 0 to 60000".into())
    }

    fn space(&mut self) {
        while self
            .peek()
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }

    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.space();
        if self.take(byte) {
            Ok(())
        } else {
            Err(format!(
                "SIM | invalid JSON at byte {}: expected {}",
                self.at,
                char::from(byte)
            ))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut value = String::new();
        loop {
            let character = self.text[self.at..]
                .chars()
                .next()
                .ok_or("SIM | unterminated JSON string")?;
            self.at += character.len_utf8();
            match character {
                '"' => return Ok(value),
                '\\' => {
                    let escaped = self.peek().ok_or("SIM | unfinished JSON escape")?;
                    self.at += 1;
                    value.push(match escaped {
                        b'"' => '"', b'\\' => '\\', b'/' => '/',
                        b'u' => {
                            let first = self.hex_quad()?;
                            let code = if (0xd800..=0xdbff).contains(&first) {
                                if !self.take(b'\\') || !self.take(b'u') { return Err("SIM | invalid Unicode surrogate pair".into()); }
                                let second = self.hex_quad()?;
                                if !(0xdc00..=0xdfff).contains(&second) { return Err("SIM | invalid Unicode surrogate pair".into()); }
                                0x10000 + ((first - 0xd800) << 10) + second - 0xdc00
                            } else { first };
                            let decoded = char::from_u32(code).ok_or("SIM | invalid Unicode escape")?;
                            if decoded.is_control() { return Err("SIM | control characters are not allowed in strings".into()); }
                            decoded
                        }
                        b'b' | b'f' | b'n' | b'r' | b't' => return Err("SIM | control characters are not allowed in strings".into()),
                        _ => return Err("SIM | invalid JSON escape".into()),
                    });
                }
                character if character.is_control() => {
                    return Err("SIM | control characters are not allowed in strings".into());
                }
                character => value.push(character),
            }
        }
    }

    fn hex_quad(&mut self) -> Result<u32, String> {
        let mut code = 0;
        for _ in 0..4 {
            let byte = self.peek().ok_or("SIM | incomplete Unicode escape")?;
            let digit = char::from(byte)
                .to_digit(16)
                .ok_or("SIM | invalid Unicode escape")?;
            self.at += 1;
            code = code * 16 + digit;
        }
        Ok(code)
    }

    fn strings(&mut self) -> Result<Vec<String>, String> {
        self.expect(b'[')?;
        self.space();
        let mut values = Vec::new();
        if self.take(b']') {
            return Ok(values);
        }
        loop {
            values.push(self.string()?);
            if values.len() > 16 {
                return Err("SIM | at most 16 events are supported".into());
            }
            self.space();
            if self.take(b']') {
                return Ok(values);
            }
            self.expect(b',')?;
        }
    }

    fn numbers(&mut self) -> Result<Vec<u8>, String> {
        self.expect(b'[')?;
        self.space();
        let mut values = Vec::new();
        if self.take(b']') {
            return Ok(values);
        }
        loop {
            self.space();
            let start = self.at;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.at += 1;
            }
            let number = &self.text[start..self.at];
            if number.is_empty() || (number.len() > 1 && number.starts_with('0')) {
                return Err("SIM | mode entries must be JSON whole numbers from 0 to 99".into());
            }
            values.push(
                number
                    .parse::<u8>()
                    .ok()
                    .filter(|value| *value <= 99)
                    .ok_or("SIM | mode entries must be JSON whole numbers from 0 to 99")?,
            );
            if values.len() > 16 {
                return Err("SIM | at most 16 mode entries are supported".into());
            }
            self.space();
            if self.take(b']') {
                return Ok(values);
            }
            self.expect(b',')?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const PAYLOAD: &str = r#"{"events":["vectis prime -mod 20 impact", "lex prime -mod 20 puncture"],"mode":[1,2,2],"target":"-ehp 195 125 175"}"#;

    #[test]
    fn requested_payload_preserves_values_without_claiming_real_ttk() {
        let lines = run(PAYLOAD).unwrap();
        assert!(lines[0].contains("UNCOMPUTED"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("vectis prime") && line.contains("modifier=20"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("lex prime") && line.contains("puncture"))
        );
        assert!(lines.iter().any(|line| line.contains("1 -> 2 -> 2")));
        assert!(lines.iter().any(|line| line.contains("195 125 175")));
        assert!(lines.last().unwrap().contains("TTK pending"));
    }

    #[test]
    fn malformed_and_ambiguous_inputs_are_rejected() {
        for payload in ["", "{}", "[]", "{\"events\":[]}", "{\"unknown\":1}"] {
            assert!(run(payload).is_err());
        }
        for changed in [
            PAYLOAD.replace("[1,2,2]", "[1,2,2,]"),
            PAYLOAD.replace("[1,2,2]", "[01]"),
            PAYLOAD.replace("[1,2,2]", "[-1]"),
            PAYLOAD.replace("[1,2,2]", "[100]"),
            PAYLOAD.replace("20 impact", "NaN impact"),
            PAYLOAD.replace("-ehp 195", "-ehp -195"),
            PAYLOAD.replace("\"target\":", "\"mode\":[1],\"target\":"),
            format!("{PAYLOAD} trailing"),
            " ".repeat(4097),
        ] {
            assert!(run(&changed).is_err(), "accepted {changed}");
        }
    }

    #[test]
    fn whitespace_and_unicode_escapes_decode_safely() {
        let changed = PAYLOAD
            .replace("vectis", r"\u0076ectis")
            .replace("lex", "烈");
        assert!(run(&format!(" \n {changed} \t")).is_ok());
        assert!(run(&PAYLOAD.replace("vectis", r"\uD83D\uDE00")).is_ok());
        assert!(run(&PAYLOAD.replace("vectis", r"\uD800")).is_err());
        assert!(run(&PAYLOAD.replace("vectis", r"\u001b")).is_err());
    }

    #[test]
    fn mode_indices_resolve_exact_weapon_order_and_reject_missing_weapons() {
        let lines = run(PAYLOAD).unwrap();
        assert!(
            lines
                .iter()
                .any(|line| line == "SEQUENCE | vectis prime -> lex prime -> lex prime")
        );
        for invalid in ["[0]", "[3]", "[1,2,0]"] {
            assert!(run(&PAYLOAD.replace("[1,2,2]", invalid)).is_err());
        }
        assert!(lines.last().unwrap().contains("Weapon order is resolved"));
    }

    fn test_weapon(name: &str) -> ShotWeapon {
        ShotWeapon {
            name: name.into(),
            preset: Weapon {
                ips: [100.0, 0.0, 0.0],
                rate: 2.0,
                magazine: 2,
                reload: 0.9,
                head: 1.0,
            },
            health_damage: 75.0,
            shield_damage: 150.0,
        }
    }

    #[test]
    fn sequence_carries_target_damage_between_weapons_and_counts_switch_delay() {
        let weapons = [test_weapon("first"), test_weapon("second")];
        let output = ordered_sequence(&weapons, &[1, 2, 2], 100.0, 150.0, Some(250));
        assert!(output[1].contains("shield 150.0000->0.0000 | health 100.0000->100.0000"));
        assert!(output[2].contains("health 100.0000->25.0000"));
        assert!(output.last().unwrap().contains("TTK 0.7500s"));
        let missing_timing = ordered_sequence(&weapons, &[1, 2, 2], 100.0, 150.0, None);
        assert!(
            missing_timing
                .last()
                .unwrap()
                .contains("TTK requires swap_ms")
        );
        let alive = ordered_sequence(&weapons, &[1, 2], 100.0, 150.0, Some(250));
        assert!(alive.last().unwrap().contains("target survives"));
        assert!(alive.last().unwrap().contains("health 25.0000"));
    }

    #[test]
    fn sequence_uses_per_weapon_magazines_and_stops_at_the_killing_shot() {
        let weapons = [test_weapon("first"), test_weapon("second")];
        let output = ordered_sequence(&weapons, &[1, 1, 1], 500.0, 0.0, Some(0));
        assert!(output[3].contains("t=1.4000s"));
        let output = ordered_sequence(&weapons, &[1, 2, 2], 1.0, 0.0, Some(300));
        assert_eq!(
            output
                .iter()
                .filter(|line| line.starts_with("SHOT "))
                .count(),
            1
        );
        assert!(output.last().unwrap().contains("TTK 0.0000s"));
    }

    #[test]
    fn sequence_override_and_swap_time_are_validated() {
        let payload = PAYLOAD.replacen('{', r#"{"calculation":"sequence-body","swap_ms":500,"#, 1);
        assert!(
            run(&payload)
                .unwrap()
                .iter()
                .any(|line| line.starts_with("SHOT 2 | weapon 2 lex prime"))
        );
        assert!(run(&payload.replace("sequence-body", "sequence-head")).is_ok());
        for value in ["-1", "60001", "01", "0.5", "null", "\"500\""] {
            assert!(
                run(&payload.replace(r#""swap_ms":500"#, &format!("\"swap_ms\":{value}"))).is_err()
            );
        }
    }

    #[test]
    fn conversion_preserves_total_and_armor_uses_old_typed_modifiers() {
        let converted = convert([40.0, 45.0, 15.0], 0.2, 0);
        assert_eq!(converted, [52.0, 36.0, 12.0]);
        assert_eq!(converted.iter().sum::<f64>(), 100.0);
        assert!((health_damage([100.0, 0.0, 0.0], 0.0) - 75.0).abs() < 1e-9);
        assert!((health_damage([0.0, 100.0, 0.0], 300.0) - 100.0).abs() < 1e-9);
        assert!((health_damage([100.0, 0.0, 0.0], 300.0) - 37.5).abs() < 1e-9);
    }

    #[test]
    fn shield_overflow_uses_fraction_and_reload_timing_starts_at_zero() {
        // A 100-impact shot deals 150 shield damage and 75 unarmored health damage.
        // 75 shield consumes half the shot, leaving 37.5 health damage.
        assert_eq!(shots_required(37.5, 75.0, 75.0, 150.0), 1);
        assert_eq!(shots_required(60.0, 75.0, 75.0, 150.0), 2);
        assert_eq!(shots_required(75.0, 150.0, 75.0, 150.0), 2);
        assert_eq!(shots_required(150.0, 0.0, 75.0, 150.0), 2);
        assert_eq!(shot_time(1, 2.0, 2, 0.9), 0.0);
        assert!((shot_time(3, 2.0, 2, 0.9) - 1.4).abs() < 1e-9);
        assert!((shot_time(5, 2.0, 2, 0.9) - 2.8).abs() < 1e-9);
        assert!((shot_time(3, 2.0, 2, 0.1) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn explicit_override_computes_each_weapon_with_disclosed_assumptions() {
        let changed = PAYLOAD.replacen('{', r#"{"calculation":"individual-body","#, 1);
        let output = run(&changed).unwrap();
        assert!(output[0].contains("IDEALIZED MODEL ESTIMATE"));
        assert_eq!(
            output
                .iter()
                .filter(|line| line.starts_with("ESTIMATE |"))
                .count(),
            2
        );
        assert!(
            output
                .iter()
                .any(|line| line.contains("assumed conversion"))
        );
        assert!(
            output
                .iter()
                .any(|line| line.contains("not verified current"))
        );
        assert!(run(&changed.replace("individual-body", "individual-head")).is_ok());
        assert!(run(&changed.replace("20 impact", "21 impact")).is_err());
        assert!(run(&changed.replace("vectis prime", "unknown weapon")).is_err());
        assert!(run(&changed.replace("195 125 175", "0 125 175")).is_err());
        // Large legal inputs use algebra rather than an unbounded per-shot simulation.
        assert!(run(&changed.replace("195 125 175", "1000000000 1000000000 1000000000")).is_ok());
    }
}
