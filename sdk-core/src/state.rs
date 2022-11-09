use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::strategy_parser::compile_rule;
use crate::IPAddress;

pub type CompiledState = HashMap<String, CompiledToggle>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InnerContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub remote_address: Option<IPAddress>,
    pub properties: Option<HashMap<String, String>>,
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
    match strategy.name.as_str() {
        "default" => "true".into(),
        "userWithId" => upgrade_user_id_strategy(strategy),
        _ => "true".into(),
    }
    .into()
}

fn upgrade_user_id_strategy(strategy: &Strategy) -> String {
    match &strategy.parameters {
        Some(parameters) => match parameters.get("userIds") {
            Some(user_ids) => format!("user_id in [{}]", user_ids),
            None => "".into(),
        },
        None => "".into(),
    }
}

fn upgrade_constraint(constraint: &Constraint) {}
