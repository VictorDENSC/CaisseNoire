use diesel::{Insertable, Queryable};
use diesel_as_jsonb::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::schema::teams;

#[derive(Deserialize)]
pub struct UpdateTeamRequest {
    pub name: String,
    pub rules: Vec<Rules>,
}

impl From<UpdateTeamRequest> for Team {
    fn from(update_request: UpdateTeamRequest) -> Team {
        Team {
            id: Uuid::new_v4(),
            name: update_request.name,
            rules: update_request.rules,
        }
    }
}

impl From<UpdateTeamRequest> for UpdateTeam {
    fn from(update_request: UpdateTeamRequest) -> UpdateTeam {
        UpdateTeam {
            name: update_request.name,
            rules: update_request.rules,
        }
    }
}

#[derive(Debug, Queryable, Insertable, Serialize, PartialEq)]
#[table_name = "teams"]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub rules: Vec<Rules>,
}

#[derive(AsChangeset)]
#[table_name = "teams"]
pub struct UpdateTeam {
    pub name: String,
    pub rules: Vec<Rules>,
}

#[derive(AsJsonb, Debug, Serialize, Deserialize, PartialEq)]
pub struct Rules {
    pub name: String,
    pub category: RuleCategory,
    pub description: String,
    pub kind: RuleKind,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleCategory {
    GameDay,
    TrainingDay,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleKind {
    Basic {
        price: f32,
    },
    BasedOnMultiple {
        price_to_multiply: f32,
    },
    BasedOnTime {
        price_per_time_unit: f32,
        unit: TimeUnit,
    },
    EachTimeInterval {
        price: f32,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}
