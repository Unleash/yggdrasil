use std::{collections::HashMap, hash::Hash};

pub struct InnerContext {
    pub environment: String,
}

pub fn is_enabled(name: String, context: InnerContext) -> bool {
    let matches_context = match context {
        InnerContext { ref environment } if environment == "dev" => true,
        _ => false,
    };
    name == "AlwaysOn" && matches_context
}

pub struct EngineState {
    toggles: HashMap<String, bool>,
}

impl EngineState {
    pub fn new() -> EngineState {
        EngineState {
            toggles: HashMap::new(),
        }
    }

    pub fn is_enabled(&self, name: String, context: InnerContext) -> bool {
        let matches_context = match context {
            InnerContext { ref environment } if environment == "dev" => true,
            _ => false,
        };
        name == "AlwaysOn" && matches_context
    }

    pub fn take_state(&mut self, toggles: HashMap<String, bool>) {
        self.toggles = toggles;
    }

    pub fn get_variant() -> bool {
        true
    }
}
