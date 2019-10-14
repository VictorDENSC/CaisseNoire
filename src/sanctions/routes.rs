use rouille::{input::json::json_input, router, Request};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    interface::SanctionsDb,
    models::{CreateSanction, Sanction, UpdateSanctionRequest},
    utils::{formatter::map_by_users, parameters::ParametersHandler},
};
use crate::api::models::ErrorResponse;
use crate::database::postgres::DbError;
use crate::teams::interface::TeamsDb;

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ResultWrapper {
    Sanctions(Vec<Sanction>),
    MappedSanctions(HashMap<Uuid, Vec<Sanction>>),
    Sanction(Sanction),
}

pub fn handle_request<T>(request: &Request, db: &T) -> Result<ResultWrapper, ErrorResponse>
where
    T: SanctionsDb + TeamsDb,
{
    router!(request,
    (GET) (/teams/{team_id: Uuid}/sanctions) => {
        let parameters_handler = ParametersHandler::from_request(request)?;

        let result = db.get_sanctions(team_id, parameters_handler.date_interval())?;

        match parameters_handler.must_be_formatted() {
            true => Ok(ResultWrapper::MappedSanctions(map_by_users(result))),
            false => Ok(ResultWrapper::Sanctions(result))
        }
    },
    (POST) (/teams/{team_id: Uuid}/sanctions) => {
        let input = json_input::<UpdateSanctionRequest>(request)?;

        let rule = db
            .get_team(team_id)
            .map_err(|err| match err {
                DbError::NotFound => {
                    DbError::ForeignKeyViolation(String::from("The key team_id doesn't refer to anything"))
                }
                _ => err,
            })?
            .get_rule(input.sanction_info.associated_rule)
            .ok_or(DbError::ForeignKeyViolation(String::from(
                    "The key associated_rule doesn't refer to anything",
            )))?;

        let price = input.sanction_info.get_price(rule)?;

        let sanction: CreateSanction = (input, team_id, price).into();

        let result = db.create_sanction(&sanction)?;

        Ok(ResultWrapper::Sanction(result))
    },
    (DELETE) (/teams/{team_id: Uuid}/sanctions/{sanction_id: Uuid}) => {
        let result = db.delete_sanction(team_id, sanction_id)?;

        Ok(ResultWrapper::Sanction(result))
    },
    _ => Err(ErrorResponse::not_found())
    )
}

#[cfg(test)]
mod tests {
    use chrono::naive::NaiveDate;
    use serde_json::json;

    use super::super::models::{ExtraInfo, SanctionInfo};
    use super::*;
    use crate::api::models::{test_utils::RequestBuilder, ErrorKind};
    use crate::database::postgres::DbError;

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

    enum SanctionsDbMock {
        Success,
        NotFound,
    }

    impl SanctionsDb for SanctionsDbMock {
        fn get_sanctions(
            &self,
            team_id: Uuid,
            date_interval: Option<(NaiveDate, NaiveDate)>,
        ) -> Result<Vec<Sanction>, DbError> {
            match self {
                SanctionsDbMock::Success => {
                    let basic_result = vec![
                        create_default_sanction(
                            None,
                            team_id,
                            Some(NaiveDate::from_ymd(2019, 10, 5)),
                        ),
                        create_default_sanction(
                            None,
                            team_id,
                            Some(NaiveDate::from_ymd(2019, 10, 15)),
                        ),
                        create_default_sanction(
                            None,
                            team_id,
                            Some(NaiveDate::from_ymd(2019, 11, 5)),
                        ),
                    ];
                    Ok(match date_interval {
                        Some((min, max)) => basic_result
                            .into_iter()
                            .filter(|sanction| {
                                sanction.created_at >= min && sanction.created_at <= max
                            })
                            .collect(),
                        None => basic_result,
                    })
                }
                _ => unimplemented!(),
            }
        }

