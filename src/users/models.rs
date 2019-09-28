use diesel::{Insertable, Queryable};
use uuid::Uuid;

use crate::database::schema::users;

#[derive(Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    id: Uuid,
    team_id: Uuid,
    firstname: String,
    lastname: String,
    nickname: Option<String>,
    login: String,
    password: String,
    email: Option<String>
}