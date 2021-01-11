use rand::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use twitch_chat_wrapper::{run, ChatMessage};

mod bot;
mod config;
mod runtime;

use bot::Bot;
use runtime::Runtime;

const CONFIG_FILE: &str = "config.toml";

fn give_points(rt: &mut Runtime) {
    for (_, chatter) in rt.data.chatters.iter_mut() {
        if chatter.remaining_ticks > 0 {
            chatter.points += rt.config.points.points_per_tick;
            chatter.remaining_ticks -= 1;
        }
    }
}

fn get_name_from_string(s: &str) -> String {
    s.trim().replace("@", "")
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<ChatMessage>();

    let mut data_dir = dirs::home_dir().unwrap();
    data_dir.push(".config/hardbot");

    let config_file = data_dir.join(CONFIG_FILE);

    let config = config::Config::load(config_file);
    let mut bot = Bot::new(config);

    bot.register(
        "hype",
        |config, _commands, _data, _user_access, _name, msg| {
            let response = msg
                .chars()
                .filter(|&c| c == 'e')
                .take(30)
                .map(|_| config.general.hype_emote.clone())
                .collect::<Vec<_>>()
                .join(" ");
            Some(response)
        },
    );

    bot.register(
        "points",
        |_config, _commandss, data, _user_access, name, msg| {
            let message = get_name_from_string(&msg[6..]);
            let name = if message.is_empty() { name } else { &message };
            let response = format!(
                "@{} currently has {} points!",
                name,
                data.chatters
                    .get(name)
                    .map(|chatter| chatter.points)
                    .unwrap_or_default()
            );
            Some(response)
        },
    );

    bot.register(
        "mod",
        move |_config, _commands, _data, user_access, name, _msg| {
            if user_access.is_mod || user_access.is_admin {
                Some(format!("@{} you are a mod!", name))
            } else {
                None
            }
        },
    );

    bot.register(
        "top",
        move |config, _commands, data, _user_access, name, _msg| {
            let mut chatters = data.chatters.iter().collect::<Vec<_>>();
            chatters.sort_by_key(|value| value.1.points);
            let top5 = chatters
                .iter()
                .rev()
                .map(|(name, _)| *name)
                .cloned()
                .filter(|name| {
                    !config
                        .user_access
                        .entry(name.to_owned())
                        .or_insert_with(|| Default::default())
                        .is_ignored
                })
                .enumerate()
                .take(5)
                .map(|(i, name)| format!("{}. {}", i + 1, name))
                .collect::<Vec<_>>();
            let response = format!("Top 5, requested by @{}: {}", name, top5.join(" | "));
            Some(response)
        },
    );

    bot.register(
        "steal",
        |config, _commands, data, _user_access, name, msg| {
            let mut rng = rand::thread_rng();
            let parts = msg.split(' ').collect::<Vec<_>>();

            let response = if parts.len() == 2 {
                let other = get_name_from_string(&parts[1]);
                let mut amount = 0;
                let response = if let Some(chatter) = data.chatters.get_mut(&other) {
                    let roll: f32 = rng.gen();
                    if dbg!(roll) < config.points.steal_chance {
                        amount = rng.gen_range(config.points.steal_min..=config.points.steal_max);
                        if chatter.points >= amount {
                            chatter.points = chatter.points.saturating_sub(amount);
                        } else {
                            amount = chatter.points;
                            chatter.points = 0;
                        }
                        None
                    } else {
                        Some(format!("@{} stealing is bad, mkayyy", name))
                    }
                } else {
                    Some(format!(
                        "@{} can't steal from {} because they have no points!",
                        name, other
                    ))
                };

                match response {
                    None => {
                        (*data.chatters.entry(name.to_owned()).or_default()).points = amount;
                        format!("@{} stole {} points from {}!", name, amount, other)
                    }
                    Some(s) => s,
                }
            } else {
                format!(
          "{} You need to specify a person you want to steal from! For example: !steal iamhardbot",
          name
        )
            };
            Some(response)
        },
    );

    bot.register(
            "give",
            |_config, _commands, data, _user_access, name, msg| {
                let parts = msg.split(' ').collect::<Vec<_>>();

                let response = if parts.len() == 3 {
                    let receiver_name = get_name_from_string(&parts[1]);
                    match parts[2].parse::<u64>() {
                        Ok(amount) => {
                            let sender_error = if let Some(sender) = data.chatters.get(name) {
                                if amount <= sender.points {
                                    None
                                } else {
                                    Some(format!("{} you don't have enough points!", name))
                                }
                            } else {
                                Some(
                                    format!("{} You do not have any points to spend!", name)
                                        .to_string(),
                                )
                            };

                            let receiver_error = if data.chatters.contains_key(&receiver_name) {
                                None
                            } else {
                                Some(format!(
                                    "{} The receiver {} is not a registered chatter!",
                                    name, receiver_name
                                ))
                            };

                            if let Some(sender_error) = sender_error {
                                sender_error
                            } else if let Some(receiver_error) = receiver_error {
                                receiver_error
                            } else {
                                data.chatters
                                    .entry(name.to_owned())
                                    .and_modify(|sender| sender.points -= amount);
                                data.chatters
                                    .entry(receiver_name.clone())
                                    .and_modify(|receiver| receiver.points += amount);
                                format!("{} sent {} points to {}!", name, amount, receiver_name)
                            }
                        }
                        _ => format!("{} The amount you specified is not a number!", name)
                            .to_string(),
                    }
                } else {
                    format!(
                        "{} You need to specify a receiver and an amount! For example: !give iamhardbot 999",
                        name
                    ).to_string()
                };
                Some(response)
            },
        );

    bot.register(
        "lurk",
        |_config, _commands, _data, _user_access, name, _msg| {
            Some(format!("Have fun lurking @{}! iamhar2Bob", name))
        },
    );

    bot.register(
        "unlurk",
        |_config, _commands, _data, _user_access, name, _msg| {
            Some(format!("Welcome back from lurking @{}! iamhar2Bob", name))
        },
    );

    bot.register("hi", |_config, _commands, _data, _user_access, name, _msg| {
        Some(format!("Hello, {}!", name))
    });

    let handler_names = bot.handlers.keys().cloned().collect::<Vec<_>>();

    bot.register(
        "commands",
        move |_config, commands, _data, _user_access, _name, _msg| {
            let mut handlers = handler_names.clone();
            let mut commands = commands.keys().cloned().collect::<Vec<_>>();
            commands.append(&mut handlers);
            commands.iter_mut().for_each(|c| c.insert(0, '!'));
            let response = commands.join(" | ");
            Some(response)
        },
    );

    let bot = Arc::new(Mutex::new(bot));

    let bot_sender = tx2.clone();

    std::thread::spawn(move || -> anyhow::Result<()> {
        let sig_term = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&sig_term))?;

        let sig_int = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sig_int))?;

        let mut last_check = SystemTime::now();
        let mut last_save = SystemTime::now();
        let mut last_auto_action = SystemTime::now();

        let (auto_commands, message_timeout) = {
            let bot = bot.lock().unwrap();
            let message_timeout = bot.runtime.config.general.message_timeout.unwrap_or(1);
            let auto_commands = bot.runtime.config.auto_commands.commands.clone();
            (auto_commands, message_timeout)
        };
        let mut auto_commands = auto_commands.iter().cycle();

        while !sig_int.load(Ordering::Relaxed) && !sig_term.load(Ordering::Relaxed) {
            while let Ok(msg) = rx2.recv_timeout(Duration::from_secs(message_timeout)) {
                let mut bot = bot.lock().unwrap();
                if let Some(response) = { bot.handle_message(&msg.name, &msg.message) } {
                    for msg in response.split('\n') {
                        tx.send(msg.to_owned()).unwrap();
                    }
                }
            }

            let now = SystemTime::now();

            {
                let mut bot = bot.lock().unwrap();
            let time_since_last_check = now.duration_since(last_check).unwrap().as_secs_f32();
            if time_since_last_check > bot.runtime.config.points.tick_speed as f32 {
                for _ in 0..(time_since_last_check as u64 / bot.runtime.config.points.tick_speed) {
                    give_points(&mut bot.runtime);
                }
                last_check = now;
            }

            if now.duration_since(last_save).unwrap().as_secs_f32()
                > bot.runtime.config.general.save_time
            {
                last_save = now;
                let _ = bot.save_data();
            }

            if now.duration_since(last_auto_action).unwrap().as_secs_f32()
                > bot.runtime.config.auto_commands.time
            {
                last_auto_action = now;
                if let Some(command) = auto_commands.next() {
                    let _ = bot_sender.send(ChatMessage::builder("<auto>".to_string(), format!("!{}", command)).build());
                }
            }
            }
        }

        let _ = bot.lock().unwrap().save_data();

        std::process::exit(0);
    });

    Ok(run(rx, tx2).unwrap())
}
