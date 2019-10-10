use diesel::{Insertable, Queryable};
use diesel_as_jsonb::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::schema::teams;

#[derive(Deserialize)]
pub struct UpdateTeamRequest {
    pub id: Option<Uuid>,
    pub name: String,
    pub admin_password: String,
    pub rules: Vec<UpdateRuleRequest>,
}

impl From<UpdateTeamRequest> for Team {
    fn from(update_request: UpdateTeamRequest) -> Team {
        Team {
            id: update_request.id.unwrap_or(Uuid::new_v4()),
            name: update_request.name,
            admin_password: update_request.admin_password,
            rules: update_request
                .rules
                .into_iter()
                .map(|update_rule_request| update_rule_request.into())
                .collect(),
        }
    }
}

impl From<UpdateTeamRequest> for UpdateTeam {
    fn from(update_request: UpdateTeamRequest) -> UpdateTeam {
        UpdateTeam {
            name: update_request.name,
            admin_password: update_request.admin_password,
            rules: update_request
                .rules
                .into_iter()
                .map(|update_rule_request| update_rule_request.into())
                .collect(),
        }
    }
}

#[derive(Debug, Queryable, Insertable, Serialize, PartialEq, Clone)]
#[table_name = "teams"]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub admin_password: String,
    pub rules: Vec<Rule>,
}

#[derive(AsChangeset)]
#[table_name = "teams"]
pub struct UpdateTeam {
    pub name: String,
    pub admin_password: String,
    pub rules: Vec<Rule>,
}

#[derive(Deserialize)]
pub struct UpdateRuleRequest {
    pub id: Option<Uuid>,
    pub name: String,
    pub category: RuleCategory,
    pub description: String,
    pub kind: RuleKind,
}

impl From<UpdateRuleRequest> for Rule {
    fn from(update_request: UpdateRuleRequest) -> Rule {
        Rule {
            id: update_request.id.unwrap_or(Uuid::new_v4()),
            name: update_request.name,
            category: update_request.category,
            description: update_request.description,
            kind: update_request.kind,
        }
    }
}

#[derive(AsJsonb, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub category: RuleCategory,
    pub description: String,
    pub kind: RuleKind,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleCategory {
    GameDay,
    TrainingDay,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleKind {
    Basic {
        price: f32,
    },
    Multiplication {
        price_to_multiply: f32,
    },
    TimeMultiplication {
        price_per_time_unit: f32,
        time_unit: TimeUnit,
    },
    RegularIntervals {
        price: f32,
        interval_in_time_unit: u32,
        time_unit: TimeUnit,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Week,
    Month,
    Year,
}
