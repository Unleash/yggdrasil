use std::collections::HashMap;
use std::io::Cursor;
use std::net::IpAddr;
use std::num::ParseIntError;

use ipnet::IpNet;
use murmur3::murmur3_32;
use rand::Rng;

use crate::state::Strategy;
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
}

impl Strategy {
    pub fn is_enabled(&self, context: &InnerContext) -> bool {
        match self.name.as_str() {
            "userWithId" => {
                let params = UserWithIdParams::from(self.parameters.as_ref());

                match &context.user_id {
                    Some(user_id) => params.user_ids.contains(&user_id.parse::<u32>().expect("")),
                    None => false,
                }
            }
            "gradualRolloutUserId" => {
                let params = GradualRolloutUserIdParams::from(self.parameters.as_ref());
                let user_id = &context.user_id;

                match user_id {
                    Some(user_id) => match normalized_hash(&params.group_id, &user_id, 100) {
                        Ok(normalized_user_id) => params.percentage >= normalized_user_id,
                        Err(_) => false,
                    },
                    None => false,
                }
            }
            "gradualRolloutSessionId" => {
                let params = GradualRolloutSessionParams::from(self.parameters.as_ref());
                let session_id = &context.session_id;
                match session_id {
                    Some(session_id) => match normalized_hash(&params.group_id, &session_id, 100) {
                        Ok(normalized_user_id) => params.percentage >= normalized_user_id,
                        Err(_) => false,
                    },
                    None => false,
                }
            }
            "gradualRolloutRandom" => {
                let params = GradualRolloutRandomParams::from(self.parameters.as_ref());
                let random = rand::thread_rng().gen_range(0..100);
                params.percentage >= random
            }
            "remoteAddress" => {
                let params = RemoteAddressParams::from(self.parameters.as_ref());
                let remote_address = &context.remote_address;
                match remote_address {
                    Some(remote_address) => {
                        for ip in &params.ips {
                            if ip.contains(&remote_address.0) {
                                return true;
                            }
                        }
                        false
                    }

                    None => false,
                };
                false
            }
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
