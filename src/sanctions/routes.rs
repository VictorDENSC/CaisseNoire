use rouille::{input::json::json_input, router, Request};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    interface::SanctionsDb,
    models::{CreateSanction, Sanction, UpdateSanctionRequest},
    utils::{formatter::map_by_users, parameters::ParametersHandler},
};
use crate::api::models::ErrorResponse;

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ResultWrapper {
    Sanctions(Vec<Sanction>),
    MappedSanctions(HashMap<Uuid, Vec<Sanction>>),
    Sanction(Sanction),
}

pub fn handle_request<T>(request: &Request, db: &T) -> Result<ResultWrapper, ErrorResponse>
where
    T: SanctionsDb,
{
    router!(request,
    (GET) (/teams/{team_id: Uuid}/sanctions) => {
        let parameters_handler = ParametersHandler::from_request(request)?;

        let result = db.get_sanctions(team_id, parameters_handler.date_interval())?;

        match parameters_handler.must_be_formatted() {
            true => Ok(ResultWrapper::MappedSanctions(map_by_users(result))),
            false => Ok(ResultWrapper::Sanctions(result))
        }
    },
    (POST) (/teams/{team_id: Uuid}/sanctions) => {
        let input: CreateSanction = (json_input::<UpdateSanctionRequest>(request)?, team_id).into();

        let result = db.create_sanction(&input)?;

        Ok(ResultWrapper::Sanction(result))
    },
    (DELETE) (/teams/{team_id: Uuid}/sanctions/{sanction_id: Uuid}) => {
        let result = db.delete_sanction(team_id, sanction_id)?;

        Ok(ResultWrapper::Sanction(result))
    },
    _ => Err(ErrorResponse::not_found())
    )
}
