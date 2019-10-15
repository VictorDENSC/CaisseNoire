use crate::database::postgres::*;

//TODO
pub struct DbConnectionBuilder;

impl DbConnectionBuilder {
    pub fn new() -> DbConnection {
        init_db_connection("postgres://postgres:password@localhost/caisse_noire")
            .expect("Something went wrong while getting the connection")
    }
}
