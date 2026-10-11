//! A bounded PCM mixer feeding the Windows default output. No external audio runtime.
use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

pub const SOUNDS: [&str; 11] = [
    "Nightfall soundtrack",
    "Drummer explosion",
    "SSD explosion",
    "Pedal stomp",
    "Rhythm BT hit",
    "Rhythm FX hit",
    "Laser knob",
    "Page navigation",
    "Command submission",
    "Error alert",
    "Celebration",
];
#[derive(Clone, Debug)]
pub struct Gains {
    pub master: f32,
    pub music: f32,
    pub effects: f32,
    pub interface: f32,
    pub sounds: [f32; 11],
}
impl Default for Gains {
    fn default() -> Self {
        Self {
            master: 0.65,
            music: 0.75,
            effects: 0.6,
            interface: 0.35,
            sounds: [1.; 11],
        }
    }
}
impl Gains {
    fn gain(&self, sound: usize) -> f32 {
        let bus = if sound == 0 {
            self.music
        } else if (7..=9).contains(&sound) {
            self.interface
        } else {
            self.effects
        };
        self.master * bus * self.sounds[sound]
    }
}
#[derive(Default)]
struct State {
    gains: Gains,
    music: bool,
    events: Vec<usize>,
}
struct Voice {
    sound: usize,
    frame: usize,
}
fn sample_voice(sound: usize, frame: usize) -> f32 {
    let t = frame as f32 / 44100.;
    let durations = [0., 1.2, 0.9, 0.35, 0.15, 0.35, 0.18, 0.1, 0.13, 0.3, 0.8];
    if t >= durations[sound] {
        return 0.;
    }
    let envelope = (1. - t / durations[sound]).powi(2) * (t * 800.).min(1.);
    let phase = std::f32::consts::TAU * t;
    // Deterministic noise makes explosions testable; tones identify the other cues.
    let n = (frame as u32)
        .wrapping_mul(747796405)
        .wrapping_add(2891336453);
    let noise = ((n ^ (n >> 16)) & 65535) as f32 / 32768. - 1.;
    let tone = match sound {
        1 | 2 => noise * 0.6 + (phase * (48. + 12. * (-t * 15.).exp())).sin() * 0.4,
        3 => (phase * 110.).sin(),
        4 => (phase * 660.).sin(),
        5 => (phase * (220. + 700. * t)).sin(),
        6 => (phase * (900. - 2000. * t)).sin(),
        7 => (phase * 880.).sin(),
        8 => (phase * 1320.).sin(),
        9 => (phase * 180.).sin(),
        _ => ((phase * 523.25).sin() + (phase * 659.25).sin() + (phase * 783.99).sin()) / 3.,
    };
    tone * envelope * 0.3
}
fn mix_block(state: &mut State, position: &mut usize, voices: &mut Vec<Voice>, out: &mut [i16]) {
    for sound in state.events.drain(..) {
        if voices.len() < 32 {
            voices.push(Voice { sound, frame: 0 });
        }
    }
    let track = crate::soundtrack::embedded_pcm();
    let frames = track.len() / 4;
    for pair in out.chunks_exact_mut(2) {
        let fx: f32 = voices
            .iter_mut()
            .map(|voice| {
                let value = sample_voice(voice.sound, voice.frame) * state.gains.gain(voice.sound);
                voice.frame += 1;
                value
            })
            .sum();
        for (channel, value) in pair.iter_mut().enumerate() {
            let offset = (*position % frames) * 4 + channel * 2;
            let music = if state.music {
                i16::from_le_bytes([track[offset], track[offset + 1]]) as f32 / 32768.
                    * state.gains.gain(0)
            } else {
                0.
            };
            *value = ((music + fx).clamp(-1., 1.) * 32767.) as i16;
        }
        if state.music {
            *position = (*position + 1) % frames;
        }
    }
    voices.retain(|voice| voice.frame < 44100 * 2);
}

