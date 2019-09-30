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
    use crate::database::postgres::DbError;

    pub enum TeamsDbMock {
        Success,
        NotFound,
        Unknown,
    }

    impl TeamsDb for TeamsDbMock {
        fn get_team(&self, id: Uuid) -> Result<Team, DbError> {
            match self {
                TeamsDbMock::Success => Ok(Team {
                    id: id,
                    name: String::from("Test_team"),
                    rules: vec![],
                }),
                TeamsDbMock::NotFound => Err(DbError::NotFound),
                TeamsDbMock::Unknown => Err(DbError::Unknown),
            }
        }

        fn create_team(&self, team: &Team) -> Result<Team, DbError> {
            match self {
                TeamsDbMock::Success => Ok(team.clone()),
                TeamsDbMock::Unknown => Err(DbError::Unknown),
                _ => unimplemented!(),
            }
        }

        fn update_team(&self, id: Uuid, team: &UpdateTeam) -> Result<Team, DbError> {
            match self {
                TeamsDbMock::Success => Ok(Team {
                    id,
                    name: team.name.clone(),
                    rules: team.rules.clone(),
                }),
                TeamsDbMock::NotFound => Err(DbError::NotFound),
                TeamsDbMock::Unknown => Err(DbError::Unknown),
            }
        }
    }

    #[test]
    fn test_get_team() {
        let id = Uuid::new_v4();
        let response = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &TeamsDbMock::Success,
        )
        .unwrap();

        assert_eq!(response.id, id);
    }

    #[test]
    fn test_get_team_fails() {
        let id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &TeamsDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            &TeamsDbMock::Unknown,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);
    }

    #[test]
    fn test_create_team() {
        let team = json!({
            "name": "Test_team",
            "rules": [{
                "name": "Rule_Test",
                "category": "TRAINING_DAY",
                "description": "This is a description !",
                "kind": {
                    "BASED_ON_TIME": {
                        "price_per_time_unit": 0.2,
                        "time_unit": "MINUTES"
                    }
                }
            }]
        });

        let response = handle_request(
            &RequestBuilder::post(String::from("/teams"), &team),
            &TeamsDbMock::Success,
        )
        .unwrap();

        assert_eq!(response.name, team["name"]);
        assert_eq!(response.rules[0].name, team["rules"][0]["name"]);
    }

    #[test]
    fn test_create_team_fails() {
        let team = json!({
            "name": "Test_team",
            "rules": []
        });

        let error = handle_request(
            &RequestBuilder::post(String::from("/teams"), &team),
            &TeamsDbMock::Unknown,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);

        let invalid_json = json!({});

        let error = handle_request(
            &RequestBuilder::post(String::from("/teams"), &invalid_json),
            &TeamsDbMock::Success,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Json);
    }

    #[test]
    fn test_update_team() {
        let id = Uuid::new_v4();
        let team = json!({
            "name": "Test_team",
            "rules": []
        });

        let response = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &TeamsDbMock::Success,
        )
        .unwrap();

        assert_eq!(response.id, id);
    }

    #[test]
    fn test_update_team_fails() {
        let id = Uuid::new_v4();
        let team = json!({
            "name": "Test_team",
            "rules": []
        });

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &TeamsDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team),
            &TeamsDbMock::Unknown,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);
    }
}
