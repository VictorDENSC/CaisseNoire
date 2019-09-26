use super::models::ErrorResponse;
use crate::teams::{interface::TeamsDb, routes::handle_request as teams_handling};
use rouille::{find_route, Request, Response};
use serde::Serialize;

fn extract_response<T>(result: Result<T, ErrorResponse>) -> Response
where
    T: Serialize,
{
    match result {
        Ok(response) => Response::json(&response),
        Err(e) => e.into(),
    }
}

pub fn handle_request<T: TeamsDb>(request: &Request, db: T) -> Response {
    find_route!(extract_response(teams_handling(request, db)))
}
