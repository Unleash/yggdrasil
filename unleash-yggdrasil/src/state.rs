use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InnerContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "remove_null_properties")]
    pub properties: Option<HashMap<String, String>>,
}

pub struct EnrichedContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
    pub(crate) toggle_name: String,
}

impl InnerContext {
    pub fn with_toggle_name(&self, toggle_name: String) -> EnrichedContext {
        EnrichedContext {
            user_id: self.user_id.clone(),
            session_id: self.session_id.clone(),
            environment: self.environment.clone(),
            app_name: self.app_name.clone(),
            current_time: self.current_time.clone(),
            remote_address: self.remote_address.clone(),
            properties: self.properties.clone(),
            toggle_name,
        }
    }
}

// I know this looks silly but it's also important for two reasons:
// The first is that the client spec tests have a test case that has a context defined like:
// {
//   "properties": {
//      "someValue": null
//    }
// }
// Passing around an Option<HashMap<String, Option<String>>> is awful and unnecessary, we should scrub ingested data
// before trying to execute our logic, so we scrub out those empty values instead, they do nothing useful for us.
// The second reason is that we can't shield the Rust code from consumers using the FFI layers and potentially doing
// exactly the same thing in languages that allow it. They should not do that. But if they do we have enough information
// to understand the intent of the executed code clearly and there's no reason to fail
fn remove_null_properties<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let props: Option<HashMap<String, Option<String>>> = Option::deserialize(deserializer)?;
    Ok(props.map(|props| {
        props
            .into_iter()
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap()))
            .collect()
    }))
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
