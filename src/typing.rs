use log::error;
use regex::{Match, Regex};
use serde::{Deserialize, Serialize};
use std::panic::catch_unwind;
use std::time::Duration;
use winput::{send, Vk};

static TAB_REGEX: &str = r"\{TAB\}";
static ENTER_REGEX: &str = r"\{ENTER\}";
static SLEEP_REGEX: &str = r"\{SLEEP=(\d+)\}";
static DEFAULT_SLEEP_MILLIS: u64 = 20;

pub fn send_raw_string(string: String) {
    for cmd in to_command_stream(string) {
        std::thread::sleep(Duration::from_millis(DEFAULT_SLEEP_MILLIS));
        handle_cmd(cmd)
    }
}

pub fn to_command_stream(string: String) -> Vec<Command> {
    let string = string;
    let mut commands_positions: Vec<(usize, Command)> = string
        .char_indices()
        .map(|(i, c)| (i, Command::Char(c)))
        .collect();

    replace_chars_with_cmd(&string, TAB_REGEX, &mut commands_positions, |_| {
        Command::Tab
    });
    replace_chars_with_cmd(&string, ENTER_REGEX, &mut commands_positions, |_| {
        Command::Enter
    });
    replace_chars_with_cmd(&string, SLEEP_REGEX, &mut commands_positions, |group_one| {
        let millis_str = group_one.unwrap().as_str();
        match millis_str.parse() {
            Ok(millis) => Command::Sleep(millis),
            Err(_) => {
                error!(
                    "Failed to parse millis from string {}, ignoring command",
                    millis_str
                );
                Command::Void
            }
        }
    });

    commands_positions.sort_by_key(|tuple| tuple.0);
    commands_positions.drain(..).map(|tuple| tuple.1).collect()
}

fn replace_chars_with_cmd<F>(
    string: &str,
    pattern: &str,
    commands_set: &mut Vec<(usize, Command)>,
    cmd_gen: F,
) where
    F: Fn(Option<Match>) -> Command,
{
    let regex = Regex::new(pattern).unwrap();
    for c in regex.captures_iter(string) {
        let full_match = c.get(0).unwrap();
        let group_one = c.get(1);
        // Retain the Command::Char that are not covered by the match
        commands_set.retain(|(i, _)| i < &full_match.start() || &full_match.end() <= i);
        // Add our own
        commands_set.push((full_match.start(), cmd_gen(group_one)));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialOrd, PartialEq, Hash, Eq)]
pub enum Command {
    Tab,
    Enter,
    Char(char),
    Sleep(u64), // in milliseconds
    Void,       // Does nothing
}

pub fn send_serialized_cmd(cmd: String) {
    match serde_json::from_str(&cmd) {
        Ok(cmd) => handle_cmd(cmd),
        Err(e) => error!("Failed to deserialize command: {}, {}", cmd, e),
    }
}

pub fn handle_cmd(cmd: Command) {
    match cmd {
        Command::Tab => send_char('\t'),
        Command::Enter => send_char('\n'),
        Command::Char(c) => send_char(c),
        Command::Sleep(millis) => std::thread::sleep(Duration::from_millis(millis)),
        Command::Void => {}
    }
}

fn send_char(c: char) {
    if catch_unwind(|| match c {
        '\t' => send(Vk::Tab),
        '\n' => send(Vk::Enter),
        _ => send(c),
    })
    .is_err()
    {
        error!("Failed to send keystroke for character");
    }
}
