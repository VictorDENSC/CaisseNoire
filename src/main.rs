use caisse_noire::api::routes::handle_request;
use caisse_noire::database::postgres::Database;
use std::env::{var, VarError};

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

    rouille::start_server(format!("0.0.0.0:{}", port), move |request| {
        let pool = &Database::connect(&database_url);

        handle_request(request, pool)
    });
}
