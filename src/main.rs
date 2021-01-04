use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use stringlit::s;
use twitch_chat_wrapper::{run, ChatMessage};

mod config;
mod runtime;

use runtime::Runtime;

const CONFIG_FILE: &str = "config.toml";
const DATA_FILE: &str = "data.toml";

fn handle_msg(rt: &mut Runtime, msg: &ChatMessage) -> Option<String> {
    println!("{}: {}", msg.name, msg.message);
    let message = msg.message.trim().to_lowercase();

    if !message.starts_with("!") {
        (*rt
            .data
            .chatters
            .entry(msg.name.clone())
            .or_insert(Default::default()))
        .remaining_ticks = rt.points_config.ticks_per_message;
    }

    rt.commands.get(&message).cloned().or_else(|| {
        if message.starts_with("!commands") {
            Some(rt.commands.keys().cloned().collect::<Vec<_>>().join(" | "))
        } else if message.starts_with("!points") {
            Some(format!(
                "@{} currently has {} points!",
                msg.name,
                rt.data
                    .chatters
                    .get(&msg.name)
                    .map(|chatter| chatter.points)
                    .unwrap_or_default()
            ))
        } else if message.starts_with("!hype") {
            Some(
                message
                    .chars()
                    .filter(|&c| c == 'e')
                    .take(30)
                    .map(|_| "iamhar2Bob")
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        } else if message.starts_with("!") && !message.chars().all(|c| c == '!') {
            Some(s!(
                "No hacking allowed!\nUse !commands to see available commands."
            ))
        } else {
            None
        }
    })
}

fn give_points(rt: &mut Runtime) {
    for (_, chatter) in rt.data.chatters.iter_mut() {
        if chatter.remaining_ticks > 0 {
            chatter.points += rt.points_config.points_per_tick;
            chatter.remaining_ticks -= 1;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel::<String>();
    let (tx2, rx2) = channel::<ChatMessage>();

    std::thread::spawn(move || -> anyhow::Result<()> {
        let mut data_dir = dirs::home_dir().unwrap();
        data_dir.push(".config/hardbot");

        let config_file = data_dir.join(CONFIG_FILE);
        let data_file = data_dir.join(DATA_FILE);

        let sig_term = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&sig_term))?;

        let sig_int = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sig_int))?;

        let config = config::Config::load(config_file);
        let mut rt: Runtime = config.into();

        rt.data = std::fs::read_to_string(&data_file)
            .iter()
            .flat_map(|s| toml::from_str(&s))
            .next()
            .unwrap_or_default();

        let mut last_check = SystemTime::now();
        let mut last_save = SystemTime::now();
        while !sig_int.load(Ordering::Relaxed) && !sig_term.load(Ordering::Relaxed) {
            while let Ok(msg) = rx2.recv_timeout(Duration::from_secs(1)) {
                if let Some(response) = { handle_msg(&mut rt, &msg) } {
                    for msg in response.split('\n') {
                        tx.send(msg.to_owned()).unwrap();
                    }
                }
            }

            let now = SystemTime::now();

            let time_since_last_check = now
                .duration_since(last_check)
                .unwrap()
                .as_secs_f32();

            if time_since_last_check > rt.points_config.tick_speed {
                for _ in 0..(time_since_last_check as u128) {
                    give_points(&mut rt);
                }
                last_check = now;
            }

            if now.duration_since(last_save).unwrap().as_secs_f32() > 60.0 {
                std::fs::write(&data_file, toml::to_string(&rt.data)?)?;
                last_save = now;
            }
        }

        std::fs::write(data_file, toml::to_string(&rt.data)?)?;

        std::process::exit(0);
    });

    Ok(run(rx, tx2).unwrap())
}
