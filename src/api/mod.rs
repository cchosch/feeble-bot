pub(crate) mod session;
pub mod auth;
mod err;

use std::env::var;
use axum::Router;
use axum::routing::post;
use crate::api::auth::sign_in;
use crate::api::session::layer::PgSessionLayer;
use crate::PROD;

/// authenticates session
#[macro_export]
macro_rules! auth_session {
    (
        $sess:expr
    ) => {
        match $sess.get_user_id() {
            None => return Err(ApiError::Unauthenticated),
            Some(uid) => uid,
        }
    };
}

pub fn get_router() -> anyhow::Result<Router> {
    let session_layer = PgSessionLayer::new(
        hex::decode(var("COOKIE_SECRET").expect("COOKIE_SECRET isn't valid"))?.as_slice(),
        PROD
    );
    Ok(Router::new().route("/login", post(sign_in)).layer(session_layer))
}

