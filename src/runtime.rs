use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Chatter {
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Data {
    pub chatters: HashMap<String, Chatter>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Runtime {
    pub points_config: crate::config::Points,
    pub commands: HashMap<String, String>,
    pub data: Data,
}

impl Into<Runtime> for crate::config::Config {
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
            data: Data {
                chatters: HashMap::new(),
            },
            points_config: self.points.clone(),
        }
    }
}

