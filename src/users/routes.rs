use rouille::{input::json::json_input, router, Request};
use serde::Serialize;
use uuid::Uuid;

use super::{
    interface::UsersDb,
    models::{UpdateUser, UpdateUserRequest, User},
};
use crate::api::models::ErrorResponse;

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ResultWrapper {
    Users(Vec<User>),
    User(User),
}

pub fn handle_request<T>(request: &Request, db: &T) -> Result<ResultWrapper, ErrorResponse>
where
    T: UsersDb,
{
    router!(request,
        (GET) (/teams/{team_id: Uuid}/users) => {
            let result = db.get_users(team_id)?;

            Ok(ResultWrapper::Users(result))
        },
        (POST) (/teams/{team_id: Uuid}/users) => {
            let input: User = (json_input::<UpdateUserRequest>(request)?, team_id).into();

            let result = db.create_user(&input)?;

            Ok(ResultWrapper::User(result))

        },
        (GET) (/teams/{team_id: Uuid}/users/{user_id: Uuid}) => {
            let result = db.get_user(team_id, user_id)?;

            Ok(ResultWrapper::User(result))
        },
        (POST) (/teams/{team_id: Uuid}/users/{user_id: Uuid}) => {
            let input: UpdateUser = json_input::<UpdateUserRequest>(request)?.into();

            let result = db.update_user(team_id, user_id, &input)?;

            Ok(ResultWrapper::User(result))
        },
        _ => {
            Err(ErrorResponse::not_found())
        }
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::api::models::{test_utils::RequestBuilder, ErrorKind};
    use crate::database::postgres::DbError;

    pub fn default_user(team_id: Uuid, user_id: Uuid) -> User {
        User {
            id: user_id,
            team_id,
            firstname: String::from("firstname"),
            lastname: String::from("lastname"),
            nickname: None,
            login: String::from("login"),
            password: String::from("password"),
            email: None,
            is_admin: false,
        }
    }

    pub enum UsersDbMock {
        Success,
        NotFound,
        UnexistingTeam,
        DuplicatedField,
    }

    impl UsersDb for UsersDbMock {
        fn get_users(&self, team_id: Uuid) -> Result<Vec<User>, DbError> {
            match self {
                UsersDbMock::Success => Ok(vec![default_user(team_id, Uuid::new_v4())]),
                _ => unimplemented!(),
            }
        }

        fn get_user(&self, team_id: Uuid, user_id: Uuid) -> Result<User, DbError> {
            match self {
                UsersDbMock::Success => Ok(default_user(team_id, user_id)),
                UsersDbMock::NotFound => Err(DbError::NotFound),
                _ => unimplemented!(),
            }
        }

        fn create_user(&self, user: &User) -> Result<User, DbError> {
            match self {
                UsersDbMock::Success => Ok(user.clone()),
                UsersDbMock::UnexistingTeam => {
                    Err(DbError::ForeignKeyViolation(String::from("Error")))
                }
                UsersDbMock::DuplicatedField => {
                    Err(DbError::UniqueViolation(String::from("Error")))
                }
                _ => unimplemented!(),
            }
        }

        fn update_user(
            &self,
            team_id: Uuid,
            user_id: Uuid,
            user: &UpdateUser,
        ) -> Result<User, DbError> {
            match self {
                UsersDbMock::Success => Ok(User {
                    id: user_id,
                    team_id,
                    firstname: user.firstname.clone(),
                    lastname: user.lastname.clone(),
                    nickname: user.nickname.clone(),
                    login: user.login.clone(),
                    password: user.password.clone(),
                    email: user.email.clone(),
                    is_admin: user.is_admin.clone(),
                }),
                UsersDbMock::NotFound => Err(DbError::NotFound),
                _ => unimplemented!(),
            }
        }
    }

    #[test]
    fn test_get_users() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/users", team_id)),
            &UsersDbMock::Success,
        )
        .unwrap());

        assert_eq!(response[0]["team_id"], json!(team_id));
    }

    #[test]
    fn test_get_user() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/users/{}", team_id, user_id)),
            &UsersDbMock::Success,
        )
        .unwrap());

        assert_eq!(response["id"], json!(user_id));
        assert_eq!(response["team_id"], json!(team_id));
    }

    #[test]
    fn test_get_user_fails() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}/users/{}", team_id, user_id)),
            &UsersDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);
    }

    #[test]
    fn test_create_user() {
        let team_id = Uuid::new_v4();
        let user = json!({
            "firstname": "John",
            "lastname": "Snow",
            "nickname": "King of the north",
            "login": "login",
            "password": "password",
            "is_admin": true
        });

        let response = json!(handle_request(
            &RequestBuilder::post(format!("/teams/{}/users", team_id), &user),
            &UsersDbMock::Success,
        )
        .unwrap());

        assert_eq!(response["team_id"], json!(team_id));
        assert_eq!(response["firstname"], "John");
        assert_eq!(response["email"], serde_json::Value::Null);
    }

    #[test]
    fn test_create_user_fails() {
        let team_id = Uuid::new_v4();
        let user = json!({
            "firstname": "John",
            "lastname": "Snow",
            "nickname": "King of the north",
            "login": "login",
            "password": "password",
            "is_admin": true
        });

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/users", team_id), &user),
            &UsersDbMock::UnexistingTeam,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotAllowed);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/users", team_id), &user),
            &UsersDbMock::DuplicatedField,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotAllowed);

        let invalid_json = json!({});

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/users", team_id), &invalid_json),
            &UsersDbMock::Success,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Json);
    }

    #[test]
    fn test_update_user() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let user = json!({
            "firstname": "John",
            "lastname": "Snow",
            "nickname": "King of the north",
            "login": "login",
            "password": "password",
            "is_admin": true
        });

        let response = json!(handle_request(
            &RequestBuilder::post(format!("/teams/{}/users/{}", team_id, user_id), &user),
            &UsersDbMock::Success,
        )
        .unwrap());

        assert_eq!(response["id"], json!(user_id));
        assert_eq!(response["team_id"], json!(team_id));
    }

    #[test]
    fn test_update_user_fails() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let user = json!({
            "firstname": "John",
            "lastname": "Snow",
            "nickname": "King of the north",
            "login": "login",
            "password": "password",
            "is_admin": true
        });

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/users/{}", team_id, user_id), &user),
            &UsersDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);
    }
}
