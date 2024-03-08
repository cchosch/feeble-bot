use axum::Json;
use axum_core::response::IntoResponse;
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use crate::api::session::WritableSession;
use crate::api::err::ApiError;
use crate::auth_session;
use crate::bots::account_client::AccountClient;
use crate::schemas::account_mapping::DiscordAccount;

#[derive(Deserialize)]
pub struct CreateBotPayload {
    token: String
}

pub async fn post_bot(
    sess: WritableSession,
    WithRejection(payload, _): WithRejection<Json<CreateBotPayload>, ApiError>
) -> impl IntoResponse {
    let uid = auth_session!(sess);
    let acc_client = AccountClient::new(payload.token.clone()).await?;

    Ok("post bot")
}

