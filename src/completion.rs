//! Bounded, read-only command and path completion with character-based cursors.
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const CHOICE_LIMIT: usize = 512;
const ENTRY_LIMIT: usize = 4096;
const BUILTINS: &[&str] = &[
    "help",
    "novel",
    "soundtrack",
    "dac",
    "psyche",
    "status",
    "cinematic",
    "callsign",
    "dossier",
    "mascot",
    "poopsec",
    "mission",
    "oracle",
    "matrix",
    "theme",
    "mainframe",
    "cd",
    "pwd",
    "history",
    "clear",
    "exit",
    "egg",
    "ritual",
    "bido",
    "bucket",
    "tailscale",
    "icons",
    "doctor",
    "mode",
    "expansion",
    "mcp",
    "ssd",
    "rt60",
    "room",
    "sdvx",
    "coins",
    "focus",
    "music",
    "graph",
    "shell",
    "export",
    "unset",
    "..advancedttk",
    "hoi4",
    "band",
    "pedal",
    "headphones",
];

#[derive(Debug)]
pub struct Completion {
    pub input: String,
    pub cursor: usize,
    pub choices: Vec<String>,
}

#[cfg(test)]
pub fn complete(input: &str, cursor: usize) -> Completion {
    complete_with_environment(input, cursor, &BTreeMap::new())
}

pub fn complete_with_environment(
    input: &str,
    cursor: usize,
    environment: &BTreeMap<String, Option<String>>,
) -> Completion {
    let cursor = cursor.min(input.chars().count());
    let cursor_byte = byte_index(input, cursor);
    let prefix = &input[..cursor_byte];
    let start = word_start(prefix);
    let (word, quote) = decode(&prefix[start..]);
    let before = prefix[..start].trim();
    let first_word = before.is_empty() || before.ends_with(['|', ';', '&']);
    let mut choices = BTreeSet::new();
    let mut scanned = 0;
    if first_word && !word.contains('/') {
        for name in BUILTINS {
            if name.starts_with(&word) {
                choices.insert((*name).to_string());
            }
        }
        if let Some(path) = environment_value("PATH", environment) {
            for directory in env::split_paths(&path) {
                if scanned >= ENTRY_LIMIT || choices.len() >= CHOICE_LIMIT {
                    break;
                }
                let directory = if directory.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    directory
                };
                scan(
                    &directory,
                    &mut scanned,
                    |entry| {
                        let name = entry.file_name().to_str()?.to_string();
                        if name.starts_with(&word)
                            && !name.chars().any(char::is_control)
                            && entry.metadata().ok().is_some_and(|metadata| {
                                metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
                            })
                        {
                            Some(name)
                        } else {
                            None
                        }
                    },
                    &mut choices,
                );
            }
        }
    }
    let (directory_text, name_prefix) = word.rfind('/').map_or(("", word.as_str()), |slash| {
        (&word[..slash + 1], &word[slash + 1..])
    });
    let directory = expand_directory(directory_text, environment);
    // File completion receives its own bounded budget even after a long PATH scan.
    let mut file_scanned = 0;
    scan(
        &directory,
        &mut file_scanned,
        |entry| {
            let name = entry.file_name().to_str()?.to_string();
            if name.chars().any(char::is_control)
                || !name.starts_with(name_prefix)
                || (name.starts_with('.') && !name_prefix.starts_with('.'))
            {
                return None;
            }
            let suffix = if entry.file_type().ok()?.is_dir() || entry.path().is_dir() {
                "/"
            } else {
                ""
            };
            Some(format!("{directory_text}{name}{suffix}"))
        },
        &mut choices,
    );
    let choices: Vec<_> = choices.into_iter().take(CHOICE_LIMIT).collect();
    let replacement = if choices.len() == 1 {
        choices[0].clone()
    } else {
        common_prefix(&choices)
    };
    if replacement.is_empty()
        || (choices.len() != 1 && replacement.chars().count() <= word.chars().count())
    {
        return Completion {
            input: input.into(),
            cursor,
            choices,
        };
    }
    let unique_file = choices.len() == 1 && !replacement.ends_with('/');
    let end = word_end(input, start, cursor_byte);
    let suffix = &input[end..];
    let mut rendered = encode(&replacement, quote);
    if unique_file {
        if let Some(mark) = quote {
            if !suffix.starts_with(mark) {
                rendered.push(mark);
                if !suffix.starts_with(char::is_whitespace) {
                    rendered.push(' ');
                }
            }
        } else if !suffix.starts_with(char::is_whitespace) {
            rendered.push(' ');
        }
    }
    if !unique_file
        && let Some(mark) = quote
        && input[start..end].ends_with(mark)
    {
        rendered.push(mark);
    }
    let completed = format!("{}{}{}", &input[..start], rendered, suffix);
    let new_cursor = input[..start].chars().count() + rendered.chars().count();
    Completion {
        input: completed,
        cursor: new_cursor,
        choices,
    }
}

