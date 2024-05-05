use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Redemption {
    pub broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    pub id: String,
    pub user_login: String,
    pub user_id: String,
    pub user_name: String,
    pub user_input: Option<String>,
    pub status: RedemptionStatus,
    pub redeemed_at: DateTime<Utc>,
    pub reward: SimpleReward,
}

impl Redemption {
    pub async fn complete(&self, client: &crate::twitch::client::Client) -> anyhow::Result<()> {
        client
            .complete_redemption(&self.id, &self.broadcaster_id, &self.reward.id)
            .await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum RedemptionStatus {
    Canceled,
    Fulfilled,
    Unfulfilled,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleReward {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub cost: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedemptionsResponse {
    pub data: Vec<Redemption>,
}
