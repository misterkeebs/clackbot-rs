#![allow(unused)]
use std::env;

use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::twitch::{client::Client, types::Reward};

const REWARDS: &[(&str, u16, &str)] = &[
    ("3 Clacks", 100, "3 Clacks"),
    ("10 Clacks", 350, "5 Clacks"),
    ("20 Clacks", 600, "20 Clacks"),
];

struct EventHandler {
    client: Client,
    rewards: Vec<Reward>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rewards: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.init_rewards().await?;
        self.manage_rewards().await?;
        self.process_redemptions().await?;

        Ok(())
    }

    async fn init_rewards(&mut self) -> anyhow::Result<()> {
        let rewards = self.client.get_rewards().await?;
        self.rewards = rewards.data;

        Ok(())
    }

    async fn manage_rewards(&self) -> anyhow::Result<()> {
        for (title, cost, prompt) in REWARDS {
            if self.rewards.iter().any(|r| r.title == *title) {
                continue;
            }
            println!("{}", title);
            let reward = Reward::builder(title.to_string(), *cost as i32)
                .prompt(prompt.to_string())
                .build();
            reward.create(&self.client).await?;
        }

        Ok(())
    }

    async fn process_redemptions(&self) -> anyhow::Result<()> {
        for reward in &self.rewards {
            let redemptions = reward.get_pending_redemptions(&self.client).await?;
            for redemption in redemptions.data {
                println!("{:#?}", redemption);
            }
        }

        Ok(())
    }
}

pub async fn init_events() -> anyhow::Result<()> {
    let mut handler = EventHandler::new();
    handler.run().await?;

    Ok(())
}

// pub async fn init_webhook() -> anyhow::Result<()> {
//     let client = Client::new().with_user_access_token().await?;
//     println!("{:?}", client);
//     let res = client
//         .subscribe(
//             // "channel.channel_points_custom_reward_redemption.add",
//             "channel.follow",
//             env::var("CALLBACK_SECRET").unwrap().as_str(),
//             env::var("CALLBACK_URL").unwrap().as_str(),
//         )
//         .await?;
//     println!("{:?}", res);
//
//     Ok(())
// }

// pub async fn old_init_events() -> anyhow::Result<()> {
//     let client = Client::new()
//         .with_token(env::var("TWITCH_WEBHOOK_TOKEN"))
//         .await?;
//
//     let user = client.get_users().await?;
//     let user_id = user["data"][0]["id"].as_str().unwrap();
//
//     // Twitch WebSocket URL for PubSub
//     let url = Url::parse(&env::var("TWITCH_WEBSOCKET_URL").unwrap())?;
//
//     // Connect to the WebSocket server
//     let (ws_stream, res) = connect_async(url).await?;
//     println!("Connected to the server: {res:?}");
//
//     // Split the WebSocket stream into a sender and receiver
//     let (_write, mut read) = ws_stream.split();
//
//     // Your OAuth token and channel ID
//     // let token = client
//     //     .get_token(vec!["bits:read", "channel:read:redemptions"])
//     //     .await?;
//     let channel_id = user_id;
//     let token = env::var("TWITCH_WEBHOOK_TOKEN").unwrap();
//
//     println!("Token: {}", token);
//     println!("Channel ID: {}", channel_id);
//
//     // Receive messages indefinitely
//     let mut session_id = None;
//     while let Some(message) = read.next().await {
//         match message? {
//             Message::Text(text) => {
//                 let data: serde_json::Value = serde_json::from_str(&text)?;
//                 println!("{}", serde_json::to_string_pretty(&data).unwrap());
//
//                 if data["metadata"]["message_type"] == "session_welcome" {
//                     session_id = Some(data["payload"]["session"]["id"].as_str().unwrap());
//                     let res = client
//                         .sub_event(
//                             session_id.unwrap(),
//                             "1",
//                             "channel.channel_points_custom_reward_redemption.add",
//                         )
//                         .await?;
//                     println!("{}", serde_json::to_string_pretty(&res).unwrap());
//
//                     let res = client
//                         .sub_event(session_id.unwrap(), "1", "channel.chat.message")
//                         .await?;
//                     println!("{}", serde_json::to_string_pretty(&res).unwrap());
//                 }
//             }
//             Message::Binary(_bin) => println!("Received binary data"),
//             _ => (),
//         }
//     }
//
//     Ok(())
// }

async fn handle_message(msg: &str) -> anyhow::Result<()> {
    let _data: serde_json::Value = serde_json::from_str(msg)?;

    Ok(())
}
