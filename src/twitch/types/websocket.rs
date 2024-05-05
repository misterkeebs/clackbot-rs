use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::SimpleReward;

// Base structure for a WebSocket message from Twitch
#[derive(Debug, Serialize)]
pub struct WebSocketMessage {
    pub metadata: MessageMetadata,
    pub payload: Payload,
}

// Metadata common across all message types
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub message_id: String,
    pub message_type: String,
    pub message_timestamp: String,
}

// Enum to handle different payload types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum Payload {
    #[serde(rename = "session_welcome")]
    Welcome(WelcomeData),

    #[serde(rename = "session_keepalive")]
    Keepalive,

    #[serde(rename = "notification")]
    Notification(NotificationData),

    #[serde(rename = "session_reconnect")]
    Reconnect(ReconnectData),

    #[serde(rename = "revocation")]
    Revocation(RevocationData),
}

// Welcome message data
#[derive(Debug, Serialize, Deserialize)]
pub struct WelcomeData {
    pub session: SessionData,
}

// Session data used in Welcome and Reconnect messages
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub id: String,
    pub status: String,
    pub keepalive_timeout_seconds: Option<i32>,
    pub reconnect_url: Option<String>,
    pub connected_at: String,
}

// Notification message data
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationData {
    pub subscription: SubscriptionDetail,
    pub event: EventDetail,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventDetail {
    ChannelPointsCustomRewardRedemptionAdd(ChannelPointsCustomRewardRedemptionAdd),
    Generic(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelPointsCustomRewardRedemptionAdd {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub redeemed_at: String,
    pub reward: SimpleReward,
    pub user_input: Option<String>,
}

// Revocation message data
#[derive(Debug, Serialize, Deserialize)]
pub struct RevocationData {
    pub subscription: SubscriptionDetail,
}

// Reconnect message data
#[derive(Debug, Serialize, Deserialize)]
pub struct ReconnectData {
    pub session: SessionData,
}

// Subscription details used in Notification and Revocation messages
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionDetail {
    pub id: String,
    pub status: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub version: String,
    pub cost: i32,
    pub condition: HashMap<String, serde_json::Value>, // Generic JSON object as condition structure can vary
    pub transport: TransportData,
    pub created_at: String,
}

// Transport data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TransportData {
    pub method: String,
    pub session_id: String,
}

impl<'de> Deserialize<'de> for WebSocketMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a temporary structure with metadata only
        #[derive(Deserialize)]
        struct TempWebSocketMessage {
            metadata: MessageMetadata,
            payload: Value,
        }

        let temp_message = TempWebSocketMessage::deserialize(deserializer)?;

        // Parse the payload based on the message_type
        let payload = match temp_message.metadata.message_type.as_str() {
            "session_welcome" => serde_json::from_value::<WelcomeData>(temp_message.payload)
                .map(Payload::Welcome)
                .map_err(serde::de::Error::custom)?,
            "session_keepalive" => Payload::Keepalive,
            "notification" => serde_json::from_value::<NotificationData>(temp_message.payload)
                .map(Payload::Notification)
                .map_err(serde::de::Error::custom)?,
            "session_reconnect" => serde_json::from_value::<ReconnectData>(temp_message.payload)
                .map(Payload::Reconnect)
                .map_err(serde::de::Error::custom)?,
            "revocation" => serde_json::from_value::<RevocationData>(temp_message.payload)
                .map(Payload::Revocation)
                .map_err(serde::de::Error::custom)?,
            _ => return Err(serde::de::Error::custom("unknown message type")),
        };

        Ok(WebSocketMessage {
            metadata: temp_message.metadata,
            payload,
        })
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_welcome_message() {
        let json = r#"{
            "metadata": {
                "message_id": "1",
                "message_type": "session_welcome",
                "message_timestamp": "2021-09-01T00:00:00Z"
            },
            "payload": {
                "session": {
                    "id": "123",
                    "status": "enabled",
                    "keepalive_timeout_seconds": 60,
                    "reconnect_url": "wss://example.com",
                    "connected_at": "2021-09-01T00:00:00Z"
                }
            }
        }"#;

        let message: WebSocketMessage = serde_json::from_str(json).unwrap();

        match message.payload {
            Payload::Welcome(data) => {
                assert_eq!(data.session.id, "123");
                assert_eq!(data.session.status, "enabled");
                assert_eq!(data.session.keepalive_timeout_seconds, Some(60));
                assert_eq!(
                    data.session.reconnect_url,
                    Some("wss://example.com".to_string())
                );
                assert_eq!(data.session.connected_at, "2021-09-01T00:00:00Z");
            }
            _ => panic!("Unexpected payload type"),
        }
    }

    #[test]
    fn test_event_detail() {
        let detail = json!( {
          "broadcaster_user_name": "MisterKeebs",
          "user_name": "MisterKeebs",
          "status": "unfulfilled",
          "redeemed_at": "2024-05-04T02:16:30.409485142Z",
          "id": "169d650a-d573-41f2-bff0-8e51ac6f8d40",
          "user_id": "11028617",
          "broadcaster_user_id": "11028617",
          "broadcaster_user_login": "misterkeebs",
          "user_login": "misterkeebs",
          "reward": {
            "cost": 100,
            "id": "4d2f55b1-15ed-45b2-9902-ddff7db91c7c",
            "prompt": "3 Clacks",
            "title": "3 Clacks"
          },
          "user_input": ""
        });

        _ = serde_json::from_value::<EventDetail>(detail).unwrap();
    }
}
