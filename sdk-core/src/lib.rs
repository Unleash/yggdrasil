use std::collections::HashMap;
use std::net::IpAddr;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pest_derive;

pub mod state;
pub mod strategy;
pub mod strategy_parsing;
pub mod strategy_upgrade;

use serde::{de, Deserialize};
use state::InnerContext;
use strategy_parsing::compile_rule;
use strategy_upgrade::upgrade;
use unleash_types::client_features::{ClientFeature, ClientFeatures, Payload, Variant};

pub type CompiledState = HashMap<String, CompiledToggle>;

pub struct CompiledToggle {
    pub enabled: bool,
    pub compiled_strategy: Box<dyn Fn(&InnerContext) -> bool>,
    // pub variants: Option<Vec<VariantDef>>,
}

pub fn compile_state(state: &ClientFeatures) -> HashMap<String, CompiledToggle> {
    let mut compiled_state = HashMap::new();
    for toggle in &state.features {
        let rule = upgrade(&toggle.strategies.clone().unwrap_or(vec![]));
        compiled_state.insert(
            toggle.name.clone(),
            CompiledToggle {
                enabled: toggle.enabled,
                compiled_strategy: compile_rule(rule.as_str()).unwrap(),
                // variants: toggle.variants.clone(),
            },
        );
    }

    compiled_state
}

#[derive(Debug)]
pub struct IPAddress(pub IpAddr);

impl<'de> de::Deserialize<'de> for IPAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            s.parse::<IpAddr>()
                .map_err(de::Error::custom)
                .map(IPAddress)
        } else {
            unimplemented!();
        }
    }
}

pub struct EngineState {
    compiled_state: Option<CompiledState>,
}

impl EngineState {
    pub fn new() -> EngineState {
        EngineState {
            compiled_state: None,
        }
    }

    fn get_toggle(&self, name: String) -> Option<&CompiledToggle> {
        match &self.compiled_state {
            Some(state) => state.get(&name),
            None => None,
        }
    }

    fn enabled(&self, toggle: Option<&CompiledToggle>, context: &InnerContext) -> bool {
        toggle
            .map(|toggle| toggle.enabled && (toggle.compiled_strategy)(&context))
            .unwrap_or(false)
    }

    pub fn is_enabled(&self, name: String, context: InnerContext) -> bool {
        match &self.compiled_state {
            Some(_) => {
                let toggle = self.get_toggle(name);
                self.enabled(toggle, &context)
            }
            None => false,
        }
    }

    pub fn get_variant(&self, name: String, context: InnerContext) -> VariantDef {
        VariantDef::default()
    }

    pub fn take_state(&mut self, toggles: ClientFeatures) {
        self.compiled_state = Some(compile_state(&toggles));
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct VariantDef {
    name: String,
    payload: Option<Payload>,
    enabled: bool,
}

impl Default for VariantDef {
    fn default() -> Self {
        Self {
            name: "disabled".into(),
            payload: None,
            enabled: false,
        }
    }
}

#[cfg(test)]
mod test {
    use serde::Deserialize;
    use std::fs;
    use test_case::test_case;
    use unleash_types::client_features::ClientFeatures;
    use unleash_types::client_features::Variant;

    use crate::{EngineState, InnerContext, VariantDef};

    const SPEC_FOLDER: &str = "../client-specification/specifications";

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct TestSuite {
        pub(crate) state: ClientFeatures,
        pub(crate) tests: Option<Vec<TestCase>>,
        pub(crate) variant_tests: Option<Vec<VariantTestCase>>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct TestCase {
        pub(crate) description: String,
        pub(crate) context: InnerContext,
        pub(crate) toggle_name: String,
        pub(crate) expected_result: bool,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct VariantTestCase {
        pub(crate) description: String,
        pub(crate) context: InnerContext,
        pub(crate) toggle_name: String,
        pub(crate) expected_result: VariantDef,
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
    #[test_case("05-gradual-rollout-random-strategy.json"; "Gradual Rollout random")]
    #[test_case("06-remote-address-strategy.json"; "Remote address")]
    #[test_case("07-multiple-strategies.json"; "Multiple strategies")]
    #[test_case("08-variants.json"; "Variants")]
    #[test_case("09-strategy-constraints.json"; "Strategy constraints")]
    #[test_case("10-flexible-rollout-strategy.json"; "Flexible rollout strategy")]
    #[test_case("11-strategy-constraints-edge-cases.json"; "Strategy constraint edge cases")]
    // #[test_case("12-custom-stickiness.json"; "Custom stickiness")]
    fn run_client_spec(spec_name: &str) {
        let spec = load_spec(spec_name);
        let mut engine = EngineState::new();
        engine.take_state(spec.state);

        if let Some(mut tests) = spec.tests {
            while let Some(test_case) = tests.pop() {
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
        };
        if let Some(mut variant_tests) = spec.variant_tests {
            while let Some(test_case) = variant_tests.pop() {
                println!(
                    "Executing test {:?} with toggle name{:?} against context{:?}",
                    &test_case.description, &test_case.toggle_name, &test_case.context
                );
                let expected = test_case.expected_result;
                let actual = engine.get_variant(test_case.toggle_name, test_case.context);
                assert_eq!(expected, actual);
            }
        }
    }
}
