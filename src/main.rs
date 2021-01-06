use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use twitch_chat_wrapper::{run, ChatMessage};
use rand::random;

mod bot;
mod config;
mod runtime;

use bot::Bot;
use runtime::Runtime;

const CONFIG_FILE: &str = "config.toml";

fn give_points(rt: &mut Runtime) {
  for (_, chatter) in rt.data.chatters.iter_mut() {
    if chatter.remaining_ticks > 0 {
      chatter.points += rt.points_config.points_per_tick;
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

  std::thread::spawn(move || -> anyhow::Result<()> {
    let mut data_dir = dirs::home_dir().unwrap();
    data_dir.push(".config/hardbot");

    let config_file = data_dir.join(CONFIG_FILE);

    let sig_term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&sig_term))?;

    let sig_int = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sig_int))?;

    let config = config::Config::load(config_file);

    let mut bot = Bot::new(config);

    bot.register("hype", |_runtime, _name, msg| {
      msg
        .chars()
        .filter(|&c| c == 'e')
        .take(30)
        .map(|_| "iamhar2Bob")
        .collect::<Vec<_>>()
        .join(" ")
    });

    bot.register("points", |runtime, name, msg| {
      let message = get_name_from_string(&msg[6..]);
      let name = if message.is_empty() { name } else { &message };
      format!(
        "@{} currently has {} points!",
        name,
        runtime
          .data
          .chatters
          .get(name)
          .map(|chatter| chatter.points)
          .unwrap_or_default()
      )
    });

    let ignored_users = vec![
      "iamhardbot".to_string(),
      "pretzelrocks".to_string(),
      "iamhardliner".to_string(),
    ];

    bot.register("top", move |runtime, name, _msg| {
      let mut chatters = runtime.data.chatters.iter().collect::<Vec<_>>();
      chatters.sort_by_key(|value| value.1.points);
      let top5 = chatters
        .iter()
        .rev()
        .filter(|(name, _)| !ignored_users.contains(name))
        .enumerate()
        .take(5)
        .map(|(i, (name, _))| format!("{}. {}", i + 1, name))
        .collect::<Vec<_>>();
      format!("Top 5 requested by @{}: {}", name, top5.join(" | "))
    });

    // bot.register("steal", |runtime, name, msg| {
    //   let parts = msg.split(' ').collect::<Vec<_>>();

    //   if parts.len() == 2 {
    //       let other = get_name_from_string(&parts[1]);
    //       if let Some(chatter) = runtime.data.chatters.get_mut(&other) {
    //           let roll: f32 = random();
    //           if roll < 0.000000000000001 {

    //           } else {
    //             format!("@{} stealing is bad, mkayyy", name)
    //           }
    //       } else {
    //           format!("@{} can't steal from {} because they have no points!", name, other)
    //       }
    //   } else {
    //     format!(
    //       "{} You need to specify a person you want to steal from! For example: !steal iamhardbot 999",
    //       name
    //     )
    //   }
    // });

    bot.register("give", |runtime, name, msg| {
      let parts = msg.split(' ').collect::<Vec<_>>();

      if parts.len() == 3 {
        let receiver_name = get_name_from_string(&parts[1]);
        match parts[2].parse::<u64>() {
          Ok(amount) => {
            let sender_error = if let Some(sender) = runtime.data.chatters.get(name) {
              if amount <= sender.points {
                None
              } else {
                Some(format!("{} you don't have enough points!", name))
              }
            } else {
              Some(format!("{} You do not have any points to spend!", name).to_string())
            };

            let receiver_error = if runtime.data.chatters.contains_key(&receiver_name) {
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
              runtime
                .data
                .chatters
                .entry(name.to_owned())
                .and_modify(|sender| sender.points -= amount);
              runtime
                .data
                .chatters
                .entry(receiver_name.clone())
                .and_modify(|receiver| receiver.points += amount);
              format!("{} sent {} points to {}!", name, amount, receiver_name)
            }
          }
          _ => format!("{} The amount you specified is not a number!", name).to_string(),
        }
      } else {
        format!(
          "{} You need to specify a receiver and an amount! For example: !give iamhardbot 999",
          name
        )
        .to_string()
      }
    });

    let handler_names = bot.handlers.keys().cloned().collect::<Vec<_>>();

    bot.register("commands", move |runtime, _name, _msg| {
      let mut handlers = handler_names.clone();
      let mut commands = runtime.commands.keys().cloned().collect::<Vec<_>>();
      commands.append(&mut handlers);
      commands.iter_mut().for_each(|c| c.insert(0, '!'));
      commands.join(" | ")
    });
    let mut last_check = SystemTime::now();
    let mut last_save = SystemTime::now();
    while !sig_int.load(Ordering::Relaxed) && !sig_term.load(Ordering::Relaxed) {
      while let Ok(msg) = rx2.recv_timeout(Duration::from_secs(1)) {
        if let Some(response) = { bot.handle_message(&msg) } {
          for msg in response.split('\n') {
            tx.send(msg.to_owned()).unwrap();
          }
        }
      }

      let now = SystemTime::now();

      let time_since_last_check = now.duration_since(last_check).unwrap().as_secs_f32();

      if time_since_last_check > bot.runtime.points_config.tick_speed as f32 {
        for _ in 0..(time_since_last_check as u64 / bot.runtime.points_config.tick_speed) {
          give_points(&mut bot.runtime);
        }
        last_check = now;
      }

      if now.duration_since(last_save).unwrap().as_secs_f32() > 60.0 {
        last_save = now;
        let _ = bot.save_data();
      }
    }

    let _ = bot.save_data();

    std::process::exit(0);
  });

  Ok(run(rx, tx2).unwrap())
}
