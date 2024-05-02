use std::env;

use serde::de::DeserializeOwned;

use super::types::{RedemptionsResponse, RewardsResponse};

#[allow(unused)]
#[derive(Debug)]
pub struct Client {
    client_id: String,
    secret: String,
    token: String,

    access_token: Option<String>,
    user_id: Option<String>,
}

#[allow(unused)]
impl Client {
    pub fn new() -> Self {
        Self {
            client_id: env::var("TWITCH_CLIENT_ID").unwrap(),
            secret: env::var("TWITCH_SECRET").unwrap(),
            token: env::var("TWITCH_TOKEN").unwrap(),
            access_token: None,
            user_id: None,
        }
    }

    pub async fn with_access_token(mut self) -> anyhow::Result<Self> {
        self.access_token = Some(self.get_access_token().await?);
        Ok(self)
    }

    pub async fn with_token(mut self, token: String) -> anyhow::Result<Self> {
        self.access_token = Some(token);
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
        log::trace!("{}", serde_json::to_string_pretty(&response).unwrap());
        let access_token = response["access_token"].as_str().unwrap();

        Ok(access_token.to_string())
    }

    pub async fn api_request<T: DeserializeOwned>(&self, url: &str) -> anyhow::Result<T> {
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        let response = response.json().await?;

        Ok(response)
    }

    #[allow(unused)]
    pub async fn helix_request(&self, endpoint: &str) -> anyhow::Result<serde_json::Value> {
        let url = format!("https://api.twitch.tv/helix/{}", endpoint);
        self.api_request(&url).await
    }

    pub async fn create_reward(
        &self,
        title: &str,
        cost: i32,
        prompt: &str,
    ) -> anyhow::Result<RewardsResponse> {
        let user_id = self.get_user().await?;
        let url = format!(
            "https://api.twitch.tv/helix/channel_points/custom_rewards?broadcaster_id={user_id}"
        );
        let client = reqwest::Client::new();
        let req = serde_json::json!({
            "title": title,
            "cost": cost,
            "prompt": prompt,
            "is_enabled": true,
            "is_user_input_required": false,
            "is_max_per_stream_enabled": false,
            "is_max_per_user_per_stream_enabled": false,
            "background_color": "#6441A4",
            "image": {
                "url_1x": "https://static-cdn.jtvnw.net/custom-reward-images/default-1.png",
                "url_2x": "https://static-cdn.jtvnw.net/custom-reward-images/default-2.png",
                "url_4x": "https://static-cdn.jtvnw.net/custom-reward-images/default-4.png"
            }
        });
        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&req)
            .send()
            .await?;

        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_rewards(&self) -> anyhow::Result<RewardsResponse> {
        let user_id = self.get_user().await?;
        let url = format!(
            "https://api.twitch.tv/helix/channel_points/custom_rewards?broadcaster_id={user_id}"
        );
        self.api_request(&url).await
    }

    pub async fn get_pending_redemptions(
        &self,
        reward_id: String,
    ) -> anyhow::Result<RedemptionsResponse> {
        let user_id = self.get_user().await?;
        let url = format!(
            "https://api.twitch.tv/helix/channel_points/custom_rewards/redemptions?broadcaster_id={user_id}&reward_id={reward_id}&status=UNFULFILLED"
        );
        self.api_request(&url).await
    }

    pub async fn complete_redemption(
        &self,
        redemption_id: &str,
        broadcaster_id: &str,
        reward_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!(
            "https://api.twitch.tv/helix/channel_points/custom_rewards/redemptions?broadcaster_id={broadcaster_id}&id={redemption_id}&reward_id={reward_id}"
        );
        let client = reqwest::Client::new();
        let req = serde_json::json!({
            "status": "FULFILLED"
        });
        let response = client
            .patch(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&req)
            .send()
            .await?;

        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_user(&self) -> anyhow::Result<String> {
        let users = self.get_users().await?;
        let user_id = users["data"][0]["id"].as_str().unwrap();
        Ok(user_id.to_string())
    }

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
        log::trace!(
            "sub_event result = {}",
            serde_json::to_string_pretty(&req).unwrap()
        );
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
        log::trace!(
            "eventsub/subscriptions = {}",
            serde_json::to_string_pretty(&req).unwrap()
        );
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
