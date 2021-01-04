use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Points {
    pub ticks_per_message: i64,
    pub points_per_tick: u64,
    pub tick_speed: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub points: Points,
    pub variables: HashMap<String, String>,
    pub commands: HashMap<String, String>,
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
