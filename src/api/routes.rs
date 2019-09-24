use rouille::{find_route, Request, Response};

use super::models::ErrorResponse;
use super::teams::{interface::TeamsDb, routes::handle_request as teams_handling};

pub fn extract_response(result: Result<Response, ErrorResponse>) -> Response {
    match result {
        Ok(response) => response,
        Err(e) => e.into(),
    }
}

pub fn handle_request<T: TeamsDb>(request: &Request, db: &T) -> Response {
    find_route!(extract_response(teams_handling(request, db)))
}
