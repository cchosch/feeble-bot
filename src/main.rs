use std::env;
use std::net::SocketAddr;
use axum::Router;
use dotenv::dotenv;
use tokio::net::TcpListener;
use crate::api::get_router;

mod db;
mod discord_api;
mod api;
mod bots;

const BASE_URL: &'static str = "https://discord.com/api/v10";
const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

async fn init_app() -> anyhow::Result<Router> {
    Ok(Router::new().nest_service("/api", get_router()))
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();

    // AccountClient::new();
    // let acc = AccountClient::new(String::from(env::var("TEST_TOKEN").unwrap())).await.unwrap();
    let app = init_app().await.unwrap();
    // test_layout();

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("Hello, world!");
}
