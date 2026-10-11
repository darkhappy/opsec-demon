//! Original, non-explicit fan tribute; all story and dialogue are new.
const STORY: &str = include_str!("../assets/story/lost-signals.md");

#[derive(Default)]
pub struct Novel {
    scene: Option<usize>,
    open_choices: u8,
}
impl Novel {
    pub fn page(&self) -> String {
        let Some(scene) = self.scene else {
            return "LOST SIGNALS // original Lost Child-inspired tribute\nnovel start / novel next / novel choose 1|2 / psyche".into();
        };
        let text = STORY.split("\n---\n").nth(scene).unwrap_or_default().trim();
        let prompt = match scene {
            3 => "\n[1] Knock and listen.  [2] Restore power and identify.\nnovel choose 1|2",
            6 => "\n[1] Keep the recording.  [2] Accept the bargain.\nnovel choose 1|2",
            10 | 11 => "\nTHE END / novel reset to return to the station",
            _ => "\nnovel next / PgUp and PgDn scroll the full transcript",
        };
        format!("LOST SIGNALS / {text}\n{prompt}")
    }
    pub fn command(&mut self, args: &[String]) -> Result<String, String> {
        match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] | ["status"] => Ok(self.page()),
            ["start"] | ["reset"] => {
                self.scene = Some(0);
                self.open_choices = 0;
                Ok(self.page())
            }
            ["next"] => {
                let scene = self.scene.ok_or("Use novel start first")?;
                if matches!(scene, 3 | 6) {
                    return Err("Choose 1 or 2 before continuing".into());
                }
                if scene >= 10 {
                    return Err("The story has ended; use novel reset".into());
                }
                self.scene = Some(if scene == 9 {
                    if self.open_choices == 2 { 10 } else { 11 }
                } else {
                    scene + 1
                });
                Ok(self.page())
            }
            ["choose", choice @ ("1" | "2")] => {
                let scene = self.scene.ok_or("Use novel start first")?;
                if !matches!(scene, 3 | 6) {
                    return Err("There is no pending choice in this scene".into());
                }
                if *choice == "1" {
                    self.open_choices += 1;
                }
                self.scene = Some(scene + 1);
                Ok(self.page())
            }
            _ => Err("usage: novel [start|next|choose 1|2|status|reset]".into()),
        }
    }
    pub fn psyche(&self) -> String {
        let tendency = match self.open_choices {
            0 => "The protagonist reaches for structure when uncertainty becomes loud.",
            1 => "The protagonist alternates between protecting a boundary and inviting contact.",
            _ => "The protagonist makes room for another voice before resolving the uncertainty.",
        };
        format!(
            "[PSYCHE / FICTIONAL CHARACTER ANALYSIS]\n{tendency}\nTheme: armor as a coping metaphor; the blue cup as continuity; Tweeter as witness; Krillion as plural memory.\nThis describes the story protagonist, not the reader or a clinical diagnosis.\nChoices toward listening: {} / 2; story active: {}",
            self.open_choices,
            self.scene.is_some()
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn route(choice: &str) -> Novel {
        let mut n = Novel::default();
        n.command(&["start".into()]).unwrap();
        for _ in 0..3 {
            n.command(&["next".into()]).unwrap();
        }
        assert!(n.command(&["next".into()]).is_err());
        n.command(&["choose".into(), choice.into()]).unwrap();
        for _ in 0..2 {
            n.command(&["next".into()]).unwrap();
        }
        n.command(&["choose".into(), choice.into()]).unwrap();
        for _ in 0..3 {
            n.command(&["next".into()]).unwrap();
        }
        n
    }
    #[test]
    fn choices_produce_distinct_endings_and_reset_clears_state() {
        let mut open = route("1");
        let guarded = route("2");
        assert!(open.page().contains("ENDING: THE OPEN CHANNEL"));
        assert!(guarded.page().contains("ENDING: THE GUARDED CHANNEL"));
        assert!(open.command(&["next".into()]).is_err());
        open.command(&["reset".into()]).unwrap();
        assert_eq!(open.open_choices, 0);
        assert!(open.page().contains("01 / THE STATION"));
    }
    #[test]
    fn invalid_choices_do_not_change_state_and_story_is_about_two_thousand_words() {
        let mut n = Novel::default();
        assert!(n.command(&["choose".into(), "1".into()]).is_err());
        assert!(n.command(&["choose".into(), "9".into()]).is_err());
        let count = STORY.split_whitespace().count();
        assert!((1900..=2400).contains(&count), "story has {count} words");
        assert_eq!(STORY.split("\n---\n").count(), 12);
    }
}
