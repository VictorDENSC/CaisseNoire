use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum DbError {
    ServiceUnavailable,
    NotFound,
    Unknown,
}

impl From<r2d2::Error> for DbError {
    fn from(_: r2d2::Error) -> DbError {
        DbError::ServiceUnavailable
    }
}

impl From<diesel::result::Error> for DbError {
    fn from(error: diesel::result::Error) -> DbError {
        match error {
            diesel::result::Error::NotFound => DbError::NotFound,
            _ => DbError::Unknown,
        }
    }
}

pub fn init_db_connection(database_url: &str) -> Result<DbConnection, DbError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager)?;
    let connection = pool.get()?;
    Ok(DbConnection(connection))
}

pub struct DbConnection(r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for DbConnection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub mod test_utils {
    use super::*;

    pub struct DbConnectionBuilder;

    impl DbConnectionBuilder {
        pub fn new() -> DbConnection {
            init_db_connection("postgres://postgres:password@localhost/caisse_noire")
                .expect("Something went wrong while getting the connection")
        }
    }
}
