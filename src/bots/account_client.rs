use log::error;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use crate::{BASE_URL, USER_AGENT};
use crate::api::err::{ApiError, ApiResult};
use crate::db::gen_id;
use crate::discord_api::{DiscordApiResponse};
use crate::schemas::account_mapping::DiscordAccount;

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
    pub account_token: String,
    pub created_by: String
}

fn map_err_invalid (e: impl std::error::Error) -> ApiError {
    error!("{e}");
    ApiError::InternalError
}

impl AccountClient {
    pub async fn new(token: String, created_by: String) -> ApiResult<AccountClient> {
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

        Ok(AccountClient {
            req_client,
            account_id: user.id,
            username: user.username,
            account_token: token,
            created_by
        })
    }

    pub fn to_discord_account(&self) -> DiscordAccount {
        DiscordAccount::new(&self)
    }
}