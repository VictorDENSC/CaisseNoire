use diesel::{Insertable, Queryable};
use uuid::Uuid;

use crate::database::schema::users;

#[derive(Debug, Queryable, Insertable, PartialEq)]
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
}
