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
    use crate::test_utils::routes::{DbMock, UsersDbMock};

    #[test]
    fn test_get_users() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/users", team_id)),
            &DbMock::default(),
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
            &DbMock::default(),
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
            &DbMock {
                users_db: UsersDbMock::NotFound,
                ..Default::default()
            },
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
            &DbMock::default(),
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
            &DbMock {
                users_db: UsersDbMock::UnexistingTeam,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadReference);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/users", team_id), &user),
            &DbMock {
                users_db: UsersDbMock::DuplicatedField,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::DuplicatedField);

        let invalid_json = json!({});

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/users", team_id), &invalid_json),
            &DbMock::default(),
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
            &DbMock::default(),
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
            &DbMock {
                users_db: UsersDbMock::NotFound,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);
    }
}
