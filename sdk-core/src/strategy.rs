use std::collections::HashMap;
use std::io::Cursor;
use std::num::ParseIntError;

use murmur3::murmur3_32;

use crate::state::Strategy;
use crate::InnerContext;

pub fn normalized_hash(group: &str, identifier: &str, modulus: u32) -> std::io::Result<u32> {
    let mut reader = Cursor::new(format!("{}:{}", &group, &identifier));
    murmur3_32(&mut reader, 0).map(|hash_result| hash_result % modulus)
}

pub(crate) struct RolloutParams {
    pub(crate) percentage: u32,
    pub(crate) group_id: String,
}

trait ToRolloutParameters {
    fn to_strategy_parameters(&self, parameter_name: &str) -> RolloutParams;
}

impl ToRolloutParameters for Option<&HashMap<String, String>> {
    fn to_strategy_parameters(&self, parameter_name: &str) -> RolloutParams {
        match self {
            Some(parameters) => {
                let percentage = parameters
                    .get("percentage")
                    .map(|x| x.parse::<u32>().unwrap_or(0))
                    .unwrap_or(0);
                let group_id: String = parameters
                    .get(parameter_name)
                    .map(|x| x.clone())
                    .unwrap_or("".to_string());
                RolloutParams {
                    percentage,
                    group_id,
                }
            }
            None => RolloutParams {
                percentage: 0,
                group_id: "".to_string(),
            },
        }
    }
}

impl Strategy {
    pub fn is_enabled(&self, context: &InnerContext) -> bool {
        match self.name.as_str() {
            "userWithId" => match &self.parameters {
                Some(parameters) => {
                    let strategy_user_ids = match parameters.get("userIds") {
                        Some(user_ids) => user_ids
                            .split(",")
                            .map(|s| s.trim())
                            .map(|s| s.parse::<u32>())
                            .collect::<Result<Vec<u32>, ParseIntError>>()
                            .unwrap_or(vec![]),
                        None => vec![],
                    };

                    match &context.user_id {
                        Some(user_id) => {
                            strategy_user_ids.contains(&user_id.parse::<u32>().expect(""))
                        }
                        None => false,
                    }
                }
                None => false,
            },
            "gradualRolloutUserId" => {
                if self.parameters.is_some() {
                    let params: RolloutParams =
                        self.parameters.as_ref().to_strategy_parameters("groupId");
                    let user_id = &context.user_id;

                    match user_id {
                        Some(user_id) => match normalized_hash(&params.group_id, &user_id, 100) {
                            Ok(normalized_user_id) => params.percentage >= normalized_user_id,
                            Err(_) => false,
                        },
                        None => false,
                    }
                } else {
                    false
                }
            }
            "gradualRolloutSessionId" => {
                if self.parameters.is_some() {
                    let params: RolloutParams =
                        self.parameters.as_ref().to_strategy_parameters("sessionId");
                    let session_id = &context.session_id;
                    match session_id {
                        Some(session_id) => match normalized_hash(&params.group_id, &session_id, 100) {
                            Ok(normalized_user_id) => params.percentage >= normalized_user_id,
                            Err(_) => false,
                        },
                        None => false,
                    }
                } else {
                    false
                }
            },
            _ => {
                println!(
                    "Unknown strategy type: {:?}, defaulting to disabled",
                    self.name
                );
                true
            }
        }
    }
}
