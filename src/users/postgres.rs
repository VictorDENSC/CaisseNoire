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

    fn get_user_by_id(&self, team_id: Uuid, user_id: Uuid) -> Result<User, DbError> {
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
    use crate::database::postgres::test_utils::{
        create_default_team, create_default_user, DbConnectionBuilder,
    };

    #[test]
    fn test_get_users() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let new_user = create_default_user(&conn, "login");

            create_default_user(&conn, "login_2");

            let users = conn.get_users(new_user.team_id).unwrap();

            assert_eq!(vec![new_user], users);

            Ok(())
        })
    }

    #[test]
    fn test_get_user() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let new_user = create_default_user(&conn, "login");

            let user = conn.get_user_by_id(new_user.team_id, new_user.id).unwrap();

            assert_eq!(new_user, user);

            Ok(())
        })
    }

    #[test]
    fn test_get_unexisting_user() {
        let conn = DbConnectionBuilder::new();

        let error = conn
            .get_user_by_id(Uuid::new_v4(), Uuid::new_v4())
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }

    #[test]
    fn test_create_user() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team = create_default_team(&conn);

            let new_user = User {
                id: Uuid::new_v4(),
                team_id: team.id,
                firstname: String::from("firstname"),
                lastname: String::from("lastname"),
                nickname: None,
                login: String::from("login"),
                password: String::from("password"),
                email: None,
            };

            let user = conn.create_user(&new_user).unwrap();

            assert_eq!(new_user, user);

            Ok(())
        })
    }

    #[test]
    fn test_create_uncorrect_user() {
        let conn = DbConnectionBuilder::new();

        let mut user = User {
            id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            firstname: String::from("firstname"),
            lastname: String::from("lastname"),
            nickname: None,
            login: String::from("login"),
            password: String::from("password"),
            email: None,
        };

        conn.deref().test_transaction::<_, Error, _>(|| {
            let error = conn.create_user(&user).unwrap_err();

            assert_eq!(
                error,
                DbError::ForeignKeyViolation(String::from(
                    "The key team_id doesn\'t refer to anything"
                ))
            );

            Ok(())
        });

        conn.deref().test_transaction::<_, Error, _>(|| {
            let default_user = create_default_user(&conn, "login");

            user.team_id = default_user.team_id;

            let error = conn.create_user(&user).unwrap_err();

            assert_eq!(
                error,
                DbError::UniqueViolation(String::from(
                    "The field login is already used by another user"
                ))
            );

            Ok(())
        })
    }

    #[test]
    fn test_update_user() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let default_user = create_default_user(&conn, "login");

            let user = conn
                .update_user(
                    default_user.team_id,
                    default_user.id,
                    &UpdateUser {
                        firstname: String::from("name"),
                        lastname: default_user.lastname,
                        nickname: default_user.nickname,
                        login: default_user.login,
                        password: default_user.password,
                        email: default_user.email,
                    },
                )
                .unwrap();

            assert_eq!(user.firstname, String::from("name"));
            assert_eq!(user.id, default_user.id);

            Ok(())
        })
    }

    #[test]
    fn test_update_unexisting_user() {
        let conn = DbConnectionBuilder::new();

        let error = conn
            .update_user(
                Uuid::new_v4(),
                Uuid::new_v4(),
                &UpdateUser {
                    firstname: String::from("firstname"),
                    lastname: String::from("lastname"),
                    nickname: None,
                    login: String::from("login"),
                    password: String::from("password"),
                    email: None,
                },
            )
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }
}
