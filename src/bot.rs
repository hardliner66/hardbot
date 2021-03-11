use crate::config::Config;
use crate::runtime::{Commands, Data, Runtime};
use std::collections::HashMap;

pub struct Handler {
    handle_function:
        Box<dyn Fn(&mut Config, &Commands, &mut Data, &str, &str) -> Option<String> + 'static>,
    allow_list: Option<Vec<String>>,
}

pub struct Bot {
    pub handlers: HashMap<String, Handler>,
    pub runtime: Runtime,
}

impl Bot {
    pub fn new(config: Config) -> Self {
        Bot {
            handlers: HashMap::new(),
            runtime: config.into(),
        }
    }

    pub fn register<
        F: Fn(&mut Config, &Commands, &mut Data, &str, &str) -> Option<String> + 'static,
    >(
        &mut self,
        prefix: &str,
        allow_list: Option<Vec<String>>,
        f: F,
    ) {
        self.handlers.insert(
            prefix.to_owned(),
            Handler {
                handle_function: Box::new(f),
                allow_list,
            },
        );
    }

    pub fn save_data(&self) -> anyhow::Result<()> {
        self.runtime.save()
    }

    pub fn handle_message(&mut self, name: &str, message: &str) -> Option<String> {
        println!("{}: {}", name, message);
        let name = name.trim().to_lowercase();
        let mut message = message.trim().to_lowercase();

        if message.starts_with("$") {
            message = format!("!so {}", &message[1..]);
        }

        if !message.starts_with("!") {
            (*self
                .runtime
                .data
                .chatters
                .entry(name.to_owned())
                .or_insert(Default::default()))
            .remaining_ticks = self.runtime.config.points.ticks_per_message;
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
                            Some(handler) => {
                                let allowed = if let Some(allow_list) = &handler.allow_list {
                                    allow_list.contains(&format!("@{}", name))
                                        || self
                                            .runtime
                                            .config
                                            .groups
                                            .iter()
                                            .filter(|(_, g)| g.users.contains(&name.to_owned()))
                                            .any(|(name, _)| {
                                                allow_list.contains(&format!("#{}", name))
                                            })
                                } else {
                                    true
                                };
                                if allowed {
                                    (handler.handle_function)(
                                        &mut self.runtime.config,
                                        &self.runtime.commands,
                                        &mut self.runtime.data,
                                        &name,
                                        &message,
                                    )
                                } else {
                                    None
                                }
                            }
                            None => None,
                        }
                    } else {
                        Some(
                            "No hacking allowed!\nUse !commands to see available commands."
                                .to_string(),
                        )
                    }
                } else {
                    Some(
                        "No hacking allowed!\nUse !commands to see available commands.".to_string(),
                    )
                }
            })
        }
    }
}
