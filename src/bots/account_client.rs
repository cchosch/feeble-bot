use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::Duration;
use async_channel::{Receiver, RecvError, Sender, unbounded};
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::tungstenite::{Message, WebSocket};
use async_tungstenite::{tungstenite, WebSocketStream};
use futures_util::{future, SinkExt, StreamExt};
use futures_util::stream::SplitStream;
use log::{error, info};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::runtime::Handle;
use tokio::time;
use crate::{BASE_URL, USER_AGENT};
use crate::api::DbConn;
use crate::api::err::{ApiError, ApiResult};
use crate::bots::manager::BotCommand;
use crate::bots::ws::{WsMessage, WsMessageType};
use crate::db::gen_id;
use crate::discord_api::{DiscordApiResponse};
use crate::schema::controlled_account::dsl::controlled_account;
use crate::schemas::controlled_account::ControlledAccount;

/// Partial version of Discord's internal user struct
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
}

#[derive(Clone, Debug)]
pub struct BotClient {
    pub req_client: reqwest::Client,
    pub command_chan: Option<Receiver<BotCommand>>,
    pub account_id: String,
    pub username: String,
    pub account_token: String,
    pub created_by: String
}

fn map_err_invalid (e: impl std::error::Error) -> ApiError {
    error!("{e}");
    ApiError::InternalError
}

impl BotClient {
    pub async fn new(token: String, created_by: String) -> ApiResult<BotClient> {
        // let (mut conn, _r) = connect_async("wss://discord.com").await?;
        let req_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).default_headers({
            let mut dft_headers = HeaderMap::new();
            dft_headers.insert(AUTHORIZATION, HeaderValue::from_str(token.as_str()).map_err(map_err_invalid)?);
            dft_headers
        }).build().map_err(map_err_invalid.clone())?;

        let resp = req_client.get(format!("{}/users/@me", BASE_URL)).send().await.map_err(map_err_invalid)?.json::<DiscordApiResponse<DiscordUser>>().await.map_err(map_err_invalid)?;
        let user = match resp.into_result() {
            Err(_err) => {
                return Err(ApiError::BadRequest(String::from("Invalid token")))
            },
            Ok(user) => user
        };

        Ok(BotClient {
            req_client,
            command_chan: None,
            account_id: user.id,
            username: user.username,
            account_token: token,
            created_by
        })
    }

    pub fn spawn_ws_conn(&self) -> impl Future<Output=Sender<BotCommand>> {
        let token = self.account_token.clone();
        async move {
            let (s, r) = unbounded();
            let handle = Handle::current();
            thread::spawn(move || {
                handle.spawn(async move {
                    let handle = Handle::current();
                    let (ws, _r) = match connect_async("wss://gateway.discord.gg/?v=10&encoding=json").await {
                        Err(e) => {
                            error!("{}", e);
                            return;
                        },
                        Ok(ws) => ws
                    };
                    let (write, read) = init_ws_conn(token, ws).await;
                    let last_ack = Arc::new(AtomicI32::new(-1));
                    handle.spawn(spawn_heartbeat(write.clone(), last_ack.clone()));
                    handle.spawn(read.for_each(move |item| {
                        match (|| -> anyhow::Result<()> {
                            let msg = serde_json::from_str::<WsMessage>(item?.into_text()?.as_str())?;
                            last_ack.clone().swap(msg.s.unwrap_or(-1), Ordering::Relaxed);
                            info!("{msg:?}");
                            Ok(())
                        })() {
                            Err(e) => {
                                error!("{e}");
                            },
                            _ => {}
                        };
                        future::ready(())
                    }));
                });
            });
            s
        }
    }

    pub fn to_discord_account(&self) -> ControlledAccount {
        ControlledAccount::new(&self)
    }
}
async fn spawn_heartbeat(write_chan: Sender<WsMessageType>, last_ack: Arc<AtomicI32>) {
    let mut interval = time::interval(Duration::from_secs(20));
    interval.tick().await;
    loop {
        interval.tick().await;
        let last_ack = if last_ack.load(Ordering::Relaxed) < 0 {
            None
        } else {
            Some(last_ack.load(Ordering::Relaxed))
        };
        match write_chan.send(WsMessageType::Heartbeat(last_ack)).await {
            Err(_e) => {
                break;
            },
            Ok(()) => {}
        };
    }
}

async fn init_ws_conn(token: String, ws: WebSocketStream<ConnectStream>) -> (Sender<WsMessageType>, SplitStream<WebSocketStream<ConnectStream>>) {
    let (mut write, read) = ws.split();
    let (write_s, write_r) = unbounded::<WsMessageType>();
    let handle = Handle::current();
    handle.spawn(async move {
        loop {
            match write_r.recv().await {
                Err(_e) => {
                    write_r.close();
                    return;
                },
                Ok(v) => {
                    let str_msg = match serde_json::to_string(&v.into_ws_message()) {
                        Err(e) => {
                            error!("{e}");
                            continue;
                        },
                        Ok(v) => v
                    };
                    info!("{str_msg}");
                    match write.send(Message::Text(str_msg)).await {
                        Err(e) => {
                            match e {
                                tungstenite::Error::ConnectionClosed |
                                tungstenite::Error::AlreadyClosed => {
                                    write_r.close();
                                    return;
                                },
                                _ => {
                                    error!("{e}");
                                }
                            }
                        }
                        Ok(_) => {}
                    };
                }
            }
        }
    });
    write_s.send(WsMessageType::Identify {
        token,
        os: String::from("win"),
        browser: String::from("disco"),
        device: String::from("disco"),
    }).await.ok();

    (write_s, read)
}
