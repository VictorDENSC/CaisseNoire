use diesel::prelude::*;
use std::ops::Deref;
use uuid::Uuid;

use super::{
    interface::UsersDb,
    models::{UpdateUser, User},
};
use crate::database::{
    postgres::{DbConnection, DbError},
    schema::users,
};

impl UsersDb for DbConnection {
    fn get_users(&self, team_id: Uuid) -> Result<Vec<User>, DbError> {
        let users: Vec<User> = users::table
            .filter(users::team_id.eq(team_id))
            .get_results(self.deref())?;

        Ok(users)
    }

    fn get_user(&self, team_id: Uuid, user_id: Uuid) -> Result<User, DbError> {
        let user: User = users::table
            .filter(users::team_id.eq(team_id).and(users::id.eq(user_id)))
            .get_result(self.deref())?;

        Ok(user)
    }

    fn create_user(&self, user: &User) -> Result<User, DbError> {
        let user: User = diesel::insert_into(users::table)
            .values(user)
            .get_result(self.deref())?;

        Ok(user)
    }

    fn update_user(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        user: &UpdateUser,
    ) -> Result<User, DbError> {
        let user: User = diesel::update(
            users::table.filter(users::team_id.eq(team_id).and(users::id.eq(user_id))),
        )
        .set(user)
        .get_result(self.deref())?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use diesel::result::Error;

    use super::*;
    use crate::teams::{interface::TeamsDb, models::Team};
    use crate::test_utils::postgres::init_connection;

    #[test]
    fn test_get_users() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team_id = conn.create_team(&Team::default()).unwrap().id;
            let user = conn
                .create_user(&User {
                    team_id,
                    ..Default::default()
                })
                .unwrap();

            let team_id_2 = conn
                .create_team(&Team {
                    id: Uuid::new_v4(),
                    name: String::from("CHBC"),
                    ..Default::default()
                })
                .unwrap()
                .id;
            let user_2 = conn
                .create_user(&User {
                    id: Uuid::new_v4(),
                    team_id: team_id_2,
                    ..Default::default()
                })
                .unwrap();

            let users = conn.get_users(team_id).unwrap();
            let users_2 = conn.get_users(team_id_2).unwrap();

            assert_eq!(vec![user], users);
            assert_eq!(vec![user_2], users_2);

            Ok(())
        })
    }

    #[test]
    fn test_get_user() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team_id = conn.create_team(&Team::default()).unwrap().id;
            let user_id = conn
                .create_user(&User {
                    team_id,
                    ..Default::default()
                })
                .unwrap()
                .id;

            let user = conn.get_user(team_id, user_id).unwrap();

            assert_eq!(user_id, user.id);
            assert_eq!(team_id, user.team_id);

            Ok(())
        })
    }

    #[test]
    fn test_get_unexisting_user() {
        let conn = init_connection();

        let error = conn.get_user(Uuid::new_v4(), Uuid::new_v4()).unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }

    #[test]
    fn test_create_user() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let id = Uuid::new_v4();
            let team_id = conn.create_team(&Team::default()).unwrap().id;

            let user = conn
                .create_user(&User {
                    id,
                    team_id,
                    ..Default::default()
                })
                .unwrap();

            assert_eq!(id, user.id);
            assert_eq!(team_id, user.team_id);

            Ok(())
        })
    }

    #[test]
    fn test_create_uncorrect_user() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let error = conn.create_user(&User::default()).unwrap_err();

            assert_eq!(
                error,
                DbError::ForeignKeyViolation(String::from(
                    "The key team_id doesn\'t refer to anything"
                ))
            );

            Ok(())
        });

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team_id = conn.create_team(&Team::default()).unwrap().id;

            conn.create_user(&User {
                team_id,
                email: Some(String::from("email@gmail.com")),
                ..Default::default()
            })
            .unwrap();

            let error = conn
                .create_user(&User {
                    id: Uuid::new_v4(),
                    team_id,
                    email: Some(String::from("email@gmail.com")),
                    ..Default::default()
                })
                .unwrap_err();

            assert_eq!(
                error,
                DbError::UniqueViolation(String::from(
                    "The field email is already used by another user"
                ))
            );

            Ok(())
        })
    }

    #[test]
    fn test_update_user() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team_id = conn.create_team(&Team::default()).unwrap().id;
            let user_id = conn
                .create_user(&User {
                    team_id,
                    ..Default::default()
                })
                .unwrap()
                .id;

            let user = conn
                .update_user(
                    team_id,
                    user_id,
                    &UpdateUser {
                        firstname: String::from("name"),
                        ..Default::default()
                    },
                )
                .unwrap();

            assert_eq!(team_id, user.team_id);
            assert_eq!(user_id, user.id);
            assert_eq!(user.firstname, String::from("name"));

            Ok(())
        })
    }

    #[test]
    fn test_update_unexisting_user() {
        let conn = init_connection();

        let error = conn
            .update_user(Uuid::new_v4(), Uuid::new_v4(), &UpdateUser::default())
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }
}
