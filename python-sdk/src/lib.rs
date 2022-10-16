use pyo3::prelude::*;
use sdk_core::{EngineState, InnerContext};

#[pyclass]
struct UnleashEngine {
    engine_state: EngineState,
}

#[pyclass]
pub struct Context {
    environment: String,
}

#[pymethods]
impl Context {
    #[new]
    pub fn new(environment: String) -> Context {
        Context { environment }
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

impl From<&Context> for InnerContext {
    fn from(context_wrapper: &Context) -> Self {
        InnerContext {
            environment: context_wrapper.environment.clone(),
        }
    }
}

#[pymodule]
fn python_sdk(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<UnleashEngine>()?;
    m.add_class::<Context>()?;
    Ok(())
}
