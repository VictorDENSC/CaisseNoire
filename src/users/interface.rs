use uuid::Uuid;

use super::models::{UpdateUser, User};
use crate::database::postgres::DbError;

pub trait UsersDb {
    fn get_users(&self, team_id: Uuid) -> Result<Vec<User>, DbError>;

    fn get_user_by_id(&self, team_id: Uuid, user_id: Uuid) -> Result<User, DbError>;

    fn create_user(&self, user: &User) -> Result<User, DbError>;

    fn update_user(&self, team_id: Uuid, user_id: Uuid, user: &UpdateUser)
        -> Result<User, DbError>;
}
