use chrono::naive::NaiveDate;
use diesel::prelude::*;
use std::ops::Deref;
use uuid::Uuid;

use super::{
    interface::SanctionsDb,
    models::{CreateSanction, Sanction},
};
use crate::database::{
    postgres::{DbConnection, DbError},
    schema::sanctions,
};

impl SanctionsDb for DbConnection {
    fn get_sanctions(
        &self,
        team_id: Uuid,
        date_interval: Option<(NaiveDate, NaiveDate)>,
    ) -> Result<Vec<Sanction>, DbError> {
        let sanctions = match date_interval {
            Some((min, max)) => sanctions::table
                .filter(
                    sanctions::created_at
                        .between(min, max)
                        .and(sanctions::team_id.eq(team_id)),
                )
                .get_results(self.deref())?,
            None => sanctions::table
                .filter(sanctions::team_id.eq(team_id))
                .get_results(self.deref())?,
        };

        Ok(sanctions)
    }

    fn create_sanction(&self, sanction: &CreateSanction) -> Result<Sanction, DbError> {
        let sanction: Sanction = diesel::insert_into(sanctions::table)
            .values(sanction)
            .get_result(self.deref())?;

        Ok(sanction)
    }

    fn delete_sanction(&self, team_id: Uuid, sanction_id: Uuid) -> Result<Sanction, DbError> {
        let sanction: Sanction = diesel::delete(
            sanctions::table.filter(
                sanctions::team_id
                    .eq(team_id)
                    .and(sanctions::id.eq(sanction_id)),
            ),
        )
        .get_result(self.deref())?;

        Ok(sanction)
    }
}

#[cfg(test)]
mod tests {
    use diesel::result::Error;

    use super::super::models::{ExtraInfo, SanctionInfo};
    use super::*;
    use crate::database::postgres::test_utils::{
        create_default_sanction, create_default_team, create_default_user, DbConnectionBuilder,
    };

    #[test]
    fn test_get_sanctions() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let sanction =
                create_default_sanction(&conn, &create_default_user(&conn, None, None), None);

            create_default_sanction(
                &conn,
                &create_default_user(
                    &conn,
                    Some(create_default_team(&conn, Some(String::from("Team_Test_2"))).id),
                    None,
                ),
                None,
            );

            let sanctions: Vec<Sanction> = conn.get_sanctions(sanction.team_id, None).unwrap();

            assert_eq!(vec![sanction], sanctions);

            Ok(())
        });
    }

    #[test]
    fn test_get_sanctions_with_date_interval() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let default_user = create_default_user(&conn, None, None);

            let sanction = create_default_sanction(
                &conn,
                &default_user,
                Some(&NaiveDate::from_ymd(2019, 10, 13)),
            );
            create_default_sanction(
                &conn,
                &default_user,
                Some(&NaiveDate::from_ymd(2019, 10, 5)),
            );
            create_default_sanction(
                &conn,
                &default_user,
                Some(&NaiveDate::from_ymd(2019, 10, 25)),
            );

            let sanctions: Vec<Sanction> = conn
                .get_sanctions(
                    default_user.team_id,
                    Some((
                        NaiveDate::from_ymd(2019, 10, 6),
                        NaiveDate::from_ymd(2019, 10, 20),
                    )),
                )
                .unwrap();

            assert_eq!(vec![sanction], sanctions);

            Ok(())
        })
    }

    #[test]
    fn test_create_sanction() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let sanction_id = Uuid::new_v4();
            let user = create_default_user(&conn, None, None);
            let team = conn.get_team(user.team_id).unwrap();

            let sanction: Sanction = conn
                .create_sanction(&CreateSanction {
                    id: sanction_id,
                    user_id: user.id,
                    team_id: user.team_id,
                    sanction_info: SanctionInfo {
                        associated_rule: team.rules[0].id,
                        extra_info: ExtraInfo::None,
                    },
                    price: 0.0,
                })
                .unwrap();

            assert_eq!(sanction.id, sanction_id);

            Ok(())
        });
    }

    #[test]
    fn test_create_sanction_fails() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let default_user = create_default_user(&conn, None, None);
            let mut sanction = CreateSanction {
                id: Uuid::new_v4(),
                team_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                sanction_info: SanctionInfo {
                    associated_rule: Uuid::new_v4(),
                    extra_info: ExtraInfo::None,
                },
                price: 0.0,
            };

            let error = conn.create_sanction(&sanction).unwrap_err();
            assert_eq!(
                error,
                DbError::ForeignKeyViolation(String::from(
                    "The key team_id doesn\'t refer to anything"
                ))
            );

            sanction.team_id = default_user.team_id;
            let error = conn.create_sanction(&sanction).unwrap_err();
            assert_eq!(
                error,
                DbError::ForeignKeyViolation(String::from(
                    "The key user_id doesn\'t refer to anything"
                ))
            );

            sanction.user_id = default_user.id;
            let error = conn.create_sanction(&sanction).unwrap_err();
            assert_eq!(
                error,
                DbError::ForeignKeyViolation(String::from(
                    "The key associated_rule doesn\'t refer to anything"
                ))
            );

            Ok(())
        });
    }

    #[test]
    fn test_delete_sanction() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let default_user = create_default_user(&conn, None, None);
            let sanction = create_default_sanction(&conn, &default_user, None);

            let sanction_deleted = conn.delete_sanction(sanction.team_id, sanction.id).unwrap();

            let error = sanctions::table
                .find(sanction.id)
                .get_result::<Sanction>(conn.deref())
                .unwrap_err();

            assert_eq!(sanction.id, sanction_deleted.id);
            assert_eq!(error, Error::NotFound);

            Ok(())
        });
    }

    #[test]
    fn test_delete_sanction_fails() {
        let conn = DbConnectionBuilder::new();

        let error = conn
            .delete_sanction(Uuid::new_v4(), Uuid::new_v4())
            .unwrap_err();

        assert_eq!(error, DbError::NotFound);
    }
}
