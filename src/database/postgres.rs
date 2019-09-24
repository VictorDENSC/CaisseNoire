use diesel::{pg::PgConnection, result::Error};
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

pub enum DbError {
    PoolBuilding,
    ConnectionAccess,
    NotFound,
    Unknown,
}

impl From<Error> for DbError {
    fn from(error: Error) -> DbError {
        match error {
            Error::NotFound => DbError::NotFound,
            _ => DbError::Unknown,
        }
    }
}

pub struct Database {
    pool: Option<Pool<ConnectionManager<PgConnection>>>,
}

impl Database {
    pub fn connect(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        match Pool::builder().build(manager) {
            Ok(pool) => Database { pool: Some(pool) },
            Err(_) => Database { pool: None },
        }
    }

    pub fn get_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, DbError> {
        match &self.pool {
            Some(pool) => match pool.get() {
                Ok(connection) => Ok(connection),
                Err(_) => Err(DbError::ConnectionAccess),
            },
            None => Err(DbError::PoolBuilding),
        }
    }
}
