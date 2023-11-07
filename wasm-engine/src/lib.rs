use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;

use unleash_yggdrasil::{
    state::EnrichedContext, strategy_parsing::compile_rule, Context as YggdrasilContext,
};
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct Context {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "environment")]
    pub environment: Option<String>,
    #[serde(rename = "appName")]
    pub app_name: Option<String>,
    #[serde(rename = "currentTime")]
    pub current_time: Option<String>,
    #[serde(rename = "remoteAddress")]
    pub remote_address: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "properties")]
    pub properties: Option<HashMap<String, String>>,
}

impl Context {
    fn to_context(self) -> EnrichedContext {
        let yggdrasil_context = YggdrasilContext {
            user_id: self.user_id.clone(),
            session_id: self.session_id.clone(),
            environment: self.environment.clone(),
            app_name: self.app_name.clone(),
            current_time: self.current_time.clone(),
            remote_address: self.remote_address.clone(),
            properties: self.properties.clone(),
        };

        EnrichedContext::from(yggdrasil_context, self.group_id.unwrap_or("".into()))
    }
}

#[wasm_bindgen]
pub fn evaluate(dsl_fragment: &str, context: JsValue) -> Result<bool, JsValue> {
    let internal_context: Context = from_value(context)?;
    let context = internal_context.to_context();
    let rule = compile_rule(dsl_fragment).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    Ok((rule)(&context))
}
