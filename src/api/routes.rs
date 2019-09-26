use rouille::{find_route, Request, Response};

use super::models::{ErrorResponse, SuccessResponse};
use crate::teams::{interface::TeamsDb, routes::handle_request as teams_handling};

fn extract_response(result: Result<SuccessResponse, ErrorResponse>) -> Response {
    match result {
        Ok(response) => response.into(),
        Err(e) => e.into(),
    }
}

pub fn handle_request<T: TeamsDb>(request: &Request, db: T) -> Response {
    find_route!(extract_response(teams_handling(request, db)))
}
