use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum DbError {
    ServiceUnavailable,
    NotFound,
    ForeignKeyViolation(String),
    UniqueViolation(String),
    Unknown,
}

impl From<r2d2::Error> for DbError {
    fn from(_: r2d2::Error) -> DbError {
        DbError::ServiceUnavailable
    }
}

impl From<diesel::result::Error> for DbError {
    fn from(error: diesel::result::Error) -> DbError {
        println!("{:?}", error);
        match error {
            diesel::result::Error::NotFound => DbError::NotFound,
            diesel::result::Error::DatabaseError(kind, information) => match kind {
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    DbError::ForeignKeyViolation(match information.constraint_name() {
                        Some(constraint_name) => {
                            format!("The key {} doesn't refer to anything", constraint_name,)
                        }
                        None => String::from("An error occured due to a foreign key violation"),
                    })
                }
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    DbError::UniqueViolation(match information.constraint_name() {
                        Some(constraint_name) => format!(
                            "The field {} is already used by another user",
                            constraint_name,
                        ),
                        None => String::from("An error occured due to a unique violation"),
                    })
                }
                _ => DbError::Unknown,
            },
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
