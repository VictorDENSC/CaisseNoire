use rouille::{input::json::json_input, router, Request};
use uuid::Uuid;

use crate::api::models::{ErrorKind, ErrorResponse};
use super::{
    interface::TeamsDb,
    models::{Team, UpdateTeam, UpdateTeamRequest},
};

pub fn handle_request<T: TeamsDb>(request: &Request, db: T) -> Result<Team, ErrorResponse> {
    router!(request,
        (POST) (/teams) => {
            let input: Team = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.create_team(&input)?;

            Ok(result)
        },
        (GET) (/teams/{id:Uuid}) => {
            let result: Team = db.get_team_by_id(id)?;

            Ok(result)
        },
        (POST) (/teams/{id:Uuid}) => {
            let input: UpdateTeam = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.update_team(id, &input)?;

            Ok(result)
        },
        _ => {
            Err(ErrorResponse {
                kind: ErrorKind::NotFound,
                description: String::from("Not found")
            })
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::test_utils::RequestBuilder;
    use crate::database::postgres::DbError;

    pub enum TeamsDbMock {
        Success,
        NotFound,
        Unknown,
    }

    impl TeamsDb for TeamsDbMock {
        fn get_team_by_id(&self, id: Uuid) -> Result<Team, DbError> {
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
            TeamsDbMock::Success,
        )
        .unwrap();

        assert_eq!(response.id, id);
    }

    #[test]
    fn test_get_team_fails() {
        let id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            TeamsDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}", id)),
            TeamsDbMock::Unknown,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);
    }

    #[test]
    fn test_create_team() {
        let team = UpdateTeamRequest {
            name: String::from("Test_team"),
            rules: vec![],
        };

        let response = handle_request(
            &RequestBuilder::post(String::from("/teams"), &team).unwrap(),
            TeamsDbMock::Success,
        )
        .unwrap();

        assert_eq!(response.name, team.name);
    }

    #[test]
    fn test_create_team_fails() {
        let team = UpdateTeamRequest {
            name: String::from("Test_team"),
            rules: vec![],
        };

        let error = handle_request(
            &RequestBuilder::post(String::from("/teams"), &team).unwrap(),
            TeamsDbMock::Unknown,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown)
    }

    #[test]
    fn test_update_team() {
        let id = Uuid::new_v4();
        let team = UpdateTeamRequest {
            name: String::from("Test_team"),
            rules: vec![],
        };

        let response = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team).unwrap(),
            TeamsDbMock::Success,
        )
        .unwrap();

        assert_eq!(response.id, id);
    }

    #[test]
    fn test_update_team_fails() {
        let id = Uuid::new_v4();
        let team = UpdateTeamRequest {
            name: String::from("Test_team"),
            rules: vec![],
        };

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team).unwrap(),
            TeamsDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}", id), &team).unwrap(),
            TeamsDbMock::Unknown,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Unknown);
    }
}
