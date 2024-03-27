use std::env;
use std::net::SocketAddr;
use axum::Router;
use diesel_async::RunQueryDsl;
use dotenv::dotenv;
use log::error;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use crate::api::{DbConn, get_router};
use crate::bots::account_client::BotClient;
use crate::db::{gen_pool, init_db};
use crate::schema::users::dsl::users;
use crate::schemas::User;
use crate::util::log::init_logger;

mod db;
mod discord_api;
mod api;
mod bots;
mod schemas;
pub(crate) mod util;
pub mod schema;

const BASE_URL: &'static str = "https://discord.com/api/v10";
const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";
pub const PROD: bool = cfg!(not(debug_assertions));

async fn create_first_user(c: &mut DbConn) {
    match users.first::<User>(c).await {
        Ok(_) => return,
        Err(e) => {
            match e {
                diesel::result::Error::NotFound => (),
                _ => {
                    error!("{e}");
                    return;
                }
            }
        }
    }
    let u = User::new(String::from("cchosch"), String::from("admin"), String::new());
    diesel::insert_into(users).values(&u).execute(c).await.unwrap();
}

async fn init_app() -> anyhow::Result<Router> {
    let pool = gen_pool();
    init_db(pool.clone()).await?;
    create_first_user(&mut pool.get().await?).await;
    let cors = if PROD {
        CorsLayer::new()
    } else {
        CorsLayer::very_permissive()
    };

    Ok(Router::new().nest_service("/api", get_router(pool)?).layer(cors))
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();
    init_logger().unwrap();

    // AccountClient::new();
    let acc = BotClient::new(String::from(env::var("TEST_TOKEN").unwrap()), String::from("jajajaj")).await.unwrap();
    tokio::spawn(acc.spawn_ws_conn());
    let app = init_app().await.unwrap();
    // test_layout();

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
