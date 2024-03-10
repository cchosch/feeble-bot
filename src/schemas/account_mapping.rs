use diesel::{ExpressionMethods, Insertable, Queryable, QueryDsl, Selectable};
use diesel_async::RunQueryDsl;
use log::error;
use serde::{Deserialize, Serialize};
use crate::api::DbConn;
use crate::api::err::{ApiError, ApiResult};
use crate::db::gen_id;
use crate::schema::account_mapping::dsl::account_mapping;
use crate::schema::account_mapping::id;
use crate::schemas::controlled_account::ControlledAccount;

#[derive(Serialize, Deserialize, Clone, Debug, Selectable, Queryable, Insertable)]
#[diesel(table_name = crate::schema::account_mapping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountMapping {
    id: String,
    mapped_discord_id: String,
    controlled_username: String,
    controlled_discord_id: String,
    controlled_internal_id: String,
}


impl AccountMapping {
    pub fn new(controlled_account: &ControlledAccount, mapped_discord_id: String) -> Self {
        Self {
            id: gen_id(),
            mapped_discord_id,
            controlled_username: controlled_account.username.clone(),
            controlled_internal_id: controlled_account.id.clone(),
            controlled_discord_id: controlled_account.discord_id.clone()
        }
    }

    pub async fn create(&self, conn: &mut DbConn) -> ApiResult<()> {
        match diesel::insert_into(account_mapping).values(self).execute(conn).await {
            Err(e) => {
                error!("{e}");
                Err(ApiError::InternalError)
            },
            Ok(_v) => {
                Ok(())
            }
        }
    }
}

