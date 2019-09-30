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

    use super::super::models::{Rule, RuleCategory, RuleKind, TimeUnit};
    use super::*;
    use crate::database::postgres::test_utils::{create_default_team, DbConnectionBuilder};

    #[test]
    fn test_get_team() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let new_team = create_default_team(&conn);

            let team = conn.get_team(new_team.id).unwrap();

            assert_eq!(team, new_team);

            Ok(())
        });
    }

    #[test]
    fn test_get_unexisting_team() {
        let conn = DbConnectionBuilder::new();

        let error = conn.get_team(Uuid::new_v4()).unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }

    #[test]
    fn test_create_team() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let team = Team {
                id: Uuid::new_v4(),
                name: String::from("Team_Test"),
                rules: vec![Rule {
                    id: Uuid::new_v4(),
                    name: String::from("Rule_Test"),
                    category: RuleCategory::TrainingDay,
                    description: String::from("This is a description !"),
                    kind: RuleKind::TimeMultiplication {
                        price_per_time_unit: 0.2,
                        time_unit: TimeUnit::Minutes,
                    },
                }],
            };

            let team_created = conn.create_team(&team).unwrap();

            assert_eq!(team_created, team);

            Ok(())
        })
    }

    #[test]
    fn test_update_team() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let new_team = create_default_team(&conn);

            let team = conn
                .update_team(
                    new_team.id,
                    &UpdateTeam {
                        name: String::from("New name"),
                        rules: new_team.rules,
                    },
                )
                .unwrap();

            assert_eq!(&team.name, "New name");

            Ok(())
        });
    }

    #[test]
    fn test_update_unexisting_team() {
        let conn = DbConnectionBuilder::new();

        let error = conn
            .update_team(
                Uuid::new_v4(),
                &UpdateTeam {
                    name: String::from(""),
                    rules: vec![],
                },
            )
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }
}
