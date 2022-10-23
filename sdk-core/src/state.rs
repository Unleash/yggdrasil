use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub parameters: Option<HashMap<String, String>>,
    constraints: Option<Vec<Constraint>>,
    segments: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Operator {
    In,
    NotIn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Constraint {
    context_name: String,
    values: Vec<String>,
    value: String,
    operator: Operator,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
    id: i32,
    constraints: Vec<Constraint>,
}

