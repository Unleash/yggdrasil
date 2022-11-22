use std::collections::HashMap;
use std::io::Cursor;
use std::net::IpAddr;
use std::num::ParseIntError;

use ipnet::IpNet;
use murmur3::murmur3_32;
use rand::Rng;

use unleash_types::{Constraint, Operator, Strategy};
use crate::InnerContext;

pub fn normalized_hash(group: &str, identifier: &str, modulus: u32) -> std::io::Result<u32> {
    let mut reader = Cursor::new(format!("{}:{}", &group, &identifier));
    murmur3_32(&mut reader, 0).map(|hash_result| hash_result % modulus)
}

fn get_string_param(parameter_name: &str, parameters: &HashMap<String, String>) -> String {
    parameters
        .get(parameter_name)
        .map(|x| x.clone())
        .unwrap_or("".to_string())
}

fn get_int_param(parameter_name: &str, parameters: &HashMap<String, String>) -> u32 {
    parameters
        .get(parameter_name)
        .map(|x| x.parse::<u32>().unwrap_or(0))
        .unwrap_or(0)
}

pub(crate) struct UserWithIdParams {
    pub(crate) user_ids: Vec<u32>,
}

impl From<Option<&HashMap<String, String>>> for UserWithIdParams {
    fn from(parameters: Option<&HashMap<String, String>>) -> Self {
        let user_ids = match parameters {
            Some(parameters) => match parameters.get("userIds") {
                Some(user_ids) => user_ids
                    .split(",")
                    .map(|s| s.trim())
                    .map(|s| s.parse::<u32>())
                    .collect::<Result<Vec<u32>, ParseIntError>>()
                    .unwrap_or(vec![]),
                None => vec![],
            },
            None => vec![],
        };
        UserWithIdParams { user_ids }
    }
}

pub(crate) struct GradualRolloutUserIdParams {
    pub(crate) percentage: u32,
    pub(crate) group_id: String,
}

impl From<Option<&HashMap<String, String>>> for GradualRolloutUserIdParams {
    fn from(parameters: Option<&HashMap<String, String>>) -> Self {
        let props = match parameters {
            Some(parameters) => {
                let percentage = get_int_param("percentage", parameters);
                let group_id: String = get_string_param("groupId", parameters);

                (percentage, group_id)
            }
            None => (0, "".to_string()),
        };
        GradualRolloutUserIdParams {
            percentage: props.0,
            group_id: props.1,
        }
    }
}

pub(crate) struct GradualRolloutSessionParams {
    pub(crate) percentage: u32,
    pub(crate) group_id: String,
}

impl From<Option<&HashMap<String, String>>> for GradualRolloutSessionParams {
    fn from(parameters: Option<&HashMap<String, String>>) -> Self {
        let props = match parameters {
            Some(parameters) => {
                let percentage = get_int_param("percentage", parameters);
                let group_id: String = get_string_param("sessionId", parameters);

                (percentage, group_id)
            }
            None => (0, "".to_string()),
        };
        GradualRolloutSessionParams {
            percentage: props.0,
            group_id: props.1,
        }
    }
}

pub(crate) struct GradualRolloutRandomParams {
    pub(crate) percentage: u32,
}

impl From<Option<&HashMap<String, String>>> for GradualRolloutRandomParams {
    fn from(parameters: Option<&HashMap<String, String>>) -> Self {
        let percentage = match parameters {
            Some(parameters) => get_int_param("percentage", parameters),
            None => 0,
        };
        GradualRolloutRandomParams { percentage }
    }
}

pub(crate) struct RemoteAddressParams {
    pub(crate) ips: Vec<IpNet>,
}

fn parse_ip(ip: &str) -> Result<IpNet, std::net::AddrParseError> {
    ip.parse::<IpNet>()
        .or_else(|_| ip.parse::<IpAddr>().map(|addr| addr.into()))
}

impl From<Option<&HashMap<String, String>>> for RemoteAddressParams {
    fn from(parameters: Option<&HashMap<String, String>>) -> Self {
        let ips = match parameters {
            Some(parameters) => match parameters.get("IPs") {
                Some(ips) => {
                    let mut parsed_ips: Vec<IpNet> = Vec::new();
                    for ip_str in ips.split(',') {
                        let ip_parsed = parse_ip(ip_str.trim());
                        if let Ok(ip) = ip_parsed {
                            parsed_ips.push(ip)
                        }
                    }
                    parsed_ips
                }
                None => vec![],
            },
            None => vec![],
        };
        RemoteAddressParams { ips }
    }
}

pub(crate) struct FlexibleRolloutParams {
    pub(crate) rollout: u32,
    pub(crate) group_id: String,
    pub(crate) stickiness: String,
}

impl From<Option<&HashMap<String, String>>> for FlexibleRolloutParams {
    fn from(parameters: Option<&HashMap<String, String>>) -> Self {
        if let Some(parameters) = parameters {
            let rollout = get_int_param("rollout", parameters);
            let stickiness = get_string_param("stickiness", parameters);
            let group_id = get_string_param("groupId", parameters);
            return FlexibleRolloutParams {
                rollout,
                group_id,
                stickiness,
            };
        }
        FlexibleRolloutParams {
            rollout: 0,
            group_id: "".to_string(),
            stickiness: "".to_string(),
        }
    }
}

fn resolve_context_prop(name: &str, context: &InnerContext) -> Option<String> {
    match name {
        "userId" => return context.user_id.clone(),
        "sessionId" => return context.session_id.clone(),
        "remoteAddress" => {}
        "environment" => return context.environment.clone(),
        "appName" => return context.app_name.clone(),
        _ => {
            println!("Resolving an unknown context {} {:?}", name, context);
            if let Some(props) = &context.properties {
                return props.get(name).map(|x| x.to_string());
            }
        }
    }
    None
}

fn check_constraints(constraints: &Option<Vec<Constraint>>, context: &InnerContext) -> bool {
    if let Some(constraints) = constraints {
        constraints
            .iter()
            .all(|constraint| check_constraint(constraint, context))
    } else {
        true
    }
}

fn check_constraint(constraint: &Constraint, context: &InnerContext) -> bool {
    let context_value = resolve_context_prop(&constraint.context_name, context);
    let constraint_value = &constraint.values;

    match &constraint.operator {
        Operator::In => {
            if context_value.is_none() || constraint_value.is_none() {
                return false;
            };
            let context_value = context_value.unwrap();
            let constraint_value = constraint_value.as_ref().unwrap();

            constraint_value.contains(&context_value)
        }
        Operator::NotIn => {
            if context_value.is_none() || constraint_value.is_none() {
                println!(
                    "Not in early exit {:?} {:?}",
                    context_value, constraint_value
                );
                return true;
            };
            let context_value = context_value.unwrap();
            let constraint_value = constraint_value.as_ref().unwrap();
            !constraint_value.contains(&context_value)
        }
        _ => todo!()
    }
}
