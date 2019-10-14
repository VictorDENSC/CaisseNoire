use rouille::{input::json::json_input, router, Request};
use uuid::Uuid;

use super::{
    interface::TeamsDb,
    models::{Team, UpdateTeam, UpdateTeamRequest},
};
use crate::api::models::ErrorResponse;

pub fn handle_request<T>(request: &Request, db: &T) -> Result<Team, ErrorResponse>
where
    T: TeamsDb,
{
    router!(request,
        (POST) (/teams) => {
            let input: Team = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.create_team(&input)?;

            Ok(result)
        },
        (GET) (/teams/{id:Uuid}) => {
            let result: Team = db.get_team(id)?;

            Ok(result)
        },
        (POST) (/teams/{id:Uuid}) => {
            let input: UpdateTeam = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.update_team(id, &input)?;

            Ok(result)
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
    fn test_get_team() {
        let id = Uuid::new_v4();
        let response = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &DbMock::default(),
        )
        .unwrap();

        assert_eq!(response.id, id);
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
        let team = json!({
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

        let response = handle_request(
            &RequestBuilder::post(String::from("/teams"), &team),
            &DbMock::default(),
        )
        .unwrap();

        assert_eq!(response.name, team["name"]);
        assert_eq!(response.rules[0].name, team["rules"][0]["name"]);
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

        let response = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &DbMock::default(),
        )
        .unwrap();

        assert_eq!(response.id, id);
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
