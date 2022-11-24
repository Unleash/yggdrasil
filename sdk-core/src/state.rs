use serde::{Deserialize};
use std::{collections::HashMap, hash::Hash};

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
            properties: Some(HashMap::new()),
        }
    }
}
