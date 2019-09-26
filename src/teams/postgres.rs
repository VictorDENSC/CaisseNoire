use diesel::prelude::*;
use std::ops::Deref;
use uuid::Uuid;

use super::interface::TeamsDb;
use super::models::{Team, UpdateTeam};
use crate::database::{
    postgres::{DbConnection, DbError},
    schema::teams,
};

impl TeamsDb for DbConnection {
    fn get_team_by_id(&self, id: Uuid) -> Result<Team, DbError> {
        let team: Team = teams::table.find(id).get_result::<Team>(self.deref())?;

        Ok(team)
    }

    fn create_team(&self, team: Team) -> Result<Team, DbError> {
        let team: Team = diesel::insert_into(teams::table)
            .values(team)
            .get_result(self.deref())?;

        Ok(team)
    }

    fn update_team(&self, id: Uuid, team: UpdateTeam) -> Result<Team, DbError> {
        let team: Team = diesel::update(teams::table.find(id))
            .set(&team)
            .get_result(self.deref())?;

        Ok(team)
    }
}
