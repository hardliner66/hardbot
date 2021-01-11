use crate::config::{Config, UserAccess};
use crate::runtime::{Commands, Data, Runtime};
use std::collections::HashMap;

pub struct Bot {
    pub handlers: HashMap<
        String,
        Box<
            dyn Fn(&mut Config, &Commands, &mut Data, &UserAccess, &str, &str) -> Option<String>
                + 'static,
        >,
    >,
    pub runtime: Runtime,
}

// eww, don't do that
// TODO: fix that properly!
unsafe impl Send for Bot {}
unsafe impl Sync for Bot {}

impl Bot {
    pub fn new(config: Config) -> Self {
        Bot {
            handlers: HashMap::new(),
            runtime: config.into(),
        }
    }

    pub fn register<
        F: Fn(&mut Config, &Commands, &mut Data, &UserAccess, &str, &str) -> Option<String> + 'static,
    >(
        &mut self,
        prefix: &str,
        f: F,
    ) {
        self.handlers.insert(prefix.to_owned(), Box::new(f));
    }

    pub fn save_data(&self) -> anyhow::Result<()> {
        self.runtime.save()
    }

    pub fn handle_message(&mut self, name: &str, message: &str) -> Option<String> {
        println!("{}: {}", name, message);
        let message = message.trim().to_lowercase();

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
            let user_access = self
                .runtime
                .config
                .user_access
                .get(name)
                .copied()
                .unwrap_or_default();
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
                            Some(handler) => handler(
                                &mut self.runtime.config,
                                &self.runtime.commands,
                                &mut self.runtime.data,
                                &user_access,
                                &name,
                                &message,
                            ),
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
