use chrono::naive::NaiveDate;
use diesel::{Insertable, Queryable};
use diesel_as_jsonb::AsJsonb;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::schema::sanctions;

#[derive(Queryable, Debug, PartialEq)]
pub struct Sanction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub sanction_info: SanctionInfo,
    pub created_at: NaiveDate,
}

#[derive(Insertable)]
#[table_name = "sanctions"]
pub struct CreateSanction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub sanction_info: SanctionInfo,
}

#[derive(Debug, AsJsonb, Serialize, Deserialize, PartialEq)]
pub struct SanctionInfo {
    pub associated_rule: Uuid,
    pub sanction_data: SanctionData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SanctionData {
    Basic,
    Multiplication { multiple: u32 },
    TimeMultiplication { times_unit: u32 },
}
