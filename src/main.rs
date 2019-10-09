use rouille::{start_server, Response};
use std::env::{var, VarError};

use caisse_noire::api::{models::ErrorResponse, routes::handle_request};
use caisse_noire::database::postgres::init_db_connection;

fn extract_var(var_name: &str) -> Result<String, VarError> {
    use dotenv::dotenv;

    dotenv().ok();
    var(var_name)
}

fn with_cors_enabled(response: Response, enabled_cors: bool) -> Response {
    match enabled_cors {
        true => response
            .with_additional_header("Access-Control-Allow-Origin", "*")
            .with_additional_header("Access-Control-Allow-Headers", "content-type"),
        false => response,
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

    let enabled_cors = match extract_var("ENABLED_CORS") {
        Ok(enabled_cors) => enabled_cors.parse().unwrap_or(false),
        Err(_) => false,
    };

    start_server(format!("0.0.0.0:{}", port), move |request| {
        with_cors_enabled(
            match init_db_connection(&database_url) {
                Ok(db_connection) => handle_request(request, &db_connection),
                Err(err) => {
                    let error_response: ErrorResponse = err.into();
                    error_response.into()
                }
            },
            enabled_cors,
        )
    });
}
