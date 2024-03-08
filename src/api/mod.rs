pub(crate) mod session;
pub mod auth;
pub mod err;

use std::env::var;
use std::sync::Arc;
use axum::Router;
use axum::routing::post;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Object;
use log::error;
use tower_http::add_extension::AddExtensionLayer;
use tower::ServiceBuilder;
use crate::api::auth::sign_in;
use crate::api::err::ApiError;
use crate::api::session::layer::PgSessionLayer;
use crate::db::ConnPool;
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

pub type DbConn = Object<AsyncPgConnection>;
#[derive(Clone)]
pub struct ApiContext {
    pub db: Arc<ConnPool>,
}

impl ApiContext {
    pub async fn get_conn(&self) -> Result<Object<AsyncPgConnection>, ApiError> {
        match self.db.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => {
                error!("Error getting connection: {}", e);
                Err(ApiError::InternalError)
            }
        }
    }
}


pub fn get_router(p: ConnPool) -> anyhow::Result<Router> {
    let conn_pool = Arc::new(p);
    let session_layer = PgSessionLayer::new(
        hex::decode(var("COOKIE_SECRET").expect("COOKIE_SECRET isn't valid"))?.as_slice(),
        PROD,
        conn_pool.clone(),
    );
    Ok(Router::new().route("/login", post(sign_in)).layer(session_layer).layer(ServiceBuilder::new().layer(AddExtensionLayer::new(
        ApiContext {
            db: conn_pool,
        }
    ))))
}

