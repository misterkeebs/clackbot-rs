use std::env;

use futures_util::StreamExt;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::twitch::{Payload, WebSocketMessage};

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

pub async fn init_event_handler() -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, JoinHandle<()>)>
{
    let (tx, mut rx) = channel(100);

    let prod_handle = tokio::spawn(async move {
        _ = start_event_listener(tx).await.map_err(|e| {
            log::error!("Error starting EventSub listener: {e}");
            e
        });

        Ok(())
    });

    let consumer_handle = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            log::info!("Received EventSub notification: {data:#?}");
        }
        log::info!("EventSub notification channel closed");
    });

    Ok((prod_handle, consumer_handle))
}
