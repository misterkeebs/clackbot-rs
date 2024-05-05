use core::fmt;
use std::{fmt::Formatter, sync::Arc};

use anyhow::Context;
use regex::Regex;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex, OnceCell,
};
use twitch_irc::{
    login::StaticLoginCredentials, ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::WPM_GAME;

pub static TWITCH: OnceCell<Twitch> = OnceCell::const_new();

pub struct Twitch {
    tx: Sender<String>,
    rx: Arc<Mutex<Receiver<String>>>,
}

impl fmt::Debug for Twitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Twitch").finish()
    }
}

impl Twitch {
    fn new() -> Self {
        let (tx, rx) = channel(100);
        Self {
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn send(&self, msg: String) {
        log::info!("Sending message: {}", msg);
        self.tx.send(msg).await.unwrap();
    }
}

pub async fn init_twitch() -> anyhow::Result<()> {
    log::debug!("Initializing Twitch...");
    TWITCH.set(Twitch::new())?;
    log::debug!("Done initializing Twitch...");

    let token = std::env::var("TWITCH_TOKEN").context("TWITCH_TOKEN is not set")?;
    let channel = std::env::var("TWITCH_CHANNEL").context("TWITCH_CHANNEL is not set")?;

    let config =
        ClientConfig::new_simple(StaticLoginCredentials::new(channel.clone(), Some(token)));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let cli = client.clone();
    let ch = channel.clone();
    let listener_join_handle = tokio::spawn(async move {
        log::debug!("Listening to channel: {}", ch);
        while let Some(message) = incoming_messages.recv().await {
            handle_message(&ch, &cli, message).await;
        }
    });

    let cli = client.clone();
    let sender_join_handle = tokio::spawn(async move {
        let channel = std::env::var("TWITCH_CHANNEL").expect("TWITCH_CHANNEL is not set");
        let twitch = TWITCH.get().unwrap();

        log::debug!("Sending messages to channel: {}", channel);
        while let Some(ref msg) = twitch.rx.lock().await.recv().await {
            if let Err(err) = cli.privmsg(channel.clone(), msg.clone()).await {
                log::error!("Error sending message '{msg}': {err}");
            }
        }
    });

    client.join(channel)?;
    listener_join_handle.await?;
    sender_join_handle.await?;

    Ok(())
}

async fn handle_message(
    channel: &str,
    client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    message: twitch_irc::message::ServerMessage,
) {
    match message {
        twitch_irc::message::ServerMessage::Privmsg(msg) => {
            if msg.channel_login == channel {
                let text = msg.message_text.clone();
                let sender = msg.sender.login.clone();
                log::debug!("{}: {}", sender, text);
                let badges = msg
                    .badges
                    .iter()
                    .map(|b| b.name.clone())
                    .collect::<Vec<_>>();
                if text == "!wpm start" && badges.contains(&"broadcaster".to_string()) {
                    WPM_GAME.get().unwrap().write().await.start();
                    client
                        .privmsg(channel.to_string(), "A new WPM guessing game has started. Send your guess by using !wpm <guess>.".to_string())
                        .await
                        .unwrap();
                    return;
                }

                if text.starts_with("!wpm") {
                    let re = Regex::new(r"!wpm (\d+)").unwrap();
                    if let Some(caps) = re.captures(&text) {
                        if let Some(n) = caps.get(1) {
                            let num = n.as_str().parse::<usize>().unwrap();
                            let res = WPM_GAME
                                .get()
                                .unwrap()
                                .write()
                                .await
                                .add_guess(&sender, num);

                            let reply = match res {
                                Ok(_) => format!("{} got your {} WPM guess", sender, num),
                                Err(e) => format!("{} {}", sender, e),
                            };
                            // reply the user with guess
                            client.privmsg(channel.to_string(), reply).await.unwrap();
                        }
                    } else {
                        // reply the user with error
                        client
                            .privmsg(
                                channel.to_string(),
                                format!("{} invalid guess, use !wpm <wpm estimate>", sender),
                            )
                            .await
                            .unwrap();
                    }
                }
            }
        }
        _ => {}
    }
}
