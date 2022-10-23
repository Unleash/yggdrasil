use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use state::State;
pub mod state;
pub mod strategy;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InnerContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

pub struct EngineState {
    toggles: Option<State>,
}

impl EngineState {
    pub fn new() -> EngineState {
        EngineState { toggles: None }
    }

    pub fn is_enabled(&self, name: String, context: InnerContext) -> bool {
        match &self.toggles {
            Some(toggles) => {
                let toggle = toggles.features.iter().find(|toggle| toggle.name == name);
                match toggle {
                    Some(toggle) => {
                        if !toggle.enabled {
                            return false;
                        }

                        let strategy_enabled = if toggle.strategies.len() > 0 {
                            toggle
                                .strategies
                                .iter()
                                .any(|strategy| strategy.is_enabled(&context))
                        } else {
                            true
                        };

                        strategy_enabled && toggle.enabled
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    pub fn take_state(&mut self, toggles: State) {
        self.toggles = Some(toggles);
    }

    pub fn get_variant() -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use std::fs;
    use test_case::test_case;

    use crate::{state::State, EngineState, InnerContext};

    const SPEC_FOLDER: &str = "../client-specification/specifications";

    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct TestSuite {
        pub(crate) name: String,
        pub(crate) state: State,
        pub(crate) tests: Vec<TestCase>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct TestCase {
        pub(crate) description: String,
        pub(crate) context: InnerContext,
        pub(crate) toggle_name: String,
        pub(crate) expected_result: bool,
    }

    fn load_spec(spec_name: &str) -> TestSuite {
        let spec_path = format!("{}/{}", SPEC_FOLDER, spec_name);
        let spec_data =
            fs::read_to_string(spec_path).expect("Should have been able to read the file");
        serde_json::from_str(&spec_data).expect("Failed to parse client spec")
    }

    #[test_case("01-simple-examples.json"; "Basic client spec")]
    #[test_case("02-user-with-id-strategy.json"; "User Id with strategy")]
    #[test_case("03-gradual-rollout-user-id-strategy.json"; "Gradual Rollout user id strategy")]
    #[test_case("04-gradual-rollout-session-id-strategy.json"; "Gradual Rollout session-id strategy")]
    fn run_client_spec(spec_name: &str) {
        let mut spec = load_spec(spec_name);
        // println!("Loaded testcase {:?}", &spec);
        let mut engine = EngineState::new();
        engine.take_state(spec.state);
        while let Some(test_case) = spec.tests.pop() {
            println!(
                "Executing test {:?} with toggle name{:?} against context{:?}",
                &test_case.description, &test_case.toggle_name, &test_case.context
            );
            let expected = test_case.expected_result;
            let actual = engine.is_enabled(test_case.toggle_name, test_case.context);
            if expected != actual {
                panic!(
                    "Test case: '{}' does not match. Expected: {}, actual: {}",
                    test_case.description, expected, actual
                );
            }
        }
    }
}
