use super::models::ErrorResponse;
use crate::teams::{interface::TeamsDb, routes::handle_request as teams_request_handling};
use crate::users::{interface::UsersDb, routes::handle_request as users_request_handling};
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

pub fn handle_request<T>(request: &Request, db: &T) -> Response
where
    T: TeamsDb + UsersDb,
{
    find_route!(
        extract_response(teams_request_handling(request, db)),
        extract_response(users_request_handling(request, db))
    )
}
