use maplit::hashmap;
use std::sync::mpsc::channel;
use stringlit::s;
use twitch_chat_wrapper::{run, ChatMessage};

const COMMANDS: [&str; 11] = [
    "!ping",
    "!github | !git | !gh",
    "!boss",
    "!togglebit | !toggle",
    "!catpeasant | !cat",
    "!arcticspacefox | !arctic",
    "!bob",
    "!energy",
    "!pog",
    "!lul",
    "!hype",
];
const ARCTIC: &str = "Look what @ArcticSpaceFox made: iamhar2Bob iamhar2Energy";
const CATPEASANT: &str = "toggle8Catpeasant";
const TOGGLEBIT: &str = "https://www.twitch.tv/togglebit";
const GITHUB: &str = "https://github.com/hardliner66";

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<ChatMessage>();

    let simple = hashmap! {
        s!("!ping") => "pong".to_string(),
        s!("!git") => GITHUB.to_string(),
        s!("!github")  => GITHUB.to_string(),
        s!("!gh") => GITHUB.to_string(),
        s!("!boss") => "toggle8Boss".to_string(),
        s!("!toggle") => TOGGLEBIT.to_string(),
        s!("!togglebit") => TOGGLEBIT.to_string(),
        s!("!cat") => CATPEASANT.to_string(),
        s!("!catpeasant") => CATPEASANT.to_string(),
        s!("!arctic") => ARCTIC.to_string(),
        s!("!arcticspacefox") => ARCTIC.to_string(),
        s!("!bob") => "iamhar2Bob".to_string(),
        s!("!energy") => "iamhar2Energy".to_string(),
        s!("!pog") => "PogChamp".to_string(),
        s!("!lul") => "LUL".to_string(),
        s!("!lurk") => "Have fun lurking! iamhar2Bob".to_string(),
        s!("!commands") => COMMANDS.join(" | "),
    };

    std::thread::spawn(move || {
        while let Ok(msg) = rx2.recv() {
            if let Some(response) = {
                let message = msg.message.to_lowercase();
                simple.get(&message).cloned().or_else(|| {
                    if message.starts_with("!hype") {
                        Some(
                            message
                                .chars()
                                .filter(|&c| c == 'e')
                                .take(30)
                                .map(|_| "sinticaHype")
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
                    } else if message.starts_with("!") && !message.chars().all(|c| c == '!') {
                        Some(s!("No hacking allowed!\nUse !commands to see available commands."))
                    } else {
                        None
                    }
                })
            } {
                for msg in response.split("\n") {
                    tx.send(msg.to_owned()).unwrap();
                }
            }
        }
    });

    Ok(run(rx, tx2).unwrap())
}
