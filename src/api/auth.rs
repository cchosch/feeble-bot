use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use crate::api::ApiContext;
use crate::api::err::{ApiError, ApiResult};
use crate::api::session::WritableSession;
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
