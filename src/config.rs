use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Points {
    pub ticks_per_message: i64,
    pub points_per_tick: u64,
    pub tick_speed: u64,
    pub steal_chance: f32,
    pub steal_min: u64,
    pub steal_max: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoCommands {
    pub time: f32,
    #[serde(default)]
    pub commands: Vec<String>,
}

impl Default for AutoCommands {
    fn default() -> Self {
        AutoCommands {
            time: 600.0,
            commands: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Group {
    pub users: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct General {
    pub save_time: f32,
    pub hype_emote: String,
    pub message_timeout: Option<u64>,
    #[serde(default)]
    pub ignored_users: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub aliases: Vec<String>,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringOrCommand {
    Text(String),
    Cmd(Command),
}

impl ToString for StringOrCommand {
    fn to_string(&self) -> String {
        match self {
            StringOrCommand::Text(txt) => txt.clone(),
            StringOrCommand::Cmd(cmd) => cmd.value.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub general: General,
    pub points: Points,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub commands: HashMap<String, StringOrCommand>,
    #[serde(default)]
    pub auto_commands: AutoCommands,
    #[serde(default)]
    pub groups: HashMap<String, Group>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Config {
        // std::fs::read_to_string(path)
        //     .iter()
        //     .flat_map(|s| toml::from_str(&s))
        //     .next()
        //     .unwrap_or_default()
        std::fs::read_to_string(path)
            .iter()
            .map(|s| toml::from_str(&s).unwrap())
            .next()
            .unwrap()
    }
}

const DEFAULT_CONFIG: &str = include_str!("../defaults.toml");
impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }
}
