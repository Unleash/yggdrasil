use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::AtomicU32;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pest_derive;

mod sendable_closures;
pub mod state;
pub mod strategy_parsing;
pub mod strategy_upgrade;

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{de, Deserialize, Serialize};
use state::EnrichedContext;
use std::sync::atomic::Ordering;
use strategy_parsing::{compile_rule, normalized_hash, RuleFragment};
use strategy_upgrade::upgrade;
pub use unleash_types::client_features::Context;
use unleash_types::client_features::{ClientFeatures, Override, Payload, Segment, Variant};
use unleash_types::client_metrics::{MetricBucket, ToggleStats};

pub type CompiledState = HashMap<String, CompiledToggle>;

pub const SUPPORTED_SPEC_VERSION: &str = "4.2.2";

pub struct CompiledToggle {
    pub name: String,
    pub enabled: bool,
    pub compiled_strategy: RuleFragment,
    pub variants: Vec<CompiledVariant>,
    pub yes: AtomicU32,
    pub no: AtomicU32,
    pub default_variant: AtomicU32,
    pub impression_data: bool,
    pub project: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledVariant {
    pub name: String,
    pub weight: i32,
    pub stickiness: Option<String>,
    pub payload: Option<Payload>,
    pub overrides: Option<Vec<Override>>,
    pub count: AtomicU32,
}

impl Default for CompiledToggle {
    fn default() -> Self {
        Self {
            name: Default::default(),
            enabled: Default::default(),
            compiled_strategy: Box::new(|_| true),
            variants: Default::default(),
            yes: Default::default(),
            no: Default::default(),
            default_variant: Default::default(),
            impression_data: false,
            project: "default".to_string(),
        }
    }
}

impl From<&Variant> for CompiledVariant {
    fn from(value: &Variant) -> Self {
        CompiledVariant {
            name: value.name.clone(),
            weight: value.weight,
            stickiness: value.stickiness.clone(),
            payload: value.payload.clone(),
            overrides: value.overrides.clone(),
            count: AtomicU32::new(0),
        }
    }
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
                variants: compile_variants(&toggle.variants),
                compiled_strategy: compile_rule(rule.as_str()).unwrap(),
                yes: AtomicU32::new(0),
                no: AtomicU32::new(0),
                default_variant: AtomicU32::new(0),
                impression_data: toggle.impression_data.unwrap_or_default(),
                project: toggle.project.clone().unwrap_or("default".to_string()),
            },
        );
    }

    compiled_state
}

