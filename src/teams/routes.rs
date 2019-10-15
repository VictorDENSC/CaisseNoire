use rouille::{input::json::json_input, router, Request};
use serde::Serialize;
use uuid::Uuid;

use super::{
    interface::TeamsDb,
    models::{LoginRequest, LoginResponse, Team, UpdateTeam, UpdateTeamRequest},
};
use crate::api::models::ErrorResponse;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ResponseWrapper {
    Login(LoginResponse),
    Team(Team),
}

pub fn handle_request<T>(request: &Request, db: &T) -> Result<ResponseWrapper, ErrorResponse>
where
    T: TeamsDb,
{
    router!(request,
        (POST) (/login) => {
            let input: LoginRequest = json_input(request)?;

            let team_id = db.login(&input.name, &input.admin_password)?;

            let result: LoginResponse = (input, team_id).into();

            Ok(ResponseWrapper::Login(result))
        },
        (POST) (/teams) => {
            let input: Team = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.create_team(&input)?;

            Ok(ResponseWrapper::Team(result))
        },
        (GET) (/teams/{id:Uuid}) => {
            let result: Team = db.get_team(id)?;

            Ok(ResponseWrapper::Team(result))
        },
        (POST) (/teams/{id:Uuid}) => {
            let input: UpdateTeam = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.update_team(id, &input)?;

            Ok(ResponseWrapper::Team(result))
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
    use crate::test_utils::routes::{DbMock, TeamsDbMock};

    #[test]
    fn test_login() {
        let login_request = json!({
            "name": "CHBC",
        });

        let response = json!(handle_request(
            &RequestBuilder::post(String::from("/login"), &login_request),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response["admin_password"], serde_json::Value::Null);
    }

    #[test]
    fn test_login_with_password() {
        let login_request = json!({
            "name": "CHBC",
            "admin_password": "password"
        });

        let response = json!(handle_request(
            &RequestBuilder::post(String::from("/login"), &login_request),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response["admin_password"], login_request["admin_password"]);
    }

    #[test]
    fn test_login_fails() {
        let mut login_request = json!({
            "name": "CHBC",
        });

        let error = handle_request(
            &RequestBuilder::post(String::from("/login"), &login_request),
            &DbMock {
                teams_db: TeamsDbMock::NotFound,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        login_request["admin_password"] = json!("password");

        let error = handle_request(
            &RequestBuilder::post(String::from("/login"), &login_request),
            &DbMock {
                teams_db: TeamsDbMock::NotFound,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);
    }

    #[test]
    fn test_get_team() {
        let id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response["id"], json!(id));
    }

    #[test]
    fn test_get_team_fails() {
        let id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &DbMock {
                teams_db: TeamsDbMock::NotFound,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &DbMock {
                teams_db: TeamsDbMock::Unknown,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);
    }

    #[test]
    fn test_create_team() {
        let team_id = Uuid::new_v4();

        let team = json!({
            "id": team_id,
            "name": "Test_team",
            "admin_password": "password",
            "rules": [{
                "name": "Rule_Test",
                "category": "TRAINING_DAY",
                "description": "This is a description !",
                "kind": {
                    "type": "TIME_MULTIPLICATION",
                    "price_per_time_unit": 0.2,
                    "time_unit": "MINUTE"
                }
            }]
        });

        let response = json!(handle_request(
            &RequestBuilder::post(String::from("/teams"), &team),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response["id"], json!(team_id));
    }

    #[test]
    fn test_create_team_fails() {
        let team = json!({
            "name": "Test_team",
            "admin_password": "password",
            "rules": []
        });

        let error = handle_request(
            &RequestBuilder::post(String::from("/teams"), &team),
            &DbMock {
                teams_db: TeamsDbMock::Unknown,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);

        let invalid_json = json!({});

        let error = handle_request(
            &RequestBuilder::post(String::from("/teams"), &invalid_json),
            &DbMock::default(),
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Json);
    }

    #[test]
    fn test_update_team() {
        let id = Uuid::new_v4();

        let team = json!({
            "name": "Test_team",
            "admin_password": "password",
            "rules": []
        });

        let response = json!(handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response["id"], json!(id));
    }

    #[test]
    fn test_update_team_fails() {
        let id = Uuid::new_v4();
        let team = json!({
            "name": "Test_team",
            "admin_password": "password",
            "rules": []
        });

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &DbMock {
                teams_db: TeamsDbMock::NotFound,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &DbMock {
                teams_db: TeamsDbMock::Unknown,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);
    }
}
