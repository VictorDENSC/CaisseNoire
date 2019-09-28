use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum DbError {
    ServiceUnavailable,
    NotFound,
    ForeignKeyViolation,
    UniqueViolation,
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
            //TODO
            diesel::result::Error::DatabaseError(kind, _information) => match kind {
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => DbError::ForeignKeyViolation,
                diesel::result::DatabaseErrorKind::UniqueViolation => DbError::UniqueViolation,
                _ => DbError::Unknown
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

pub mod test_utils {
    use diesel::prelude::*;
    use uuid::Uuid;

    use super::super::schema::*;
    use super::*;
    use crate::teams::models::Team;
    use crate::users::models::User;

    pub struct DbConnectionBuilder;

    impl DbConnectionBuilder {
        pub fn new() -> DbConnection {
            init_db_connection("postgres://postgres:password@localhost/caisse_noire")
                .expect("Something went wrong while getting the connection")
        }
    }

    pub fn create_default_team(conn: &DbConnection) -> Team {
        let default_team = Team {
            id: Uuid::new_v4(),
            name: String::from("Test_team"),
            rules: vec![],
        };

        diesel::insert_into(teams::table)
            .values(&default_team)
            .get_result(conn.deref())
            .expect("Failed to create default team")
    }

    pub fn create_default_user(conn: &DbConnection, login: &str, password: &str) -> User {
        let default_team = create_default_team(conn);

        let default_user = User {
            id: Uuid::new_v4(),
            team_id: default_team.id,
            firstname: String::from("firstname"),
            lastname: String::from("lastname"),
            nickname: None,
            login: String::from(login),
            password: String::from(password),
            email: None,
        };

        diesel::insert_into(users::table)
            .values(&default_user)
            .get_result(conn.deref())
            .expect("Failed to create default user")
    }
}
