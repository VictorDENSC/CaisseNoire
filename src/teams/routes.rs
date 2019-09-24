use rouille::{input::json::json_input, router, Request, Response};
use uuid::Uuid;

use super::api::models::{ErrorKind, ErrorResponse};
use super::{
    interface::TeamsDb,
    models::{Team, UpdateTeam, UpdateTeamRequest},
};

pub fn handle_request<T: TeamsDb>(request: &Request, db: &T) -> Result<Response, ErrorResponse> {
    router!(request,
        (POST) (/teams) => {
            let input: Team = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.create_team(input)?;

            Ok(Response::json(&result))
        },
        (GET) (/teams/{id:Uuid}) => {
            let result: Team = db.get_team_by_id(id)?;

            Ok(Response::json(&result))
        },
        (POST) (/teams/{id:Uuid}) => {
            let input: UpdateTeam = json_input::<UpdateTeamRequest>(request)?.into();

            let result: Team = db.update_team(id, input)?;

            Ok(Response::json(&result))
        },
        _ => {
            Err(ErrorResponse {
                kind: ErrorKind::NotFound,
                description: String::from("Not found")
            })
        }
    )
}
