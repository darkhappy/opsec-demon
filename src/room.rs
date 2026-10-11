//! Idealized broadband Sabine room acoustics, without audio measurement or I/O.
pub struct Room {
    width: f64,
    length: f64,
    height: f64,
    absorption: f64,
}

impl Room {
    pub fn new() -> Self {
        Self {
            width: 5.0,
            length: 4.0,
            height: 3.0,
            absorption: 0.2,
        }
    }

    pub fn rt60(&self) -> f64 {
        let volume = self.width * self.length * self.height;
        let surface =
            2.0 * (self.width * self.length + self.width * self.height + self.length * self.height);
        0.161 * volume / (surface * self.absorption)
    }

    /// Straight-line idealized energy decay, relative to the initial energy.
    pub fn decay_db(&self, time_s: f64) -> f64 {
        if !time_s.is_finite() || time_s <= 0.0 {
            0.0
        } else {
            -60.0 * time_s / self.rt60()
        }
    }

    pub fn status(&self) -> String {
        format!(
            "SIM RT60 | {:.2} x {:.2} x {:.2} m | absorption {:.3} | Sabine {:.3}s\nSIM idealized diffuse-field broadband decay / not measured audio / no audio I/O",
            self.width,
            self.length,
            self.height,
            self.absorption,
            self.rt60()
        )
    }

    /// Arguments exclude the `rt60` command prefix.
    pub fn command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        match args {
            [] => Ok(vec![self.status()]),
            [action] if action == "status" => Ok(vec![self.status()]),
            [action, width, length, height, absorption] if action == "set" => {
                // Validate all four fields before mutating the model.
                let width = parse_dimension(width)?;
                let length = parse_dimension(length)?;
                let height = parse_dimension(height)?;
                let absorption = absorption
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && (0.001..=1.0).contains(value))
                    .ok_or("SIM | absorption must be finite from 0.001 to 1")?;
                self.width = width;
                self.length = length;
                self.height = height;
                self.absorption = absorption;
                Ok(vec![self.status()])
            }
            _ => Err("SIM | usage: rt60 status | set WIDTH LENGTH HEIGHT ABSORPTION".into()),
        }
    }
}

fn parse_dimension(text: &str) -> Result<f64, String> {
    text.parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && (0.1..=1000.0).contains(value))
        .ok_or_else(|| "SIM | room dimensions must be finite from 0.1 to 1000 meters".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn known_sabine_room_and_minus_sixty_decay() {
        let room = Room::new();
        let expected = 0.161 * 60.0 / (94.0 * 0.2);
        assert!((room.rt60() - expected).abs() < 1e-12);
        assert!((room.decay_db(room.rt60()) + 60.0).abs() < 1e-12);
        assert!((room.decay_db(room.rt60() / 2.0) + 30.0).abs() < 1e-12);
        assert_eq!(room.decay_db(0.0), 0.0);
        assert_eq!(room.decay_db(-1.0), 0.0);
        assert!(room.status().contains("not measured audio"));
    }

    #[test]
    fn updated_room_uses_volume_and_surface_not_only_one_dimension() {
        let mut room = Room::new();
        room.command(&args(&["set", "10", "8", "6", "0.2"]))
            .unwrap();
        assert!((room.rt60() - 0.161 * 480.0 / (376.0 * 0.2)).abs() < 1e-12);
        room.command(&args(&["set", "10", "8", "6", "0.4"]))
            .unwrap();
        assert!((room.rt60() - 0.161 * 480.0 / (376.0 * 0.4)).abs() < 1e-12);
    }

    #[test]
    fn invalid_values_are_rejected_atomically() {
        let mut room = Room::new();
        let before = room.status();
        for values in [
            ["set", "0", "4", "3", "0.2"],
            ["set", "NaN", "4", "3", "0.2"],
            ["set", "10", "inf", "3", "0.2"],
            ["set", "10", "8", "1001", "0.2"],
            ["set", "10", "8", "6", "0"],
            ["set", "10", "8", "6", "1e-320"],
            ["set", "10", "8", "6", "-0.2"],
            ["set", "10", "8", "6", "1.01"],
            ["set", "10", "8", "6", "NaN"],
            ["set", "10", "8", "6", "inf"],
        ] {
            assert!(room.command(&args(&values)).is_err());
            assert_eq!(room.status(), before);
        }
    }

    #[test]
    fn bounds_and_nonfinite_time_never_make_nan_graph_values() {
        let mut room = Room::new();
        room.command(&args(&["set", "0.1", "1000", "1", "1"]))
            .unwrap();
        assert!(room.rt60().is_finite() && room.rt60() > 0.0);
        assert_eq!(room.decay_db(f64::NAN), 0.0);
        assert_eq!(room.decay_db(f64::INFINITY), 0.0);
        assert!(room.command(&args(&["set", "1", "2"])).is_err());
    }
}
