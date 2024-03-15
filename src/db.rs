use std::env;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use rand::{random};
use crate::schema::users::dsl::users;
use crate::schemas::User;

pub type ConnPool = Pool<AsyncPgConnection>;

/// Generates random 8 byte integer encod as hex
pub fn gen_id() -> String {
    hex::encode(random::<[u8; 8]>())
}

pub async fn init_db(pool: ConnPool) -> anyhow::Result<()> {
    let mut conn = pool.get().await?;
    match users.first::<User>(&mut conn).await {
        Err(e) => {
            match e {
                diesel::result::Error::NotFound => {
                    let u = User::new(String::from("cchosch"), String::from("admin"), String::new());
                    diesel::insert_into(users).values(&u).execute(&mut conn).await?;
                    return Ok(())
                },
                _ => {
                    Err(e)?
                }
            }
        },
        Ok(_u) => {return Ok(())}
    }
    Ok(())
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

