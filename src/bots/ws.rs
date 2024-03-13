use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub enum WsMessageType {
    Heartbeat(Option<i32>),
    Identify {
        token: String,
        os: String,
        browser: String,
        device: String,
    },
    UpdateVoiceState {
        guild_id: String,
        channel_id: Option<String>,
        self_mute: bool,
        self_deaf: bool,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<i32>,
    pub op: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<Value>
}

impl WsMessageType {
    pub fn into_ws_message(self) -> WsMessage {
        match self {
            WsMessageType::Heartbeat(last_ack) => {
                WsMessage {
                    t: None,
                    s: None,
                    op: 1,
                    d: last_ack.map(|v| Value::Number(serde_json::Number::from(v)))
                }
            },
            WsMessageType::Identify {token, os, browser, device} => {
                WsMessage {
                    t: None,
                    s: None,
                    op: 2,
                    d: Some(json!({
                        "token": token,
                        "properties": {
                            "$os": os,
                            "$browser": browser,
                            "$device": device,
                        }
                    }))
                }
            },
            WsMessageType::UpdateVoiceState {guild_id, channel_id, self_mute, self_deaf} => {
                WsMessage {
                    t: None,
                    s: None,
                    op: 4,
                    d: Some(json!({
                        "guild_id": guild_id,
                    }))
                }
            }
        }
    }
}
