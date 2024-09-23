#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use unleash_yggdrasil::{Context, EngineState, ExtendedVariantDef};
use unleash_types::client_features::ClientFeatures;
use std::collections::HashMap;

type CustomStrategyResults = HashMap<String, bool>;

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub status_code: ResponseCode,
    pub value: Option<T>,
    pub error_message: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Ok = 1,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Engine {
    engine: EngineState,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine::default()
    }

    #[wasm_bindgen(js_name = takeState)]
    pub fn take_state(&mut self, state: JsValue) -> JsValue {
        let state: ClientFeatures = match from_value(state) {
            Ok(state) => state,
            Err(e) => {
                let response = Response::<()> {
                    status_code: ResponseCode::Error,
                    value: None,
                    error_message: Some(format!("Error parsing state: {}", e)),
                };
                return serde_wasm_bindgen::to_value(&response).unwrap();
            }
        };

        let warnings = self.engine.take_state(state);

        let response = if warnings.is_some() {
            Response {
                status_code: ResponseCode::Error,
                value: None,
                error_message: Some("Partial update with warnings".to_string()),
            }
        } else {
            Response {
                status_code: ResponseCode::Ok,
                value: Some(()),
                error_message: None,
            }
        };

        serde_wasm_bindgen::to_value(&response).unwrap()
    }

    #[wasm_bindgen(js_name = checkEnabled)]
    pub fn check_enabled(&self, toggle_name: &str, context: JsValue, custom_strategy_results: JsValue) -> JsValue {
        let context: Context = match from_value(context) {
            Ok(ctx) => ctx,
            Err(e) => {
                let response = Response::<bool> {
                    status_code: ResponseCode::Error,
                    value: None,
                    error_message: Some(format!("Invalid context object: {}", e)),
                };
                return serde_wasm_bindgen::to_value(&response).unwrap();
            }
        };

        let custom_strategy_results: Option<CustomStrategyResults> = if custom_strategy_results.is_null() || custom_strategy_results.is_undefined() {
            None
        } else {
            match from_value(custom_strategy_results) {
                Ok(results) => Some(results),
                Err(e) => {
                    let response = Response::<bool> {
                        status_code: ResponseCode::Error,
                        value: None,
                        error_message: Some(format!("Invalid strategy results: {}", e)),
                    };
                    return serde_wasm_bindgen::to_value(&response).unwrap();
                }
            }
        };

        let check_enabled = self.engine.check_enabled(toggle_name, &context, &custom_strategy_results);

        let response = Response {
            status_code: ResponseCode::Ok,
            value: check_enabled,
            error_message: None,
        };

        serde_wasm_bindgen::to_value(&response).unwrap()
    }

    #[wasm_bindgen(js_name = checkVariant)]
    pub fn check_variant(&self, toggle_name: &str, context: JsValue, custom_strategy_results: JsValue) -> JsValue {
        let context: Context = match from_value(context) {
            Ok(ctx) => ctx,
            Err(e) => {
                let response = Response::<ExtendedVariantDef> {
                    status_code: ResponseCode::Error,
                    value: None,
                    error_message: Some(format!("Invalid context object: {}", e)),
                };
                return serde_wasm_bindgen::to_value(&response).unwrap();
            }
        };

        let custom_strategy_results: Option<CustomStrategyResults> = if custom_strategy_results.is_null() || custom_strategy_results.is_undefined() {
            None
        } else {
            match from_value(custom_strategy_results) {
                Ok(results) => Some(results),
                Err(e) => {
                    let response = Response::<ExtendedVariantDef> {
                        status_code: ResponseCode::Error,
                        value: None,
                        error_message: Some(format!("Invalid strategy results: {}", e)),
                    };
                    return serde_wasm_bindgen::to_value(&response).unwrap();
                }
            }
        };

        let base_variant = self.engine.check_variant(toggle_name, &context, &custom_strategy_results);
        let toggle_enabled = self.engine.is_enabled(toggle_name, &context, &custom_strategy_results);

        let enriched_variant = base_variant.map(|variant| variant.to_enriched_response(toggle_enabled));

        let response = Response {
            status_code: ResponseCode::Ok,
            value: enriched_variant,
            error_message: None,
        };

        serde_wasm_bindgen::to_value(&response).unwrap()
    }

    #[wasm_bindgen(js_name = getMetrics)]
    pub fn get_metrics(&mut self) -> JsValue {
        if let Some(metrics) = self.engine.get_metrics() {
            serde_wasm_bindgen::to_value(&metrics).unwrap()
        } else {
            let response = Response::<()> {
                status_code: ResponseCode::NotFound,
                value: None,
                error_message: Some("No metrics available".to_string()),
            };
            serde_wasm_bindgen::to_value(&response).unwrap()
        }
    }

    #[wasm_bindgen(js_name = countToggle)]
    pub fn count_toggle(&self, toggle_name: &str, enabled: bool) -> JsValue {
        self.engine.count_toggle(toggle_name, enabled);

        let response = Response {
            status_code: ResponseCode::Ok,
            value: Some(()),
            error_message: None,
        };

        serde_wasm_bindgen::to_value(&response).unwrap()
    }

    #[wasm_bindgen(js_name = countVariant)]
    pub fn count_variant(&self, toggle_name: &str, variant_name: &str) -> JsValue {
        self.engine.count_variant(toggle_name, variant_name);

        let response = Response {
            status_code: ResponseCode::Ok,
            value: Some(()),
            error_message: None,
        };

        serde_wasm_bindgen::to_value(&response).unwrap()
    }
}
