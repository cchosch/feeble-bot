use std::env;
use dotenv::dotenv;
use crate::account_client::AccountClient;

mod account_client;
mod db;
mod discord_api;
mod api;

const BASE_URL: &'static str = "https://discord.com/api/v10";
const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

#[tokio::main]
async fn main() {
    dotenv().unwrap();

    // AccountClient::new();
    // let acc = AccountClient::new(String::from(env::var("TEST_TOKEN").unwrap())).await.unwrap();

    println!("Hello, world!");
}
