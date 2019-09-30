use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::schema::users;

#[derive(Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub firstname: String,
    pub lastname: String,
    pub nickname: Option<String>,
    pub login: String,
    pub password: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

impl From<(UpdateUserRequest, Uuid)> for User {
    fn from((update_request, team_id): (UpdateUserRequest, Uuid)) -> User {
        User {
            id: Uuid::new_v4(),
            team_id,
            firstname: update_request.firstname,
            lastname: update_request.lastname,
            nickname: update_request.nickname,
            login: update_request.login,
            password: update_request.password,
            email: update_request.email,
            is_admin: update_request.is_admin,
        }
    }
}

impl From<UpdateUserRequest> for UpdateUser {
    fn from(update_request: UpdateUserRequest) -> UpdateUser {
        UpdateUser {
            firstname: update_request.firstname,
            lastname: update_request.lastname,
            nickname: update_request.nickname,
            login: update_request.login,
            password: update_request.password,
            email: update_request.email,
            is_admin: update_request.is_admin,
        }
    }
}

#[derive(Debug, Clone, Queryable, Insertable, PartialEq, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub team_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub nickname: Option<String>,
    pub login: String,
    pub password: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
    pub firstname: String,
    pub lastname: String,
    pub nickname: Option<String>,
    pub login: String,
    pub password: String,
    pub email: Option<String>,
    pub is_admin: bool,
}