fn compile_variants(variants: &Option<Vec<Variant>>) -> Vec<CompiledVariant> {
    if let Some(variants) = variants {
        variants.iter().map(CompiledVariant::from).collect()
    } else {
        vec![]
    }
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
    pub started: DateTime<Utc>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            compiled_state: Default::default(),
            started: Utc::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedToggle {
    pub enabled: bool,
    pub impression_data: bool,
    pub project: String,
    pub variant: VariantDef,
}

impl EngineState {
    fn get_toggle(&self, name: &str) -> Option<&CompiledToggle> {
        match &self.compiled_state {
            Some(state) => state.get(name),
            None => None,
        }
    }

    fn harvest_metrics(&mut self) -> Option<MetricBucket> {
        if let Some(state) = &self.compiled_state {
            let metrics: HashMap<String, ToggleStats> = state
                .values()
                .map(|toggle| {
                    let yes = toggle.yes.swap(0, Ordering::Relaxed);
                    let no = toggle.no.swap(0, Ordering::Relaxed);

                    let mut variants: HashMap<String, u32> = toggle
                        .variants
                        .iter()
                        .map(|variant| {
                            (
                                variant.name.clone(),
                                variant.count.swap(0, Ordering::Relaxed),
                            )
                        })
                        .collect();

                    variants.insert(
                        VariantDef::default().name,
                        toggle.default_variant.swap(0, Ordering::Relaxed),
                    );

                    (toggle.name.clone(), ToggleStats { yes, no, variants })
                })
                .collect();
            let timestamp = Utc::now();
            let bucket = MetricBucket {
                toggles: metrics,
                start: self.started,
                stop: timestamp,
            };
            self.started = timestamp;
            Some(bucket)
        } else {
            None
        }
    }

    pub fn get_metrics(&mut self) -> Option<MetricBucket> {
        self.harvest_metrics()
    }

    fn enabled(&self, toggle: &CompiledToggle, context: &Context) -> bool {
        let context = EnrichedContext::from(context.clone(), toggle.name.clone());
        let is_enabled = toggle.enabled && (toggle.compiled_strategy)(&context);
        if is_enabled {
            toggle.yes.fetch_add(1, Ordering::Relaxed);
        } else {
            toggle.no.fetch_add(1, Ordering::Relaxed);
        }
        is_enabled
    }

    pub fn resolve_all(&self, context: &Context) -> Option<HashMap<String, ResolvedToggle>> {
        self.compiled_state.as_ref().map(|state| {
            state
                .iter()
                .map(|(name, toggle)| {
                    (
                        name.clone(),
                        ResolvedToggle {
                            enabled: self.enabled(toggle, context),
                            impression_data: toggle.impression_data,
                            variant: self.get_variant(name, context),
                            project: toggle.project.clone(),
                        },
                    )
                })
                .collect()
        })
    }

    pub fn resolve(&self, name: &str, context: &Context) -> Option<ResolvedToggle> {
        self.compiled_state
            .as_ref()
            .map(|state| {
                state.get(name).map(|compiled_toggle| ResolvedToggle {
                    enabled: self.enabled(compiled_toggle, context),
                    impression_data: compiled_toggle.impression_data,
                    variant: self.get_variant(name, context),
                    project: compiled_toggle.project.clone(),
                })
            })
            .flatten()
    }

    pub fn is_enabled(&self, name: &str, context: &Context) -> bool {
        self.compiled_state
            .as_ref()
            .and_then(|_| {
                self.get_toggle(name)
                    .map(|toggle| self.enabled(toggle, context))
            })
            .unwrap_or_default()
    }

    fn resolve_variant<'a>(
        &self,
        toggle: &CompiledToggle,
        variants: &'a Vec<CompiledVariant>,
        context: &Context,
    ) -> Option<&'a CompiledVariant> {
        if variants.is_empty() {
            return None;
        }
        if let Some(found_override) = check_for_variant_override(variants, context) {
            return Some(found_override);
        }
        let total_weight: u32 = variants.iter().map(|var| var.weight as u32).sum();

        let stickiness = variants
            .get(0)
            .and_then(|variant| variant.stickiness.clone());

        let target = get_seed(stickiness, context)
            .map(|seed| normalized_hash(&toggle.name, &seed, total_weight).unwrap())
            .unwrap_or_else(|| rand::thread_rng().gen_range(0..total_weight));

        let mut total_weight = 0;
        for variant in variants {
            total_weight += variant.weight as u32;
            if total_weight > target {
                return Some(variant);
            }
        }
        None
    }

    pub fn get_variant(&self, name: &str, context: &Context) -> VariantDef {
        let toggle = self.get_toggle(name);

        toggle
            .map(|toggle| {
                let variant = self.resolve_variant(toggle, &toggle.variants, context);
                let enabled = self.enabled(toggle, context);

                if !enabled {
                    toggle.default_variant.fetch_add(1, Ordering::Relaxed);
                    return VariantDef::default();
                };

                if let Some(variant) = variant {
                    variant.count.fetch_add(1, Ordering::Relaxed);
                    VariantDef {
                        name: variant.name.clone(),
                        payload: variant.payload.clone(),
                        enabled,
                    }
                } else {
                    toggle.default_variant.fetch_add(1, Ordering::Relaxed);
                    VariantDef::default()
                }
            })
            .unwrap_or_default()
    }

    pub fn take_state(&mut self, toggles: ClientFeatures) -> Option<MetricBucket> {
        let metrics = self.harvest_metrics();
        self.compiled_state = Some(compile_state(&toggles));
        metrics
    }
}

fn get_seed(stickiness: Option<String>, context: &Context) -> Option<String> {
    match stickiness.as_deref() {
        Some("default") | None => context
            .user_id
            .clone()
            .or_else(|| context.session_id.clone()),
        Some(custom_stickiness) => match custom_stickiness {
            "userId" => context.user_id.clone(),
            "sessionId" => context.session_id.clone(),
            "environment" => context.environment.clone(),
            "appName" => context.app_name.clone(),
            "remoteAddress" => context.remote_address.clone(),
            _ => context
                .properties
                .as_ref()
                .and_then(|props| props.get(custom_stickiness))
                .cloned(),
        },
    }
}

fn lookup_override_context<'a>(
    variant_override: &Override,
    context: &'a Context,
) -> Option<&'a String> {
    match variant_override.context_name.as_ref() as &str {
        "userId" => context.user_id.as_ref(),
        "sessionId" => context.session_id.as_ref(),
        "environment" => context.environment.as_ref(),
        "appName" => context.app_name.as_ref(),
        "currentTime" => context.current_time.as_ref(),
        "remoteAddress" => context.remote_address.as_ref(),
        _ => context
            .properties
            .as_ref()
            .and_then(|props| props.get(&variant_override.context_name)),
    }
}

