use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::strategy_parser::compile_rule;

pub type CompiledState = HashMap<String, CompiledToggle>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InnerContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

impl Default for InnerContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            environment: None,
            app_name: None,
            remote_address: None,
            properties: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    version: i8,
    pub features: Vec<Toggle>,
    pub segments: Option<Vec<Segment>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Toggle {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub strategies: Vec<Strategy>,
    #[serde(default = "empty_vec")]
    pub variants: Vec<VariantDef>,
}

fn empty_vec() -> Vec<VariantDef> {
    vec![]
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub payload: Option<VariantPayload>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub weight: u8,
    pub payload: VariantPayload,
    pub overrides: Option<Vec<Override>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    pub context_name: String,
    pub values: Vec<String>,
}

impl Default for Variant {
    fn default() -> Self {
        Variant {
            name: "disabled".to_string(),
            payload: None,
            enabled: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VariantPayload {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub variant_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub parameters: Option<HashMap<String, String>>,
    pub constraints: Option<Vec<Constraint>>,
    segments: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operator {
    In,
    NotIn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Constraint {
    pub context_name: String,
    pub values: Option<Vec<String>>,
    pub value: Option<String>,
    pub operator: Operator,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
    id: i32,
    constraints: Vec<Constraint>,
}

pub struct CompiledToggle {
    pub enabled: bool,
    pub compiled_strategy: Box<dyn Fn(&InnerContext) -> bool>,
    pub variants: Vec<VariantDef>,
}

pub fn compile_state(state: &State) -> HashMap<String, CompiledToggle> {
    let mut compiled_state = HashMap::new();
    for toggle in &state.features {
        let rule = upgrade(&toggle.strategies);
        compiled_state.insert(
            toggle.name.clone(),
            CompiledToggle {
                enabled: toggle.enabled,
                compiled_strategy: compile_rule(rule.as_str()).unwrap(),
                variants: toggle.variants.clone(),
            },
        );
    }

    compiled_state
}

pub fn upgrade(strategies: &Vec<Strategy>) -> String {
    if strategies.is_empty() {
        return "true".into();
    }
    strategies
        .iter()
        .map(|x| upgrade_strategy(x))
        .collect::<Vec<String>>()
        .join(" or ")
}

fn upgrade_strategy(strategy: &Strategy) -> String {
    let strategy_rule = match strategy.name.as_str() {
        "default" => "true".into(),
        "userWithId" => upgrade_user_id_strategy(strategy),
        "gradualRolloutUserId" => upgrade_user_id_rollout_strategy(strategy),
        "gradualRolloutSessionId" => upgrade_session_id_rollout_strategy(strategy),
        "gradualRolloutRandom" => upgrade_random(strategy),
        "remoteAddress" => upgrade_remote_address(strategy),
        _ => "true".into(),
    };

    let constraints = upgrade_constraints(&strategy.constraints);

    format!("({})", strategy_rule)
}

trait PropResolver {
    fn get_param(&self, key: &str) -> Option<&String>;
}

impl PropResolver for Strategy {
    fn get_param(&self, key: &str) -> Option<&String> {
        self.parameters
            .as_ref()
            .map(|params| params.get(key))
            .unwrap_or(None)
    }
}

fn upgrade_user_id_strategy(strategy: &Strategy) -> String {
    match strategy.get_param("userIds") {
        Some(user_ids) => format!("user_id in [{}]", user_ids),
        None => "".into(),
    }
}

fn upgrade_remote_address(strategy: &Strategy) -> String {
    match strategy.get_param("IPs") {
        Some(addresses) => {
            let ips = addresses
                .split(",")
                .collect::<Vec<&str>>()
                .iter()
                .map(|x| x.trim())
                .map(|x| format!("\"{}\"", x))
                .collect::<Vec<String>>()
                .join(", ");
            format!("remote_address in [{}]", ips)
        }
        None => "".into(),
    }
}

fn upgrade_session_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = strategy.get_param("percentage");
    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!(
                "{}% sticky on session_id with group_id of \"{}\"",
                percentage, group_id
            )
        }
        _ => "".into(),
    }
}

fn upgrade_user_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = strategy.get_param("percentage");
    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!(
                "{}% sticky on user_id with group_id of \"{}\"",
                percentage, group_id
            )
        }
        _ => "".into(),
    }
}

fn upgrade_random(strategy: &Strategy) -> String {
    match strategy.get_param("percentage") {
        Some(percent) => format!("random() < {}", percent),
        None => "".into(),
    }
}

fn upgrade_constraints(constraints: &Option<Vec<Constraint>>) {
    if constraints.is_none() {
        return;
    }
    let constraints = constraints.as_ref().unwrap();
    constraints.iter().map(|x| upgrade_constraint(x));
}

fn upgrade_constraint(constraint: &Constraint) -> String {
    let context_name = constraint.context_name.clone();
    let values = constraint.values.clone();
    let op = constraint.operator.clone();
    "".into()
}
