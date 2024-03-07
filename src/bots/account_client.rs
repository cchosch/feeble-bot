use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::Message;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use crate::{BASE_URL, USER_AGENT};
use crate::discord_api::{DiscordApiError, DiscordApiResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
}

#[derive(Clone, Debug)]
pub struct AccountClient {
    pub req_client: reqwest::Client,
    pub account_id: String,
    pub username: String,
    pub account_token: String
}

impl AccountClient {
    pub async fn new(token: String) -> anyhow::Result<AccountClient> {
        let (mut conn, _r) = connect_async("wss://discord.com").await?;
        let req_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).default_headers({
            let mut dft_headers = HeaderMap::new();
            dft_headers.insert(AUTHORIZATION, HeaderValue::from_str(token.as_str())?);
            dft_headers
        }).build()?;

        let resp = req_client.get(format!("{}/users/@me", BASE_URL)).send().await?.json::<DiscordApiResponse<DiscordUser>>().await?;
        let user = match resp.into_result() {
            Err(err) => {
                return Err(anyhow::anyhow!("{}", err.message))
            },
            Ok(user) => user
        };

        Ok(AccountClient {
            req_client,
            account_id: user.id,
            username: user.username,
            account_token: token
        })
    }
}