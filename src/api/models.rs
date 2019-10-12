use rouille::input::json::JsonError;
use serde::Serialize;

use crate::database::postgres::DbError;
use crate::sanctions::utils::parameters::{ParameterError, ParameterErrorKind};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub kind: ErrorKind,
    pub description: String,
}

impl ErrorResponse {
    pub fn not_found() -> ErrorResponse {
        ErrorResponse {
            kind: ErrorKind::NotFound,
            description: String::from("Not found"),
        }
    }

    pub fn bad_parameter(description: String) -> ErrorResponse {
        ErrorResponse {
            kind: ErrorKind::BadParameter,
            description,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    ServiceUnavailable,
    Unknown,
    NotFound,
    Json,
    BadReference,
    DuplicatedField,
    BadParameter,
}

impl ErrorKind {
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorKind::ServiceUnavailable => 500,
            ErrorKind::Unknown => 500,
            ErrorKind::NotFound => 404,
            ErrorKind::Json => 400,
            ErrorKind::BadReference => 400,
            ErrorKind::DuplicatedField => 400,
            ErrorKind::BadParameter => 400,
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
            DbError::ForeignKeyViolation(description) => ErrorResponse {
                kind: ErrorKind::BadReference,
                description,
            },
            DbError::UniqueViolation(description) => ErrorResponse {
                kind: ErrorKind::DuplicatedField,
                description,
            },
        }
    }
}

impl From<ParameterError> for ErrorResponse {
    fn from(error: ParameterError) -> ErrorResponse {
        match error.kind {
            ParameterErrorKind::UnvalidValue {
                parameter_value,
                reason,
            } => ErrorResponse::bad_parameter(format!(
                "{} is not a possible value for the {} parameter. {}.",
                parameter_value, error.parameter_name, reason
            )),
            ParameterErrorKind::UnvalidType { expected_type } => {
                ErrorResponse::bad_parameter(format!(
                    "The {} parameter must be a {}.",
                    error.parameter_name, expected_type
                ))
            }
            ParameterErrorKind::UnvalidCombination { missing_parameters } => {
                ErrorResponse::bad_parameter(format!(
                    "The {} parameter must be combined with these parameters : {}.",
                    error.parameter_name,
                    missing_parameters.join(", ")
                ))
            }
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

pub mod test_utils {
    use rouille::Request;
    use serde_json::Value;

    pub struct RequestBuilder;

    impl RequestBuilder {
        fn json_header() -> (String, String) {
            ("Content-Type".to_string(), "application/json".to_string())
        }

        pub fn get(url: String) -> Request {
            Request::fake_http("GET", url, vec![RequestBuilder::json_header()], vec![])
        }

        pub fn post(url: String, data: &Value) -> Request {
            let serialized_data = serde_json::to_vec(data).expect("Failed to serialize data");

            Request::fake_http(
                "POST",
                url,
                vec![RequestBuilder::json_header()],
                serialized_data,
            )
        }

        pub fn delete(url: String) -> Request {
            Request::fake_http("DELETE", url, vec![RequestBuilder::json_header()], vec![])
        }
    }

}
