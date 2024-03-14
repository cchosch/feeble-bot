use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DumbUser {
    id: String,
    username: Option<String>,
    public_flags: Option<i64>,
    global_name: Option<String>,
    discriminator: Option<String>,
    avatar: Option<String>
}

pub struct DiscordUser {
    avatar: Option<String>,
    communication_disabled_until: Option<String>,
    deaf: Option<bool>,
    flags: i32,
    joined_at: String,
    mute: Option<bool>,
    nick: Option<String>,
    pending: Option<bool>,
    premium_since: Option<String>,
    roles: Vec<String>,
    user: DumbUser
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadyGuild {
    /*
    system_channel_flags: i32,
    system_channel_id: String,
    vanity_url_code: Option<String>,
    verification_level: i32,
    version: i64,

     */
    pub id: String,
    pub name: String,

    pub joined_at: String,
    pub large: bool,
    pub unavailable: Option<bool>,
    pub member_count: i32,
    pub voice_states: Vec<Value>,
    pub members: Vec<Value>,
    pub channels: Vec<Value>,
    // threads: Vec<Value>,
    // presences: Vec<Value>,
    // stage_instances: Vec<Value>,
    // guild_scheduled_events: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IncomingWsEvent {
    PresenceUpdate {
        user: DumbUser,
        status: String,
        guild_id: String,
    },
    Hello {
        heartbeat_interval: i32,
    },
    Ready {
        v: i32,
        user: DumbUser,
        guilds: Vec<ReadyGuild>,
        session_id: String,
        resume_gateway_url: String,
        shard: Option<Vec<i32>>,
    },
}

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
    },
    UpdatePresence {
        since: Option<i64>,
        activities: Option<Vec<Value>>,
        status: String,
        afk: bool,
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
                        "channel_id": channel_id,
                        "self_mute": self_mute,
                        "self_deaf": self_deaf,
                    }))
                }
            },
            WsMessageType::UpdatePresence {since, activities, status, afk} => {
                WsMessage {
                    t: None,
                    s: None,
                    op: 3,
                    d: Some(json!({
                        "since": since,
                        "activities": activities.unwrap_or(vec![]),
                        "status": status,
                        "afk": afk,
                    }))
                }
            }
        }
    }
}
