//! Writable SET_ACTIVITY fields, bounded before sending to the local client.
use serde_json::{Value, json};
use std::io;

pub const REPOSITORY: &str = "https://github.com/darkhappy/opsec-demon";

#[derive(Clone)]
pub struct Profile {
    pub overrides: Value,
    pub large_images: Vec<String>,
    pub small_images: Vec<String>,
    pub timer: Timer,
}

#[derive(Clone, Copy)]
pub enum Timer {
    Elapsed,
    Off,
    Countdown { start: u64, end: u64 },
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            overrides: json!({
                "type": 0, "status_display_type": 2,
                "details_url": REPOSITORY, "state_url": format!("{REPOSITORY}#yoggies-mainframe-tray"),
                "assets": { "large_url": REPOSITORY, "small_url": format!("{REPOSITORY}#yoggies-mainframe-tray") },
                "buttons": [
                    { "label": "ENTER THE BLACK CHOIR", "url": REPOSITORY },
                    { "label": "REQUISITION YOGGIES", "url": format!("{REPOSITORY}#yoggies-mainframe-tray") }
                ],
                "instance": false
            }),
            large_images: vec!["nightfall".into(), "black_choir".into(), "yoggies".into()],
            small_images: vec!["yoggies".into(), "porcelain".into(), "ghost_kernel".into()],
            timer: Timer::Elapsed,
        }
    }
}

impl Profile {
    pub fn parse(text: &str) -> io::Result<Self> {
        let input: Value = serde_json::from_str(text)?;
        object(&input, &["activity", "large_images", "small_images"])?;
        let mut profile = Self::default();
        if let Some(activity) = input.get("activity") {
            validate_activity(activity)?;
            profile.overrides = activity.clone();
        }
        for (field, images) in [
            ("large_images", &mut profile.large_images),
            ("small_images", &mut profile.small_images),
        ] {
            if let Some(value) = input.get(field) {
                let values = value
                    .as_array()
                    .filter(|values| values.len() <= 32)
                    .ok_or_else(|| {
                        invalid("image rotations must be arrays of at most 32 assets")
                    })?;
                *images = values
                    .iter()
                    .map(|value| {
                        image(value)?;
                        Ok(value.as_str().unwrap().to_owned())
                    })
                    .collect::<io::Result<_>>()?;
            }
        }
        Ok(profile)
    }

    pub fn apply(&self, payload: &mut Value, transmission: usize) {
        for (key, value) in self.overrides.as_object().unwrap() {
            if key == "assets" {
                for (key, value) in value.as_object().unwrap() {
                    payload["assets"][key] = value.clone();
                }
            } else {
                payload[key] = value.clone();
            }
        }
        for (field, images) in [
            ("large_image", &self.large_images),
            ("small_image", &self.small_images),
        ] {
            if !images.is_empty() && self.overrides["assets"].get(field).is_none() {
                payload["assets"][field] = json!(images[transmission % images.len()]);
            }
        }
        // Explicit JSON timestamps take precedence over the session timer.
        if self.overrides.get("timestamps").is_none() {
            match self.timer {
                Timer::Elapsed => {}
                Timer::Off => {
                    payload.as_object_mut().unwrap().remove("timestamps");
                }
                Timer::Countdown { start, end } => {
                    payload["timestamps"] = json!({"start": start, "end": end});
                }
            }
        }
        for field in ["details", "state"] {
            if let Some(text) = payload[field].as_str()
                && !text.starts_with("SIM |")
            {
                payload[field] = json!(bounded(&format!("SIM | {text}"), 128));
            }
        }
    }
}

pub fn bounded(text: &str, limit: usize) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(limit)
        .collect()
}

fn invalid(message: &str) -> io::Error {
    io::Error::other(message)
}
fn object<'a>(value: &'a Value, keys: &[&str]) -> io::Result<&'a serde_json::Map<String, Value>> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("profile field must be an object"))?;
    if object.keys().any(|key| !keys.contains(&key.as_str())) {
        return Err(invalid("unknown or read-only Discord profile field"));
    }
    Ok(object)
}
fn text(value: &Value, max: usize) -> io::Result<()> {
    let text = value
        .as_str()
        .ok_or_else(|| invalid("profile field must be text"))?;
    if text.is_empty() || text.chars().count() > max || text.chars().any(char::is_control) {
        return Err(invalid(
            "profile text is empty, too long, or contains control characters",
        ));
    }
    Ok(())
}
fn url(value: &Value) -> io::Result<()> {
    text(value, 512)?;
    let url = value.as_str().unwrap();
    if !url.starts_with("https://")
        || url[8..].split('/').next().unwrap_or("").is_empty()
        || url.chars().any(char::is_whitespace)
    {
        return Err(invalid("presence links must be public HTTPS URLs"));
    }
    Ok(())
}
fn image(value: &Value) -> io::Result<()> {
    text(value, 512)?;
    if value.as_str().unwrap().contains("://") {
        url(value)?;
    } else if !value
        .as_str()
        .unwrap()
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
    {
        return Err(invalid(
            "images must be Discord asset keys/IDs or public HTTPS URLs",
        ));
    }
    Ok(())
}

