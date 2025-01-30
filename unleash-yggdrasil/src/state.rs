use std::collections::HashMap;
use unleash_types::client_features::Context;

pub struct EnrichedContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
    pub external_results: Option<HashMap<String, bool>>,
    pub(crate) toggle_name: String,
}

impl EnrichedContext {
    pub fn from(
        context: Context,
        toggle_name: String,
        external_results: Option<HashMap<String, bool>>,
    ) -> Self {
        EnrichedContext {
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            environment: context.environment.clone(),
            app_name: context.app_name.clone(),
            current_time: context.current_time.clone(),
            remote_address: context.remote_address.clone(),
            properties: context.properties,
            external_results,
            toggle_name,
        }
    }
}

#[derive(Debug)]
pub enum SdkError {
    StrategyEvaluationError,
    StrategyParseError(String),
}
