mod utils;

use sdk_core::{EngineState, state::InnerContext};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Context {
    environment: String,
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new(environment: String) -> Context {
        Context {
            environment: environment,
        }
    }

    pub fn get(&self) -> String {
        self.environment.clone()
    }

    pub fn set(&mut self, val: String) {
        self.environment = val;
    }
}

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
    pub fn is_enabled(&self, name: String, context: &Context) -> bool {
        let context = context.into();
        self.engine_state.is_enabled(name, context)
    }

    pub fn take_state() {}
}

impl From<&Context> for InnerContext {
    fn from(context_wrapper: &Context) -> Self {
        InnerContext {
            user_id: None,
            session_id: None,
            remote_address: None,
            environment: None,
            app_name: None,
            properties: None,
        }
    }
}
