use axum::Json;
use axum_core::response::IntoResponse;
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use crate::api::session::WritableSession;
use crate::api::err::ApiError;
use crate::auth_session;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInPayload {
    username: String,
    password: String,
}

pub async fn sign_in(
    sess: WritableSession,
    WithRejection(Json(payload), _): WithRejection<Json<SignInPayload>, ApiError>
) -> impl IntoResponse {
    auth_session!(sess);
    Ok("")
}
