use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use unleash_types::client_features::{ClientFeatures, Override, Segment};
use unleash_yggdrasil::state::{EnrichedContext, SdkError};
use unleash_yggdrasil::strategy_parsing::{compile_rule, normalized_hash};
use unleash_yggdrasil::strategy_upgrade::upgrade;
use unleash_yggdrasil::{
    compile_state, CompiledState, CompiledToggle, CompiledVariant, Context, EvalWarning,
    ExtendedVariantDef, ToggleDefinition, VariantDef,
};
use wasm_bindgen::prelude::*;

const VARIANT_NORMALIZATION_SEED: u32 = 86028157;

#[wasm_bindgen]
pub struct WasmEngine {
    compiled_state: CompiledState,
    grammars: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct ToggleEvaluation {
    enabled: bool,
    variant: ExtendedVariantDef,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEngine {
        WasmEngine {
            compiled_state: CompiledState::default(),
            grammars: HashMap::new(),
        }
    }

    #[wasm_bindgen(js_name = "loadClientFeatures")]
    pub fn load_client_features(&mut self, features_json: &str) -> Result<JsValue, JsValue> {
        let warnings = self
            .load_client_features_internal(features_json)
            .map_err(|err| js_error(err))?;
        to_value(&warnings).map_err(|err| js_error(err.to_string()))
    }

    #[wasm_bindgen(js_name = "listToggles")]
    pub fn list_toggles(&self) -> Result<JsValue, JsValue> {
        let toggles = self.toggle_definitions();
        to_value(&toggles).map_err(|err| js_error(err.to_string()))
    }

    #[wasm_bindgen(js_name = "getGrammars")]
    pub fn get_grammars(&self) -> Result<JsValue, JsValue> {
        to_value(&self.grammars).map_err(|err| js_error(err.to_string()))
    }

    #[wasm_bindgen(js_name = "getToggleGrammar")]
    pub fn get_toggle_grammar(&self, toggle_name: &str) -> Option<String> {
        self.grammars.get(toggle_name).cloned()
    }

    #[wasm_bindgen(js_name = "setToggleGrammar")]
    pub fn set_toggle_grammar(&mut self, toggle_name: &str, grammar: &str) -> Result<(), JsValue> {
        self.apply_grammar_internal(toggle_name, grammar)
            .map_err(|err| js_error(err))
    }

    #[wasm_bindgen(js_name = "setGrammars")]
    pub fn set_grammars(&mut self, grammars: JsValue) -> Result<(), JsValue> {
        let updates: HashMap<String, String> = from_value(grammars)
            .map_err(|err| js_error(format!("Invalid grammar payload: {err}")))?;

        for (toggle_name, grammar) in updates {
            self.apply_grammar_internal(&toggle_name, &grammar)
                .map_err(|err| js_error(err))?;
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "evaluate")]
    pub fn evaluate(&self, toggle_name: &str, context_json: &str) -> Result<JsValue, JsValue> {
        let evaluation = self
            .evaluate_internal(toggle_name, context_json)
            .map_err(|err| js_error(err))?;
        to_value(&evaluation).map_err(|err| js_error(err.to_string()))
    }
}

impl WasmEngine {
    fn load_client_features_internal(
        &mut self,
        features_json: &str,
    ) -> Result<Vec<EvalWarning>, String> {
        let features: ClientFeatures = serde_json::from_str(features_json)
            .map_err(|err| format!("Invalid Unleash response JSON: {err}"))?;

        let (compiled_state, warnings) = compile_state(&features);
        self.grammars = compute_grammars(&features);
        self.compiled_state = compiled_state;

        Ok(warnings)
    }

    fn toggle_definitions(&self) -> Vec<ToggleDefinition> {
        self.compiled_state
            .iter()
            .map(|(_, toggle)| ToggleDefinition {
                name: toggle.name.clone(),
                feature_type: toggle.feature_type.clone(),
                project: toggle.project.clone(),
                enabled: toggle.enabled,
            })
            .collect()
    }

    fn apply_grammar_internal(&mut self, toggle_name: &str, grammar: &str) -> Result<(), String> {
        let compiled = compile_rule(grammar).map_err(format_sdk_error)?;

        let toggle = self
            .compiled_state
            .get_mut(toggle_name)
            .ok_or_else(|| format!("Unknown toggle '{toggle_name}'"))?;

        toggle.compiled_strategy = compiled;
        self.grammars
            .insert(toggle_name.to_string(), grammar.to_string());
        Ok(())
    }

    fn evaluate_internal(
        &self,
        toggle_name: &str,
        context_json: &str,
    ) -> Result<ToggleEvaluation, String> {
        let toggle = self
            .compiled_state
            .get(toggle_name)
            .ok_or_else(|| format!("Unknown toggle '{toggle_name}'"))?;

        let context = parse_context(context_json)?;
        let enriched_context =
            EnrichedContext::from(context.clone(), toggle_name.to_string(), None);

        let enabled = self.compiled_toggle_enabled(toggle, &enriched_context);
        let variant = if enabled {
            self.check_variant_by_toggle(toggle, &enriched_context)
                .unwrap_or_default()
        } else {
            VariantDef::default()
        };

        Ok(ToggleEvaluation {
            enabled,
            variant: variant.to_enriched_response(enabled),
        })
    }

