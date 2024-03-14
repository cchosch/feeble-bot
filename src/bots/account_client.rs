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
use rand::Rng;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::runtime::Handle;
use tokio::time;
use crate::{BASE_URL, USER_AGENT};
use crate::api::DbConn;
use crate::api::err::{ApiError, ApiResult};
use crate::bots::manager::BotCommand;
use crate::bots::ws::{DumbUser, IncomingWsEvent, WsMessage, WsMessageType};
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

pub struct StallChanClosed<T> {
    inner: Arc<Sender<T>>
}

impl<T> Future for StallChanClosed<T> {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        if self.inner.is_closed() {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
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
            let r = Arc::new(r);
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
                    handle.spawn(read.for_each(move |item| {
                        let mut str_version = String::new();
                        match (|| -> anyhow::Result<()> {
                            str_version = item?.into_text()?;
                            // info!("{}", serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(str_version.as_str())?)?);
                            let msg = serde_json::from_str::<WsMessage>(str_version.as_str())?;
                            match msg.s {
                                Some(s) => {
                                    last_ack.clone().swap(s, Ordering::Relaxed);
                                },
                                None => {}
                            }
                            match msg.d {
                                Some(data) => {
                                    let incoming = match serde_json::from_value::<IncomingWsEvent>(data) {
                                        Err(_e) => {
                                            info!("message opcode {} ({}) not implemented", msg.op, msg.t.unwrap_or(String::from("none")));
                                            return Ok(())
                                        },
                                        Ok(v) => v
                                    };
                                    match incoming {
                                        IncomingWsEvent::Hello {heartbeat_interval} => {
                                            let handle = Handle::current();
                                            handle.spawn(spawn_heartbeat(write.clone(), last_ack.clone(), heartbeat_interval as u64));
                                        },
                                        IncomingWsEvent::Ready {
                                            v: _v,
                                            user: _u,
                                            guilds,
                                            session_id: _s,
                                            resume_gateway_url: _r,
                                            shard: _sh,
                                        } => {
                                            info!("{}", serde_json::to_string_pretty(&guilds[4].members)?);
                                        },
                                        _ => {
                                            info!("{incoming:?}");
                                        }
                                    }
                                },
                                None => {}
                            }
                            Ok(())
                        })() {
                            Err(e) => {
                                info!("{}", str_version);
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
fn spawn_heartbeat(write_chan: Sender<WsMessageType>, last_ack: Arc<AtomicI32>, heartbeat_interval: u64) -> impl Future<Output=()> {
    let mut thread_rng= rand::thread_rng();
    let random_sleep = thread_rng.gen_range(0..heartbeat_interval);
    async move {
        time::sleep(Duration::from_millis(random_sleep)).await;
        let mut interval = time::interval(Duration::from_millis(heartbeat_interval));
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
    let inside_wrs = write_s.clone();
    handle.spawn(async move {
        inside_wrs.send(WsMessageType::Identify {
            token,
            os: String::from("win"),
            browser: String::from("disco"),
            device: String::from("disco"),
        }).await
    });

    (write_s, read)
}
