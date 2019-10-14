use chrono::naive::NaiveDate;
use diesel::{
    dsl::{date, now},
    prelude::*,
};
use std::ops::Deref;
use uuid::Uuid;

use crate::database::{postgres::*, schema::*};
use crate::sanctions::models::{ExtraInfo, Sanction, SanctionInfo};
use crate::teams::models::{Rule, RuleCategory, RuleKind, Team};
use crate::users::models::User;

pub struct DbConnectionBuilder;

impl DbConnectionBuilder {
    pub fn new() -> DbConnection {
        init_db_connection("postgres://postgres:password@localhost/caisse_noire")
            .expect("Something went wrong while getting the connection")
    }
}

pub fn insert_default_team(conn: &DbConnection, name: Option<String>) -> Team {
    let default_team = Team {
        id: Uuid::new_v4(),
        name: name.unwrap_or(String::from("Test_team")),
        admin_password: String::from("password"),
        rules: vec![Rule {
            id: Uuid::new_v4(),
            name: String::from("rule_1"),
            category: RuleCategory::TrainingDay,
            description: String::from("Basic rule"),
            kind: RuleKind::Basic { price: 2.5 },
        }],
    };

    diesel::insert_into(teams::table)
        .values(&default_team)
        .get_result(conn.deref())
        .expect("Failed to create default team")
}

pub fn insert_default_user(
    conn: &DbConnection,
    team_id: Option<Uuid>,
    email: Option<String>,
) -> User {
    let default_team_id = team_id.unwrap_or_else(|| insert_default_team(conn, None).id);

    let default_user = User {
        id: Uuid::new_v4(),
        team_id: default_team_id,
        firstname: String::from("firstname"),
        lastname: String::from("lastname"),
        nickname: None,
        email,
    };

    diesel::insert_into(users::table)
        .values(&default_user)
        .get_result(conn.deref())
        .expect("Failed to create default user")
}

pub fn insert_default_sanction(
    conn: &DbConnection,
    user: &User,
    created_at: Option<&NaiveDate>,
) -> Sanction {
    diesel::insert_into(sanctions::table)
        .values((
            sanctions::id.eq(Uuid::new_v4()),
            sanctions::user_id.eq(user.id),
            sanctions::team_id.eq(user.team_id),
            sanctions::price.eq(0.0),
            sanctions::sanction_info.eq(SanctionInfo {
                associated_rule: Uuid::new_v4(),
                extra_info: ExtraInfo::None,
            }),
            sanctions::created_at
                .eq(created_at.unwrap_or(&diesel::select(date(now)).first(conn.deref()).unwrap())),
        ))
        .get_result(conn.deref())
        .expect("Failed to create default sanction")
}
