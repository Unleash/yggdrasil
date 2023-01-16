use serde::Deserialize;
use std::{collections::HashMap};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InnerContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

impl Default for InnerContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            environment: None,
            current_time: None,
            app_name: None,
            remote_address: None,
            properties: Some(HashMap::new()),
        }
    }
}

#[derive(Debug)]
pub enum SdkError {
    StrategyEvaluationError,
    StrategyParseError,
}
