use tungstenite::{connect, Message};
use url::Url;
use maplit::hashmap;
use std::sync::mpsc::channel;
use stringlit::s;

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

fn handle_msg(message: &str) -> Option<String> {
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
        s!("!os") => "Hardliner is using Manjaro.".to_string(),
        s!("!commands") => COMMANDS.join(" | "),
    };

    simple.get(message).cloned().or_else(|| {
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
}

const WS_URL: &str = "wss://pubsub-edge.twitch.tv";

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<String>();


    std::thread::spawn(move || {
        // handle messages
    });

    let (mut socket, response) =
        connect(Url::parse(WS_URL).unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    // socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
    
    Ok(())
}
