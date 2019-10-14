use chrono::naive::NaiveDate;
use uuid::Uuid;

use crate::database::postgres::DbError;
use crate::sanctions::{interface::SanctionsDb, models::*};
use crate::teams::{interface::TeamsDb, models::*};
use crate::users::{interface::UsersDb, models::*};

#[derive(Default)]
pub struct DbMock {
    pub teams_db: TeamsDbMock,
    pub users_db: UsersDbMock,
    pub sanctions_db: SanctionsDbMock,
}

pub enum TeamsDbMock {
    Success,
    NotFound,
    Unknown,
}

impl Default for TeamsDbMock {
    fn default() -> TeamsDbMock {
        TeamsDbMock::Success
    }
}

impl TeamsDb for DbMock {
    fn get_team(&self, id: Uuid) -> Result<Team, DbError> {
        match self.teams_db {
            TeamsDbMock::Success => Ok(Team {
                id: id,
                name: String::from("Test_team"),
                admin_password: String::from("password"),
                rules: vec![],
            }),
            TeamsDbMock::NotFound => Err(DbError::NotFound),
            TeamsDbMock::Unknown => Err(DbError::Unknown),
        }
    }

    fn create_team(&self, team: &Team) -> Result<Team, DbError> {
        match self.teams_db {
            TeamsDbMock::Success => Ok(team.clone()),
            TeamsDbMock::Unknown => Err(DbError::Unknown),
            _ => unimplemented!(),
        }
    }

    fn update_team(&self, id: Uuid, team: &UpdateTeam) -> Result<Team, DbError> {
        match self.teams_db {
            TeamsDbMock::Success => Ok(Team {
                id,
                name: team.name.clone(),
                admin_password: team.admin_password.clone(),
                rules: team.rules.clone(),
            }),
            TeamsDbMock::NotFound => Err(DbError::NotFound),
            TeamsDbMock::Unknown => Err(DbError::Unknown),
        }
    }
}

//TO CHANGE
fn default_user(team_id: Uuid, user_id: Uuid) -> User {
    User {
        id: user_id,
        team_id,
        firstname: String::from("firstname"),
        lastname: String::from("lastname"),
        nickname: None,
        email: None,
    }
}

pub enum UsersDbMock {
    Success,
    NotFound,
    UnexistingTeam,
    DuplicatedField,
}

impl Default for UsersDbMock {
    fn default() -> UsersDbMock {
        UsersDbMock::Success
    }
}

impl UsersDb for DbMock {
    fn get_users(&self, team_id: Uuid) -> Result<Vec<User>, DbError> {
        match self.users_db {
            UsersDbMock::Success => Ok(vec![default_user(team_id, Uuid::new_v4())]),
            _ => unimplemented!(),
        }
    }

    fn get_user(&self, team_id: Uuid, user_id: Uuid) -> Result<User, DbError> {
        match self.users_db {
            UsersDbMock::Success => Ok(default_user(team_id, user_id)),
            UsersDbMock::NotFound => Err(DbError::NotFound),
            _ => unimplemented!(),
        }
    }

    fn create_user(&self, user: &User) -> Result<User, DbError> {
        match self.users_db {
            UsersDbMock::Success => Ok(user.clone()),
            UsersDbMock::UnexistingTeam => Err(DbError::ForeignKeyViolation(String::from("Error"))),
            UsersDbMock::DuplicatedField => Err(DbError::UniqueViolation(String::from("Error"))),
            _ => unimplemented!(),
        }
    }

    fn update_user(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        user: &UpdateUser,
    ) -> Result<User, DbError> {
        match self.users_db {
            UsersDbMock::Success => Ok(User {
                id: user_id,
                team_id,
                firstname: user.firstname.clone(),
                lastname: user.lastname.clone(),
                nickname: user.nickname.clone(),
                email: user.email.clone(),
            }),
            UsersDbMock::NotFound => Err(DbError::NotFound),
            _ => unimplemented!(),
        }
    }
}

//TO CHANGE
fn create_default_sanction(
    id: Option<Uuid>,
    team_id: Uuid,
    created_at: Option<NaiveDate>,
) -> Sanction {
    Sanction {
        id: id.unwrap_or(Uuid::new_v4()),
        user_id: Uuid::new_v4(),
        team_id,
        sanction_info: SanctionInfo {
            associated_rule: Uuid::new_v4(),
            extra_info: ExtraInfo::None,
        },
        price: 0.0,
        created_at: created_at.unwrap_or(NaiveDate::from_ymd(2019, 10, 15)),
    }
}

pub enum SanctionsDbMock {
    Success,
    NotFound,
}

impl Default for SanctionsDbMock {
    fn default() -> SanctionsDbMock {
        SanctionsDbMock::Success
    }
}

impl SanctionsDb for DbMock {
    fn get_sanctions(
        &self,
        team_id: Uuid,
        date_interval: Option<(NaiveDate, NaiveDate)>,
    ) -> Result<Vec<Sanction>, DbError> {
        match self.sanctions_db {
            SanctionsDbMock::Success => {
                let basic_result = vec![
                    create_default_sanction(None, team_id, Some(NaiveDate::from_ymd(2019, 10, 5))),
                    create_default_sanction(None, team_id, Some(NaiveDate::from_ymd(2019, 10, 15))),
                    create_default_sanction(None, team_id, Some(NaiveDate::from_ymd(2019, 11, 5))),
                ];
                Ok(match date_interval {
                    Some((min, max)) => basic_result
                        .into_iter()
                        .filter(|sanction| sanction.created_at >= min && sanction.created_at <= max)
                        .collect(),
                    None => basic_result,
                })
            }
            _ => unimplemented!(),
        }
    }

    fn create_sanction(&self, sanction: &CreateSanction) -> Result<Sanction, DbError> {
        match self.sanctions_db {
            SanctionsDbMock::Success => Ok(Sanction {
                id: sanction.id,
                user_id: sanction.user_id,
                team_id: sanction.team_id,
                sanction_info: sanction.sanction_info.clone(),
                price: 0.0,
                created_at: NaiveDate::from_ymd(2019, 10, 15),
            }),
            SanctionsDbMock::NotFound => Err(DbError::ForeignKeyViolation(String::from("Error"))),
        }
    }

    fn delete_sanction(&self, team_id: Uuid, sanction_id: Uuid) -> Result<Sanction, DbError> {
        match self.sanctions_db {
            SanctionsDbMock::Success => {
                Ok(create_default_sanction(Some(sanction_id), team_id, None))
            }
            SanctionsDbMock::NotFound => Err(DbError::NotFound),
        }
    }
}
