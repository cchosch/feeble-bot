use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::Json;
use axum_core::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use log::error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiErrorMessage {
    error_msg: String,
    user_msg: String,
}

fn error_msg(err: &str, usr_msg: Option<&str>) -> Json<ApiErrorMessage> {
    Json(ApiErrorMessage {
        error_msg: err.to_string(),
        user_msg: usr_msg.unwrap_or("Something went wrong").to_string(),
    })
}

#[macro_export]
macro_rules! conv_search_err {
    (
        $e:expr
    ) => {
        match $e {
            diesel::result::Error::NotFound => ApiError::NotFound,
            _ => {
                error!("{}", $e);
                ApiError::InternalError
            }
        }
    };
}

pub type ApiResult<T> = Result<T, ApiError>;
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Unauthenticated")]
    Unauthenticated,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not found")]
    NotFound,
    #[error("Something went wrong")]
    InternalError,
    #[error("{1}")]
    Custom(StatusCode, String, Option<String>),
    #[error("")]
    BadRequest(String),
    #[error(transparent)]
    JsonDecodeError(#[from] JsonRejection),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Unauthenticated => (
                StatusCode::FORBIDDEN,
                error_msg("Unauthenticated", Some("Please log in")),
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, error_msg("Resource not found", None)),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, error_msg(msg.as_str(), None)),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_msg("Internal server error", None),
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                error_msg(
                    "Unauthorized",
                    Some("You don't have the permissions necessary to view this resource"),
                ),
            ),
            ApiError::Custom(stat, msg, None) => (stat, error_msg(msg.as_str(), None)),
            ApiError::Custom(stat, msg, Some(usr_msg)) => {
                (stat, error_msg(msg.as_str(), Some(usr_msg.as_str())))
            }
            ApiError::JsonDecodeError(e) => (
                StatusCode::BAD_REQUEST,
                error_msg(e.to_string().as_str(), None),
            ),
        }
            .into_response()
    }
}