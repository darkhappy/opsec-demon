use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const SIGIL: &str = r"
             /\                 /\
            /  \___         ___/  \
           /       \_______/       \
           \   ▄██▄         ▄██▄   /
            \  ▀██▀   /\    ▀██▀  /
             \       /  \        /
              \  \_________/   /
               \  V V V V V   /
                \____________/
       O P S E C   //   D E M O N
          THE MAINFRAME KNOWS YOUR NAME";

pub const MISSIONS: &[(&str, &str)] = &[
    (
        "ACT I // THE SIGNAL",
        "Run mainframe ping. Make the machine answer.",
    ),
    (
        "ACT II // KNOW YOUR TERRITORY",
        "Run pwd, then ls -lah. Every legend starts somewhere.",
    ),
    (
        "ACT III // THE PIPELINE",
        "Run printf 'signal acquired\\n' | cat. Route the signal.",
    ),
    (
        "ACT IV // CLAIM YOUR NAME",
        "Run callsign YOUR_NAME. Leave a name in the mythology.",
    ),
    (
        "ACT V // THE OPERATOR",
        "Build something real. The console is yours now.",
    ),
];

pub fn rank(operations: u64) -> &'static str {
    match operations {
        0..=4 => "GHOST IN THE WIRES",
        5..=14 => "MAINFRAME REVENANT",
        15..=29 => "NEON ARCHDEMON",
        _ => "ROOT OF ALL LORE",
    }
}

pub const BOOT_DURATION: Duration = Duration::from_secs(15);

pub const POOP_SIGIL: &str = r"
                    (  )
                     )(     ~ ~
                    /  \
                   /____\
                  / o  o \
                 /   __   \
                /__________\
           P O O P S E C  D E M O N
           THE MAINFRAME HAS BEEN CLOGGED";

pub fn boot(timed: bool, color: bool, accent: &str, poopsec: bool) -> io::Result<()> {
    let started = Instant::now();
    let stages = if poopsec {
        [
            "SUMMONING THE PORCELAIN CORE",
            "ROUTING THROUGH THE SEWER MAINFRAME",
            "FLUSHING THE FICTIONAL FIREWALL",
            "ACQUIRING THE GOLDEN THRONE",
        ]
    } else {
        [
            "SUMMONING THE RUST CORE",
            "DECODING THE FORBIDDEN SIGIL",
            "INFILTRATING THE SIMULATED MAINFRAME",
            "CALIBRATING PROTAGONIST ENERGY",
        ]
    };
    for (index, stage) in stages.iter().enumerate() {
        for fill in 0..=12 {
            if color {
                print!("\x1b[{accent}m");
            }
            print!(
                "\r  {:02} [{}{}] {stage}",
                index + 1,
                "━".repeat(fill),
                "░".repeat(12 - fill)
            );
            if color {
                print!("\x1b[0m");
            }
            io::stdout().flush()?;
            if timed {
                let deadline = BOOT_DURATION.mul_f64((index * 13 + fill + 1) as f64 / 52.0);
                if let Some(delay) = deadline.checked_sub(started.elapsed()) {
                    thread::sleep(delay);
                }
            }
        }
        println!();
    }
    Ok(())
}

pub fn matrix(rows: usize) -> String {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |time| time.as_nanos() as u64)
        | 1;
    let glyphs = b"0123456789ABCDEF:./<>";
    let mut output = String::with_capacity(rows * 65);
    for _ in 0..rows {
        for _ in 0..64 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            output.push(if seed.is_multiple_of(7) {
                ' '
            } else {
                glyphs[seed as usize % glyphs.len()] as char
            });
        }
        output.push('\n');
    }
    output
}

pub fn validate_callsign(name: &str) -> io::Result<()> {
    if name.is_empty()
        || name.len() > 24
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
    {
        return Err(io::Error::other(
            "callsign needs 1–24 ASCII letters, numbers, underscores, or hyphens",
        ));
    }
    Ok(())
}
