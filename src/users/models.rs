use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::schema::users;

#[derive(Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub id: Option<Uuid>,
    pub firstname: String,
    pub lastname: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
}

impl From<(UpdateUserRequest, Uuid)> for User {
    fn from((update_request, team_id): (UpdateUserRequest, Uuid)) -> User {
        User {
            id: update_request.id.unwrap_or_else(Uuid::new_v4),
            team_id,
            firstname: update_request.firstname,
            lastname: update_request.lastname,
            nickname: update_request.nickname,
            email: update_request.email,
        }
    }
}

impl From<UpdateUserRequest> for UpdateUser {
    fn from(update_request: UpdateUserRequest) -> UpdateUser {
        UpdateUser {
            firstname: update_request.firstname,
            lastname: update_request.lastname,
            nickname: update_request.nickname,
            email: update_request.email,
        }
    }
}

#[derive(Debug, Clone, Queryable, Insertable, PartialEq, Serialize, Deserialize, Default)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub team_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
}

#[derive(AsChangeset, Default)]
#[table_name = "users"]
pub struct UpdateUser {
    pub firstname: String,
    pub lastname: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
}
