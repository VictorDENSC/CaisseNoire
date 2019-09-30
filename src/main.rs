use rouille::start_server;
use std::env::{var, VarError};

use caisse_noire::api::{models::ErrorResponse, routes::handle_request};
use caisse_noire::database::postgres::init_db_connection;

pub fn extract_var(var_name: &str) -> Result<String, VarError> {
    use dotenv::dotenv;

    dotenv().ok();
    var(var_name)
}

fn main() {
    let port = match extract_var("PORT") {
        Ok(port) => port,
        Err(_) => panic!("PORT must be set"),
    };

    let database_url = match extract_var("DATABASE_URL") {
        Ok(port) => port,
        Err(_) => panic!("DATABASE_URL must be set"),
    };

    start_server(
        format!("0.0.0.0:{}", port),
        move |request| match init_db_connection(&database_url) {
            Ok(db_connection) => handle_request(request, &db_connection),
            Err(err) => {
                let error_response: ErrorResponse = err.into();
                error_response.into()
            }
        },
    );
}
