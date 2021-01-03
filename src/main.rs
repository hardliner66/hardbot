use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime};
use stringlit::s;
use twitch_chat_wrapper::{run, ChatMessage};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct PointConfig {
    ticks_per_message: i64,
    points_per_tick: u64,
    tick_speed: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    points: PointConfig,
    variables: HashMap<String, String>,
    commands: HashMap<String, String>,
}

struct Chatter {
    pub points: u64,
    pub remaining_ticks: i64,
}

impl Default for Chatter {
    fn default() -> Self {
        Chatter {
            points: 0,
            remaining_ticks: 0,
        }
    }
}

struct Runtime {
    pub points_config: PointConfig,
    pub commands: HashMap<String, String>,
    pub chatters: HashMap<String, Chatter>,
}

impl Into<Runtime> for Config {
    fn into(self) -> Runtime {
        let mut commands = HashMap::new();
        for (name, value) in self.commands {
            let mut value = value;
            for (variable_name, variable_value) in &self.variables {
                value = value.replace(&format!("${}", variable_name), &variable_value);
            }
            commands.insert(format!("!{}", name), value);
        }
        Runtime {
            commands,
            chatters: HashMap::new(),
            points_config: self.points.clone(),
        }
    }
}

const DEFAULT_CONFIG: &str = include_str!("../defaults.toml");

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }
}

fn handle_msg(rt: &mut Runtime, msg: &ChatMessage) -> Option<String> {
    println!("{}: {}", msg.name, msg.message);
    let message = msg.message.trim().to_lowercase();

    if !message.starts_with("!") {
        (*rt.chatters
            .entry(msg.name.clone())
            .or_insert(Default::default()))
        .remaining_ticks = rt.points_config.ticks_per_message;
    }

    rt.commands.get(&message).cloned().or_else(|| {
        if message.starts_with("!commands") {
            Some(rt.commands.keys().cloned().collect::<Vec<_>>().join(" | "))
        } else if message.starts_with("!points") {
            Some(format!(
                "@{} currently has {} points!",
                msg.name,
                rt.chatters
                    .get(&msg.name)
                    .map(|chatter| chatter.points)
                    .unwrap_or_default()
            ))
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
            Some(s!(
                "No hacking allowed!\nUse !commands to see available commands."
            ))
        } else {
            None
        }
    })
}

fn give_points(rt: &mut Runtime) {
    for (_, chatter) in rt.chatters.iter_mut() {
        if chatter.remaining_ticks > 0 {
            chatter.points += rt.points_config.points_per_tick;
            chatter.remaining_ticks -= 1;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<ChatMessage>();

    std::thread::spawn(move || {
        let mut home = dirs::home_dir().unwrap();
        home.push(".config/hardbot/config.toml");

        let config: Config = std::fs::read_to_string(home)
            .iter()
            .flat_map(|s| toml::from_str(&s))
            .next()
            .unwrap_or_default();
        let mut rt: Runtime = config.into();
        let mut last_check = SystemTime::now();
        loop {
            while let Ok(msg) = rx2.recv_timeout(Duration::from_secs(1)) {
                if let Some(response) = { handle_msg(&mut rt, &msg) } {
                    for msg in response.split('\n') {
                        tx.send(msg.to_owned()).unwrap();
                    }
                }
            }
            let time_since_last_check = SystemTime::now()
                .duration_since(last_check)
                .unwrap()
                .as_secs_f32();
            if time_since_last_check > rt.points_config.tick_speed {
                for _ in 0..(time_since_last_check as u128) {
                    give_points(&mut rt);
                }
                last_check = SystemTime::now();
            }
        }
    });

    Ok(run(rx, tx2).unwrap())
}
