use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use crate::bots::account_client::AccountClient;

#[derive(Serialize, Deserialize, Clone, Debug, Selectable, Queryable, Insertable)]
#[diesel(table_name = crate::schema::account_mapping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountMapping {
    id: String,
    real_account_id: String,
    controlled_account_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Selectable, Queryable, Insertable)]
#[diesel(table_name = crate::schema::discord_account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DiscordAccount {
    id: String,
    account_id: String,
    account_name: String,
    account_token: String,
    created_by: String,
}

impl DiscordAccount {
    pub fn new(account_client: &AccountClient) -> DiscordAccount {
        DiscordAccount {
            id: crate::db::gen_id(),
            account_id: account_client.account_id.clone(),
            account_name: account_client.username.clone(),
            account_token: account_client.account_token.clone(),
            created_by: account_client.created_by.clone(),
        }
    }
}
