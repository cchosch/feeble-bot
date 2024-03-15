use diesel::{ExpressionMethods, Insertable, Queryable, Selectable};
use diesel_async::RunQueryDsl;
use log::error;
use serde::{Deserialize, Serialize};
use crate::api::DbConn;
use crate::api::err::{ApiError, ApiResult};
use crate::bots::account_client::BotClient;
use crate::db::gen_id;
use crate::schema::controlled_account::dsl::controlled_account;
use crate::schema::controlled_account::id;

#[derive(Serialize, Deserialize, Clone, Debug, Selectable, Queryable, Insertable)]
#[diesel(table_name = crate::schema::controlled_account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ControlledAccount {
    /// ID for database
    pub id: String,
    pub discord_id: String,
    pub username: String,
    #[serde(skip_serializing)]
    token: String,
    created_by: String,
}

impl ControlledAccount {
    pub fn new(account_client: &BotClient) -> Self {
        Self {
            id: gen_id(),
            discord_id: account_client.account_id.clone(),
            username: account_client.username.clone(),
            token: account_client.account_token.clone(),
            created_by: account_client.created_by.clone(),
        }
    }


    pub async fn delete_by_id(internal_id: String, conn: &mut DbConn) -> ApiResult<()> {
        match diesel::delete(controlled_account).filter(id.eq(internal_id)).execute(conn).await {
            Err(e) => {
                match e {
                    diesel::result::Error::NotFound => {
                        Err(ApiError::NotFound)
                    },
                    _ => {
                        error!("{e}");
                        Err(ApiError::InternalError)
                    }
                }
            },
            Ok(_v) => Ok(())

        }
    }

    pub async fn create(&self, conn: &mut DbConn) -> ApiResult<()> {
        match diesel::insert_into(controlled_account).values(self).execute(conn).await {
            Err(e) => {
                error!("{e}");
                Err(ApiError::InternalError)
            },
            Ok(_) => {
                Ok(())
            }
        }
    }
}
