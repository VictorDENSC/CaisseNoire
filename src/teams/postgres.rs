use diesel::prelude::*;
use uuid::Uuid;

use super::database::{
    postgres::{Database, DbError},
    schema::teams,
};
use super::interface::TeamsDb;
use super::models::{Team, UpdateTeam};

impl TeamsDb for Database {
    fn get_team_by_id(&self, id: Uuid) -> Result<Team, DbError> {
        let conn = self.get_connection()?;

        let team: Team = teams::table.find(id).get_result::<Team>(&*conn)?;

        Ok(team)
    }

    fn create_team(&self, team: Team) -> Result<Team, DbError> {
        let conn = self.get_connection()?;

        let team: Team = diesel::insert_into(teams::table)
            .values(team)
            .get_result(&*conn)?;

        Ok(team)
    }

    fn update_team(&self, id: Uuid, team: UpdateTeam) -> Result<Team, DbError> {
        let conn = self.get_connection()?;

        let team: Team = diesel::update(teams::table.find(id))
            .set(&team)
            .get_result(&*conn)?;

        Ok(team)
    }
}
