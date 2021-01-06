use crate::config::Config;
use crate::runtime::Runtime;
use std::collections::HashMap;
use twitch_chat_wrapper::ChatMessage;

pub struct Bot {
  pub handlers: HashMap<String, Box<dyn Fn(&mut Runtime, &str, &str) -> String + 'static>>,
  pub runtime: Runtime,
}

impl Bot {
  pub fn new(config: Config) -> Self {
    Bot {
      handlers: HashMap::new(),
      runtime: config.into(),
    }
  }

  pub fn register<F: Fn(&mut Runtime, &str, &str) -> String + 'static>(
    &mut self,
    prefix: &str,
    f: F,
  ) {
    self.handlers.insert(prefix.to_owned(), Box::new(f));
  }

  pub fn save_data(&self) -> anyhow::Result<()> {
    self.runtime.save()
  }

  pub fn handle_message(&mut self, msg: &ChatMessage) -> Option<String> {
    println!("{}: {}", msg.name, msg.message);
    let message = msg.message.trim().to_lowercase();

    if !message.starts_with("!") {
      (*self
        .runtime
        .data
        .chatters
        .entry(msg.name.clone())
        .or_insert(Default::default()))
      .remaining_ticks = self.runtime.points_config.ticks_per_message;
      None
    } else {
      let message = message
        .chars()
        .skip_while(|&c| c == '!')
        .collect::<String>();
      self.runtime.commands.get(&message).cloned().or_else(|| {
        if let Some(user_command) = message.split(' ').next() {
          if let Some(command) = self
            .handlers
            .keys()
            .filter(|key| user_command.starts_with(*key))
            .next()
          {
            match self.handlers.get(command) {
              Some(handler) => Some(handler(&mut self.runtime, &msg.name, &message)),
              None => None,
            }
          } else {
            Some("No hacking allowed!\nUse !commands to see available commands.".to_string())
          }
        } else {
          Some("No hacking allowed!\nUse !commands to see available commands.".to_string())
        }
      })
    }
  }
}
