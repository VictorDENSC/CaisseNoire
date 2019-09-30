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

    use super::super::models::{SanctionData, SanctionInfo};
    use super::*;
    use crate::database::postgres::test_utils::{
        create_default_sanction, create_default_user, DbConnectionBuilder,
    };

    #[test]
    fn test_get_sanctions() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let sanction = create_default_sanction(&conn);

            let sanctions: Vec<Sanction> = conn.get_sanctions(sanction.team_id, None).unwrap();

            assert_eq!(vec![sanction], sanctions);

            Ok(())
        });
    }

    #[test]
    fn test_create_sanction() {
        let conn = DbConnectionBuilder::new();

        conn.deref().test_transaction::<_, Error, _>(|| {
            let sanction_id = Uuid::new_v4();
            let user = create_default_user(&conn, "login");

            let sanction: Sanction = conn
                .create_sanction(&CreateSanction {
                    id: sanction_id,
                    user_id: user.id,
                    team_id: user.team_id,
                    sanction_info: SanctionInfo {
                        id: Uuid::new_v4(),
                        sanction_data: SanctionData::Basic,
                    },
                })
                .unwrap();

            assert_eq!(sanction.id, sanction_id);

            Ok(())
        });
    }
}
