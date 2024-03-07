use std::env;
use std::time::Duration;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use lazy_static::lazy_static;
use rand::Rng;

lazy_static! {
    pub static ref CONN_POOL: Pool<ConnectionManager<PgConnection>> = gen_pool();
}

// PooledConnection<ConnectionManager<PgConnection>>
// impl Connection<Backend=Pg> + LoadConnection
/// Get `PooledConnection` from `CONN_POOL`
pub fn get_conn() -> PooledConnection<ConnectionManager<PgConnection>> {
    return CONN_POOL.get().unwrap();
}

/// Generates random 8 byte integer encod as hex
pub fn gen_id() -> String {
    hex::encode(rand::thread_rng().gen::<[u8; 8]>())
}

/// Creates new `Pool<ConnectionManager<PgConnection>>>`
fn gen_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv::dotenv().expect("no .env file found");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    let pool = Pool::builder()
        .test_on_check_out(true)
        .max_size(30)
        .connection_timeout(Duration::from_secs(1))
        .build(manager)
        .expect("Could not build connection pool");

    pool
}