fn check_for_variant_override<'a>(
    variants: &'a Vec<CompiledVariant>,
    context: &Context,
) -> Option<&'a CompiledVariant> {
    for variant in variants {
        if let Some(overrides) = &variant.overrides {
            for o in overrides {
                let context_property = lookup_override_context(o, context);
                if let Some(context_property) = context_property {
                    if o.values.contains(context_property) {
                        return Some(variant);
                    }
                }
            }
        }
    }
    None
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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
    use std::{collections::HashMap, fs, sync::atomic::AtomicU32};
    use test_case::test_case;
    use unleash_types::client_features::{ClientFeatures, Override};

    use crate::{
        check_for_variant_override, get_seed, CompiledToggle, CompiledVariant, Context,
        EngineState, VariantDef,
    };

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
        pub(crate) context: Context,
        pub(crate) toggle_name: String,
        pub(crate) expected_result: bool,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct VariantTestCase {
        pub(crate) description: String,
        pub(crate) context: Context,
        pub(crate) toggle_name: String,
        pub(crate) expected_result: VariantDef,
    }

    fn load_spec(spec_name: &str) -> TestSuite {
        let spec_path = format!("{SPEC_FOLDER}/{spec_name}");
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
                let actual = engine.is_enabled(&test_case.toggle_name, &test_case.context);
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
                let actual = engine.get_variant(&test_case.toggle_name, &test_case.context);
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
                variants: vec![CompiledVariant {
                    name: "iguana".into(),
                    weight: 100,
                    stickiness: Some("userId".into()),
                    payload: None,
                    overrides: None,
                    count: AtomicU32::new(0),
                }],
                ..CompiledToggle::default()
            },
        );
        let state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };
        let context = Context::default();

        state.get_variant("cool-animals".into(), &context);
    }

    #[test]
    pub fn get_variant_resolves_to_default_variant_when_variants_is_empty() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "test".to_string(),
            CompiledToggle {
                name: "test".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                ..CompiledToggle::default()
            },
        );
        let state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };
        let context = Context::default();

        assert_eq!(
            state.get_variant("test".into(), &context),
            VariantDef::default()
        );
    }

    #[test]
    pub fn checking_a_toggle_also_increments_metrics() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|context| context.user_id == Some("7".into())),
                variants: vec![],
                ..CompiledToggle::default()
            },
        );

        let mut state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };
        let context_with_user_id_of_7 = Context {
            user_id: Some("7".into()),
            ..Context::default()
        };

        let blank_context = Context::default();

        state.is_enabled("some-toggle".into(), &context_with_user_id_of_7);
        state.is_enabled("some-toggle".into(), &context_with_user_id_of_7);

        //No user id, no enabled state, this should increment the "no" metric
        state.is_enabled("some-toggle".into(), &blank_context);

        let metrics = state.get_metrics().unwrap();
        assert_eq!(metrics.toggles.get("some-toggle").unwrap().yes, 2);
        assert_eq!(metrics.toggles.get("some-toggle").unwrap().no, 1);
    }

    #[test]
    pub fn checking_a_variant_also_increments_metrics_including_toggle_metrics() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|context| context.user_id == Some("7".into())),
                variants: vec![CompiledVariant {
                    name: "test-variant".into(),
                    weight: 100,
                    stickiness: None,
                    payload: None,
                    overrides: None,
                    count: AtomicU32::new(0),
                }],
                ..CompiledToggle::default()
            },
        );

        let mut state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };

        let blank_context = Context::default();
        let context_with_user_id_of_7 = Context {
            user_id: Some("7".into()),
            ..Context::default()
        };

        state.get_variant("some-toggle".into(), &blank_context);
        state.get_variant("some-toggle".into(), &context_with_user_id_of_7);

        let metrics = state.get_metrics().unwrap();
        let toggle_metric = metrics.toggles.get("some-toggle").unwrap();

        let variant_metric = metrics
            .toggles
            .get("some-toggle")
            .unwrap()
            .variants
            .get("test-variant")
            .unwrap();

        let disabled_variant_metric = metrics
            .toggles
            .get("some-toggle")
            .unwrap()
            .variants
            .get("disabled")
            .unwrap();

        assert_eq!(variant_metric, &1);
        assert_eq!(disabled_variant_metric, &1);
        assert_eq!(toggle_metric.yes, 1);

        assert_eq!(toggle_metric.no, 1);
    }

    #[test]
    pub fn take_state_yields_unhandled_metrics() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                ..CompiledToggle::default()
            },
        );

        let mut state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };

        let blank_context = Context::default();

        state.is_enabled("some-toggle".into(), &blank_context);

        let metrics = state
            .take_state(ClientFeatures {
                version: 2,
                features: vec![],
                segments: None,
                query: None,
            })
            .unwrap();
        let toggle_metric = metrics.toggles.get("some-toggle").unwrap();

        assert_eq!(toggle_metric.yes, 1);
    }

    #[test_case(Some("default"), Some("sessionId"), Some("userId"), Some("userId"); "should return userId for default stickiness")]
    #[test_case(None, Some("sessionId"), Some("userId"), Some("userId"); "should use default stickiness if none is defined")]
    #[test_case(Some("userId"), Some("sessionId"), None, None; "should use custom userId stickiness")]
    #[test_case(Some("sessionId"), Some("sessionId"), Some("userId"), Some("sessionId"); "should use custom sessionId stickiness")]
    #[test_case(Some("random"), Some("sessionId"), Some("userId"), None; "should return no seed for random stickiness")]
    #[test_case(Some("customId"), Some("sessionId"), Some("userId"), Some("customId"); "should use custom stickiness")]
    pub fn test_get_seed(
        stickiness: Option<&str>,
        session_id: Option<&str>,
        user_id: Option<&str>,
        expected: Option<&str>,
    ) {
        let mut context = Context {
            session_id: session_id.map(String::from),
            user_id: user_id.map(String::from),
            ..Default::default()
        };

        context
            .properties
            .as_mut()
            .unwrap()
            .insert("customId".to_string(), "customId".to_string());

        assert_eq!(
            get_seed(stickiness.map(String::from), &context),
            expected.map(String::from)
        );
    }

    #[test]
    fn resolves_all_toggles() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![CompiledVariant {
                    name: "test-variant".into(),
                    weight: 100,
                    stickiness: None,
                    payload: None,
                    overrides: None,
                    count: AtomicU32::new(0),
                }],
                ..CompiledToggle::default()
            },
        );

        compiled_state.insert(
            "some-toggle-other".to_string(),
            CompiledToggle {
                name: "some-toggle-other".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                ..CompiledToggle::default()
            },
        );

        let state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };

        let blank_context = Context::default();
        let toggles = state.resolve_all(&blank_context).unwrap();
        let resolved_variant = toggles.get("some-toggle").unwrap().variant.name.clone();
        let unresolved_variant = toggles
            .get("some-toggle-other")
            .unwrap()
            .variant
            .name
            .clone();

        assert_eq!(resolved_variant, "test-variant".to_string());
        assert_eq!(unresolved_variant, "disabled".to_string());
        assert_eq!(toggles.len(), 2);
    }

    // The client spec doesn't actually enforce anything except userId for variant overrides, so this is
    // getting its own test set until the client spec can take over that responsibility
    #[test_case("userId", "7", &["7"], true; "Basic example")]
    #[test_case("userId", "7", &["7", "8"], true; "With multiple values")]
    #[test_case("userId", "7", &["2", "9"], false; "Expected not to match against missing property")]
    #[test_case("sessionId", "7", &["2", "7"], true; "Resolves against session id")]
    #[test_case("remoteAddress", "7", &["2", "7"], true; "Resolves against remote address")]
    #[test_case("environment", "7", &["2", "7"], true; "Resolves against environment")]
    #[test_case("currentTime", "7", &["2", "7"], true; "Resolves against currentTime")]
    #[test_case("appName", "7", &["2", "7"], true; "Resolves against app name")]
    #[test_case("someArbContext", "7", &["2", "7"], true; "Resolves against arbitrary context field")]
    fn variant_override_resolves_with_arbitrary_context_fields(
        context_name: &str,
        context_values: &str,
        override_values: &[&str],
        expected: bool,
    ) {
        let context = to_context(context_name, context_values);

        let variants = vec![CompiledVariant {
            name: "test".into(),
            weight: 1000,
            stickiness: None,
            payload: None,
            overrides: Some(vec![Override {
                context_name: context_name.into(),
                values: override_values.iter().map(|s| s.to_string()).collect(),
            }]),
            count: 0.into(),
        }];
        let result = check_for_variant_override(&variants, &context);
        assert_eq!(result.is_some(), expected);
    }

    fn to_context(name: &str, value: &str) -> Context {
        match name {
            "userId" => Context {
                user_id: Some(value.into()),
                ..Context::default()
            },
            "sessionId" => Context {
                session_id: Some(value.into()),
                ..Context::default()
            },
            "environment" => Context {
                environment: Some(value.into()),
                ..Context::default()
            },
            "appName" => Context {
                app_name: Some(value.into()),
                ..Context::default()
            },
            "currentTime" => Context {
                current_time: Some(value.into()),
                ..Context::default()
            },
            "remoteAddress" => Context {
                remote_address: Some(value.into()),
                ..Context::default()
            },
            _ => {
                let mut context = Context::default();
                let mut props = HashMap::new();
                props.insert(name.into(), value.into());
                context.properties = Some(props);
                context
            }
        }
    }
}
