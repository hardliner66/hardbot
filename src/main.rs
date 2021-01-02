use std::collections::HashMap;
use std::sync::mpsc::channel;
use stringlit::s;
use twitch_chat_wrapper::{run, ChatMessage};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    variables: HashMap<String, String>,
    commands: HashMap<String, String>,
}

const DEFAULT_CONFIG: &str = include_str!("../defaults.toml");

impl Default for Config {
  fn default() -> Self {
      toml::from_str(DEFAULT_CONFIG).unwrap()
  }
}

fn handle_msg(cfg: &Config, msg: &ChatMessage) -> Option<String> {
    let message = msg.message.to_lowercase();

    cfg.commands.get(&message).cloned().map(|msg| {
        let mut msg = msg;
        if msg.contains('$') {
            for (name, value) in &cfg.variables {
                msg = msg.replace(&format!("${}", name), &value);
            }
        }
        msg
    }).or_else(|| {
        if message.starts_with("!commands") {
            Some( cfg.commands.keys().cloned().collect::<Vec<_>>().join(" | ") )
        } else if message.starts_with("!hype") {
            Some(
                message
                    .chars()
                    .filter(|&c| c == 'e')
                    .take(30)
                    .map(|_| "iamhar2Bob")
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

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<ChatMessage>();

    std::thread::spawn(move || {
        let mut home = dirs::home_dir().unwrap();
        home.push(".config/hardbot/config.toml");
        
        let config: Config = std::fs::read_to_string(home).iter().flat_map(|s| {
          toml::from_str(&s)
        }).next().unwrap_or_default();
        while let Ok(msg) = rx2.recv() {
            if let Some(response) = {
                handle_msg(&config, &msg)
            } {
                for msg in response.split('\n') {
                    tx.send(msg.to_owned()).unwrap();
                }
            }
        }
    });

    Ok(run(rx, tx2).unwrap())
}
