//! Synth arrangements, including a short Snow halation melody quotation; no recordings.
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
    "Snow Halation / OpSecULAR 97 Credits",
];

pub fn catalog() -> String {
    let mut output = "[SYNTH AUDIO] SiIvaGunner-inspired melody-switch humor; unaffiliated.\nTrack 9 quotes Snow halation (composer: Takahiro Yamada / performed by μ's).\nNo sampled recordings. rips render ID FILE.wav (ID 1..9); soundtrack play FILE.wav\n".to_string();
    for (index, title) in TITLES.iter().enumerate() {
        output.push_str(&format!(
            "{}: {title} / {} BPM / 16 bars\n",
            index + 1,
            tempo(index + 1)
        ));
    }
    output
}

fn tempo(id: usize) -> usize {
    if id == 9 { 174 } else { 112 + (id - 1) * 7 }
}

// Short opening chorus hook, interpreted in concert A major. Durations are beats.
// Melody reference: https://notangkalagujepang.blogspot.com/2014/04/snow-halation-s-not-angka-pianika.html
const SNOW_HOOK: [(i32, f64); 10] = [
    (73, 1.5),
    (68, 0.25),
    (76, 0.25),
    (78, 2.0),
    (78, 0.5),
    (76, 0.5),
    (78, 0.5),
    (76, 0.5),
    (74, 0.5),
    (73, 1.5),
];

fn snow_note(beats: f64) -> (i32, f64) {
    let mut local = beats.rem_euclid(8.0);
    for (note, duration) in SNOW_HOOK {
        if local < duration {
            return (note, local);
        }
        local -= duration;
    }
    (73, 0.0)
}

pub fn siiva_catalog() -> String {
    let tracks = [
        ("Lifelight", "Geshtro", "2019/12/lifelight"),
        (
            "8 PM (Fishing Night in Heaven)",
            "Roc the Dilettante",
            "2019/12/8-pm-fishing-night-in-heaven",
        ),
        ("Main Menu", "Hinchy", "2019/11/main-menu"),
        ("Hilltop Rave", "RHMan", "2019/11/hilltop-rave"),
        ("Launch Base '93", "Hinchy", "2019/11/launch-base-93"),
        (
            "Funny Man Goes to the Tool Shop",
            "BobTheTacocat",
            "2019/11/funny-man-goes-to-the-tool-shop",
        ),
        ("NEO KOBE外伝", "Niko+", "2019/11/neo-kobe"),
        (
            "A Spacecraft Fueled on Fright",
            "Peacock Roy",
            "2019/11/a-spacecraft-fueled-on-fright",
        ),
        (
            "Reggie's Late Nite",
            "Dj CUTMAN",
            "2019/11/reggies-late-nite",
        ),
    ];
    let mut output = "[EXTERNAL MUSIC] SiIvaGunner / King for Another Day / MOJO!!\nOpen a credited track page in your browser to listen. Online availability may change.\nSource: https://mojo.highquality.rip/category/music/\n".to_string();
    output.push_str("Snow halation but it shreds / Joshua Taipale & Nolithium / SiIvaGunner\nhttps://www.youtube.com/watch?v=AEtEBleNwCQ\n");
    for (index, (title, arranger, slug)) in tracks.iter().enumerate() {
        output.push_str(&format!(
            "{}: {title} / arrangement: {arranger}\nhttps://mojo.highquality.rip/{slug}/\n",
            index + 1
        ));
    }
    output
}

fn score(id: usize) -> io::Result<Vec<u8>> {
    if !(1..=TITLES.len()).contains(&id) {
        return Err(io::Error::other("track ID must be 1..9"));
    }
    let beat = 60.0 / tempo(id) as f64;
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
        let snow_section = id == 9 && bar >= 8;
        let (note, note_time) = if snow_section {
            let (note, elapsed) = snow_note(beats - 32.0);
            (note, elapsed * beat)
        } else {
            (
                60 + id as i32 + root + melody[(step + id) % 8],
                t % (beat / 2.0),
            )
        };
        let frequency = 440.0 * 2_f64.powf((note as f64 - 69.0) / 12.0);
        let envelope = if snow_section {
            (note_time / 0.008).min(1.0) * (-1.8 * note_time).exp()
        } else {
            (-12.0 * note_time).exp()
        };
        let phase = std::f64::consts::TAU * frequency * t;
        let lead = (phase.sin() + 0.22 * (phase * (2 + id % 3) as f64).sin()) * envelope * 0.22;
        let bass_note = if snow_section {
            [38, 40, 42, 45][(bar - 8) / 2 % 4] as f64
        } else {
            36.0 + id as f64 + root as f64
        };
        let bass_frequency = 440.0 * 2_f64.powf((bass_note - 69.0) / 12.0);
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
    fn credits_audio_contains_snow_halation_pitches_after_the_switch() {
        let bytes = score(9).unwrap();
        let beat = 60.0 / 174.0;
        // Inspect the rendered PCM, not just the note table: C#5, G#4, E5.
        for (offset, midi) in [(0.15, 73.0), (1.56 * beat, 68.0), (1.81 * beat, 76.0)] {
            let start = ((32.0 * beat + offset) * RATE as f64) as usize;
            let magnitude = |note: f64| {
                let frequency = 440.0 * 2_f64.powf((note - 69.0) / 12.0);
                let (mut real, mut imaginary) = (0.0, 0.0);
                for sample in 0..882 {
                    let byte = 44 + (start + sample) * 4;
                    let value =
                        i16::from_le_bytes(bytes[byte..byte + 2].try_into().unwrap()) as f64;
                    let window = 0.5 - 0.5 * (std::f64::consts::TAU * sample as f64 / 881.0).cos();
                    let angle = std::f64::consts::TAU * frequency * sample as f64 / RATE as f64;
                    real += value * window * angle.cos();
                    imaginary += value * window * angle.sin();
                }
                real.hypot(imaginary)
            };
            assert!(magnitude(midi) > 2.0 * magnitude(midi - 2.0));
            assert!(magnitude(midi) > 2.0 * magnitude(midi + 2.0));
        }
    }

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
