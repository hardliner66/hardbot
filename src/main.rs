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
        "!ping" => "pong".to_string(),
        "!git" => GITHUB.to_string(),
        "!github" => GITHUB.to_string(),
        "!gh" => GITHUB.to_string(),
        "!boss" => "toggle8Boss".to_string(),
        "!toggle" => TOGGLEBIT.to_string(),
        "!togglebit" => TOGGLEBIT.to_string(),
        "!cat" => CATPEASANT.to_string(),
        "!catpeasant" => CATPEASANT.to_string(),
        "!arctic" => ARCTIC.to_string(),
        "!arcticspacefox" => ARCTIC.to_string(),
        "!bob" => "iamhar2Bob".to_string(),
        "!energy" => "iamhar2Energy".to_string(),
        "!pog" => "PogChamp".to_string(),
        "!lul" => "LUL".to_string(),
        "!lurk" => "Have fun lurking! iamhar2Bob".to_string(),
        "!os" => "Hardliner is using Manjaro.".to_string(),
        "!commands" => COMMANDS.join(" | "),
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

    println!("{:?}", handle_msg("!os"));

    Ok(())
}
