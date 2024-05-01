use serde::{Deserialize, Serialize};

use super::{
    redemption::RedemptionsResponse, CooldownSetting, Image, RedemptionPerStreamLimitSetting,
    RedemptionPerUserStreamLimitSetting,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Reward {
    pub broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub cost: i32,
    pub image: Option<Image>,
    pub default_image: Image,
    pub background_color: String,
    pub is_enabled: bool,
    pub is_user_input_required: bool,
    pub max_per_stream_setting: RedemptionPerStreamLimitSetting,
    pub max_per_user_per_stream_setting: RedemptionPerUserStreamLimitSetting,
    pub global_cooldown_setting: CooldownSetting,
    pub is_paused: bool,
    pub is_in_stock: bool,
    pub should_redemptions_skip_request_queue: bool,
    pub redemptions_redeemed_current_stream: Option<i32>,
    pub cooldown_expires_at: Option<String>,
}

impl Reward {
    pub fn builder(title: String, cost: i32) -> RewardBuilder {
        RewardBuilder::new(title, cost)
    }

    pub async fn create(
        &self,
        client: &crate::twitch::client::Client,
    ) -> anyhow::Result<RewardsResponse> {
        client
            .create_reward(&self.title, self.cost, &self.prompt)
            .await
    }

    pub async fn get_pending_redemptions(
        &self,
        client: &crate::twitch::client::Client,
    ) -> anyhow::Result<RedemptionsResponse> {
        client.get_pending_redemptions(self.id.clone()).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RewardsResponse {
    pub data: Vec<Reward>,
}

#[derive(Debug)]
pub struct RewardBuilder {
    // Mandatory fields
    title: String,
    cost: i32,

    // Optional fields with defaults
    prompt: String,
    is_enabled: bool,
    is_user_input_required: bool,
    background_color: String,
    image: Option<Image>,
    default_image: Image,
    max_per_stream_setting: RedemptionPerStreamLimitSetting,
    max_per_user_per_stream_setting: RedemptionPerUserStreamLimitSetting,
    global_cooldown_setting: CooldownSetting,
    is_paused: bool,
    is_in_stock: bool,
    should_redemptions_skip_request_queue: bool,
    redemptions_redeemed_current_stream: Option<i32>,
    cooldown_expires_at: Option<String>,
}

impl RewardBuilder {
    // Constructor with mandatory fields
    pub fn new(title: String, cost: i32) -> Self {
        RewardBuilder {
            title,
            cost,
            prompt: String::new(),
            is_enabled: true,
            is_user_input_required: false,
            background_color: String::from("#6441A4"),
            image: Some(Image {
                url_1x: Some(String::from(
                    "https://static-cdn.jtvnw.net/custom-reward-images/default-1.png",
                )),
                url_2x: Some(String::from(
                    "https://static-cdn.jtvnw.net/custom-reward-images/default-2.png",
                )),
                url_4x: Some(String::from(
                    "https://static-cdn.jtvnw.net/custom-reward-images/default-4.png",
                )),
            }),
            default_image: Image {
                url_1x: Some(String::from(
                    "https://static-cdn.jtvnw.net/custom-reward-images/default-1.png",
                )),
                url_2x: Some(String::from(
                    "https://static-cdn.jtvnw.net/custom-reward-images/default-2.png",
                )),
                url_4x: Some(String::from(
                    "https://static-cdn.jtvnw.net/custom-reward-images/default-4.png",
                )),
            },
            max_per_stream_setting: RedemptionPerStreamLimitSetting {
                is_enabled: false,
                max_per_stream: None,
            },
            max_per_user_per_stream_setting: RedemptionPerUserStreamLimitSetting {
                is_enabled: false,
                max_per_user_per_stream: None,
            },
            global_cooldown_setting: CooldownSetting {
                is_enabled: false,
                global_cooldown_seconds: 0,
            },
            is_paused: false,
            is_in_stock: true,
            should_redemptions_skip_request_queue: false,
            redemptions_redeemed_current_stream: None,
            cooldown_expires_at: None,
        }
    }

    // Setter methods for optional fields
    pub fn prompt(mut self, prompt: String) -> Self {
        self.prompt = prompt;
        self
    }

    // Build method to finalize and construct the Reward object
    pub fn build(self) -> Reward {
        Reward {
            broadcaster_id: String::new(),
            broadcaster_login: String::new(),
            broadcaster_name: String::new(),
            id: String::new(),
            title: self.title,
            prompt: self.prompt,
            cost: self.cost,
            image: self.image,
            default_image: self.default_image,
            background_color: self.background_color,
            is_enabled: self.is_enabled,
            is_user_input_required: self.is_user_input_required,
            max_per_stream_setting: self.max_per_stream_setting,
            max_per_user_per_stream_setting: self.max_per_user_per_stream_setting,
            global_cooldown_setting: self.global_cooldown_setting,
            is_paused: self.is_paused,
            is_in_stock: self.is_in_stock,
            should_redemptions_skip_request_queue: self.should_redemptions_skip_request_queue,
            redemptions_redeemed_current_stream: self.redemptions_redeemed_current_stream,
            cooldown_expires_at: self.cooldown_expires_at,
        }
    }
}
