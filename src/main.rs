use rand::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use stringlit::s;
use twitch_irc::login::{CredentialsPair, StaticLoginCredentials};
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::TCPTransport;
use twitch_irc::TwitchIRCClient;

mod bot;
mod config;
mod runtime;

use bot::Bot;
use runtime::Runtime;

const CONFIG_FILE: &str = "config.toml";

fn channel_to_join() -> Result<String, Box<dyn std::error::Error>> {
    let channel = get_env_var("TWITCH_CHANNEL")?;
    Ok(channel)
}

fn get_env_var(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let my_var = std::env::var(key)?;
    Ok(my_var)
}

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

#[tokio::main]
async fn main() {
    let mut data_dir = dirs::home_dir().unwrap();
    data_dir.push(".config/hardbot");

    let config_file = data_dir.join(CONFIG_FILE);

    let config = config::Config::load(config_file);
    let mut bot = Bot::new(config);

    bot.register("calc", None, |_config, _commands, _data, _name, msg| {
        let mut ns = fasteval::EmptyNamespace;
        match fasteval::ez_eval(&msg[4..], &mut ns) {
            Ok(v) => Some(v.to_string()),
            Err(e) => Some(e.to_string()),
        }
    });

    // bot.register("hype", None, |config, _commands, _data, _name, msg| {
    //     let response = msg
    //         .chars()
    //         .filter(|&c| c == 'e')
    //         .take(30)
    //         .map(|_| config.general.hype_emote.clone())
    //         .collect::<Vec<_>>()
    //         .join(" ");
    //     Some(response)
    // });

    bot.register("points", None, |_config, _commandss, data, name, msg| {
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
    });

    bot.register(
        "mod",
        Some(vec![s!("#admin"), s!("#mod")]),
        move |_config, _commands, _data, name, _msg| Some(format!("@{} you are a mod!", name)),
    );

    bot.register("top", None, move |config, _commands, data, name, _msg| {
        let mut chatters = data.chatters.iter().collect::<Vec<_>>();
        chatters.sort_by_key(|value| value.1.points);
        let top5 = chatters
            .iter()
            .rev()
            .map(|(name, chatter)| (*name, chatter))
            .filter(|(name, _)| !config.general.ignored_users.contains(name))
            .enumerate()
            .take(5)
            .map(|(i, (name, chatter))| format!("{}. {} ({} Points)", i + 1, name, chatter.points))
            .collect::<Vec<_>>();
        let response = format!("Top 5, requested by @{}: {}", name, top5.join(" | "));
        Some(response)
    });

    bot.register("steal", None, |config, _commands, data, name, msg| {
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
    });

    bot.register(
            "give",
            None,
            |_config, _commands, data, name, msg| {
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

    bot.register("lurk", None, |_config, _commands, _data, name, _msg| {
        Some(format!("Have fun lurking @{}! iamhar2Bob", name))
    });

    bot.register("unlurk", None, |_config, _commands, _data, name, _msg| {
        Some(format!("Welcome back from lurking @{}! iamhar2Bob", name))
    });

    bot.register("hi", None, |_config, _commands, _data, name, _msg| {
        Some(format!("Hello, {}!", name))
    });

    let handler_names = bot.handlers.keys().cloned().collect::<Vec<_>>();

    bot.register(
        "commands",
        None,
        move |_config, commands, _data, _name, _msg| {
            let mut handlers = handler_names.clone();
            let mut commands = commands.keys().cloned().collect::<Vec<_>>();
            commands.append(&mut handlers);
            commands.iter_mut().for_each(|c| c.insert(0, '!'));
            let response = commands.join(" | ");
            Some(response)
        },
    );

    bot.register(
        "so",
        Some(vec![s!("#admin")]),
        |_config, _commands, _data, _name, msg| {
            let parts = msg.split(" ").collect::<Vec<_>>();
            if let Some(part) = parts.get(1) {
                let name = get_name_from_string(part);
                Some(format!(
                    "Shoutout to @{}. You can check out their stream at https://www.twitch.tv/{}",
                    name, name
                ))
            } else {
                None
            }
        },
    );

    let sig_term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&sig_term)).unwrap();

    let sig_int = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sig_int)).unwrap();

    let mut last_check = SystemTime::now();
    let mut last_save = SystemTime::now();
    let mut last_auto_action = SystemTime::now();

    let (auto_commands, message_timeout) = {
        let message_timeout = bot.runtime.config.general.message_timeout.unwrap_or(1);
        let auto_commands = bot.runtime.config.auto_commands.commands.clone();
        (auto_commands, message_timeout)
    };
    let mut auto_commands = auto_commands.iter().cycle();

    let twitch_name = get_env_var("TWITCH_NAME").unwrap();
    let twitch_token = get_env_var("TWITCH_TOKEN")
        .unwrap()
        .replacen("oauth:", "", 1);
    let channel_to_join = channel_to_join().unwrap();

    // default configuration is to join chat as anonymous.
    let config = ClientConfig {
        login_credentials: StaticLoginCredentials {
            credentials: CredentialsPair {
                login: twitch_name.clone(),
                token: Some(twitch_token),
            },
        },
        ..ClientConfig::default()
    };

    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(config);

    client.join(channel_to_join.clone());

    while !sig_int.load(Ordering::Relaxed) && !sig_term.load(Ordering::Relaxed) {
        let message = tokio::time::timeout(
            Duration::from_secs(message_timeout),
            incoming_messages.recv(),
        );
        if let Ok(Some(msg)) = message.await {
            match msg {
                ServerMessage::Privmsg(msg) => {
                    let response = bot.handle_message(&msg.sender.name, &msg.message_text);
                    if let Some(response) = response {
                        for msg in response.split('\n') {
                            client
                                .say(channel_to_join.clone(), msg.to_owned())
                                .await
                                .unwrap();
                        }
                    }
                }
                _ => (),
            }
        }

        let now = SystemTime::now();

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
                if let Some(response) = { bot.handle_message("<auto>", &format!("!{}", command)) } {
                    for msg in response.split('\n') {
                        client
                            .say(channel_to_join.clone(), msg.to_owned())
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }

    let _ = bot.save_data();

    std::process::exit(0);
}
