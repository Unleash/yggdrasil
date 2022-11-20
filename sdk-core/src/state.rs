use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub segments: Option<Vec<i32>>,
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



