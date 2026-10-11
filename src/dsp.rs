//! Synthetic response telemetry for a minimum-phase second-order low-pass filter.
//!
//! H(f) = exp(-j 2 pi f T) / (1 - (f/fc)^2 + j f/(fc Q)).
//! This models an invented signal, not measured machine or audio performance.
//! Excess group delay uses the magnitude-derived minimum-phase low-pass as its
//! reference; the remaining delay is precisely the added transport delay T.

use std::f64::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Magnitude,
    Phase,
    GroupDelay,
    ExcessDelay,
}

impl Response {
    pub const ALL: [Self; 4] = [
        Self::Magnitude,
        Self::Phase,
        Self::GroupDelay,
        Self::ExcessDelay,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Magnitude => "FREQUENCY RESPONSE",
            Self::Phase => "PHASE RESPONSE",
            Self::GroupDelay => "GROUP DELAY",
            Self::ExcessDelay => "EXCESS GROUP DELAY",
        }
    }

    pub const fn unit(self) -> &'static str {
        match self {
            Self::Magnitude => "dB",
            Self::Phase => "deg",
            Self::GroupDelay | Self::ExcessDelay => "ms",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Model {
    /// Resonant cutoff frequency in Hz.
    pub cutoff: f64,
    /// Positive resonance quality factor.
    pub q: f64,
    /// Added pure transport delay in seconds.
    pub transport_delay_s: f64,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            cutoff: 1_600.0,
            q: std::f64::consts::FRAC_1_SQRT_2,
            transport_delay_s: 0.002,
        }
    }
}

impl Model {
    /// Evaluate a positive-frequency response. Phase is unwrapped in degrees.
    /// Delay is -d(phase in radians)/d(angular frequency), returned in ms.
    /// Invalid inputs use defaults; extreme values are bounded for UI stability.
    pub fn value(self, response: Response, f_hz: f64) -> f64 {
        let defaults = Self::default();
        let cutoff = finite_or(self.cutoff, defaults.cutoff).clamp(1e-6, 1e12);
        let q = finite_or(self.q, defaults.q).clamp(1e-6, 1e6);
        let delay = finite_or(self.transport_delay_s, defaults.transport_delay_s).clamp(0.0, 1e6);
        let frequency = finite_or(f_hz, 0.0).clamp(0.0, 1e12);
        let ratio = frequency / cutoff;
        let real = 1.0 - ratio * ratio;
        let imaginary = ratio / q;
        let norm_squared = real * real + imaginary * imaginary;
        match response {
            Response::Magnitude => -10.0 * norm_squared.log10(),
            Response::Phase => (-imaginary.atan2(real) - TAU * frequency * delay) * 180.0 / PI,
            Response::GroupDelay => {
                let minimum_phase_delay = (1.0 + ratio * ratio) / (q * TAU * cutoff * norm_squared);
                (minimum_phase_delay + delay) * 1_000.0
            }
            Response::ExcessDelay => delay * 1_000.0,
        }
    }
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_and_dc_match_lowpass_response() {
        let model = Model::default();
        assert_eq!(model.value(Response::Magnitude, 0.0), 0.0);
        assert_eq!(model.value(Response::Phase, 0.0), 0.0);
        let expected_db = 20.0 * model.q.log10();
        assert!((model.value(Response::Magnitude, model.cutoff) - expected_db).abs() < 1e-12);
        let expected_phase = -90.0 - 360.0 * model.cutoff * model.transport_delay_s;
        assert!((model.value(Response::Phase, model.cutoff) - expected_phase).abs() < 1e-9);
    }

    #[test]
    fn analytic_group_delay_matches_unwrapped_phase_derivative() {
        for q in [0.2, 0.707, 1.0, 4.0] {
            let model = Model {
                q,
                ..Model::default()
            };
            for frequency in [10.0, 100.0, 1_000.0, 1_600.0, 2_000.0, 20_000.0] {
                let step = frequency * 1e-5;
                let derivative = (model.value(Response::Phase, frequency + step)
                    - model.value(Response::Phase, frequency - step))
                    / (2.0 * step);
                let numerical_ms = -derivative / 360.0 * 1_000.0;
                let analytic_ms = model.value(Response::GroupDelay, frequency);
                assert!(
                    (numerical_ms - analytic_ms).abs() < 1e-7,
                    "q={q}, f={frequency}: numeric={numerical_ms}, analytic={analytic_ms}"
                );
            }
        }
    }

    #[test]
    fn excess_delay_is_transport_delay_relative_to_minimum_phase() {
        for delay in [0.0, 0.002, 0.015] {
            let model = Model {
                transport_delay_s: delay,
                ..Model::default()
            };
            let minimum_phase = Model {
                transport_delay_s: 0.0,
                ..model
            };
            for frequency in [0.0, 20.0, 1_600.0, 20_000.0] {
                let excess = model.value(Response::ExcessDelay, frequency);
                assert_eq!(excess, delay * 1_000.0);
                assert!(
                    (model.value(Response::GroupDelay, frequency)
                        - minimum_phase.value(Response::GroupDelay, frequency)
                        - excess)
                        .abs()
                        < 1e-12
                );
                assert_eq!(
                    model.value(Response::Magnitude, frequency),
                    minimum_phase.value(Response::Magnitude, frequency)
                );
            }
        }
    }

    #[test]
    fn responses_are_finite_and_phase_continuous() {
        for model in [
            Model::default(),
            Model {
                cutoff: 0.0,
                q: f64::NAN,
                transport_delay_s: f64::INFINITY,
            },
            Model {
                cutoff: f64::MAX,
                q: f64::MAX,
                transport_delay_s: f64::MAX,
            },
        ] {
            for frequency in [
                f64::NAN,
                f64::NEG_INFINITY,
                -1.0,
                0.0,
                1.0,
                20_000.0,
                f64::MAX,
            ] {
                for response in Response::ALL {
                    assert!(model.value(response, frequency).is_finite());
                }
            }
        }
        let model = Model::default();
        let before = model.value(Response::Phase, model.cutoff - 0.001);
        let after = model.value(Response::Phase, model.cutoff + 0.001);
        assert!(after < before && before - after < 0.01);
        assert!(model.value(Response::Phase, 20_000.0) < -360.0);
    }
}
