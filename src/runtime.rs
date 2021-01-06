use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const DATA_FILE: &str = "data.toml";

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

impl Runtime {
  pub fn save(&self) -> anyhow::Result<()> {
    let data_file = Self::get_data_path();
    std::fs::write(&data_file, toml::to_string(&self.data)?)?;
    Ok(())
  }

  pub fn load(&mut self) {
    let data_file = Self::get_data_path();
    self.data = std::fs::read_to_string(&data_file)
      .iter()
      .flat_map(|s| toml::from_str(&s))
      .next()
      .unwrap_or_default();
  }

  pub fn get_data_path() -> PathBuf {
    let mut data_dir = dirs::home_dir().unwrap();
    data_dir.push(".config/hardbot");
    let data_file = data_dir.join(DATA_FILE);
    data_file
  }
}

impl Into<Runtime> for crate::config::Config {
  fn into(self) -> Runtime {
    let mut commands = HashMap::new();
    for (name, value) in self.commands {
      let mut value = value;
      for (variable_name, variable_value) in &self.variables {
        value = value.replace(&format!("${}", variable_name), &variable_value);
      }
      commands.insert(format!("{}", name), value);
    }

    let mut rt = Runtime {
      commands,
      data: Data::default(),
      points_config: self.points.clone(),
    };

    rt.load();

    rt
  }
}
