use std::future::Future;
use std::sync::{Arc, Once};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::Duration;
use async_channel::{Receiver, Sender, unbounded};
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::tungstenite::Message;
use async_tungstenite::{tungstenite, WebSocketStream};
use futures_util::{future, SinkExt, StreamExt};
use futures_util::stream::SplitStream;
use log::{error, info};
use rand::Rng;
use tokio::runtime::Handle;
use tokio::sync::{Mutex, OnceCell};
use tokio::time;
use crate::bots::api_schema::{IncomingWsEvent, WsMessage, WsMessageType};
use crate::bots::manager::BotCommand;

pub fn ws_loop(token: String, sess_id: Arc<Mutex<Option<String>>>, recv: Arc<Receiver<BotCommand>>) -> impl Future<Output=()> {
    async move {
        let once_heartbeat = Arc::new(Once::new());
        let handle = Handle::current();
        let (ws, _r) = match connect_async("wss://gateway.discord.gg/?v=10&encoding=json").await {
            Err(e) => {
                error!("{}", e);
                return;
            },
            Ok(ws) => ws
        };
        let (write, mut read) = init_ws_conn(token, ws).await;
        handle.spawn(async move {
            let last_ack = Arc::new(AtomicI32::new(-1));

            while let Some(item) = read.next().await {
                match on_incoming_msg(item, sess_id.clone(), once_heartbeat.clone(), write.clone(), last_ack.clone()).await {
                    Err((e, str_version)) => {
                        info!("{}", str_version);
                        error!("{e}");
                    },
                    _ => {}
                };
            }
            info!("over");
        });

    }
}

async fn on_incoming_msg(item: Result<Message, tungstenite::Error>, sess_id: Arc<Mutex<Option<String>>>, once_heartbeat: Arc<Once>, ws_sender: Sender<WsMessageType>, last_ack: Arc<AtomicI32>) -> anyhow::Result<(), (anyhow::Error, String)> {
    let mut str_version = String::new();
    async  {
        str_version = item?.into_text()?;
        let msg = serde_json::from_str::<WsMessage>(str_version.as_str())?;
        match msg.s {
            Some(s) => {
                Arc::clone(&last_ack).swap(s, Ordering::Relaxed);
            },
            None => {}
        }
        match msg.d {
            Some(data) => {
                let incoming = match serde_json::from_value::<IncomingWsEvent>(data.clone()) {
                    Err(_e) => {

                        let msg_type = msg.t.unwrap_or(String::from("none"));
                        if vec![String::from("SESSIONS_REPLACE"), String::from("PRESENCE_UPDATE")].contains(&msg_type) {
                            info!("{}", data);
                        }
                        info!("message type ({}) ({}) not implemented", msg_type, msg.op);
                        return Ok(())
                    },
                    Ok(v) => v
                };
                match incoming {
                    IncomingWsEvent::Hello {heartbeat_interval} => {
                        let handle = Handle::current();
                        once_heartbeat.call_once(move || {
                            handle.spawn(spawn_heartbeat(ws_sender.clone(), Arc::clone(&last_ack), heartbeat_interval as u64));
                        });
                    },
                    IncomingWsEvent::Ready {
                        v: _v,
                        user: _u,
                        guilds,
                        session_id,
                        resume_gateway_url: _r,
                        shard: _sh,
                    } => {
                        let sid = session_id.clone();
                        let binding = sess_id.clone();
                        let mut s_id_lock = binding.lock().await;
                        let _ = s_id_lock.insert(sid);
                        info!("{}", serde_json::to_string_pretty(&guilds[2].voice_states)?);
                    },
                    _ => {
                        info!("{incoming:?}");
                    }
                }
            },
            None => {}
        }
        Ok::<(), anyhow::Error>(())
    }.await.map_err(|e| (e, str_version))

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
                Err(_) | Ok(WsMessageType::InternalDisconnect) => {
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
