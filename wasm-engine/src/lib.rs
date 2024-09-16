#![allow(non_snake_case)]

use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

use unleash_yggdrasil::{Context, EngineState};
use unleash_types::client_features::ClientFeatures;

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
    pub fn take_state(&mut self, state: JsValue) -> Result<(), JsValue> {
        let state: ClientFeatures = from_value(state)
            .map_err(|e| JsValue::from_str(&format!("Error parsing state: {}", e)))?;

          if let Some(warnings) = self.engine.take_state(state) {
            let warnings_json = serde_wasm_bindgen::to_value(&warnings)
                .map_err(|e| JsValue::from_str(&format!("Error serializing warnings: {}", e)))?;
            return Err(JsValue::from_str(&format!("Warnings: {:?}", warnings_json)));
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = isEnabled)]
    pub fn is_enabled(&self, toggle_name: &str, context: JsValue) -> Result<bool, JsValue> {
      let context: Context = serde_wasm_bindgen::from_value(context)
          .map_err(|e| JsValue::from_str(&format!("Invalid context object: {}", e)))?;
      Ok(self.engine.is_enabled(toggle_name, &context, &None))
  }

  #[wasm_bindgen(js_name = checkVariant)]
    pub fn check_variant(&self, toggle_name: &str, context: JsValue) -> Result<JsValue, JsValue> {
      let context: Context = serde_wasm_bindgen::from_value(context)
          .map_err(|e| JsValue::from_str(&format!("Invalid context object: {}", e)))?;
      let variant = self
          .engine
          .check_variant(toggle_name, &context, &None)
          .unwrap_or_default();

      serde_wasm_bindgen::to_value(&variant)
          .map_err(|e| JsValue::from_str(&format!("Error serializing variant: {}", e)))
    }

    #[wasm_bindgen(js_name = getMetrics)]
    pub fn get_metrics(&mut self) -> Result<JsValue, JsValue> {
      if let Some(metrics) = self.engine.get_metrics() {
          serde_wasm_bindgen::to_value(&metrics)
              .map_err(|e| JsValue::from_str(&format!("Failed to serialize metrics: {}", e)))
      } else {
          Ok(JsValue::from_str("{}"))
      }
    }

    #[wasm_bindgen(js_name = countToggle)]
    pub fn count_toggle(&self, toggle_name: &str, enabled: bool) -> Result<(), JsValue> {
      self.engine.count_toggle(toggle_name, enabled);
      Ok(())
    }

    #[wasm_bindgen(js_name = countVariant)]
    pub fn count_variant(&self, toggle_name: &str, variant_name: &str) -> Result<(), JsValue> {
      self.engine.count_variant(toggle_name, variant_name);
      Ok(())
    }
}