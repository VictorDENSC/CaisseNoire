use chrono::naive::NaiveDate;
use diesel::{Insertable, Queryable};
use diesel_as_jsonb::AsJsonb;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::schema::sanctions;
use crate::teams::models::RuleKind;

#[derive(Deserialize)]
pub struct UpdateSanctionRequest {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub sanction_info: SanctionInfo,
}

impl From<(UpdateSanctionRequest, Uuid)> for CreateSanction {
    fn from((update_request, team_id): (UpdateSanctionRequest, Uuid)) -> CreateSanction {
        CreateSanction {
            id: update_request.id.unwrap_or(Uuid::new_v4()),
            user_id: update_request.user_id,
            team_id,
            sanction_info: update_request.sanction_info,
        }
    }
}

#[derive(Queryable, Debug, PartialEq, Serialize, Clone)]
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

#[derive(Debug, AsJsonb, Serialize, Deserialize, PartialEq, Clone)]
pub struct SanctionInfo {
    pub associated_rule: Uuid,
    pub extra_info: ExtraInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum ExtraInfo {
    None,
    Multiplication { factor: u32 },
}

impl ExtraInfo {
    pub fn r#match(&self, rule_kind: &RuleKind) -> bool {
        match self {
            ExtraInfo::None => match rule_kind {
                RuleKind::Basic { .. } => true,
                _ => false,
            },
            ExtraInfo::Multiplication { factor: _ } => match rule_kind {
                RuleKind::Multiplication { .. } | RuleKind::TimeMultiplication { .. } => true,
                _ => false,
            },
        }
    }
}
