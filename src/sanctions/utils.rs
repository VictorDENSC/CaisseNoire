pub mod parameters {
    use chrono::NaiveDate;
    use rouille::Request;

    #[derive(Debug)]
    pub struct ParameterError {
        pub parameter_name: String,
        pub kind: ParameterErrorKind,
    }

    #[derive(Debug, PartialEq)]
    pub enum ParameterErrorKind {
        UnvalidType {
            expected_type: String,
        },
        UnvalidValue {
            parameter_value: String,
            reason: String,
        },
        UnvalidCombination {
            missing_parameters: Vec<String>,
        },
    }

    #[derive(Debug)]
    pub struct ParametersHandler {
        format: Option<FormatParameter>,
        year_and_month: Option<(YearParameter, MonthParameter)>,
    }

    impl ParametersHandler {
        pub fn from_request(request: &Request) -> Result<ParametersHandler, ParameterError> {
            let format = FormatParameter::from_request(request)?;
            let year_and_month = Self::extract_year_and_month(request)?;

            Ok(ParametersHandler {
                format,
                year_and_month,
            })
        }

        fn extract_year_and_month(
            request: &Request,
        ) -> Result<Option<(YearParameter, MonthParameter)>, ParameterError> {
            match (
                YearParameter::from_request(request)?,
                MonthParameter::from_request(request)?,
            ) {
                (Some(_), None) => Err(ParameterError {
                    parameter_name: YearParameter::parameter_name(),
                    kind: ParameterErrorKind::UnvalidCombination {
                        missing_parameters: vec![MonthParameter::parameter_name()],
                    },
                }),
                (None, Some(_)) => Err(ParameterError {
                    parameter_name: MonthParameter::parameter_name(),
                    kind: ParameterErrorKind::UnvalidCombination {
                        missing_parameters: vec![YearParameter::parameter_name()],
                    },
                }),
                (None, None) => Ok(None),
                (Some(year), Some(month)) => Ok(Some((year, month))),
            }
        }

        pub fn date_interval(&self) -> Option<(NaiveDate, NaiveDate)> {
            match &self.year_and_month {
                Some((year, month)) => {
                    let year_2 = if month.0 == 12 { year.0 + 1 } else { year.0 };
                    let month_2 = if month.0 == 12 { 1 } else { month.0 + 1 };

                    Some((
                        NaiveDate::from_ymd(year.0, month.0, 1),
                        NaiveDate::from_ymd(year_2, month_2, 1).pred(),
                    ))
                }
                None => None,
            }
        }

        pub fn must_be_formatted(&self) -> bool {
            match self.format {
                Some(FormatParameter(true)) => true,
                _ => false,
            }
        }
    }

    trait Parameter<T> {
        fn from_request(request: &Request) -> Result<Option<T>, ParameterError> {
            match request.get_param(&Self::parameter_name()) {
                Some(year) => {
                    let parameter = Self::from_string(&year)?;
                    Ok(Some(parameter))
                }
                None => Ok(None),
            }
        }

        fn from_string(string: &str) -> Result<T, ParameterError>;

        fn parameter_name() -> String;
    }

    #[derive(Debug)]
    struct MonthParameter(u32);

    impl Parameter<Self> for MonthParameter {
        fn from_string(string: &str) -> Result<Self, ParameterError> {
            match string.parse::<u32>() {
                Ok(month) => {
                    Self::validate_value(month)?;
                    Ok(MonthParameter(month))
                }
                Err(_) => Err(ParameterError {
                    parameter_name: Self::parameter_name(),
                    kind: ParameterErrorKind::UnvalidType {
                        expected_type: String::from("number"),
                    },
                }),
            }
        }

        fn parameter_name() -> String {
            String::from("month")
        }
    }

    impl MonthParameter {
        fn validate_value(value: u32) -> Result<(), ParameterError> {
            if value > 12 || value < 1 {
                Err(ParameterError {
                    parameter_name: Self::parameter_name(),
                    kind: ParameterErrorKind::UnvalidValue {
                        parameter_value: value.to_string(),
                        reason: String::from("This value must be between 1 and 12"),
                    },
                })
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug)]
    struct YearParameter(i32);

    impl Parameter<Self> for YearParameter {
        fn from_string(string: &str) -> Result<Self, ParameterError> {
            match string.parse::<i32>() {
                Ok(year) => Ok(YearParameter(year)),
                Err(_) => Err(ParameterError {
                    parameter_name: Self::parameter_name(),
                    kind: ParameterErrorKind::UnvalidType {
                        expected_type: String::from("number"),
                    },
                }),
            }
        }

        fn parameter_name() -> String {
            String::from("year")
        }
    }

    #[derive(Debug)]
    struct FormatParameter(pub bool);

    impl Parameter<Self> for FormatParameter {
        fn from_string(string: &str) -> Result<Self, ParameterError> {
            match string.parse::<bool>() {
                Ok(boolean) => Ok(FormatParameter(boolean)),
                Err(_) => Err(ParameterError {
                    parameter_name: Self::parameter_name(),
                    kind: ParameterErrorKind::UnvalidType {
                        expected_type: String::from("boolean"),
                    },
                }),
            }
        }

        fn parameter_name() -> String {
            String::from("format")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::api::models::test_utils::RequestBuilder;

        #[test]
        fn test_it_inits_parameter_handler() {
            let request =
                RequestBuilder::get(String::from("/root_url?format=true&month=1&year=2019"));

            let param_handler = ParametersHandler::from_request(&request).unwrap();

            assert_eq!(param_handler.must_be_formatted(), true);
            assert_eq!(
                param_handler.date_interval(),
                Some((
                    NaiveDate::from_ymd(2019, 1, 1),
                    NaiveDate::from_ymd(2019, 1, 31)
                ))
            );
        }

        #[test]
        fn test_date_interval_edge_case() {
            let request = RequestBuilder::get(String::from("/root_url?month=12&year=2019"));

            let param_handler = ParametersHandler::from_request(&request).unwrap();

            assert_eq!(
                param_handler.date_interval(),
                Some((
                    NaiveDate::from_ymd(2019, 12, 1),
                    NaiveDate::from_ymd(2019, 12, 31)
                ))
            );
        }

        #[test]
        fn test_parameter_handler_returns_unvalid_type_error() {
            let request = RequestBuilder::get(String::from("/root_url?format=t"));

            let error = ParametersHandler::from_request(&request).unwrap_err();

            assert_eq!(error.parameter_name, String::from("format"));
            assert_eq!(
                error.kind,
                ParameterErrorKind::UnvalidType {
                    expected_type: String::from("boolean")
                }
            );
        }

        #[test]
        fn test_parameter_handler_returns_unvalid_value_error() {
            let month_value = 13;

            let request = RequestBuilder::get(format!("/root_url?month={}", month_value));

            let error = ParametersHandler::from_request(&request).unwrap_err();

            assert_eq!(error.parameter_name, String::from("month"));
            assert_eq!(
                error.kind,
                ParameterErrorKind::UnvalidValue {
                    parameter_value: month_value.to_string(),
                    reason: String::from("This value must be between 1 and 12")
                }
            );
        }

        #[test]
        fn test_parameter_handler_returns_unvalid_combination_error() {
            let request = RequestBuilder::get(String::from("/root_url?month=1"));

            let error = ParametersHandler::from_request(&request).unwrap_err();

            assert_eq!(error.parameter_name, String::from("month"));
            assert_eq!(
                error.kind,
                ParameterErrorKind::UnvalidCombination {
                    missing_parameters: vec![String::from("year")]
                }
            );

            let request = RequestBuilder::get(String::from("/root_url?year=1"));

            let error = ParametersHandler::from_request(&request).unwrap_err();

            assert_eq!(error.parameter_name, String::from("year"));
            assert_eq!(
                error.kind,
                ParameterErrorKind::UnvalidCombination {
                    missing_parameters: vec![String::from("month")]
                }
            );
        }
    }
}

pub mod formatter {
    use std::collections::HashMap;
    use uuid::Uuid;

    use super::super::models::Sanction;

    pub fn map_by_users(result: Vec<Sanction>) -> HashMap<Uuid, Vec<Sanction>> {
        let mut mapped_result = HashMap::new();

        result.into_iter().for_each(|sanction| {
            mapped_result
                .entry(sanction.user_id)
                .and_modify(|v: &mut Vec<Sanction>| v.push(sanction.clone()))
                .or_insert(vec![sanction]);
        });

        mapped_result
    }

    #[cfg(test)]
    mod tests {
        use chrono::naive::NaiveDate;

        use super::super::super::models::{SanctionData, SanctionInfo};
        use super::*;

        fn create_default_sanction(user_id: Uuid) -> Sanction {
            Sanction {
                id: Uuid::new_v4(),
                user_id,
                team_id: Uuid::new_v4(),
                sanction_info: SanctionInfo {
                    associated_rule: Uuid::new_v4(),
                    sanction_data: SanctionData::Basic,
                },
                created_at: NaiveDate::from_ymd(2019, 10, 5),
            }
        }

        #[test]
        fn test_mapping() {
            let user_1 = Uuid::new_v4();
            let user_2 = Uuid::new_v4();

            let result = vec![
                create_default_sanction(user_1),
                create_default_sanction(user_2),
                create_default_sanction(user_1),
                create_default_sanction(user_2),
                create_default_sanction(user_1),
            ];

            let mapped_users = map_by_users(result);

            assert_eq!(mapped_users.get(&user_1).unwrap().len(), 3);
            assert_eq!(mapped_users.get(&user_2).unwrap().len(), 2);
        }
    }
}
