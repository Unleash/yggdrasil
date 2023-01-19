use std::collections::HashMap;
use std::net::IpAddr;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pest_derive;

mod error;
mod sendable_closures;
pub mod state;
pub mod strategy_parsing;
pub mod strategy_upgrade;

use error::YggdrasilError;
use rand::Rng;
use serde::{de, Deserialize, Serialize};
use strategy_parsing::{compile_rule, normalized_hash, RuleFragment};
use strategy_upgrade::upgrade;
use unleash_types::client_features::{ClientFeatures, Payload, Segment, Variant};
use {state::EnrichedContext, state::InnerContext};

pub type CompiledState = HashMap<String, CompiledToggle>;

pub struct CompiledToggle {
    pub name: String,
    pub enabled: bool,
    pub compiled_strategy: RuleFragment,
    pub variants: Option<Vec<Variant>>,
}

fn build_segment_map(segments: &Option<Vec<Segment>>) -> HashMap<i32, Segment> {
    segments
        .as_ref()
        .map(|segments| {
            segments
                .iter()
                .map(|segment| (segment.id, segment.clone()))
                .collect::<HashMap<i32, Segment>>()
        })
        .unwrap_or_default()
}

pub fn compile_state(state: &ClientFeatures) -> HashMap<String, CompiledToggle> {
    let mut compiled_state = HashMap::new();
    let segment_map = build_segment_map(&state.segments);
    for toggle in &state.features {
        let rule = upgrade(&toggle.strategies.clone().unwrap_or_default(), &segment_map);
        compiled_state.insert(
            toggle.name.clone(),
            CompiledToggle {
                name: toggle.name.clone(),
                enabled: toggle.enabled,
                variants: toggle.variants.clone(),
                compiled_strategy: compile_rule(rule.as_str()).unwrap(),
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
    pub fn default() -> EngineState {
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
        if let Some(toggle) = toggle {
            let context = context.with_toggle_name(toggle.name.clone());
            toggle.enabled && (toggle.compiled_strategy)(&context)
        } else {
            false
        }
    }

    pub fn is_enabled(&self, name: String, context: &InnerContext) -> bool {
        match &self.compiled_state {
            Some(_) => {
                let toggle = self.get_toggle(name);
                self.enabled(toggle, context)
            }
            None => false,
        }
    }

    pub fn get_variant(&self, name: String, context: &InnerContext) -> VariantDef {
        let toggle = self.get_toggle(name);
        let enabled = self.enabled(toggle, context);
        match (toggle, enabled) {
            (Some(toggle), true) => match toggle.variants.as_ref() {
                Some(variants) => {
                    if let Some(found_override) = check_for_variant_override(variants, context) {
                        return VariantDef {
                            name: found_override.name,
                            payload: found_override.payload,
                            enabled: true,
                        };
                    }
                    let total_weight: u32 = variants.iter().map(|var| var.weight as u32).sum();

                    let target = match get_custom_stickiness(variants, context) {
                        Ok(stickiness) => stickiness
                            .or_else(|| {
                                context
                                    .user_id
                                    .clone()
                                    .or_else(|| context.session_id.clone())
                            })
                            .map(|stickiness| {
                                normalized_hash(&toggle.name, &stickiness, total_weight)
                                    .unwrap()
                            })
                            .unwrap_or_else(|| rand::thread_rng().gen_range(0..99) as u32),
                        Err(_) => return VariantDef::default(),
                    };

                    let mut total_weight = 0;
                    for variant in variants {
                        total_weight += variant.weight as u32;
                        if total_weight > target {
                            return VariantDef {
                                name: variant.name.clone(),
                                payload: variant.payload.clone(),
                                enabled: true,
                            };
                        }
                    }
                    VariantDef::default()
                }
                None => VariantDef::default(),
            },
            _ => VariantDef::default(), //either the toggle doesn't exist or it evaluates to false
        }
    }

    pub fn take_state(&mut self, toggles: ClientFeatures) {
        self.compiled_state = Some(compile_state(&toggles));
    }
}

fn get_custom_stickiness(
    variants: &[Variant],
    context: &InnerContext,
) -> Result<Option<String>, YggdrasilError> {
    let custom_stickiness = variants
        .get(0)
        .and_then(|variant| variant.stickiness.clone());

    if let Some(custom_stickiness) = custom_stickiness {
        let stickiness = match custom_stickiness.as_str() {
            "userId" => context.user_id.clone(),
            "sessionId" => context.session_id.clone(),
            "environment" => context.environment.clone(),
            "appName" => context.app_name.clone(),
            "remoteAddress" => context.remote_address.clone(),
            _ => context
                .properties
                .as_ref()
                .and_then(|props| props.get(&custom_stickiness))
                .cloned(),
        };
        if stickiness.is_none() {
            Err(YggdrasilError::StickinessExpectedButNotFound)
        } else {
            Ok(stickiness)
        }
    } else {
        Ok(None)
    }
}

fn check_for_variant_override(variants: &Vec<Variant>, context: &InnerContext) -> Option<Variant> {
    for variant in variants {
        if let Some(overrides) = &variant.overrides {
            for o in overrides {
                #[allow(clippy::single_match)]
                //Clippy is technically correct here but this match statement needs more arms to be feature complete
                match o.context_name.as_ref() as &str {
                    "userId" => {
                        if let Some(val) = &context.user_id {
                            if o.values.contains(val) {
                                return Some(variant.clone());
                            }
                        }
                    } //TODO: This needs to handle all the variant override cases... also... why aren't the spec tests failing this?
                    _ => {}
                }
            }
        }
    }
    None
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct VariantDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
    pub enabled: bool,
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
    use std::{collections::HashMap, fs};
    use test_case::test_case;
    use unleash_types::client_features::{ClientFeatures, Variant, WeightType};

    use crate::{CompiledToggle, EngineState, InnerContext, VariantDef};

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
    #[test_case("12-custom-stickiness.json"; "Custom stickiness")]
    #[test_case("13-constraint-operators.json"; "Advanced constraints")]
    #[test_case("14-constraint-semver-operators.json"; "Semver constraints")]
    #[test_case("15-global-constraints.json"; "Segments")]
    fn run_client_spec(spec_name: &str) {
        let spec = load_spec(spec_name);
        let mut engine = EngineState::default();
        engine.take_state(spec.state);

        if let Some(mut tests) = spec.tests {
            while let Some(test_case) = tests.pop() {
                println!(
                    "Executing test {:?} with toggle name{:?} against context{:?}",
                    &test_case.description, &test_case.toggle_name, &test_case.context
                );
                let expected = test_case.expected_result;
                let actual = engine.is_enabled(test_case.toggle_name, &test_case.context);
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
                let actual = engine.get_variant(test_case.toggle_name, &test_case.context);
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    pub fn stickiness_for_variants_falls_back_to_random_if_no_context_property_present() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "cool-animals".to_string(),
            CompiledToggle {
                name: "cool-animals".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: Some(vec![Variant {
                    name: "iguana".into(),
                    weight: 100,
                    weight_type: Some(WeightType::Fix),
                    stickiness: Some("userId".into()),
                    payload: None,
                    overrides: None,
                }]),
            },
        );
        let state = EngineState {
            compiled_state: Some(compiled_state),
        };
        let context = InnerContext::default();

        state.get_variant("cool-animals".into(), &context);
    }
}
