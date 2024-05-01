use serde::{Deserialize, Serialize};

mod reward;

pub use reward::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub url_1x: Option<String>,
    pub url_2x: Option<String>,
    pub url_4x: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CooldownSetting {
    pub is_enabled: bool,
    pub global_cooldown_seconds: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedemptionPerStreamLimitSetting {
    pub is_enabled: bool,
    pub max_per_stream: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedemptionPerUserStreamLimitSetting {
    pub is_enabled: bool,
    pub max_per_user_per_stream: Option<i64>,
}
