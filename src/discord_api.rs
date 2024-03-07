use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct DiscordApiError {
    pub code: i64,
    pub message: String,
    pub errors: Value
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiscordApiResponse<T> {
    Ok(T),
    Err(DiscordApiError)
}

impl<T> DiscordApiResponse<T> {
    pub fn into_result(self) -> Result<T, DiscordApiError> {
        match self {
            DiscordApiResponse::Ok(data) => Ok(data),
            DiscordApiResponse::Err(err) => Err(err)
        }
    }
}