fn validate_activity(value: &Value) -> io::Result<()> {
    let fields = object(
        value,
        &[
            "type",
            "status_display_type",
            "details",
            "state",
            "details_url",
            "state_url",
            "timestamps",
            "assets",
            "party",
            "secrets",
            "instance",
            "flags",
            "buttons",
        ],
    )?;
    for (key, value) in fields {
        match key.as_str() {
            "type" if !matches!(value.as_u64(), Some(0 | 2 | 3 | 5)) => {
                return Err(invalid(
                    "IPC supports playing=0, listening=2, watching=3, competing=5",
                ));
            }
            "status_display_type" if !matches!(value.as_u64(), Some(0..=2)) => {
                return Err(invalid("display must be name=0, state=1, details=2"));
            }
            "flags" if !matches!(value.as_u64(), Some(0..=511)) => {
                return Err(invalid("activity flags must be a nine-bit mask"));
            }
            "instance" if !value.is_boolean() => return Err(invalid("instance must be boolean")),
            "details" | "state" => text(value, 128)?,
            "details_url" | "state_url" => url(value)?,
            "assets" => {
                for (key, value) in object(
                    value,
                    &[
                        "large_image",
                        "large_text",
                        "large_url",
                        "small_image",
                        "small_text",
                        "small_url",
                        "invite_cover_image",
                    ],
                )? {
                    if key.ends_with("image") {
                        image(value)?;
                    } else if key.ends_with("url") {
                        url(value)?;
                    } else {
                        text(value, 128)?;
                    }
                }
            }
            "timestamps" => {
                let times = object(value, &["start", "end"])?;
                if times.values().any(|v| !v.is_u64()) {
                    return Err(invalid(
                        "timestamps must be nonnegative Unix seconds for IPC",
                    ));
                }
                if let (Some(start), Some(end)) = (times.get("start"), times.get("end"))
                    && start.as_u64() > end.as_u64()
                {
                    return Err(invalid("timer end precedes start"));
                }
            }
            "party" => {
                for (key, value) in object(value, &["id", "size", "privacy"])? {
                    match key.as_str() {
                        "id" => text(value, 128)?,
                        "privacy" if !matches!(value.as_u64(), Some(0 | 1)) => {
                            return Err(invalid("party privacy must be private=0 or public=1"));
                        }
                        "size" => {
                            let size = value
                                .as_array()
                                .filter(|v| v.len() == 2)
                                .ok_or_else(|| invalid("party size must be [current, maximum]"))?;
                            let current = size[0]
                                .as_u64()
                                .ok_or_else(|| invalid("invalid party count"))?;
                            let maximum = size[1]
                                .as_u64()
                                .filter(|v| *v > 0 && *v <= i32::MAX as u64)
                                .ok_or_else(|| invalid("invalid party capacity"))?;
                            if current > maximum {
                                return Err(invalid("party count exceeds capacity"));
                            }
                        }
                        _ => {}
                    }
                }
            }
            "secrets" => {
                for value in object(value, &["join", "spectate", "match"])?.values() {
                    text(value, 128)?;
                }
            }
            "buttons" => {
                let buttons = value
                    .as_array()
                    .filter(|v| v.len() <= 2)
                    .ok_or_else(|| invalid("Discord allows at most two buttons"))?;
                for button in buttons {
                    object(button, &["label", "url"])?;
                    text(&button["label"], 32)?;
                    url(&button["url"])?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_profile_accepts_animation_and_all_writable_fields() {
        let profile =
            Profile::parse(include_str!("../docs/discord-presence.example.json")).unwrap();
        let mut payload = json!({"assets": {}, "details": "SIM | test", "state": "SIM | test", "timestamps": {"start": 42}});
        profile.apply(&mut payload, 1);
        assert_eq!(payload["buttons"].as_array().unwrap().len(), 2);
        assert_eq!(payload["type"], 5);
        assert_eq!(payload["assets"]["small_image"], "porcelain");
        assert!(payload["details"].as_str().unwrap().starts_with("SIM |"));
    }
    #[test]
    fn social_fields_and_explicit_timestamps_survive_without_inventing_callbacks() {
        let profile = Profile::parse(&json!({"activity": {
            "party": {"id": "external-session", "size": [2, 5], "privacy": 1},
            "secrets": {"join": "session-join", "spectate": "session-watch", "match": "session-match"},
            "timestamps": {"start": 123, "end": 456},
            "flags": 15, "instance": true,
            "assets": {"large_image": "https://example.com/choir.gif", "small_image": "https://example.com/berry.webp", "invite_cover_image": "banner"}
        }}).to_string()).unwrap();
        let mut payload = json!({"assets": {}, "details": "SIM | test", "state": "SIM | test"});
        profile.apply(&mut payload, 12);
        assert_eq!(payload["party"]["size"], json!([2, 5]));
        assert_eq!(payload["secrets"]["spectate"], "session-watch");
        assert_eq!(
            payload["assets"]["large_image"],
            "https://example.com/choir.gif"
        );
        assert_eq!(payload["timestamps"]["end"], 456);
        assert_eq!(payload["flags"], 15);
    }

    #[test]
    fn invalid_fields_types_links_and_sizes_are_rejected() {
        for activity in [
            json!({"type": 1}),
            json!({"type": 4}),
            json!({"name": "spoof"}),
            json!({"flags": 512}),
            json!({"state": "\u{1b}"}),
            json!({"instance": 1}),
            json!({"details_url": "file:///secret"}),
            json!({"party": {"size": [6, 5]}}),
            json!({"timestamps": {"start": 2, "end": 1}}),
            json!({"buttons": [{"label": "missing URL"}]}),
        ] {
            assert!(Profile::parse(&json!({"activity": activity}).to_string()).is_err());
        }
        assert!(Profile::parse("{\"large_images\": [\"/tmp/local.gif\"]}").is_err());
        assert!(Profile::parse("{\"large_images\": [\"https://example.com/choir.gif\"]}").is_ok());
    }
}
