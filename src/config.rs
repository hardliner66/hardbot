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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct UserAccess {
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_mod: bool,
    #[serde(default)]
    pub is_ignored: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct General {
    pub save_time: f32,
    pub hype_emote: String,
    pub message_timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub general: General,
    pub points: Points,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default)]
    pub commands: HashMap<String, String>,
    #[serde(default)]
    pub auto_commands: AutoCommands,
    #[serde(default)]
    pub user_access: HashMap<String, UserAccess>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Config {
        std::fs::read_to_string(path)
            .iter()
            .flat_map(|s| toml::from_str(&s))
            .next()
            .unwrap_or_default()
    }
}

const DEFAULT_CONFIG: &str = include_str!("../defaults.toml");
impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }
}
