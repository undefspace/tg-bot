use futures::StreamExt;
use log::{debug, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc, time::Duration, usize};
use teloxide::{
    dispatching::update_listeners,
    payloads::SendMessageSetters,
    prelude::{GetChatId, Request, UpdateWithCx},
    types::{MessageEntityKind, ParseMode, UpdateKind},
};
use tokio::sync::mpsc::channel;
use tokio::{sync::Mutex, time::sleep};

#[tokio::main]
pub async fn main() {
    run().await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    chat_id: u64,
}

async fn open_door() -> reqwest::Result<()> {
    debug!("Opening RING1 door...");
    let token = env::var("HASS_TOKEN").unwrap();
    let a = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("entity_id", "switch.open_ring_one_door");
    {
        let res = a
            .post("http://10.0.10.8:8123/api/services/switch/turn_on")
            .bearer_auth(&token)
            .json(&map)
            .send()
            .await?;
        debug!("Response: {}", res.text().await?)
    }
    sleep(Duration::from_secs(5)).await;

    debug!("Closing RING1 door...");
    {
        let res = a
            .post("http://10.0.10.8:8123/api/services/switch/turn_off")
            .bearer_auth(&token)
            .json(&map)
            .send()
            .await?;
        debug!("Response: {}", res.text().await?)
    }

    Result::Ok(())
}

async fn run() {
    teloxide::enable_logging!();
    let control_chat_id: i64 = env::var("CONTROL_CHAT_ID")
        .map(|s| s.parse::<i64>().expect("Failed to parse CONTROL_CHAT_ID"))
        .expect("No CONTROL_CHAT_ID in environment");

    let door_opener = Arc::from(Mutex::from({
        let (tx, mut rx) = channel::<()>(1);
        tokio::spawn(async move {
            while let Some(_) = rx.recv().await {
                if let Err(e) = open_door().await {
                    warn!("Error while opening door {}", e)
                };
            }
        });
        tx
    }));

    let bot = teloxide::Bot::from_env();

    loop {
        let mut f = Box::pin(update_listeners::polling_default(bot.clone()));
        while let Some(update) = f.next().await {
            debug!("A new update arrived {:?}", update);
            let update = match update {
                Ok(update) => update,
                Err(error) => {
                    warn!("Error while receiving update: {}", error);
                    return;
                }
            };
            let result: Result<(), &str> = async {
                match update.kind {
                    UpdateKind::Message(msg) => {
                        let cx = UpdateWithCx {
                            update: msg,
                            requester: bot.clone(),
                        };

                        if cx.chat_id() != control_chat_id {
                            Err("Message outside the control chat.")?;
                        }
                        let text = cx
                            .update
                            .text()
                            .ok_or("no message in a text message :/")?
                            .to_string();
                        let commands = cx
                            .update
                            .entities()
                            .ok_or("no entities")?
                            .iter()
                            .filter(|c| c.kind == MessageEntityKind::BotCommand)
                            .map(|s| text[s.offset + 1..s.offset + s.length].to_string())
                            .filter(|s| s.ends_with("@undefspace_bot"))
                            .map(|s| s.trim_end_matches("@undefspace_bot").to_string());

                        for command in commands {
                            let res = match command.as_str() {
                                "open" => {
                                    let sender = cx.update.from().ok_or("No sender: no sending")?;
                                    &cx.reply_to(format!(
                                        "Opening a door for *{} [@{}]*",
                                        &sender.full_name(),
                                        &sender.username.clone().unwrap_or("".to_string())
                                    ))
                                    .parse_mode(ParseMode::MarkdownV2)
                                    .send()
                                    .await;
                                    door_opener
                                        .lock()
                                        .await
                                        .try_send(())
                                        .map_err(|e| e.to_string())
                                }
                                unk => Err(unk.to_string()),
                            };
                            if let Err(command) = res {
                                tokio::spawn(
                                    cx.reply_to(format!("Cannot process command '{}'!", command))
                                        .send(),
                                );
                                warn!("Cannot process command '{}'!", command)
                            }
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            .await;

            match result {
                Ok(_) => {}
                Err(e) => debug!("Error while handling an update: {}", e),
            };
        }
    }
}
