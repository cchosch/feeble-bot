use std::sync::Mutex;
use async_channel::{Receiver, Sender, unbounded};
use tokio::sync::RwLock;
use crate::bots::account_client::BotClient;

#[derive(Debug, Clone)]
pub enum BotCommand {
    LeaveChannel(String, String),
    JoinChannel(String, String),
    Disconnect,
}

#[derive(Debug)]
pub struct BotManager {
    bots: RwLock<Vec<(Option<Mutex<Sender<BotCommand>>>, BotClient)>>,
}

impl BotManager {
    pub fn new() -> Self {
        Self {
            bots: RwLock::new(vec![]),
        }
    }

    pub async fn new_bot(&self, new_client: BotClient) {
        let s = new_client.spawn_ws_conn().await;
        self.bots.write().await.push((None, new_client))
    }
}