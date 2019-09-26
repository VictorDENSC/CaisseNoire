use uuid::Uuid;

use crate::database::postgres::DbError;
use super::models::{Team, UpdateTeam};

pub trait TeamsDb {
    fn get_team_by_id(&self, id: Uuid) -> Result<Team, DbError>;

    fn create_team(&self, team: Team) -> Result<Team, DbError>;

    fn update_team(&self, id: Uuid, team: UpdateTeam) -> Result<Team, DbError>;
}
