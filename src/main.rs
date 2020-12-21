use maplit::hashmap;
use std::sync::mpsc::channel;
use stringlit::s;
use twitch_chat_wrapper::{run, ChatMessage};

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<ChatMessage>();

    let simple = hashmap! {
        s!("!ping") => s!("pong"),
        s!("!git") => s!("https://github.com/hardliner66"),
        s!("!github")  => s!("https://github.com/hardliner66"),
        s!("!gh") => s!("https://github.com/hardliner66"),
        s!("!boss") => s!("toggle8Boss"),
        s!("!cat") => s!("toggle8Catpeasant"),
        s!("!catpeasant") => s!("toggle8Catpeasant"),
        s!("!pog") => s!("PogChamp"),
        s!("!lul") => s!("LUL"),
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
                    } else if message.starts_with("!") {
                        Some(s!("No hacking allowed!"))
                    } else {
                        None
                    }
                })
            } {
                tx.send(response).unwrap();
            }
        }
    });

    Ok(run(rx, tx2).unwrap())
}
