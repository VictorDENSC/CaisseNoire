use rouille::{start_server, Response};
use std::env::{var, VarError};

use caisse_noire::api::{models::ErrorResponse, routes::handle_request};
use caisse_noire::database::postgres::init_db_connection;

fn extract_var(var_name: &str) -> Result<String, VarError> {
    use dotenv::dotenv;

    dotenv().ok();
    var(var_name)
}

fn with_cors(response: Response) -> Response {
    match extract_var("ENABLED_ORIGIN").ok() {
        Some(origin) => response
            .with_additional_header("Access-Control-Allow-Origin", origin)
            .with_additional_header("Access-Control-Allow-Headers", "content-type")
            .with_additional_header("Access-Control-Request-Method", "GET, POST, DELETE"),
        None => response,
    }
}

fn main() {
    let port = match extract_var("PORT") {
        Ok(port) => port,
        Err(_) => panic!("PORT must be set"),
    };

    let database_url = match extract_var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => panic!("DATABASE_URL must be set"),
    };

    start_server(format!("0.0.0.0:{}", port), move |request| {
        with_cors(match init_db_connection(&database_url) {
            Ok(db_connection) => handle_request(request, &db_connection),
            Err(err) => {
                let error_response: ErrorResponse = err.into();
                error_response.into()
            }
        })
    });
}
