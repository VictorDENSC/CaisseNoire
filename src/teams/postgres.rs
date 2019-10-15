use diesel::prelude::*;
use std::ops::Deref;
use uuid::Uuid;

use super::{
    interface::TeamsDb,
    models::{Team, UpdateTeam},
};
use crate::database::{
    postgres::{DbConnection, DbError},
    schema::teams,
};

impl TeamsDb for DbConnection {
    fn get_team(&self, id: Uuid) -> Result<Team, DbError> {
        let team: Team = teams::table.find(id).get_result(self.deref())?;

        Ok(team)
    }

    fn create_team(&self, team: &Team) -> Result<Team, DbError> {
        let team: Team = diesel::insert_into(teams::table)
            .values(team)
            .get_result(self.deref())?;

        Ok(team)
    }

    fn update_team(&self, id: Uuid, team: &UpdateTeam) -> Result<Team, DbError> {
        let team: Team = diesel::update(teams::table.find(id))
            .set(team)
            .get_result(self.deref())?;

        Ok(team)
    }
}

#[cfg(test)]
mod tests {
    use diesel::result::Error;

    use super::*;
    use crate::test_utils::postgres::init_connection;

    #[test]
    fn test_get_team() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let created_team = conn.create_team(&Team::default()).unwrap();

            let team = conn.get_team(created_team.id).unwrap();

            assert_eq!(team, created_team);

            Ok(())
        });
    }

    #[test]
    fn test_get_unexisting_team() {
        let conn = init_connection();

        let error = conn.get_team(Uuid::new_v4()).unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }

    #[test]
    fn test_create_team() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            conn.create_team(&Team::default()).unwrap();

            Ok(())
        })
    }

    #[test]
    fn test_update_team() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let id = conn.create_team(&Team::default()).unwrap().id;

            let name = String::from("New name");

            let team = conn
                .update_team(
                    id,
                    &UpdateTeam {
                        name: name.clone(),
                        ..Default::default()
                    },
                )
                .unwrap();

            assert_eq!(id, team.id);
            assert_eq!(team.name, name);

            Ok(())
        });
    }

    #[test]
    fn test_update_unexisting_team() {
        let conn = init_connection();

        let error = conn
            .update_team(Uuid::new_v4(), &UpdateTeam::default())
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }
}