    fn compiled_toggle_enabled(&self, toggle: &CompiledToggle, context: &EnrichedContext) -> bool {
        toggle.enabled
            && self.is_parent_dependency_satisfied(toggle, context)
            && (toggle.compiled_strategy)(context)
    }

    fn is_parent_dependency_satisfied(
        &self,
        toggle: &CompiledToggle,
        enriched_context: &EnrichedContext,
    ) -> bool {
        toggle.dependencies.iter().all(|parent_dependency| {
            let mut context = enriched_context.clone();
            context.toggle_name = toggle.name.clone();

            let Some(compiled_parent) = self.compiled_state.get(&parent_dependency.feature) else {
                return false;
            };

            if !compiled_parent.dependencies.is_empty() {
                return false;
            }

            let parent_enabled = self.compiled_toggle_enabled(compiled_parent, &context);
            let expected_parent_enabled_state = parent_dependency.enabled.unwrap_or(true);
            let parent_variant = self.check_variant_by_toggle(compiled_parent, &context);

            let is_variant_dependency_satisfied =
                if let (Some(expected_variants), Some(actual_variant)) =
                    (&parent_dependency.variants, parent_variant)
                {
                    expected_variants.is_empty() || expected_variants.contains(&actual_variant.name)
                } else {
                    true
                };

            is_variant_dependency_satisfied && parent_enabled == expected_parent_enabled_state
        })
    }

    fn check_variant_by_toggle(
        &self,
        toggle: &CompiledToggle,
        context: &EnrichedContext,
    ) -> Option<VariantDef> {
        let strategy_variants =
            toggle
                .compiled_variant_strategy
                .as_ref()
                .and_then(|variant_strategies| {
                    variant_strategies
                        .iter()
                        .find_map(|(rule, rule_variants, group_id)| {
                            (rule)(context).then_some((rule_variants, group_id))
                        })
                });

        let variant = if let Some((rule_variants, group_id)) = strategy_variants {
            if rule_variants.is_empty() {
                self.resolve_variant(&toggle.variants, &toggle.name, context)
            } else {
                self.resolve_variant(rule_variants, group_id, context)
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

    fn resolve_variant<'a>(
        &self,
        variants: &'a Vec<CompiledVariant>,
        group_id: &str,
        context: &EnrichedContext,
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
                normalized_hash(group_id, &seed, total_weight, VARIANT_NORMALIZATION_SEED)
                    .unwrap_or(1)
            })
            .unwrap_or(1);

        let mut running_weight = 0;
        for variant in variants {
            running_weight += variant.weight as u32;
            if running_weight >= target {
                return Some(variant);
            }
        }

        variants.last()
    }
}

fn parse_context(context_json: &str) -> Result<Context, String> {
    let payload = if context_json.trim().is_empty() {
        "{}"
    } else {
        context_json
    };

    serde_json::from_str(payload).map_err(|err| format!("Invalid context JSON: {err}"))
}

fn compute_grammars(features: &ClientFeatures) -> HashMap<String, String> {
    let segment_map = build_segment_map(&features.segments);

    features
        .features
        .iter()
        .map(|feature| {
            let strategies = feature.strategies.clone().unwrap_or_default();
            let grammar = upgrade(&strategies, &segment_map);
            (feature.name.clone(), grammar)
        })
        .collect()
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

fn get_seed(stickiness: Option<String>, context: &EnrichedContext) -> Option<String> {
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
    context: &'a EnrichedContext,
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
    context: &EnrichedContext,
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

fn js_error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

fn format_sdk_error(err: SdkError) -> String {
    match err {
        SdkError::StrategyEvaluationError => "Strategy evaluation error".to_string(),
        SdkError::StrategyParseError(msg) => msg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SPEC: &str = include_str!("../../test-data/simple.json");

    #[test]
    fn load_features_and_evaluate() {
        let mut engine = WasmEngine::new();
        let warnings = engine.load_client_features_internal(SIMPLE_SPEC).unwrap();
        assert!(warnings.is_empty());

        let grammar = engine.get_toggle_grammar("Feature.A").unwrap();
        assert_eq!(grammar, "true");

        let enabled_eval = engine.evaluate_internal("Feature.A", "{}").unwrap();
        assert!(enabled_eval.enabled);

        engine.apply_grammar_internal("Feature.A", "false").unwrap();

        let disabled_eval = engine.evaluate_internal("Feature.A", "{}").unwrap();
        assert!(!disabled_eval.enabled);
    }

    #[test]
    fn invalid_grammar_returns_error() {
        let mut engine = WasmEngine::new();
        engine.load_client_features_internal(SIMPLE_SPEC).unwrap();

        let result = engine.apply_grammar_internal("Feature.A", "this is not valid");
        assert!(result.is_err());
    }
}
