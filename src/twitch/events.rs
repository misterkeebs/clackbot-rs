#![allow(unused)]
use std::env;

use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

struct Client {
    client_id: String,
    secret: String,

    access_token: Option<String>,
    user_id: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client_id: env::var("TWITCH_CLIENT_ID").unwrap(),
            secret: env::var("TWITCH_SECRET").unwrap(),
            access_token: None,
            user_id: None,
        }
    }

    pub async fn with_access_token(mut self) -> anyhow::Result<Self> {
        self.access_token = Some(self.get_access_token().await?);
        self.user_id = Some(
            self.get_users().await?["data"][0]["id"]
                .as_str()
                .unwrap()
                .to_string(),
        );
        Ok(self)
    }

    pub async fn with_token(
        mut self,
        token: Result<String, env::VarError>,
    ) -> anyhow::Result<Self> {
        self.access_token = Some(token.unwrap());
        self.user_id = Some(
            self.get_users().await?["data"][0]["id"]
                .as_str()
                .unwrap()
                .to_string(),
        );
        Ok(self)
    }

    pub async fn get_access_token(&self) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", &self.secret),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;
        let access_token = response["access_token"].as_str().unwrap();

        Ok(access_token.to_string())
    }

    pub async fn get_token(&self, scopes: Vec<&str>) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", &self.secret),
                ("grant_type", "client_credentials"),
                ("scope", &scopes.join(" ")),
            ])
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;
        let access_token = response["access_token"].as_str().unwrap();

        Ok(access_token.to_string())
    }

    pub async fn api_request(&self, url: &str) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("Client-ID", &self.client_id)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.as_ref().unwrap()),
            )
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;

        Ok(response)
    }

    // pub async fn helix_request(&self, endpoint: &str) -> anyhow::Result<serde_json::Value> {
    //     let url = format!("https://api.twitch.tv/helix/{}", endpoint);
    //     self.api_request(&url).await
    // }

    pub async fn get_users(&self) -> anyhow::Result<serde_json::Value> {
        let url = format!("https://api.twitch.tv/helix/users");
        self.api_request(&url).await
    }

    pub async fn sub_event(
        &self,
        session_id: &str,
        ver: &str,
        typ: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("https://api.twitch.tv/helix/eventsub/subscriptions");
        let client = reqwest::Client::new();
        let req = serde_json::json!({
            "type": typ,
            "version": ver,
            "condition": {
                "user_id": self.user_id,
                "broadcaster_user_id": self.user_id,
            },
            "transport": {
                "method": "websocket",
                "session_id": session_id,
            }
        });
        println!("{}", serde_json::to_string_pretty(&req).unwrap());
        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.as_ref().unwrap()),
            )
            .json(&req)
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;
        Ok(response)
    }

    pub async fn subscribe(
        &self,
        typ: &str,
        secret: &str,
        callback: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("https://api.twitch.tv/helix/eventsub/subscriptions");
        let client = reqwest::Client::new();
        let req = serde_json::json!({
            "type": typ,
            "version": "2",
            "condition": {
                "broadcaster_user_id": self.user_id,
                "moderator_user_id": self.user_id,
            },
            "transport": {
                "method": "webhook",
                "callback": callback,
                "secret": secret
            }
        });
        println!("{}", serde_json::to_string_pretty(&req).unwrap());
        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.as_ref().unwrap()),
            )
            .json(&req)
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;
        Ok(response)
    }
}

pub async fn init_webhook() -> anyhow::Result<()> {
    let client = Client::new().with_access_token().await?;
    let res = client
        .subscribe(
            // "channel.channel_points_custom_reward_redemption.add",
            "channel.follow",
            env::var("CALLBACK_SECRET").unwrap().as_str(),
            env::var("CALLBACK_URL").unwrap().as_str(),
        )
        .await?;
    println!("{:?}", res);

    Ok(())
}

pub async fn init_events() -> anyhow::Result<()> {
    // let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    // let client_secret = env::var("TWITCH_SECRET").unwrap();
    // let channel_id = env::var("TWITCH_CHANNEL_ID").unwrap();
    let client = Client::new()
        .with_token(env::var("TWITCH_WEBHOOK_TOKEN"))
        .await?;

    let user = client.get_users().await?;
    let user_id = user["data"][0]["id"].as_str().unwrap();

    // Twitch WebSocket URL for PubSub
    let url = Url::parse(&env::var("TWITCH_WEBSOCKET_URL").unwrap())?;

    // Connect to the WebSocket server
    let (ws_stream, res) = connect_async(url).await?;
    println!("Connected to the server: {res:?}");

    // Split the WebSocket stream into a sender and receiver
    let (_write, mut read) = ws_stream.split();

    // Your OAuth token and channel ID
    // let token = client
    //     .get_token(vec!["bits:read", "channel:read:redemptions"])
    //     .await?;
    let channel_id = user_id;
    let token = env::var("TWITCH_WEBHOOK_TOKEN").unwrap();

    println!("Token: {}", token);
    println!("Channel ID: {}", channel_id);

    // Receive messages indefinitely
    let mut session_id = None;
    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                let data: serde_json::Value = serde_json::from_str(&text)?;
                println!("{}", serde_json::to_string_pretty(&data).unwrap());

                if data["metadata"]["message_type"] == "session_welcome" {
                    session_id = Some(data["payload"]["session"]["id"].as_str().unwrap());
                    let res = client
                        .sub_event(
                            session_id.unwrap(),
                            "1",
                            "channel.channel_points_custom_reward_redemption.add",
                        )
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&res).unwrap());

                    let res = client
                        .sub_event(session_id.unwrap(), "1", "channel.chat.message")
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&res).unwrap());
                }
            }
            Message::Binary(_bin) => println!("Received binary data"),
            _ => (),
        }
    }

    Ok(())
}

async fn handle_message(msg: &str) -> anyhow::Result<()> {
    let _data: serde_json::Value = serde_json::from_str(msg)?;

    Ok(())
}