#[repr(C)]
struct Format {
    tag: u16,
    channels: u16,
    rate: u32,
    bytes: u32,
    align: u16,
    bits: u16,
    extra: u16,
}
#[repr(C)]
struct Header {
    data: *mut i16,
    length: u32,
    recorded: u32,
    user: usize,
    flags: u32,
    loops: u32,
    next: *mut Header,
    reserved: usize,
}
#[link(name = "winmm")]
unsafe extern "system" {
    fn waveOutOpen(
        handle: *mut usize,
        device: u32,
        format: *const Format,
        callback: usize,
        instance: usize,
        flags: u32,
    ) -> u32;
    fn waveOutPrepareHeader(handle: usize, header: *mut Header, size: u32) -> u32;
    fn waveOutWrite(handle: usize, header: *mut Header, size: u32) -> u32;
    fn waveOutUnprepareHeader(handle: usize, header: *mut Header, size: u32) -> u32;
    fn waveOutReset(handle: usize) -> u32;
    fn waveOutClose(handle: usize) -> u32;
}
#[derive(Default)]
pub struct Mixer {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    playing: bool,
    completed: Arc<AtomicU64>,
}
impl Mixer {
    pub fn playing(&self) -> bool {
        self.playing
    }
    pub fn gains(&mut self, gains: Gains) {
        self.state.lock().unwrap().gains = gains;
    }
    pub fn cue(&mut self, sound: usize) {
        if self.worker.is_some() && sound > 0 && sound < SOUNDS.len() {
            let mut state = self.state.lock().unwrap();
            if state.events.len() < 32 {
                state.events.push(sound);
            }
        }
    }
    pub fn set(&mut self, enabled: bool) -> io::Result<()> {
        if enabled && self.worker.is_none() {
            let state = Arc::clone(&self.state);
            let stop = Arc::clone(&self.stop);
            let completed = Arc::clone(&self.completed);
            let (tx, rx) = std::sync::mpsc::sync_channel(1);
            let worker = thread::spawn(move || unsafe {
                let mut handle = 0;
                let format = Format {
                    tag: 1,
                    channels: 2,
                    rate: 44100,
                    bytes: 176400,
                    align: 4,
                    bits: 16,
                    extra: 0,
                };
                let result = waveOutOpen(&mut handle, u32::MAX, &format, 0, 0, 0);
                if result != 0 {
                    let _ = tx.send(result);
                    return;
                }
                let mut data: Vec<Box<[i16]>> =
                    (0..3).map(|_| vec![0; 2048].into_boxed_slice()).collect();
                let mut headers: Vec<Box<Header>> = data
                    .iter_mut()
                    .map(|buffer| {
                        Box::new(Header {
                            data: buffer.as_mut_ptr(),
                            length: 4096,
                            recorded: 0,
                            user: 0,
                            flags: 0,
                            loops: 0,
                            next: std::ptr::null_mut(),
                            reserved: 0,
                        })
                    })
                    .collect();
                let size = std::mem::size_of::<Header>() as u32;
                let mut prepared = 0;
                let mut error = 0;
                for header in &mut headers {
                    error = waveOutPrepareHeader(handle, &mut **header, size);
                    if error != 0 {
                        break;
                    }
                    prepared += 1;
                }
                let _ = tx.send(error);
                let (mut position, mut voices) = (0, Vec::new());
                if error == 0 {
                    while !stop.load(Ordering::Acquire) {
                        for (buffer, header) in data.iter_mut().zip(&mut headers) {
                            let flags = std::ptr::read_volatile(&header.flags);
                            if flags & 16 == 0 || flags & 1 != 0 {
                                if flags & 1 != 0 {
                                    completed.fetch_add(1, Ordering::Release);
                                }
                                mix_block(
                                    &mut state.lock().unwrap(),
                                    &mut position,
                                    &mut voices,
                                    buffer,
                                );
                                if waveOutWrite(handle, &mut **header, size) != 0 {
                                    stop.store(true, Ordering::Release);
                                    break;
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(2));
                    }
                }
                waveOutReset(handle);
                for header in headers.iter_mut().take(prepared) {
                    waveOutUnprepareHeader(handle, &mut **header, size);
                }
                waveOutClose(handle);
            });
            let result = rx.recv().unwrap_or(u32::MAX);
            if result != 0 {
                let _ = worker.join();
                return Err(io::Error::other(format!(
                    "Windows PCM output failed ({result})"
                )));
            }
            self.worker = Some(worker);
        }
        self.state.lock().unwrap().music = enabled;
        self.playing = enabled;
        Ok(())
    }
}
pub fn smoke() -> io::Result<()> {
    let mut mixer = Mixer::default();
    let gains = Gains {
        master: 0.,
        ..Default::default()
    };
    mixer.gains(gains);
    mixer.set(true)?;
    for sound in 1..SOUNDS.len() {
        mixer.cue(sound);
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while mixer.completed.load(Ordering::Acquire) < 6 {
        if mixer.stop.load(Ordering::Acquire) || std::time::Instant::now() > deadline {
            return Err(io::Error::other(
                "Windows PCM buffers did not complete playback",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    mixer.set(false)?;
    println!("[AUDIO] Windows PCM mixer completed six output buffers (master muted).");
    Ok(())
}
impl Drop for Mixer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sound_and_bus_mutes_are_independent_and_master_silences_everything() {
        for (sound, name) in SOUNDS.iter().enumerate().skip(1) {
            let mut state = State::default();
            state.events.push(sound);
            let mut output = [0; 2048];
            let mut position = 0;
            mix_block(&mut state, &mut position, &mut Vec::new(), &mut output);
            assert!(output.iter().any(|x| *x != 0), "{name}");
            state.gains.sounds[sound] = 0.;
            state.events.push(sound);
            mix_block(&mut state, &mut position, &mut Vec::new(), &mut output);
            assert!(output.iter().all(|x| *x == 0));
        }
        let mut state = State {
            music: true,
            ..Default::default()
        };
        state.gains.master = 0.;
        state.events.extend(1..SOUNDS.len());
        let mut output = [1; 2048];
        mix_block(&mut state, &mut 44100, &mut Vec::new(), &mut output);
        assert!(output.iter().all(|x| *x == 0));
    }
    #[test]
    fn bus_controls_preserve_the_other_buses_and_music_gain_changes_pcm() {
        let mut gains = Gains {
            effects: 0.,
            ..Default::default()
        };
        assert_eq!(gains.gain(1), 0.);
        assert!(gains.gain(0) > 0. && gains.gain(7) > 0.);
        gains.interface = 0.;
        assert_eq!(gains.gain(7), 0.);
        assert!(gains.gain(0) > 0.);
        let mut state = State {
            music: true,
            ..Default::default()
        };
        let mut full = [0; 2048];
        let mut quiet = [0; 2048];
        mix_block(&mut state, &mut 44100, &mut Vec::new(), &mut full);
        state.gains.music *= 0.5;
        mix_block(&mut state, &mut 44100, &mut Vec::new(), &mut quiet);
        assert!(full.iter().any(|x| x.abs() > 100));
        for (full, quiet) in full.iter().zip(quiet) {
            assert!((i32::from(*full) - 2 * i32::from(quiet)).abs() <= 1);
        }
    }
}
