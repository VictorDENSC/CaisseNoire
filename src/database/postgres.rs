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

#[cfg(test)]
pub mod test_utils {
    use diesel::prelude::*;
    use uuid::Uuid;

    use super::super::schema::*;
    use super::*;
    use crate::sanctions::models::{CreateSanction, Sanction, SanctionData, SanctionInfo};
    use crate::teams::models::Team;
    use crate::users::models::User;

    pub struct DbConnectionBuilder;

    impl DbConnectionBuilder {
        pub fn new() -> DbConnection {
            init_db_connection("postgres://postgres:password@localhost/caisse_noire")
                .expect("Something went wrong while getting the connection")
        }
    }

    pub fn create_default_team(conn: &DbConnection, name: Option<String>) -> Team {
        let default_team = Team {
            id: Uuid::new_v4(),
            name: name.unwrap_or(String::from("Test_team")),
            admin_password: String::from("password"),
            rules: vec![],
        };

        diesel::insert_into(teams::table)
            .values(&default_team)
            .get_result(conn.deref())
            .expect("Failed to create default team")
    }

    pub fn create_default_user(
        conn: &DbConnection,
        team_id: Option<Uuid>,
        email: Option<String>,
    ) -> User {
        let default_team_id = team_id.unwrap_or_else(|| create_default_team(conn, None).id);

        let default_user = User {
            id: Uuid::new_v4(),
            team_id: default_team_id,
            firstname: String::from("firstname"),
            lastname: String::from("lastname"),
            nickname: None,
            email,
        };

        diesel::insert_into(users::table)
            .values(&default_user)
            .get_result(conn.deref())
            .expect("Failed to create default user")
    }

    pub fn create_default_sanction(conn: &DbConnection) -> Sanction {
        let default_user = create_default_user(conn, "login");

        let default_sanction = CreateSanction {
            id: Uuid::new_v4(),
            user_id: default_user.id,
            team_id: default_user.team_id,
            sanction_info: SanctionInfo {
                id: Uuid::new_v4(),
                sanction_data: SanctionData::Basic,
            },
        };

        diesel::insert_into(sanctions::table)
            .values(&default_sanction)
            .get_result(conn.deref())
            .expect("Failed to create default sanction")
    }
}
