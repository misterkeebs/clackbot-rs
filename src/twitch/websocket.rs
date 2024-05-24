use std::env;

use futures_util::StreamExt;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::{
    db::Pool,
    models::User,
    twitch::{EventDetail, Payload, WebSocketMessage, TWITCH},
};

use super::{Client, NotificationData};

pub async fn start_event_listener(tx: Sender<NotificationData>) -> anyhow::Result<()> {
    let url = Url::parse(&env::var("TWITCH_EVENTSUB_WEBSOCKET_URL")?)?;
    log::info!("Starting EventSub WebSocket: {url}");

    let (ws_stream, res) = connect_async(url).await?;
    log::trace!("WebSocket connection response: {res:?}");

    // Split the WebSocket stream into a sender and receiver
    let (_write, mut read) = ws_stream.split();

    let client = Client::new().with_access_token().await?;
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                log::trace!("Received text message from WebSocket: {text}");
                let msg: WebSocketMessage = serde_json::from_str(&text)?;
                log::trace!("websocket message = {msg:#?}");

                match msg.payload {
                    Payload::Welcome(data) => {
                        let res = client
                            .sub_event(
                                &data.session.id,
                                "1",
                                "channel.channel_points_custom_reward_redemption.add",
                            )
                            .await
                            .unwrap();
                        log::trace!(
                            "event sub response = {}",
                            serde_json::to_string_pretty(&res)?
                        );
                        continue;
                    }
                    Payload::Keepalive => {} // noop
                    Payload::Reconnect(data) => {
                        log::warn!("Received reconnect message from WebSocket: {data:#?}");
                    }
                    Payload::Revocation(data) => {
                        log::warn!("Received revocation message from WebSocket: {data:#?}");
                    }
                    Payload::Notification(data) => {
                        log::trace!("WebSocket Notification: {data:#?}");
                        tx.send(data).await?;
                    }
                }
            }
            Ok(Message::Binary(_bin)) => log::warn!("Received binary data from WebSocket"),
            _ => (),
        }
    }

    Ok(())
}

pub async fn init_event_handler(
    pool: &Pool,
) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, JoinHandle<()>)> {
    let (tx, mut rx) = channel(100);

    let prod_handle = tokio::spawn(async move {
        _ = start_event_listener(tx).await.map_err(|e| {
            log::error!("Error starting EventSub listener: {e}");
            e
        });

        Ok(())
    });

    let pool = pool.clone();
    let consumer_handle = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            log::info!("Received EventSub notification: {data:#?}");
            match handle_event(&pool, data).await {
                Ok(_) => log::info!("Event handled successfully"),
                Err(e) => log::error!("Error handling event: {e}"),
            };
        }
        log::info!("EventSub notification channel closed");
    });

    Ok((prod_handle, consumer_handle))
}

async fn handle_event(pool: &Pool, data: NotificationData) -> anyhow::Result<()> {
    match data.event {
        EventDetail::ChannelPointsCustomRewardRedemptionAdd(event) => {
            log::info!("Redemption: {} - {}", event.user_id, event.reward.title);
            let mut conn = pool.get().await?;
            match User::get_by_twitch_id(&mut conn, event.user_id.clone()).await? {
                None => {
                    let twitch = TWITCH.get().unwrap();
                    let discord = match env::var("DISCORD_INVITE") {
                        Ok(channel) => format!(": {channel}"),
                        Err(_) => "".to_string(),
                    };
                    twitch
                        .send(format!(
                            "{} you need to link your Twitch account in our Discord{} using the /link command on the #bot-spam channel. Once you finish, all pending rewards will be processed automatically.",
                            event.user_name, discord
                        ))
                        .await;
                }
                Some(_) => {
                    // user.process_redemption(event, &mut conn).await?;
                }
            };
        }
        EventDetail::Generic(_event) => {
            log::info!("Generic Event: {}", data.subscription.typ)
        }
    }

    Ok(())
}
