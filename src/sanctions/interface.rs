use chrono::naive::NaiveDate;
use uuid::Uuid;

use super::models::{CreateSanction, Sanction};
use crate::database::postgres::DbError;

pub trait SanctionsDb {
    fn get_sanctions(
        &self,
        team_id: Uuid,
        date_interval: Option<(NaiveDate, NaiveDate)>,
    ) -> Result<Vec<Sanction>, DbError>;

    fn create_sanction(&self, sanction: &CreateSanction) -> Result<Sanction, DbError>;

    fn delete_sanction(&self, team_id: Uuid, sanction_id: Uuid) -> Result<Sanction, DbError>;
}