        fn create_sanction(&self, sanction: &CreateSanction) -> Result<Sanction, DbError> {
            match self {
                SanctionsDbMock::Success => Ok(Sanction {
                    id: sanction.id,
                    user_id: sanction.user_id,
                    team_id: sanction.team_id,
                    sanction_info: sanction.sanction_info.clone(),
                    price: 0.0,
                    created_at: NaiveDate::from_ymd(2019, 10, 15),
                }),
                SanctionsDbMock::NotFound => {
                    Err(DbError::ForeignKeyViolation(String::from("Error")))
                }
            }
        }

        fn delete_sanction(&self, team_id: Uuid, sanction_id: Uuid) -> Result<Sanction, DbError> {
            match self {
                SanctionsDbMock::Success => {
                    Ok(create_default_sanction(Some(sanction_id), team_id, None))
                }
                SanctionsDbMock::NotFound => Err(DbError::NotFound),
            }
        }
    }

    #[test]
    fn test_get_sanctions() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions", team_id)),
            &SanctionsDbMock::Success,
        )
        .unwrap());

        assert_eq!(response[0]["team_id"], json!(team_id));
    }

    #[test]
    fn test_get_sanctions_filtered() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?month=10&year=2019", team_id)),
            &SanctionsDbMock::Success,
        )
        .unwrap());

        assert_eq!(response.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_get_sanctions_formatted() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?format=true", team_id)),
            &SanctionsDbMock::Success,
        )
        .unwrap());

        assert_eq!(response.as_object().unwrap().len(), 3);
    }

    #[test]
    fn test_get_sanctions_with_uncorrect_parameters() {
        let team_id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?format=t", team_id)),
            &SanctionsDbMock::Success,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadParameter);
        assert_eq!(
            error.description,
            String::from("The format parameter must be a boolean.")
        );

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?month=1", team_id)),
            &SanctionsDbMock::Success,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadParameter);
        assert_eq!(
            error.description,
            String::from("The month parameter must be combined with these parameters : year.")
        );

        let month_value = 13;
        let error = handle_request(
            &RequestBuilder::get(format!(
                "/teams/{}/sanctions?month={}&year=2019",
                team_id, month_value
            )),
            &SanctionsDbMock::Success,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadParameter);
        assert_eq!(error.description, format!("{} is not a possible value for the month parameter. This value must be between 1 and 12.", month_value));
    }

    #[test]
    fn test_create_sanction() {
        let team_id = Uuid::new_v4();

        let sanction = json!({
            "user_id": Uuid::new_v4(),
            "sanction_info": {
                "associated_rule": Uuid::new_v4(),
                "extra_info": {
                    "type": "NONE"
                }
            }
        });

        let response = json!(handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &sanction),
            &SanctionsDbMock::Success,
        )
        .unwrap());

        assert_eq!(response["team_id"], json!(team_id));
        assert_eq!(response["user_id"], sanction["user_id"]);
        assert_eq!(response["sanction_info"], sanction["sanction_info"]);
    }

    #[test]
    fn test_create_sanction_fails() {
        let team_id = Uuid::new_v4();

        let sanction = json!({
            "user_id": Uuid::new_v4(),
            "sanction_info": {
                "associated_rule": Uuid::new_v4(),
                "extra_info": {
                    "type": "NONE"
                }
            }
        });

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &sanction),
            &SanctionsDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadReference);

        let invalid_json = json!({});

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &invalid_json),
            &SanctionsDbMock::Success,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Json);
    }

    #[test]
    fn test_delete_sanction() {
        let team_id = Uuid::new_v4();
        let sanction_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::delete(format!("/teams/{}/sanctions/{}", team_id, sanction_id)),
            &SanctionsDbMock::Success,
        )
        .unwrap());

        assert_eq!(response["team_id"], json!(team_id));
        assert_eq!(response["id"], json!(sanction_id));
    }

    #[test]
    fn test_delete_sanction_fails() {
        let team_id = Uuid::new_v4();
        let sanction_id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::delete(format!("/teams/{}/sanctions/{}", team_id, sanction_id)),
            &SanctionsDbMock::NotFound,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);
    }
}
