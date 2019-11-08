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
    fn login(&self, name: &str, admin_password: &Option<String>) -> Result<Uuid, DbError> {
        let team: Team = match admin_password {
            Some(password) => teams::table
                .filter(teams::name.eq(name).and(teams::admin_password.eq(password)))
                .get_result(self.deref())?,
            None => teams::table
                .filter(teams::name.eq(name))
                .get_result(self.deref())?,
        };

        Ok(team.id)
    }

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
    fn test_login() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let created_team = conn
                .create_team(&Team {
                    name: String::from("CHBC"),
                    ..Default::default()
                })
                .unwrap();

            let team_id = conn.login(&created_team.name, &None).unwrap();

            assert_eq!(team_id, created_team.id);

            Ok(())
        });
    }

    #[test]
    fn test_login_with_password() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let created_team = conn
                .create_team(&Team {
                    name: String::from("CHBC"),
                    admin_password: String::from("password"),
                    ..Default::default()
                })
                .unwrap();

            let team_id = conn
                .login(&created_team.name, &Some(created_team.admin_password))
                .unwrap();

            assert_eq!(team_id, created_team.id);

            Ok(())
        });
    }

    #[test]
    fn test_login_fails() {
        let conn = init_connection();

        conn.deref().test_transaction::<_, Error, _>(|| {
            conn.create_team(&Team::default()).unwrap();

            let error = conn.login(&String::from("CHBC"), &None).unwrap_err();

            assert_eq!(error, DbError::NotFound);

            Ok(())
        });

        conn.deref().test_transaction::<_, Error, _>(|| {
            let created_team = conn
                .create_team(&Team {
                    name: String::from("CHBC"),
                    ..Default::default()
                })
                .unwrap();

            let error = conn
                .login(&created_team.name, &Some(String::from("password")))
                .unwrap_err();

            assert_eq!(error, DbError::NotFound);

            Ok(())
        });
    }

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
