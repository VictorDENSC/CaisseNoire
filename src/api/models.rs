use rouille::{input::json::JsonError, Response};
use serde::Serialize;

use super::database::postgres::DbError;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub kind: ErrorKind,
    pub description: String,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    Unknown,
    NotFound,
    Json,
}

impl ErrorKind {
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorKind::Unknown => 500,
            ErrorKind::NotFound => 404,
            ErrorKind::Json => 400,
        }
    }
}

impl Into<Response> for ErrorResponse {
    fn into(self) -> Response {
        Response::json(&self).with_status_code(self.kind.status_code())
    }
}

impl From<DbError> for ErrorResponse {
    fn from(error: DbError) -> Self {
        match error {
            DbError::PoolBuilding => ErrorResponse {
                kind: ErrorKind::Unknown,
                description: String::from("The connection to the database has failed"),
            },
            DbError::ConnectionAccess => ErrorResponse {
                kind: ErrorKind::Unknown,
                description: String::from("The database has returned an unknown error"),
            },
            DbError::NotFound => ErrorResponse {
                kind: ErrorKind::NotFound,
                description: String::from("Not found"),
            },
            DbError::Unknown => ErrorResponse {
                kind: ErrorKind::Unknown,
                description: String::from("An internal error occured"),
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
