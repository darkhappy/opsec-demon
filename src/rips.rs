//! Original melody-switch arrangements, with no sampled or third-party recordings.
use std::fs::OpenOptions;
use std::io::{self, Write};

const RATE: u32 = 22050;
pub const TITLES: [&str; 9] = [
    "Recycle Bin Bossa",
    "WSL Jade Boot",
    "Krillion Lane Counterpick",
    "Larp City Bus Ride",
    "Lost Signals Afterparty",
    "Curds of Destiny",
    "Mainframe Tears",
    "Serial Clown Inferno",
    "OpSecULAR 97 Credits",
];

pub fn catalog() -> String {
    let mut output = "[ORIGINAL AUDIO] SiIvaGunner-inspired melody-switch humor; unaffiliated.\nNine original synth arrangements, no SiIvaGunner recordings or game samples.\nrips render ID FILE.wav (ID 1..9); soundtrack play FILE.wav\n".to_string();
    for (index, title) in TITLES.iter().enumerate() {
        output.push_str(&format!(
            "{}: {title} / {} BPM / 16 bars\n",
            index + 1,
            112 + index * 7
        ));
    }
    output
}

fn score(id: usize) -> io::Result<Vec<u8>> {
    if !(1..=TITLES.len()).contains(&id) {
        return Err(io::Error::other("track ID must be 1..9"));
    }
    let beat = 60.0 / (112 + (id - 1) * 7) as f64;
    let duration = beat * 64.0;
    let frames = (duration * RATE as f64) as u32;
    let data_size = frames * 4;
    let mut bytes = Vec::with_capacity(data_size as usize + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&RATE.to_le_bytes());
    bytes.extend_from_slice(&(RATE * 4).to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    let motif = [0, 3, 7, 10, 7, 5, 3, 2];
    let surprise = [12, 7, 14, 10, 3, 17, 5, 0];
    for frame in 0..frames {
        let t = frame as f64 / RATE as f64;
        let beats = t / beat;
        let bar = (beats / 4.0) as usize;
        let step = (beats * 2.0) as usize;
        let root = [0, -5, 3, -2][(bar / 2 + id) % 4];
        let melody = if bar % 8 < 4 { &motif } else { &surprise };
        let note = 60 + id as i32 + root + melody[(step + id) % 8];
        let frequency = 440.0 * 2_f64.powf((note as f64 - 69.0) / 12.0);
        let envelope = (-12.0 * (t % (beat / 2.0))).exp();
        let phase = std::f64::consts::TAU * frequency * t;
        let lead = (phase.sin() + 0.22 * (phase * (2 + id % 3) as f64).sin()) * envelope * 0.22;
        let bass_frequency = 440.0 * 2_f64.powf((36.0 + id as f64 + root as f64 - 69.0) / 12.0);
        let bass = (std::f64::consts::TAU * bass_frequency * t).sin() * 0.13;
        let local = t % beat;
        let kick = (std::f64::consts::TAU * (48.0 * local + 5.0 * (1.0 - (-local * 25.0).exp())))
            .sin()
            * (-local * 22.0).exp()
            * 0.24;
        let fade = (t / 0.1).min((duration - t) / 0.4).clamp(0.0, 1.0);
        for pan in [0.92, 1.08] {
            let sample = ((lead * pan + bass + kick).tanh() * fade * 26000.0) as i16;
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
    }
    Ok(bytes)
}

pub fn render(id: usize, path: &str) -> io::Result<()> {
    let bytes = score(id)?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nine_distinct_scores_have_valid_pcm_headers_and_audible_frames() {
        let mut sizes = std::collections::BTreeSet::new();
        for id in 1..=9 {
            let bytes = score(id).unwrap();
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(&bytes[8..12], b"WAVE");
            assert_eq!(
                u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
                bytes.len() - 44
            );
            assert!(bytes[44..].iter().any(|b| *b != 0));
            assert!(sizes.insert(bytes.len()));
        }
        assert!(score(0).is_err());
        assert!(score(10).is_err());
    }
}
