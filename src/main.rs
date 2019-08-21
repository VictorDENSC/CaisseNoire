use rouille::Request;
use rouille::Response;
use uuid::Uuid;
use std::env;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;

use caisse_noire::database::{models::ApiKeys, schema::api_keys};

pub fn establish_connection() -> PgConnection {
    // Utile ?
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}



fn main() {
    let port: String = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let connection = establish_connection();
    let apikey: ApiKeys = diesel::insert_into(api_keys::table)
        .values(&ApiKeys {
            id: Uuid::new_v4(),
            apikey: vec![2, 3, 5],
        })
        .get_result(&connection)
        .expect("Error saving new post");

    rouille::start_server(format!("0.0.0.0:{}", port), move |request| {
        Response::text("hello world")
    });
}
