use axum::{Extension, Json};
use axum_core::response::IntoResponse;
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use serde_json::json;
use crate::api::ApiContext;
use crate::api::session::WritableSession;
use crate::api::err::ApiError;
use crate::auth_session;
use crate::bots::account_client::AccountClient;
use crate::schemas::controlled_account::ControlledAccount;

#[derive(Deserialize)]
pub struct CreateBotPayload {
    token: String
}

#[derive(Deserialize)]
pub struct MapBotPayload {
    controlled_internal_id: String,
    mapped_discord_id: String,
}

pub async fn post_bot(
    sess: WritableSession,
    Extension(ctx): Extension<ApiContext>,
    WithRejection(payload, _): WithRejection<Json<CreateBotPayload>, ApiError>
) -> impl IntoResponse {
    let uid = auth_session!(sess);
    let acc_client = AccountClient::new(payload.token.clone(), uid).await?;
    let acc = acc_client.to_discord_account();
    acc.create(&mut ctx.get_conn().await?).await?;

    Ok(Json(acc))
}

pub async fn delete_mapping() {}

pub async fn map_bot(
    sess: WritableSession
) -> impl IntoResponse {
    let uid = auth_session!(sess);

    Ok(Json(json!({})))
}

