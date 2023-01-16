use std::{collections::HashMap, hash::Hash};

use pyo3::prelude::*;
use unleash_yggdrasil::{state::InnerContext, EngineState, IPAddress, VariantDef};
use serde::{de, Deserialize};

#[pyclass]
struct UnleashEngine {
    engine_state: EngineState,
}

#[pyclass]
pub struct Context {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

#[pymethods]
impl Context {
    #[new]
    pub fn new(
        user_id: Option<String>,
        session_id: Option<String>,
        remote_address: Option<String>,
        environment: Option<String>,
        app_name: Option<String>,
        current_time: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> Context {
        Context {
            user_id,
            session_id,
            remote_address,
            environment,
            app_name,
            properties,
            current_time,
        }
    }
}

#[pyclass]
struct Variant {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub payload: Option<HashMap<String, String>>,
    #[pyo3(get, set)]
    pub enabled: bool,
}

impl From<VariantDef> for Variant {
    fn from(variant: VariantDef) -> Self {
        let mut payload = HashMap::new();
        if let Some(payload_content) = variant.payload {
            payload.insert("type".into(), payload_content.payload_type.clone());
            payload.insert("value".into(), payload_content.value.clone());
        }
        Variant {
            name: variant.name,
            payload: Some(payload),
            enabled: variant.enabled,
        }
    }
}

impl From<&Context> for InnerContext {
    fn from(context_wrapper: &Context) -> Self {
        InnerContext {
            user_id: context_wrapper.user_id.clone(),
            session_id: context_wrapper.session_id.clone(),
            environment: context_wrapper.environment.clone(),
            app_name: context_wrapper.app_name.clone(),
            current_time: context_wrapper.current_time.clone(),
            remote_address: context_wrapper.remote_address.clone(),
            properties: context_wrapper.properties.clone(),
        }
    }
}

#[pymethods]
impl UnleashEngine {
    #[new]
    pub fn new() -> UnleashEngine {
        UnleashEngine {
            engine_state: EngineState::new(),
        }
    }

    pub fn take_state(&mut self, state: String) {
        let toggles = serde_json::from_str(&state).expect("Failed to parse client spec");
        self.engine_state.take_state(toggles)
    }

    pub fn is_enabled(&self, name: String, context: &Context) -> bool {
        let context = context.into();
        self.engine_state.is_enabled(name, &context)
    }

    pub fn get_variant(&self, name: String, context: &Context) -> Variant {
        let context = context.into();
        self.engine_state.get_variant(name, &context).into()
    }
}

#[pymodule]
fn python_sdk(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<UnleashEngine>()?;
    m.add_class::<Context>()?;
    Ok(())
}
