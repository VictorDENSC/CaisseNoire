use chrono::naive::NaiveDate;
use diesel::{Insertable, Queryable};
use diesel_as_jsonb::AsJsonb;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::database::schema::sanctions;
use crate::teams::models::{Rule, RuleKind};

#[derive(Deserialize)]
pub struct UpdateSanctionRequest {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub sanction_info: SanctionInfo,
    pub created_at: Option<NaiveDate>,
}

impl From<(UpdateSanctionRequest, Uuid, f32)> for CreateSanction {
    fn from(
        (update_request, team_id, price): (UpdateSanctionRequest, Uuid, f32),
    ) -> CreateSanction {
        CreateSanction {
            id: update_request.id.unwrap_or_else(Uuid::new_v4),
            user_id: update_request.user_id,
            team_id,
            sanction_info: update_request.sanction_info,
            price,
            created_at: update_request.created_at,
        }
    }
}

#[derive(Queryable, Debug, PartialEq, Serialize, Clone)]
pub struct Sanction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub sanction_info: SanctionInfo,
    pub price: f32,
    pub created_at: NaiveDate,
}

impl Default for Sanction {
    fn default() -> Sanction {
        Sanction {
            id: Default::default(),
            user_id: Default::default(),
            team_id: Default::default(),
            sanction_info: Default::default(),
            price: Default::default(),
            created_at: NaiveDate::from_ymd(2019, 10, 5),
        }
    }
}

#[derive(Insertable, Default)]
#[table_name = "sanctions"]
pub struct CreateSanction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub sanction_info: SanctionInfo,
    pub price: f32,
    pub created_at: Option<NaiveDate>,
}

pub struct SanctionInfoError {
    pub associated_rule_name: String,
    pub associated_rule_kind: String,
    pub extra_info: String,
}

#[derive(Debug, AsJsonb, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct SanctionInfo {
    pub associated_rule: Uuid,
    pub extra_info: ExtraInfo,
}

impl SanctionInfo {
    pub fn get_price(&self, rule: Rule) -> Result<f32, SanctionInfoError> {
        match self.extra_info {
            ExtraInfo::None => match rule.kind {
                RuleKind::Basic { price } => Ok(price),
                _ => Err(()),
            },
            ExtraInfo::Multiplication { factor } => match rule.kind {
                RuleKind::Multiplication {
                    price_to_multiply, ..
                } => Ok(price_to_multiply * (factor as f32)),
                RuleKind::TimeMultiplication {
                    price_per_time_unit,
                    ..
                } => Ok(price_per_time_unit * (factor as f32)),
                _ => Err(()),
            },
        }
        .map_err(|_| SanctionInfoError {
            associated_rule_name: rule.name,
            associated_rule_kind: rule.kind.to_string(),
            extra_info: self.extra_info.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum ExtraInfo {
    None,
    Multiplication { factor: u32 },
}

impl Default for ExtraInfo {
    fn default() -> ExtraInfo {
        ExtraInfo::None
    }
}

impl fmt::Display for ExtraInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtraInfo::None => write!(f, "NONE"),
            ExtraInfo::Multiplication { .. } => write!(f, "MULTIPLICATION"),
        }
    }
}
