use crate::database::postgres::{init_db_connection, DbConnection};
use dotenv::dotenv;
use std::env::var;

pub fn init_connection() -> DbConnection {
    dotenv().ok();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    init_db_connection(&database_url).expect("Something went wrong while getting the connection")
}
