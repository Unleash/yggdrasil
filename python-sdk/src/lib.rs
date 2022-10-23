use std::collections::HashMap;

use pyo3::prelude::*;
use sdk_core::{EngineState, InnerContext};

#[pyclass]
struct UnleashEngine {
    engine_state: EngineState,
}

#[pyclass]
pub struct Context {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
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
        properties: Option<HashMap<String, String>>,
    ) -> Context {
        Context {
            user_id,
            session_id,
            remote_address,
            properties,
        }
    }
}

impl From<&Context> for InnerContext {
    fn from(context_wrapper: &Context) -> Self {
        InnerContext {
            user_id: context_wrapper.user_id.clone(),
            session_id: context_wrapper.session_id.clone(),
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

    pub fn is_enabled(&self, name: String, context: &Context) -> bool {
        let context = context.into();
        self.engine_state.is_enabled(name, context)
    }
}

#[pymodule]
fn python_sdk(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<UnleashEngine>()?;
    m.add_class::<Context>()?;
    Ok(())
}
