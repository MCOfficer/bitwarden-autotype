use log::error;
use std::panic::catch_unwind;
use std::time::Duration;
use winput::send;

pub static CMD_START: char = '\x27';
pub static CMD_END: char = '\x04';
pub static SLEEP_CMD: &str = "SLEEP";
pub static TYPING_INTERVAL_MILLIS: u64 = 20;

pub fn send_char_stream<I>(i: I)
where
    I: IntoIterator<Item = char>,
{
    let mut i = i.into_iter();
    loop {
        match i.next() {
            None => {
                break;
            }
            Some(c) => {
                dbg!(&c);
                if c == CMD_START {
                    let cmd = i
                        .by_ref()
                        .take_while(|c| !CMD_END.eq(c))
                        .collect::<String>();
                    handle_cmd(cmd);
                } else {
                    send_char(c);
                    std::thread::sleep(Duration::from_millis(TYPING_INTERVAL_MILLIS));
                }
            }
        }
    }
}

fn handle_cmd(cmd: String) {
    if cmd.starts_with(SLEEP_CMD) {
        match cmd.strip_prefix(SLEEP_CMD).unwrap().parse::<u64>() {
            Ok(millis) => std::thread::sleep(Duration::from_millis(millis)),
            Err(_) => {
                error!("Failed to parse sleep command: {}", cmd)
            }
        }
    } else {
        error!("Got unknown command {}", cmd);
    }
}

fn send_char(c: char) {
    if catch_unwind(|| match c {
        '\t' => send(winput::Vk::Tab),
        '\n' => send(winput::Vk::Enter),
        _ => send(c),
    })
    .is_err()
    {
        error!("Failed to send keystroke for character");
    }
}
