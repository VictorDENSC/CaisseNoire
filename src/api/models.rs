use rouille::input::json::JsonError;
use serde::Serialize;

use crate::database::postgres::DbError;
use crate::teams::models::Team;

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum SuccessResponse {
    Team(Team),
    Text(String),
}

impl From<SuccessResponse> for rouille::Response {
    fn from(response: SuccessResponse) -> rouille::Response {
        rouille::Response::json(&response)
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub kind: ErrorKind,
    pub description: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    ServiceUnavailable,
    Unknown,
    NotFound,
    Json,
}

impl ErrorKind {
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorKind::ServiceUnavailable => 500,
            ErrorKind::Unknown => 500,
            ErrorKind::NotFound => 404,
            ErrorKind::Json => 400,
        }
    }
}

impl From<ErrorResponse> for rouille::Response {
    fn from(response: ErrorResponse) -> rouille::Response {
        rouille::Response::json(&response).with_status_code(response.kind.status_code())
    }
}

impl From<DbError> for ErrorResponse {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => ErrorResponse {
                kind: ErrorKind::NotFound,
                description: String::from("Not found"),
            },
            DbError::Unknown => ErrorResponse {
                kind: ErrorKind::Unknown,
                description: String::from("An internal error occured"),
            },
            DbError::ServiceUnavailable => ErrorResponse {
                kind: ErrorKind::ServiceUnavailable,
                description: String::from("The service is currently unavailable"),
            },
        }
    }
}

impl From<JsonError> for ErrorResponse {
    fn from(error: JsonError) -> ErrorResponse {
        ErrorResponse {
            kind: ErrorKind::Json,
            description: error.to_string(),
        }
    }
}
