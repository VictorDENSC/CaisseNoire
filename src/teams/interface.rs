use uuid::Uuid;

use super::models::{Team, UpdateTeam};
use crate::database::postgres::DbError;

pub trait TeamsDb {
    fn get_team(&self, id: Uuid) -> Result<Team, DbError>;

    fn create_team(&self, team: &Team) -> Result<Team, DbError>;

    fn update_team(&self, id: Uuid, team: &UpdateTeam) -> Result<Team, DbError>;
}
