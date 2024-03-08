use std::borrow::Cow;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::task::{Context, Poll};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use async_trait::async_trait;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::{body::Body, Extension, http::Request, response::Response};
use axum_extra::extract::cookie::Key;
use axum_extra::extract::CookieJar;

use cookie::Cookie;
use futures_util::future::BoxFuture;
use log::{debug, error, info};
use tokio::sync::OwnedRwLockWriteGuard;
use tokio::sync::RwLock;
use tower::Layer;
use tower_service::Service;
use urlencoding::decode;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use diesel::{QueryResult, SelectableHelper};
use crate::api::DbConn;
use crate::schema::sessions::dsl::sessions;
use crate::schema::sessions::{sess_cookie, sess_id};
use crate::api::session::session::{DBSession, Session, SessionHandle, SESSION_COOKIE_NAME};
use crate::db::ConnPool;

#[derive(Clone)]
pub struct PgSessionLayer {
    secure: bool,
    key: Key,
    session_cookie: &'static str,
    pool: Arc<ConnPool>
}

unsafe impl Send for PgSessionLayer {}
unsafe impl Sync for PgSessionLayer {}

const BASE64_DIGEST_LEN: usize = 44;

impl PgSessionLayer {
    pub fn new(secret: &[u8], secure: bool, pool: Arc<ConnPool>) -> Self {
        if secret.len() < 64 {
            panic!("Secret must be at least 64 bytes");
        }
        Self {
            secure,
            pool,
            key: Key::from(secret),
            session_cookie: SESSION_COOKIE_NAME,
        }
    }

    /// Remove session from database
    pub async fn destroy_sess(&self, sess: Session, conn: &mut DbConn) -> QueryResult<Vec<DBSession>> {
        info!("destroying sess");
        diesel::delete(sessions.filter(sess_id.eq(sess.id))).load::<DBSession>(conn).await
    }

    /// Store session in database
    pub async fn db_store(&self, sess: Session, conn: &mut DbConn) -> QueryResult<()> {
        diesel::delete(sessions)
            .filter(sess_id.eq(sess.clone().id))
            .load::<DBSession>(conn)
            .await
            .ok();
        let res = diesel::insert_into(sessions)
            .values(&sess.to_db_session())
            .returning(DBSession::as_returning())
            .get_result(conn).await;
        if let Err(e) = res {
            error!("{e}");
            return Err(e);
        }
        return Ok(());
    }

    /// Lookup session in db with unsigned cookie.
    pub async fn db_lookup(&self, raw_cookie: String, conn: &mut DbConn) -> QueryResult<Option<Session>> {
        let dec_cookie = decode(raw_cookie.as_str()).unwrap();
        let cookie = PgSessionLayer::split_signature(dec_cookie.deref()).1;
        let db_res: QueryResult<DBSession> = sessions
            .filter(sess_cookie.eq(cookie))
            .first::<DBSession>(conn).await;
        if let Err(e) = db_res {
            if let diesel::NotFound = e {
                return Ok(None);
            }
            return Err(e);
        }
        Ok(Some(db_res.unwrap().to_session()))
    }

    /// Load cookie from raw cookie value, if it's not found, or it's expired, this function will
    /// create a new cookie for you. Returns session handle and a bool representing whether
    /// session was just created
    async fn load_or_create(&self, conn: &mut DbConn, cookie_value: Option<&String>) -> (SessionHandle, bool) {
        let session = match cookie_value {
            Some(cookie_value) => self.db_lookup(cookie_value.clone(), conn).await.ok().flatten(),
            None => None,
        };

        if let Some(sess) = session {
            // prob not going to run because browser won't send expired cookie
            if sess.is_expired() {
                let _ = self.destroy_sess(sess, conn).await;
            } else {
                return (Arc::new(RwLock::new(sess)), true);
            }
        }

        (Arc::new(RwLock::new(Session::default())), false)
    }
    // the following is reused from
    // https://github.com/SergioBenitez/cookie-rs/blob/master/src/secure/signed.rs#L33-L43
    /// Signs the cookie's value providing integrity and authenticity.
    fn sign_cookie(&self, cookie: &mut Cookie<'_>) {
        // Compute HMAC-SHA256 of the cookie's value.
        let mut mac = Hmac::<Sha256>::new_from_slice(self.key.signing()).expect("good key");
        mac.update(cookie.value().as_bytes());

        // Cookie's new value is [MAC | original-value].
        let mut new_value = base64::encode(mac.finalize().into_bytes());
        new_value.push_str(cookie.value());
        cookie.set_value(new_value);
    }

