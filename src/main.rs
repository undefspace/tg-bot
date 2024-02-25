use std::borrow::Cow;

use eyre::Result;
use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::SendMessageSetters, prelude::*, types::ParseMode, utils::command::BotCommands,
};
use thiserror::Error;
use tracing::debug;

mod hass;
use hass::endpoints::services;
use hass::{Client, Service, State};

#[derive(Debug, Serialize, Clone)]
#[serde(transparent)]
#[repr(transparent)]
struct ButtonPress<'a>(&'a hass::Entity<'a>);

impl<'a> From<&'a hass::Entity<'a>> for ButtonPress<'a> {
    fn from(value: &'a hass::Entity<'a>) -> Self {
        Self(value)
    }
}

impl<'a> Service for ButtonPress<'a> {
    type Output = Vec<State<'a>>;

    fn domain(&self) -> &str {
        "button"
    }

    fn service(&self) -> &str {
        "press"
    }
}

#[derive(Clone, Deserialize)]
struct Config {
    #[serde(with = "http_serde_ext::authority::option")]
    hass_host: Option<http::uri::Authority>,
    hass_token: String,
    control_chat_id: ChatId,
    teloxide_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_forest::init();
    color_eyre::install()?;
    run(envy::from_env()?).await
}

static DOOR: hass::types::Entity<'_> = hass::Entity {
    id: Cow::Borrowed("button.open_ring_1"),
};

#[tracing::instrument]
async fn open_door(client: &Client) -> Result<(), hass::client::RequestError> {
    debug!("Opening RING1 door...");
    let res = client.execute(services::Post(ButtonPress(&DOOR))).await?;
    debug!("Response: {res:?}");
    Ok(())
}

/// Commands available to undef members
#[derive(Clone, Copy, BotCommands)]
#[command(rename_rule = "snake_case")]
enum ControlCommand {
    /// Open ring 1 door
    Open,
    /// Show help message
    Help,
}

#[derive(Error, Debug)]
enum ControlCommandError {
    #[error("telegram request failed: {0}")]
    TelegramRequestError(#[from] teloxide::RequestError),
    #[error("hass request failed: {0}")]
    HassRequestError(#[from] hass::client::RequestError),
}

impl ControlCommand {
    async fn answer(
        self,
        bot: teloxide::Bot,
        msg: Message,
        hass: Client,
    ) -> Result<(), ControlCommandError> {
        match self {
            Self::Help => {
                bot.send_message(msg.chat.id, Self::descriptions().to_string())
                    .reply_to_message_id(msg.id)
                    .await?;
            }
            Self::Open => {
                let sender = msg.from();

                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Opening door for {}. Bienvenue!",
                        sender.map_or("anon".to_owned(), |sender| teloxide::utils::html::link(
                            sender.url().as_str(),
                            &sender.full_name()
                        ))
                    ),
                )
                .reply_to_message_id(msg.id)
                .parse_mode(ParseMode::Html)
                .await?;

                open_door(&hass).await?;
            }
        }
        Ok(())
    }
}

async fn run(config: Config) -> Result<()> {
    let mut client =
        hass::client::Client::new(&config.hass_token).expect("successful client creation");

    if let Some(authority) = config.hass_host {
        client.authority = authority;
    }
    let bot = teloxide::Bot::new(config.teloxide_token);

    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<ControlCommand>()
            .filter(move |msg: Message| msg.chat.id == config.control_chat_id)
            .endpoint(ControlCommand::answer),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
