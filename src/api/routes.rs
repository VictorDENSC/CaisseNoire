use super::models::ErrorResponse;
use crate::sanctions::{
    interface::SanctionsDb, routes::handle_request as sanctions_request_handling,
};
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
    T: TeamsDb + UsersDb + SanctionsDb,
{
    match request.method() {
        "OPTIONS" => Response::empty_204(),
        _ => find_route!(
            extract_response(teams_request_handling(request, db)),
            extract_response(users_request_handling(request, db)),
            extract_response(sanctions_request_handling(request, db))
        ),
    }
}
