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

#[derive(Serialize, Debug)]
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

            if parameters_handler.must_be_formatted() {
                Ok(ResultWrapper::MappedSanctions(map_by_users(result)))
            } else {
                Ok(ResultWrapper::Sanctions(result))
            }
        },
        (POST) (/teams/{team_id: Uuid}/sanctions) => {
            let input = json_input::<Vec<UpdateSanctionRequest>>(request)?;

            let mut error : Option<ErrorResponse> = None;
            let mut sanctions: Vec<CreateSanction> = vec![];

            input.into_iter().map(|update_sanction| {
                let rule = db
                    .get_team(team_id)
                    .map_err(|err| match err {
                        DbError::NotFound => {
                            DbError::ForeignKeyViolation(String::from("The key team_id doesn't refer to anything"))
                        }
                        _ => err,
                    })?
                    .get_rule(update_sanction.sanction_info.associated_rule)
                    .ok_or_else(|| DbError::ForeignKeyViolation(String::from(
                            "The key associated_rule doesn't refer to anything",
                    )))?;

                let price = update_sanction.sanction_info.get_price(rule)?;

                let sanction: CreateSanction = (update_sanction, team_id, price).into();

                Ok(sanction)
            })
            .for_each(|sanction_or_error| match sanction_or_error {
                Ok(sanction)=>sanctions.push(sanction),
                Err(err)=> match error {
                    Some(_)=>{},
                    None=>error=Some(err)
                }
            });

            match error {
                Some(err)=>Err(err),
                None=> {
                    let result = db.create_sanctions(&sanctions)?;
                    Ok(ResultWrapper::Sanctions(result))
                }
            }
        },
        (DELETE) (/teams/{team_id: Uuid}/sanctions/{sanction_id: Uuid}) => {
            let result = db.delete_sanction(team_id, sanction_id)?;

            Ok(ResultWrapper::Sanction(result))
        },
        _ => {
            Err(ErrorResponse::not_found())
        }
    )
}

#[cfg(test)]
mod tests {
    use chrono::{naive::NaiveDate, Local};
    use serde_json::json;

    use super::*;
    use crate::api::models::{test_utils::RequestBuilder, ErrorKind};
    use crate::teams::models::{Rule, RuleKind};
    use crate::test_utils::routes::{DbMock, SanctionsDbMock, TeamsDbMock};

    #[test]
    fn test_get_sanctions() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions", team_id)),
            &DbMock::default(),
        )
        .unwrap());

        for i in 0..3 {
            assert_eq!(response[i]["team_id"], json!(team_id));
        }
    }

    #[test]
    fn test_get_sanctions_filtered() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?month=10&year=2019", team_id)),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_get_sanctions_formatted() {
        let team_id = Uuid::new_v4();

        let response = json!(handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?format=true", team_id)),
            &DbMock::default(),
        )
        .unwrap());

        assert_eq!(response.as_object().unwrap().len(), 3);
    }

    #[test]
    fn test_get_sanctions_with_uncorrect_parameters() {
        let team_id = Uuid::new_v4();

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?format=t", team_id)),
            &DbMock::default(),
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadParameter);
        assert_eq!(
            error.description,
            String::from("The format parameter must be a boolean.")
        );

        let error = handle_request(
            &RequestBuilder::get(format!("/teams/{}/sanctions?month=1", team_id)),
            &DbMock::default(),
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
            &DbMock::default(),
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadParameter);
        assert_eq!(error.description, format!("{} is not a possible value for the month parameter. This value must be between 1 and 12.", month_value));
    }

    #[test]
    fn test_create_sanction() {
        let team_id = Uuid::new_v4();

        let rule_1 = Rule {
            kind: RuleKind::Multiplication {
                price_to_multiply: 3.5,
            },
            ..Default::default()
        };

        let rule_2 = Rule {
            id: Uuid::new_v4(),
            ..Default::default()
        };

        let created_at = NaiveDate::from_ymd(2019, 10, 16);

        let sanctions = json!([{
            "user_id": Uuid::new_v4(),
            "sanction_info": {
                "associated_rule": rule_1.id,
                "extra_info": {
                    "type": "MULTIPLICATION",
                    "factor": 2
                }
            }
        },
        {
            "user_id": Uuid::new_v4(),
            "sanction_info": {
                "associated_rule": rule_2.id,
                "extra_info": {
                    "type": "NONE",
                }
            },
            "created_at": created_at
        }]);

        let response = json!(handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &sanctions),
            &DbMock {
                teams_db: TeamsDbMock::SuccessWithRules(vec![rule_1, rule_2]),
                ..Default::default()
            },
        )
        .unwrap());

        assert_eq!(response[0]["team_id"], json!(team_id));
        assert_eq!(response[0]["user_id"], sanctions[0]["user_id"]);
        assert_eq!(response[0]["price"], json!(7.0));
        assert_eq!(response[0]["created_at"], json!(Local::today().naive_utc()));
        assert_eq!(response[1]["created_at"], json!(created_at));
    }

    #[test]
    fn test_create_sanction_fails() {
        let team_id = Uuid::new_v4();

        let rule = Rule {
            kind: RuleKind::Multiplication {
                price_to_multiply: 3.5,
            },
            ..Default::default()
        };

        let sanction = json!([{
            "user_id": Uuid::new_v4(),
            "sanction_info": {
                "associated_rule": rule.id,
                "extra_info": {
                    "type": "NONE"
                }
            }
        }]);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &sanction),
            &DbMock::default(),
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::BadReference);

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &sanction),
            &DbMock {
                teams_db: TeamsDbMock::SuccessWithRules(vec![rule]),
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotValid);

        let invalid_json = json!({});

        let error = handle_request(
            &RequestBuilder::post(format!("/teams/{}/sanctions", team_id), &invalid_json),
            &DbMock::default(),
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
            &DbMock::default(),
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
            &DbMock {
                sanctions_db: SanctionsDbMock::NotFound,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::NotFound);
    }
}
