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
use dashmap::DashMap;
use rand::Rng;
use serde::{de, Deserialize, Serialize};
use state::EnrichedContext;
use std::sync::atomic::Ordering;
use strategy_parsing::{compile_rule, normalized_hash, RuleFragment};
use strategy_upgrade::{build_variant_rules, upgrade};
pub use unleash_types::client_features::Context;
use unleash_types::client_features::{
    ClientFeature, ClientFeatures, ClientFeaturesDelta, FeatureDependency, Override, Payload,
    Segment, Variant,
};
use unleash_types::client_metrics::{MetricBucket, ToggleStats};

pub type CompiledState = HashMap<String, CompiledToggle>;

pub const SUPPORTED_SPEC_VERSION: &str = "5.1.9";
const VARIANT_NORMALIZATION_SEED: u32 = 86028157;
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CompiledToggle {
    pub name: String,
    pub enabled: bool,
    pub feature_type: Option<String>,
    pub compiled_strategy: RuleFragment,
    pub compiled_variant_strategy: Option<Vec<(RuleFragment, Vec<CompiledVariant>, String)>>,
    pub variants: Vec<CompiledVariant>,
    pub impression_data: bool,
    pub project: String,
    pub dependencies: Vec<FeatureDependency>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleDefinition {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub feature_type: Option<String>,
    pub project: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledVariant {
    pub name: String,
    pub weight: i32,
    pub stickiness: Option<String>,
    pub payload: Option<Payload>,
    pub overrides: Option<Vec<Override>>,
}

impl Default for CompiledToggle {
    fn default() -> Self {
        Self {
            name: Default::default(),
            enabled: Default::default(),
            feature_type: None,
            compiled_strategy: Box::new(|_| true),
            compiled_variant_strategy: None,
            variants: Default::default(),
            impression_data: false,
            project: "default".to_string(),
            dependencies: Default::default(),
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

fn compile_variant_rule(
    toggle: &ClientFeature,
    segment_map: &HashMap<i32, Segment>,
) -> Option<Vec<(RuleFragment, Vec<CompiledVariant>, String)>> {
    let variant_rules: Option<Vec<(RuleFragment, Vec<CompiledVariant>, String)>> =
        build_variant_rules(
            &toggle.strategies.clone().unwrap_or_default(),
            segment_map,
            &toggle.name,
        )
        .iter()
        .map(|(rule_string, strategy_variants, stickiness, group_id)| {
            let compiled_rule: Option<RuleFragment> = compile_rule(rule_string).ok();
            compiled_rule.map(|rule| {
                (
                    rule,
                    strategy_variants
                        .iter()
                        .map(|strategy_variant| CompiledVariant {
                            name: strategy_variant.name.clone(),
                            weight: strategy_variant.weight,
                            stickiness: Some(stickiness.clone()),
                            payload: strategy_variant.payload.clone(),
                            overrides: None,
                        })
                        .collect(),
                    group_id.clone(),
                )
            })
        })
        .collect();

    variant_rules
}

#[derive(Debug, Serialize)]
pub struct EvalWarning {
    pub toggle_name: String,
    pub message: String,
}

pub fn compile_state(
    state: &ClientFeatures,
) -> (HashMap<String, CompiledToggle>, Vec<EvalWarning>) {
    let mut compiled_state = HashMap::new();
    let segment_map = build_segment_map(&state.segments);
    let mut warnings = vec![];

    for toggle in &state.features {
        compiled_state.insert(
            toggle.name.clone(),
            compile(toggle, &segment_map, &mut warnings),
        );
    }

    (compiled_state, warnings)
}

pub fn compile(
    toggle: &ClientFeature,
    segment_map: &HashMap<i32, Segment>,
    warnings: &mut Vec<EvalWarning>,
) -> CompiledToggle {
    let rule = upgrade(&toggle.strategies.clone().unwrap_or_default(), segment_map);
    let variant_rule = compile_variant_rule(&toggle, segment_map);
    CompiledToggle {
        name: toggle.name.clone(),
        enabled: toggle.enabled,
        feature_type: toggle.feature_type.clone(),
        compiled_variant_strategy: variant_rule,
        variants: compile_variants(&toggle.variants),
        compiled_strategy: compile_rule(rule.as_str()).unwrap_or_else(|e| {
            warnings.push(EvalWarning {
                toggle_name: toggle.name.clone(),
                message: format!("Failed to compile toggle, this will always be off {e:?}"),
            });
            Box::new(|_| false)
        }),

        impression_data: toggle.impression_data.unwrap_or_default(),
        project: toggle.project.clone().unwrap_or("default".to_string()),
        dependencies: toggle.dependencies.clone().unwrap_or_default(),
    }
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

struct Metric {
    yes: AtomicU32,
    no: AtomicU32,
    variants: DashMap<String, AtomicU32>,
}

pub struct EngineState {
    compiled_state: Option<CompiledState>,
    toggle_metrics: DashMap<String, Metric>,
    pub started: DateTime<Utc>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            compiled_state: Default::default(),
            toggle_metrics: Default::default(),
            started: Utc::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedToggle {
    pub enabled: bool,
    pub impression_data: bool,
    pub project: String,
    pub variant: ExtendedVariantDef,
}

impl EngineState {
    pub fn take_delta(&mut self, delta: &ClientFeaturesDelta) -> Option<Vec<EvalWarning>> {
        let mut current_state = self.compiled_state.take().unwrap_or_default();
        let segment_map = build_segment_map(&delta.segments);
        let mut warnings: Vec<EvalWarning> = vec![];
        for removed in delta.removed.clone() {
            current_state.remove(&removed);
        }
        for update in delta.updated.clone() {
            let updated_state = compile(&update, &segment_map, &mut warnings);
            current_state.insert(update.name.clone(), updated_state);
        }
        self.compiled_state = Some(current_state);
        if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        }
    }
    fn get_toggle(&self, name: &str) -> Option<&CompiledToggle> {
        self.compiled_state
            .as_ref()
            .and_then(|state| state.get(name))
    }

    pub fn count_toggle(&self, name: &str, enabled: bool) {
        self.toggle_metrics
            .entry(name.to_owned())
            .and_modify(|metric| {
                if enabled {
                    metric.yes.fetch_add(1, Ordering::Relaxed);
                } else {
                    metric.no.fetch_add(1, Ordering::Relaxed);
                }
            })
            .or_insert_with(|| {
                let yes = AtomicU32::new(0);
                let no = AtomicU32::new(0);

                if enabled {
                    yes.fetch_add(1, Ordering::Relaxed);
                } else {
                    no.fetch_add(1, Ordering::Relaxed);
                }

                Metric {
                    yes,
                    no,
                    variants: DashMap::default(),
                }
            });
    }

    pub fn count_variant(&self, toggle_name: &str, variant: &str) {
        self.toggle_metrics
            .entry(toggle_name.to_owned())
            .and_modify(|metric| {
                metric
                    .variants
                    .entry(variant.to_string())
                    .and_modify(|v| {
                        v.fetch_add(1, Ordering::Relaxed);
                    })
                    .or_insert(AtomicU32::new(1));
            })
            .or_insert_with(|| {
                let variants = DashMap::default();
                variants.insert(variant.to_string(), AtomicU32::new(1));
                Metric {
                    yes: AtomicU32::new(0),
                    no: AtomicU32::new(0),
                    variants,
                }
            });
    }

    pub fn get_metrics(&mut self) -> Option<MetricBucket> {
        let metrics: HashMap<String, ToggleStats> = self
            .toggle_metrics
            .iter()
            .filter_map(|metric_pair| {
                let toggle_name = metric_pair.key();
                let metric_info = metric_pair.value();

                let yes = metric_info.yes.swap(0, Ordering::Relaxed);
                let no = metric_info.no.swap(0, Ordering::Relaxed);

                let variants: HashMap<String, u32> = metric_info
                    .variants
                    .iter()
                    .filter_map(|pair| {
                        let variant_metric = pair.value();
                        let variant_name = pair.key();

                        let variant_count = variant_metric.swap(0, Ordering::Relaxed);
                        if variant_count > 0 {
                            Some((variant_name.clone(), variant_count))
                        } else {
                            None
                        }
                    })
                    .collect();

                if yes > 0 || no > 0 || !variants.is_empty() {
                    Some((toggle_name.clone(), ToggleStats { yes, no, variants }))
                } else {
                    None
                }
            })
            .collect();

        if !metrics.is_empty() {
            Some(MetricBucket {
                toggles: metrics,
                start: self.started,
                stop: Utc::now(),
            })
        } else {
            None
        }
    }

    fn is_parent_dependency_satisfied(&self, toggle: &CompiledToggle, context: &Context) -> bool {
        toggle.dependencies.iter().all(|parent_dependency| {
            let Some(compiled_parent) = self.get_toggle(&parent_dependency.feature) else {
                return false;
            };

            if !compiled_parent.dependencies.is_empty() {
                return false;
            }

            let parent_enabled = self.enabled(compiled_parent, context, &None); //parent toggles explicitly don't support custom strategies
            let expected_parent_enabled_state = parent_dependency.enabled.unwrap_or(true);
            let parent_variant = self.check_variant_by_toggle(compiled_parent, context);

            let is_variant_dependency_satisfied = {
                if let (Some(expected_variants), Some(actual_variant)) =
                    (&parent_dependency.variants, parent_variant)
                {
                    expected_variants.is_empty() || expected_variants.contains(&actual_variant.name)
                } else {
                    true
                }
            };

            if !is_variant_dependency_satisfied {
                return false;
            }

            parent_enabled == expected_parent_enabled_state
        })
    }

    fn enabled(
        &self,
        toggle: &CompiledToggle,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> bool {
        let enriched_context = EnrichedContext::from(
            context.clone(),
            toggle.name.clone(),
            external_values.clone(),
        );
        toggle.enabled
            && self.is_parent_dependency_satisfied(toggle, context)
            && (toggle.compiled_strategy)(&enriched_context)
    }

    pub fn resolve_all(
        &self,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> Option<HashMap<String, ResolvedToggle>> {
        self.compiled_state.as_ref().map(|state| {
            state
                .iter()
                .map(|(name, toggle)| {
                    let enabled = self.enabled(toggle, context, external_values);
                    (
                        name.clone(),
                        ResolvedToggle {
                            enabled,
                            impression_data: toggle.impression_data,
                            variant: self.get_variant(name, context, external_values),
                            project: toggle.project.clone(),
                        },
                    )
                })
                .collect()
        })
    }

    pub fn resolve(
        &self,
        name: &str,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> Option<ResolvedToggle> {
        self.compiled_state.as_ref().and_then(|state| {
            state.get(name).map(|compiled_toggle| ResolvedToggle {
                enabled: self.enabled(compiled_toggle, context, external_values),
                impression_data: compiled_toggle.impression_data,
                variant: self.get_variant(name, context, external_values),
                project: compiled_toggle.project.clone(),
            })
        })
    }

    pub fn list_known_toggles(&self) -> Vec<ToggleDefinition> {
        self.compiled_state
            .as_ref()
            .map(|state| {
                state
                    .iter()
                    .map(|pair| {
                        let toggle = pair.1;
                        ToggleDefinition {
                            feature_type: toggle.feature_type.clone(),
                            name: toggle.name.clone(),
                            project: toggle.project.clone(),
                        }
                    })
                    .collect::<Vec<ToggleDefinition>>()
            })
            .unwrap_or_default()
    }

    pub fn should_emit_impression_event(&self, name: &str) -> bool {
        self.compiled_state
            .as_ref()
            .and_then(|state| {
                state
                    .get(name)
                    .map(|compiled_toggle| compiled_toggle.impression_data)
            })
            .unwrap_or_default()
    }

    pub fn check_enabled(
        &self,
        name: &str,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> Option<bool> {
        self.get_toggle(name)
            .map(|toggle| self.enabled(toggle, context, external_values))
    }

    pub fn is_enabled(
        &self,
        name: &str,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> bool {
        let is_enabled = self
            .get_toggle(name)
            .map(|toggle| self.enabled(toggle, context, external_values))
            .unwrap_or_default();

        self.count_toggle(name, is_enabled);

        is_enabled
    }

    fn resolve_variant<'a>(
        &self,
        variants: &'a Vec<CompiledVariant>,
        group_id: &str,
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
            .first()
            .and_then(|variant| variant.stickiness.clone());

        let target = get_seed(stickiness, context)
            .map(|seed| {
                normalized_hash(group_id, &seed, total_weight, VARIANT_NORMALIZATION_SEED).unwrap()
            })
            .unwrap_or_else(|| rand::thread_rng().gen_range(0..total_weight));

        let mut total_weight = 0;
        for variant in variants {
            total_weight += variant.weight as u32;
            if total_weight >= target {
                return Some(variant);
            }
        }
        None
    }

    fn check_variant_by_toggle(
        &self,
        toggle: &CompiledToggle,
        context: &Context,
    ) -> Option<VariantDef> {
        let strategy_variants =
            toggle
                .compiled_variant_strategy
                .as_ref()
                .and_then(|variant_strategies| {
                    let context = EnrichedContext::from(context.clone(), toggle.name.clone(), None);

                    let resolved_strategy_variants: Option<(&Vec<CompiledVariant>, &String)> =
                        variant_strategies
                            .iter()
                            .find_map(|(rule, rule_variants, group_id)| {
                                (rule)(&context).then_some((rule_variants, group_id))
                            });
                    resolved_strategy_variants
                });

        let variant = if let Some(strategy_variants) = strategy_variants {
            if strategy_variants.0.is_empty() {
                self.resolve_variant(&toggle.variants, &toggle.name, context)
            } else {
                self.resolve_variant(strategy_variants.0, strategy_variants.1, context)
            }
        } else {
            self.resolve_variant(&toggle.variants, &toggle.name, context)
        };

        variant.map(|variant| VariantDef {
            name: variant.name.clone(),
            payload: variant.payload.clone(),
            enabled: true,
        })
    }

    pub fn check_variant(
        &self,
        name: &str,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> Option<VariantDef> {
        self.get_toggle(name).map(|toggle| {
            if self.enabled(toggle, context, external_values) {
                self.check_variant_by_toggle(toggle, context)
                    .unwrap_or_default()
            } else {
                VariantDef::default()
            }
        })
    }

    pub fn get_variant(
        &self,
        name: &str,
        context: &Context,
        external_values: &Option<HashMap<String, bool>>,
    ) -> ExtendedVariantDef {
        let toggle = self.get_toggle(name);
        let enabled = toggle
            .map(|t| self.enabled(t, context, external_values))
            .unwrap_or_default();

        let variant = match toggle {
            Some(toggle) if enabled => self.check_variant_by_toggle(toggle, context),
            _ => None,
        }
        .unwrap_or_default();

        self.count_toggle(name, enabled);
        self.count_variant(name, &variant.name);

        variant.to_enriched_response(enabled)
    }

    pub fn take_state(&mut self, toggles: ClientFeatures) -> Option<Vec<EvalWarning>> {
        let (compiled_state, warnings) = compile_state(&toggles);
        self.compiled_state = Some(compiled_state);
        if !warnings.is_empty() {
            Some(warnings)
        } else {
            None
        }
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

impl VariantDef {
    pub fn to_enriched_response(&self, toggle_enabled: bool) -> ExtendedVariantDef {
        ExtendedVariantDef {
            name: self.name.clone(),
            payload: self.payload.clone(),
            enabled: self.enabled,
            feature_enabled: toggle_enabled,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedVariantDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
    pub enabled: bool,
    pub feature_enabled: bool,
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
    use unleash_types::client_features::{
        ClientFeatures, ClientFeaturesDelta, FeatureDependency, Override, Payload,
    };

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
        pub(crate) expected_result: TestCaseVariantDef,
    }

    #[derive(Deserialize, Debug)]
    pub struct TestCaseVariantDef {
        pub name: String,
        pub payload: Option<Payload>,
        pub enabled: bool,
        pub feature_enabled: bool,
    }

    fn load_spec(spec_name: &str) -> TestSuite {
        let spec_path = format!("{SPEC_FOLDER}/{spec_name}");
        let spec_data =
            fs::read_to_string(spec_path).expect("Should have been able to read the file");
        serde_json::from_str(&spec_data).expect("Failed to parse client spec")
    }

    fn load_delta(delta_name: &str) -> ClientFeaturesDelta {
        let delta_path = format!("../test-data/{delta_name}");
        let delta = fs::read_to_string(delta_path).expect("Should have been able to read the file");
        serde_json::from_str(&delta).expect("Failed to parse client spec")
    }

    #[test]
    fn can_load_single() {
        let delta = load_delta("delta_base.json");
        let mut engine = EngineState::default();
        engine.take_delta(&delta);
        assert!(engine.get_toggle("test-flag").is_some())
    }

    #[test]
    fn can_update_existing_state() {
        let delta = load_delta("delta_base.json");
        let patch = load_delta("delta_patch.json");
        let mut engine = EngineState::default();
        let context = Context {
            user_id: Some("4".into()),
            ..Context::default()
        };

        engine.take_delta(&delta);
        assert!(!engine.is_enabled("test-flag", &context, &None));
        assert!(engine.get_toggle("removed-flag").is_some());
        assert!(!engine.is_enabled("segment-flag", &context, &None));
        engine.take_delta(&patch);
        assert!(engine.is_enabled("test-flag", &context, &None));
        assert!(!engine.get_toggle("removed-flag").is_some());
        assert!(engine.is_enabled("segment-flag", &context, &None));
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
    #[test_case("16-strategy-variants.json"; "Strategy variants")]
    #[test_case("17-dependent-features.json"; "Dependent features")]
    fn run_client_spec(spec_name: &str) {
        let spec = load_spec(spec_name);
        let mut engine = EngineState::default();
        let warnings = engine.take_state(spec.state);

        assert!(warnings.is_none());

        if let Some(mut tests) = spec.tests {
            while let Some(test_case) = tests.pop() {
                println!(
                    "Executing test {:?} with toggle name{:?} against context{:?}",
                    &test_case.description, &test_case.toggle_name, &test_case.context
                );
                let expected = test_case.expected_result;
                let actual = engine.is_enabled(&test_case.toggle_name, &test_case.context, &None);
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
                let actual = engine.get_variant(&test_case.toggle_name, &test_case.context, &None);
                assert_eq!(expected.enabled, actual.enabled);
                assert_eq!(expected.feature_enabled, actual.feature_enabled);
                assert_eq!(expected.name, actual.name);
                assert_eq!(expected.payload, actual.payload);
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
                }],
                ..CompiledToggle::default()
            },
        );
        let state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };
        let context = Context::default();

        state.get_variant("cool-animals", &context, &None);
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
            state.get_variant("test", &context, &None),
            VariantDef::default().to_enriched_response(true)
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

        state.is_enabled("some-toggle", &context_with_user_id_of_7, &None);
        state.is_enabled("some-toggle", &context_with_user_id_of_7, &None);

        //No user id, no enabled state, this should increment the "no" metric
        state.is_enabled("some-toggle", &blank_context, &None);

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

        state.get_variant("some-toggle", &blank_context, &None);
        state.get_variant("some-toggle", &context_with_user_id_of_7, &None);

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
    pub fn no_valid_metrics_yields_none() {
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

        let metrics = state.get_metrics();
        assert!(metrics.is_none());
    }

    #[test]
    pub fn getting_metrics_clears_existing_metrics() {
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

        state.is_enabled("some-toggle", &Context::default(), &None);

        let metrics = state.get_metrics();
        assert!(metrics.is_some());

        let metrics = state.get_metrics();
        assert!(metrics.is_none());
    }

    #[test]
    pub fn unknown_features_and_variants_get_metrics() {
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

        state.is_enabled("missing-toggle", &Context::default(), &None);
        state.get_variant("missing-toggle", &Context::default(), &None);

        let metrics = state.get_metrics().unwrap();

        let some_toggle_stats = metrics.toggles.get("missing-toggle").unwrap();
        assert_eq!(some_toggle_stats.yes, 0);
        assert_eq!(some_toggle_stats.no, 2);
        assert_eq!(some_toggle_stats.variants.len(), 1);
    }

    #[test]
    pub fn multiple_toggle_checks_stack_metrics() {
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

        for _ in 0..10 {
            state.is_enabled("some-toggle", &Context::default(), &None);
            state.get_variant("some-toggle", &Context::default(), &None);

            state.is_enabled("missing-toggle", &Context::default(), &None);
        }

        let metrics = state.get_metrics().unwrap();

        let some_toggle_stats = metrics.toggles.get("some-toggle").unwrap();
        let missing_toggle_stats = metrics.toggles.get("missing-toggle").unwrap();

        let disabled_variant_count = *some_toggle_stats.variants.get("disabled").unwrap();

        assert_eq!(some_toggle_stats.yes, 20);
        assert_eq!(some_toggle_stats.no, 0);
        assert_eq!(some_toggle_stats.variants.len(), 1);
        assert_eq!(disabled_variant_count, 10);

        assert_eq!(missing_toggle_stats.yes, 0);
        assert_eq!(missing_toggle_stats.no, 10);
    }

    #[test]
    pub fn check_enabled_and_count_metrics_yields_same_metrics_as_is_enabled() {
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

        let is_enabled = state.is_enabled("some-toggle", &Context::default(), &None);

        let is_enabled_metrics = state
            .get_metrics()
            .unwrap()
            .toggles
            .get("some-toggle")
            .unwrap()
            .yes;

        let check_enabled = state
            .check_enabled("some-toggle", &Context::default(), &None)
            .unwrap();

        state.count_toggle("some-toggle", check_enabled);

        let count_toggle_metrics = state
            .get_metrics()
            .unwrap()
            .toggles
            .get("some-toggle")
            .unwrap()
            .yes;

        assert_eq!(is_enabled_metrics, 1);
        assert_eq!(is_enabled_metrics, count_toggle_metrics);

        assert!(is_enabled);
        assert_eq!(is_enabled, check_enabled);
    }

    #[test]
    pub fn check_variant_and_count_metrics_yields_same_metrics_as_get_variant() {
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

        let first_variant = state.get_variant("some-toggle", &Context::default(), &None);
        let get_variant_metrics = state
            .get_metrics()
            .unwrap()
            .toggles
            .get("some-toggle")
            .unwrap()
            .clone();

        let second_variant = state
            .check_variant("some-toggle", &Context::default(), &None)
            .unwrap_or_default();

        state.count_toggle("some-toggle", true);
        state.count_variant("some-toggle", &second_variant.name);
        let check_variant_metrics = state
            .get_metrics()
            .unwrap()
            .toggles
            .get("some-toggle")
            .unwrap()
            .clone();

        assert_eq!(
            first_variant,
            VariantDef::default().to_enriched_response(true)
        );
        assert_eq!(first_variant.name, second_variant.name);

        assert_eq!(get_variant_metrics.variants.len(), 1);
        assert_eq!(check_variant_metrics.variants.len(), 1);

        assert_eq!(get_variant_metrics.variants.get("disabled").unwrap(), &1);
        assert_eq!(check_variant_metrics.variants.get("disabled").unwrap(), &1);

        assert_eq!(get_variant_metrics.yes, 1);
        assert_eq!(check_variant_metrics.yes, 1);

        assert_eq!(get_variant_metrics.no, 0);
        assert_eq!(check_variant_metrics.no, 0);
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
        let toggles = state.resolve_all(&blank_context, &None).unwrap();
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

    #[test]
    fn resolves_single_toggles() {
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
        let toggle = state.resolve("some-toggle", &blank_context, &None).unwrap();
        let resolved_variant = toggle.variant.name;

        assert!(toggle.enabled);
        assert_eq!(resolved_variant, "test-variant".to_string());
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

    #[test]
    pub fn strategy_variants_are_selected_over_base_variants_if_present() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![CompiledVariant {
                    name: "should-be-ignored".into(),
                    weight: 100,
                    stickiness: None,
                    payload: None,
                    overrides: None,
                }],
                compiled_variant_strategy: Some(vec![(
                    Box::new(|_| true),
                    vec![CompiledVariant {
                        name: "don't-ignore-me".into(),
                        weight: 100,
                        stickiness: None,
                        payload: None,
                        overrides: None,
                    }],
                    "some-toggle".to_string(),
                )]),
                ..CompiledToggle::default()
            },
        );
        let state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };
        let variant = state.get_variant("some-toggle", &Context::default(), &None);
        assert_eq!(variant.name, "don't-ignore-me".to_string());
    }

    #[test]
    fn strategy_variants_respect_toggles_being_disabled() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: false,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                compiled_variant_strategy: Some(vec![(
                    Box::new(|_| true),
                    vec![CompiledVariant {
                        name: "".into(),
                        weight: 100,
                        stickiness: None,
                        payload: None,
                        overrides: None,
                    }],
                    "some-toggle".to_string(),
                )]),
                ..CompiledToggle::default()
            },
        );
        let state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };
        let variant = state.get_variant("some-toggle", &Context::default(), &None);
        assert_eq!(variant.name, "disabled".to_string());
    }

    #[test]
    pub fn empty_strategy_variants_do_not_block_non_strategy_variants_from_working() {
        let raw_state = r#"
        {
            "version": 2,
            "features": [
                {
                    "name": "toggle1",
                    "type": "release",
                    "enabled": true,
                    "project": "TestProject20",
                    "stale": false,
                    "strategies": [
                        {
                            "name": "flexibleRollout",
                            "constraints": [],
                            "parameters": {
                                "groupId": "toggle1",
                                "rollout": "100",
                                "stickiness": "default"
                            },
                            "variants": []
                        }
                    ],
                    "variants": [
                        {
                            "name": "another",
                            "weight": 1000,
                            "overrides": [],
                            "stickiness": "default",
                            "weightType": "variable"
                        }
                    ],
                    "description": null,
                    "impressionData": false
                }
            ],
            "query": {
                "environment": "development",
                "inlineSegmentConstraints": true
            },
            "meta": {
                "revisionId": 12137,
                "etag": "\"76d8bb0e:12137\"",
                "queryHash": "76d8bb0e"
            }
        }
        "#;

        let feature_set: ClientFeatures = serde_json::from_str(raw_state).unwrap();
        let mut engine = EngineState::default();
        let context = Context {
            user_id: Some("7".into()),
            ..Context::default()
        };

        let warnings = engine.take_state(feature_set);

        let results = engine.resolve_all(&context, &None);
        let targeted_toggle = results.unwrap().get("toggle1").unwrap().clone();

        assert!(targeted_toggle.enabled);
        assert_eq!(targeted_toggle.variant.name, "another");
        assert!(warnings.is_none());
    }

    #[test]
    pub fn inverted_list_constraint_still_invert_when_context_field_missing() {
        let raw_state = r#"
        {
            "version": 2,
            "features": [
                {
                    "name": "toggle1",
                    "type": "release",
                    "enabled": true,
                    "project": "TestProject20",
                    "stale": false,
                    "strategies": [
                        {
                            "name": "flexibleRollout",
                            "constraints": [
                                {
                                "contextName": "missing_field",
                                "operator": "IN",
                                "values": [
                                    "17"
                                ],
                                "inverted": true,
                                "caseInsensitive": false
                                }
                            ],
                            "parameters": {
                                "groupId": "toggle1",
                                "rollout": "100",
                                "stickiness": "default"
                            },
                            "variants": []
                        }
                    ],
                    "variants": [
                        {
                            "name": "another",
                            "weight": 1000,
                            "overrides": [],
                            "stickiness": "default",
                            "weightType": "variable"
                        }
                    ],
                    "description": null,
                    "impressionData": false
                }
            ],
            "query": {
                "environment": "development",
                "inlineSegmentConstraints": true
            },
            "meta": {
                "revisionId": 12137,
                "etag": "\"76d8bb0e:12137\"",
                "queryHash": "76d8bb0e"
            }
        }
        "#;

        let feature_set: ClientFeatures = serde_json::from_str(raw_state).unwrap();
        let mut engine = EngineState::default();
        let context = Context {
            user_id: Some("7".into()),
            ..Context::default()
        };

        let warnings = engine.take_state(feature_set);
        let enabled = engine.check_enabled("toggle1", &context, &None).unwrap();

        assert!(enabled);
        assert!(warnings.is_none());
    }

    #[test]
    pub fn inverted_numerical_constraint_still_invert_when_context_field_missing() {
        let raw_state = r#"
        {
            "version": 2,
            "features": [
                {
                    "name": "toggle1",
                    "type": "release",
                    "enabled": true,
                    "project": "TestProject20",
                    "stale": false,
                    "strategies": [
                        {
                            "name": "flexibleRollout",
                            "constraints": [
                                {
                                "contextName": "missing_field",
                                "operator": "NUM_EQ",
                                "value": "17",
                                "inverted": true,
                                "caseInsensitive": false
                                }
                            ],
                            "parameters": {
                                "groupId": "toggle1",
                                "rollout": "100",
                                "stickiness": "default"
                            },
                            "variants": []
                        }
                    ],
                    "variants": [
                        {
                            "name": "another",
                            "weight": 1000,
                            "overrides": [],
                            "stickiness": "default",
                            "weightType": "variable"
                        }
                    ],
                    "description": null,
                    "impressionData": false
                }
            ],
            "query": {
                "environment": "development",
                "inlineSegmentConstraints": true
            },
            "meta": {
                "revisionId": 12137,
                "etag": "\"76d8bb0e:12137\"",
                "queryHash": "76d8bb0e"
            }
        }
        "#;

        let feature_set: ClientFeatures = serde_json::from_str(raw_state).unwrap();
        let mut engine = EngineState::default();
        let context = Context {
            user_id: Some("7".into()),
            ..Context::default()
        };

        let warnings = engine.take_state(feature_set);
        let enabled = engine.check_enabled("toggle1", &context, &None).unwrap();

        assert!(enabled);
        assert!(warnings.is_none());
    }

    #[test]
    pub fn metrics_are_not_recorded_for_parent_flags() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                dependencies: vec![FeatureDependency {
                    feature: "parent-flag".into(),
                    enabled: Some(true),
                    variants: None,
                }],
                ..CompiledToggle::default()
            },
        );

        compiled_state.insert(
            "parent-flag".to_string(),
            CompiledToggle {
                name: "parent-flag".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                ..CompiledToggle::default()
            },
        );

        let mut state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };

        let blank_context = Context::default();

        state.is_enabled("some-toggle", &blank_context, &None);

        let metrics = state.get_metrics().unwrap();
        assert_eq!(metrics.toggles.get("some-toggle").unwrap().yes, 1);
        assert!(metrics.toggles.get("parent-flag").is_none());
    }

    #[test]
    pub fn metrics_are_not_recorded_for_parent_flags_with_variants() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                dependencies: vec![FeatureDependency {
                    feature: "parent-flag".into(),
                    enabled: Some(true),
                    variants: Some(vec!["don't-ignore-me".into()]),
                }],
                ..CompiledToggle::default()
            },
        );

        compiled_state.insert(
            "parent-flag".to_string(),
            CompiledToggle {
                name: "parent-flag".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                compiled_variant_strategy: Some(vec![(
                    Box::new(|_| true),
                    vec![CompiledVariant {
                        name: "don't-ignore-me".into(),
                        weight: 100,
                        stickiness: None,
                        payload: None,
                        overrides: None,
                    }],
                    "parent-flag".to_string(),
                )]),
                ..CompiledToggle::default()
            },
        );

        let mut state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };

        let blank_context = Context::default();

        state.is_enabled("some-toggle", &blank_context, &None);

        let metrics = state.get_metrics().unwrap();
        assert_eq!(metrics.toggles.get("some-toggle").unwrap().yes, 1);
        assert!(metrics.toggles.get("parent-flag").is_none());
    }

    #[test]
    pub fn parent_flags_are_consulted_for_get_variant() {
        let mut compiled_state = HashMap::new();
        compiled_state.insert(
            "some-toggle".to_string(),
            CompiledToggle {
                name: "some-toggle".into(),
                enabled: true,
                compiled_strategy: Box::new(|_| true),
                variants: vec![CompiledVariant {
                    name: "enabled-variant".into(),
                    weight: 100,
                    stickiness: None,
                    payload: None,
                    overrides: None,
                }],
                dependencies: vec![FeatureDependency {
                    feature: "parent-flag".into(),
                    enabled: Some(true),
                    variants: Some(vec!["don't-ignore-me".into()]),
                }],
                ..CompiledToggle::default()
            },
        );

        compiled_state.insert(
            "parent-flag".to_string(),
            CompiledToggle {
                name: "parent-flag".into(),
                enabled: false,
                compiled_strategy: Box::new(|_| true),
                variants: vec![],
                ..CompiledToggle::default()
            },
        );

        let mut state = EngineState {
            compiled_state: Some(compiled_state),
            ..Default::default()
        };

        let blank_context = Context::default();

        let variant = state.get_variant("some-toggle", &blank_context, &None);

        assert_eq!(variant.name, "disabled");

        let metrics = state.get_metrics().unwrap();
        assert_eq!(metrics.toggles.get("some-toggle").unwrap().no, 1);
        assert!(metrics.toggles.get("parent-flag").is_none());
    }

    #[test]
    pub fn strategy_variants_are_selected_over_base_variants_if_present_and_also_when_previous_failing_strategy_has_none(
    ) {
        let raw_state = r#"
      {
          "version": 2,
          "features": [
              {
                  "name": "toggle1",
                  "type": "release",
                  "enabled": true,
                  "project": "TestProject20",
                  "stale": false,
                  "strategies": [
                      {
                          "name": "flexibleRollout",
                          "constraints": [
                            {
                              "contextName": "userId",
                              "operator": "IN",
                              "values": [
                                "17"
                              ],
                              "inverted": false,
                              "caseInsensitive": false
                            }
                          ],
                          "parameters": {
                              "groupId": "toggle1",
                              "rollout": "100",
                              "stickiness": "default"
                          },
                          "variants": []
                      },
                      {
                        "name": "flexibleRollout",
                        "constraints": [],
                        "parameters": {
                            "groupId": "toggle1",
                            "rollout": "100",
                            "stickiness": "default"
                        },
                        "variants": [
                          {
                            "name": "theselectedone",
                            "weight": 1000,
                            "overrides": [],
                            "stickiness": "default",
                            "weightType": "variable"
                          }
                      ]
                    }
                ],
                  "variants": [
                      {
                          "name": "notselected",
                          "weight": 1000,
                          "overrides": [],
                          "stickiness": "default",
                          "weightType": "variable"
                      }
                  ],
                  "description": null,
                  "impressionData": false
              }
          ],
          "query": {
              "environment": "development",
              "inlineSegmentConstraints": true
          },
          "meta": {
              "revisionId": 12137,
              "etag": "\"76d8bb0e:12137\"",
              "queryHash": "76d8bb0e"
          }
      }
      "#;

        let feature_set: ClientFeatures = serde_json::from_str(raw_state).unwrap();
        let mut engine = EngineState::default();
        let context = Context {
            ..Context::default()
        };

        let warnings = engine.take_state(feature_set);

        let results = engine.resolve_all(&context, &None);
        let targeted_toggle = results.unwrap().get("toggle1").unwrap().clone();

        assert!(targeted_toggle.enabled);
        assert_eq!(targeted_toggle.variant.name, "theselectedone");
        assert!(warnings.is_none());
    }

    #[test]
    fn invalid_toggles_do_not_affect_other_toggles() {
        let raw_state = r#"
        {
            "version": 2,
            "features": [
              {
                "name": "Should_always_be_off",
                "enabled": true,
                "strategies": [
                  {
                    "name": "userWithId",
                    "parameters": {
                      "userIds": "[\"this\",\"is\",\"broken\"]"
                    }
                  }
                ]
              },
              {
                "name": "This_should_be_okay",
                "enabled": true,
                "strategies": [
                  {
                    "name": "userWithId",
                    "parameters": {
                      "userIds": "this,is,okay"
                    }
                  }
                ]
              }
            ]
          }
        "#;

        let feature_set: ClientFeatures = serde_json::from_str(raw_state).unwrap();
        let mut engine = EngineState::default();

        let warnings = engine.take_state(feature_set);

        let context = Context {
            user_id: Some("okay".into()),
            ..Context::default()
        };

        assert!(!engine.is_enabled("Should_always_be_off", &context, &None));
        assert!(engine.is_enabled("This_should_be_okay", &context, &None));
        println!("{:?}", warnings);
        assert!(warnings.is_none());
    }
}
