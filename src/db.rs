use std::env;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use rand::Rng;
use crate::api::DbConn;

pub type ConnPool = Pool<AsyncPgConnection>;

/// Generates random 8 byte integer encod as hex
pub fn gen_id() -> String {
    hex::encode(rand::thread_rng().gen::<[u8; 8]>())
}

/// Creates new `Pool<ConnectionManager<PgConnection>>>`
pub(crate) fn gen_pool() -> ConnPool {
    dotenv::dotenv().expect("no .env file found");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    let pool =
        Pool::builder(manager).build().unwrap();
    pool
}

