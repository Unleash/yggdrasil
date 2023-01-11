mod utils;

use sdk_core::{state::InnerContext, EngineState};
use serde_wasm_bindgen;
use unleash_types::client_features::ClientFeatures;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct UnleashEngine {
    engine_state: EngineState,
}

#[wasm_bindgen]
impl UnleashEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> UnleashEngine {
        UnleashEngine {
            engine_state: EngineState::new(),
        }
    }

    #[wasm_bindgen(method, js_name = isEnabled)]
    pub fn is_enabled(&self, name: String, context: &JsValue) -> bool {
        let context: InnerContext = serde_wasm_bindgen::from_value(context.clone()).unwrap();
        self.engine_state.is_enabled(name, &context)
    }

    #[wasm_bindgen(method, js_name = getVariant)]
    pub fn get_variant(&self, name: String, context: &JsValue) -> JsValue {
        let context: InnerContext = serde_wasm_bindgen::from_value(context.clone()).unwrap();
        let variant = self.engine_state.get_variant(name, &context);
        serde_wasm_bindgen::to_value(&variant).expect("Failed to materialize a variant")
    }

    #[wasm_bindgen(method, js_name = takeState)]
    pub fn take_state(&mut self, state: &JsValue) {
        let state: ClientFeatures = serde_wasm_bindgen::from_value(state.clone()).unwrap();
        self.engine_state.take_state(state);
    }
}
