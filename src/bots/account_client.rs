use std::future::Future;
use std::sync::{Arc};
use tokio::sync::Mutex;
use async_channel::{Receiver, RecvError, Sender, unbounded};
use futures_util::{future, SinkExt, StreamExt};
use log::{error, info};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use crate::{BASE_URL, USER_AGENT};
use crate::api::err::{ApiError, ApiResult};
use crate::bots::manager::BotCommand;
use crate::bots::ws::ws_loop;
use crate::discord_api::{DiscordApiResponse};
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
        let session_id = Arc::new(Mutex::new(None::<String>));
        async move {
            let (s, r) = unbounded();
            let r = Arc::new(r);
            ws_loop(token.clone(), session_id, r).await;
            s
        }
    }

    pub fn to_discord_account(&self) -> ControlledAccount {
        ControlledAccount::new(&self)
    }
}