fn scan(
    directory: &Path,
    scanned: &mut usize,
    mut candidate: impl FnMut(fs::DirEntry) -> Option<String>,
    choices: &mut BTreeSet<String>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries {
        if *scanned >= ENTRY_LIMIT || choices.len() >= CHOICE_LIMIT {
            break;
        }
        *scanned += 1;
        if let Ok(entry) = entry
            && let Some(name) = candidate(entry)
        {
            choices.insert(name);
        }
    }
}

fn environment_value(
    key: &str,
    environment: &BTreeMap<String, Option<String>>,
) -> Option<std::ffi::OsString> {
    match environment.get(key) {
        Some(value) => value.as_ref().map(std::ffi::OsString::from),
        None => env::var_os(key),
    }
}

fn expand_directory(text: &str, environment: &BTreeMap<String, Option<String>>) -> PathBuf {
    if let Some(rest) = text.strip_prefix("~/")
        && let Some(home) = environment_value("HOME", environment)
    {
        return PathBuf::from(home).join(rest);
    }
    if text.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(text)
    }
}

fn byte_index(text: &str, character: usize) -> usize {
    text.char_indices()
        .nth(character)
        .map_or(text.len(), |(index, _)| index)
}

fn word_start(text: &str) -> usize {
    let (mut start, mut quote, mut escaped) = (0, None, false);
    for (index, character) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(mark) = quote {
            if character == mark {
                quote = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if character.is_whitespace() || matches!(character, '|' | ';' | '&' | '<' | '>') {
            start = index + character.len_utf8();
        }
    }
    start
}

fn word_end(text: &str, start: usize, cursor: usize) -> usize {
    let (mut quote, mut escaped) = (None, false);
    for (offset, character) in text[start..].char_indices() {
        let index = start + offset;
        if escaped {
            escaped = false;
        } else if character == '\\' && quote != Some('\'') {
            escaped = true;
        } else if let Some(mark) = quote {
            if character == mark {
                quote = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if index >= cursor
            && (character.is_whitespace() || matches!(character, '|' | ';' | '&' | '<' | '>'))
        {
            return index;
        }
    }
    text.len()
}

fn decode(text: &str) -> (String, Option<char>) {
    let (mut word, mut quote, mut escaped) = (String::new(), None, false);
    for character in text.chars() {
        if escaped {
            word.push(character);
            escaped = false;
        } else if character == '\\' && quote != Some('\'') {
            escaped = true;
        } else if let Some(mark) = quote {
            if character == mark {
                quote = None;
            } else {
                word.push(character);
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else {
            word.push(character);
        }
    }
    (word, quote)
}

fn encode(word: &str, quote: Option<char>) -> String {
    if quote == Some('\'') {
        return format!("'{}", word.replace('\'', "'\\''"));
    }
    let mut text = if quote == Some('"') {
        "\"".into()
    } else {
        String::new()
    };
    for character in word.chars() {
        let escape = if quote == Some('"') {
            matches!(character, '"' | '\\' | '$' | '`')
        } else {
            !character.is_alphanumeric() && !matches!(character, '/' | '.' | '_' | '-' | '~' | ':')
        };
        if escape {
            text.push('\\');
        }
        text.push(character);
    }
    text
}

fn common_prefix(choices: &[String]) -> String {
    let Some(first) = choices.first() else {
        return String::new();
    };
    let mut common = first.clone();
    for choice in &choices[1..] {
        let count = common
            .chars()
            .zip(choice.chars())
            .take_while(|(left, right)| left == right)
            .count();
        common.truncate(byte_index(&common, count));
    }
    common
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path =
                env::temp_dir().join(format!("opsec-completion-{}-{unique}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn unique_file_escapes_spaces_and_preserves_input_suffix() {
        let directory = Directory::new();
        fs::write(directory.0.join("space file.txt"), "").unwrap();
        let prefix = format!("cat {}/spa", directory.0.display());
        let input = format!("{prefix} | cat");
        let result = complete(&input, prefix.chars().count());
        assert_eq!(
            result.input,
            format!("cat {}/space\\ file.txt | cat", directory.0.display())
        );
        assert_eq!(result.cursor, result.input.find(" | cat").unwrap());
    }

    #[test]
    fn quoted_multibyte_completion_uses_character_cursor() {
        let directory = Directory::new();
        fs::write(directory.0.join("猫 file.txt"), "").unwrap();
        let input = format!("cat \"{}/猫", directory.0.display());
        let result = complete(&input, input.chars().count());
        assert_eq!(
            result.input,
            format!("cat \"{}/猫 file.txt\" ", directory.0.display())
        );
        assert_eq!(result.cursor, result.input.chars().count());
    }

    #[test]
    fn directories_continue_and_multiple_files_extend_only_common_prefix() {
        let directory = Directory::new();
        fs::create_dir(directory.0.join("folder")).unwrap();
        let input = format!("cd {}/fol", directory.0.display());
        assert!(
            complete(&input, input.chars().count())
                .input
                .ends_with("/folder/")
        );
        let prefix = format!("cd \"{}/fol", directory.0.display());
        let result = complete(&format!("{prefix}wrong\" next"), prefix.chars().count());
        assert_eq!(
            result.input,
            format!("cd \"{}/folder/\" next", directory.0.display())
        );
        fs::write(directory.0.join("neon-a"), "").unwrap();
        fs::write(directory.0.join("neon-b"), "").unwrap();
        let input = format!("cat {}/ne", directory.0.display());
        let result = complete(&input, input.chars().count());
        assert!(result.input.ends_with("/neon-"));
        assert_eq!(result.choices.len(), 2);
    }

    #[test]
    fn missing_path_and_unextendable_choices_preserve_input() {
        let directory = Directory::new();
        let input = format!("cat {}/missing", directory.0.display());
        let result = complete(&input, input.chars().count());
        assert_eq!(result.input, input);
        assert!(result.choices.is_empty());
        assert_eq!(common_prefix(&["猫a".into(), "猫b".into()]), "猫");
        assert_eq!(decode("space\\ file"), ("space file".into(), None));
        assert_eq!(word_start("cat 'space file"), 4);
    }
    #[test]
    fn middle_of_word_replaces_tail_and_preserves_next_argument() {
        let directory = Directory::new();
        fs::write(directory.0.join("space file.txt"), "").unwrap();
        let prefix = format!("cat {}/spa", directory.0.display());
        let result = complete(&format!("{prefix}wrong next"), prefix.chars().count());
        assert_eq!(
            result.input,
            format!("cat {}/space\\ file.txt next", directory.0.display())
        );
        let prefix = format!("cat \"{}/spa", directory.0.display());
        let result = complete(&format!("{prefix}wrong\" next"), prefix.chars().count());
        assert_eq!(
            result.input,
            format!("cat \"{}/space file.txt\" next", directory.0.display())
        );
    }

    #[test]
    fn session_environment_controls_path_and_home_without_global_changes() {
        let directory = Directory::new();
        let name = "opsec_unique_session_binary";
        let executable = directory.0.join(name);
        fs::write(&executable, "").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        let mut environment = BTreeMap::from([
            (
                "PATH".into(),
                Some(directory.0.to_string_lossy().into_owned()),
            ),
            (
                "HOME".into(),
                Some(directory.0.to_string_lossy().into_owned()),
            ),
        ]);
        let result = complete_with_environment("opsec_unique_session_b", 22, &environment);
        assert_eq!(result.input, format!("{name} "));
        environment.insert("PATH".into(), None);
        assert!(
            complete_with_environment("opsec_unique_session_b", 22, &environment)
                .choices
                .is_empty()
        );
        let input = "cat ~/opsec_unique_session_b";
        assert_eq!(
            complete_with_environment(input, input.chars().count(), &environment).input,
            format!("cat ~/{name} ")
        );
        environment.insert("HOME".into(), None);
        assert!(
            complete_with_environment(input, input.chars().count(), &environment)
                .choices
                .is_empty()
        );
    }
}
