use axum::{Extension, Json};
use axum_core::response::IntoResponse;
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use crate::api::ApiContext;
use crate::api::err::{ApiError, ApiResult};
use crate::api::session::WritableSession;
use crate::auth_session;
use crate::schemas::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInPayload {
    username: String,
    password: String,
}

pub async fn sign_in(
    mut sess: WritableSession,
    Extension(ctx): Extension<ApiContext>,
    WithRejection(Json(payload), _): WithRejection<Json<SignInPayload>, ApiError>
) -> ApiResult<Json<User>> {
    let u = User::get_by_username(payload.username, &mut ctx.get_conn().await?).await?;

    if !u.compare_passwords(payload.password) {
        return Err(ApiError::Unauthenticated);
    }
    sess.set_user(&u);

    Ok(Json(u))
}

pub async fn get_me(
    sess: WritableSession,
    Extension(ctx): Extension<ApiContext>
) -> impl IntoResponse {
    let uid = auth_session!(sess);
    User::get_by_id(uid, &mut ctx.get_conn().await?).await.map(|u| Json(u))
}

pub async fn sign_out(
    mut sess: WritableSession,
) -> impl IntoResponse {
    sess.destroy();
    "{}"
}