    // the following is reused from
    // https://github.com/SergioBenitez/cookie-rs/blob/master/src/secure/signed.rs#L45-L63
    /// Given a signed value `str` where the signature is prepended to `value`,
    /// verifies the signed value and returns it. If there's a problem, returns
    /// an `Err` with a string describing the issue.
    fn verify_signature(&self, cookie_value: &str) -> Result<String, &'static str> {
        if cookie_value.len() < BASE64_DIGEST_LEN {
            return Err("length of value is <= BASE64_DIGEST_LEN");
        }

        // Split [MAC | original-value] into its two parts.
        let (digest_str, value) = cookie_value.split_at(BASE64_DIGEST_LEN);
        let digest = BASE64_STANDARD.decode(digest_str).map_err(|_| "bad base64 digest")?;

        // Perform the verification.
        let mut mac = Hmac::<Sha256>::new_from_slice(self.key.signing()).expect("bad key");
        mac.update(value.as_bytes());
        mac.verify_slice(digest.as_slice())
            .map(|_| value.to_string())
            .map_err(|_| "value did not verify")
    }

    /// Split cookie signature between mac & original
    fn split_signature(cookie_value: &str) -> (&str, &str) {
        cookie_value.split_at(BASE64_DIGEST_LEN)
    }
}

#[derive(Clone)]
pub struct PgSessionMiddleware<S> {
    inner: S,
    layer: PgSessionLayer,
}

impl<S> Layer<S> for PgSessionLayer {
    type Service = PgSessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        return PgSessionMiddleware {
            inner,
            layer: self.clone(),
        };
    }
}

impl<S> Service<Request<Body>> for PgSessionMiddleware<S>
    where
        S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
        S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        // get cookies as hashmap
        let cookies = HashMap::<String, String>::from_iter(
            // get cookie jar from headers
            CookieJar::from_headers(&request.headers().clone())
                .iter()
                // map it to key value
                .map(|cook| (cook.name().to_string(), cook.value().to_string()))
                // collect as vector
                .collect::<Vec<(String, String)>>(),
        );

        // use this in closure
        let mut this = self.clone();

        // basically leak closure for 'static
        Box::pin(async move {
            let mut cook = cookies.get(this.layer.session_cookie);

            // if invalid cookie passed, set cook to None
            if let Err(_) = this.layer.verify_signature(
                decode(cook.unwrap_or(&String::new()).as_str())
                    .unwrap_or(Cow::default())
                    .deref(),
            ) {
                cook = None;
            }

            // get handle
            let (sess_handle, in_db) = this.layer.load_or_create(&mut this.layer.pool.get().await.unwrap(), cook).await;

            request.extensions_mut().insert(sess_handle.clone());
            // send request to next layer
            let mut response: Response = this.inner.call(request).await?;

            // get session after endpoint
            let session = sess_handle.read().await.to_owned();
            let has_changed = session.get_changed();

            // if session's not in the db and hasn't changed, don't store session in db and return
            if !in_db && !has_changed {
                return Ok(response);
            }

            let mut db_conn = this.layer.pool.get().await.unwrap();
            // if session has expressly been marked for deletion, delete it from the db, and return
            if session.to_del() {
                // if it's in the db just created, remove it from db
                if in_db {
                    this.layer.destroy_sess(session, &mut db_conn).await.unwrap();
                }
                // otherwise its not even in db so just return
                return Ok(response);
            }
            // if session has been modified, store modifications in db
            if has_changed {
                this.layer.db_store(session.clone(), &mut db_conn).await.unwrap();
            }
            drop(db_conn);

            let mut sid_cook = Cookie::build((SESSION_COOKIE_NAME, session.cookie))
                .secure(this.layer.secure.clone())
                .http_only(true)
                .path("/")
                .build();

            this.layer.sign_cookie(&mut sid_cook);

            response.headers_mut().insert(
                SET_COOKIE,
                HeaderValue::from_str(sid_cook.encoded().to_string().as_str()).unwrap(),
            );

            Ok(response)
        })
    }
}

// An extractor which provides a writable session. Sessions may have only one
// writer.
#[derive(Debug)]
pub struct WritableSession {
    session: OwnedRwLockWriteGuard<Session>,
}

impl Deref for WritableSession {
    type Target = OwnedRwLockWriteGuard<Session>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl DerefMut for WritableSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for WritableSession
    where
        S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session_handle): Extension<SessionHandle> =
            Extension::from_request_parts(parts, state)
                .await
                .expect("Session extension missing. Is the session layer installed?");
        let session = session_handle.write_owned().await;

        Ok(Self { session })
    }
}