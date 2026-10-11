use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const TRACK: &[u8] = include_bytes!("../assets/audio/nightfall.wav");

#[cfg(windows)]
pub fn bundled_player(device: Option<&str>) -> io::Result<Command> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "opsec-nightfall-{}-{stamp}.wav",
        std::process::id()
    ));
    let text = path
        .to_str()
        .ok_or_else(|| io::Error::other("audio temporary path is not Unicode"))?;
    render(text)?;
    match player(text, device) {
        Ok(mut command) => {
            command.env("OPSEC_DEMON_TEMP_WAV", "1");
            Ok(command)
        }
        Err(error) => {
            let _ = fs::remove_file(path);
            Err(error)
        }
    }
}

pub fn info() -> &'static str {
    "[SOUNDTRACK / ORIGINAL] Nightfall: Tweeter & Krillion\n148 BPM / E minor / 32 bars / 44.1 kHz stereo PCM\nsoundtrack render FILE.wav / soundtrack play FILE.wav\nUse OPSEC_DEMON_AUDIO_DEVICE=plughw:CARD=YOUR_CARD,DEV=0 for Linux USB DAC playback."
}

pub fn render(path: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(TRACK)
}

pub fn player(path: &str, device: Option<&str>) -> io::Result<Command> {
    let path = Path::new(path).canonicalize()?;
    if cfg!(windows) {
        let mut command = Command::new("powershell.exe");
        command.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
            "$p = New-Object System.Media.SoundPlayer $env:OPSEC_DEMON_WAV; try { $p.PlaySync() } finally { $p.Dispose(); if ($env:OPSEC_DEMON_TEMP_WAV -eq '1') { Remove-Item -LiteralPath $env:OPSEC_DEMON_WAV } }"]);
        command.env("OPSEC_DEMON_WAV", path);
        Ok(command)
    } else if cfg!(target_os = "macos") {
        let mut command = Command::new("afplay");
        command.arg(path);
        Ok(command)
    } else {
        let mut command = Command::new("aplay");
        if let Some(device) = device {
            if device.is_empty() || device.len() > 128 || device.chars().any(char::is_control) {
                return Err(io::Error::other(
                    "ALSA device must be 1-128 printable characters",
                ));
            }
            command.args(["-D", device]);
        }
        command.arg(path);
        Ok(command)
    }
}

pub fn dac_status(serial: bool) -> String {
    if serial {
        let ports = fs::read_dir("/dev")
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| {
                name.starts_with("ttyUSB") || name.starts_with("ttyACM") || name.starts_with("cu.")
            })
            .take(32)
            .collect::<Vec<_>>();
        return format!(
            "[SERIAL / READ-ONLY DISCOVERY] {}\nScarlett Solo 3 playback uses USB audio; no UART commands are sent.",
            if ports.is_empty() {
                "No serial ports discovered".into()
            } else {
                ports.join(", ")
            }
        );
    }
    let cards = fs::read_to_string("/proc/asound/cards").unwrap_or_else(|_| "No Linux ALSA card list is visible in this environment. On macOS, select the interface in system audio settings.".into());
    format!(
        "[DAC / USB AUDIO] Focusrite Scarlett Solo 3 playback adapter\n{}\nFor Linux device identifiers: shell aplay -L\nSet export OPSEC_DEMON_AUDIO_DEVICE=plughw:CARD=YOUR_CARD,DEV=0 then soundtrack play FILE.wav.\nPresence and audibility require a host-visible device; this report does not claim hardware detection.",
        cards.chars().take(4096).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_score_is_stereo_pcm_with_consistent_lengths() {
        assert_eq!(&TRACK[..4], b"RIFF");
        assert_eq!(&TRACK[8..12], b"WAVE");
        assert_eq!(u16::from_le_bytes(TRACK[22..24].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(TRACK[24..28].try_into().unwrap()), 44100);
        assert_eq!(
            u32::from_le_bytes(TRACK[40..44].try_into().unwrap()) as usize + 44,
            TRACK.len()
        );
    }
    #[test]
    fn device_selection_is_an_argument_not_shell_code() {
        if cfg!(target_os = "linux") {
            let command = player("Cargo.toml", Some("plughw:CARD=Solo,DEV=0")).unwrap();
            let args: Vec<_> = command.get_args().collect();
            assert_eq!(args[0], "-D");
            assert_eq!(args[1], "plughw:CARD=Solo,DEV=0");
            assert!(player("Cargo.toml", Some("bad\nname")).is_err());
        }
    }
}
