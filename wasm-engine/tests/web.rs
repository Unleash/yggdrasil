//! Test suite for the Web and headless browsers.

#![cfg(target_family = "wasm")]

extern crate wasm_bindgen_test;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use yggdrasil_engine::Engine;
use unleash_types::client_features::{ClientFeature, ClientFeatures, Constraint, Context, Operator, Strategy};

#[wasm_bindgen_test]
fn is_enabled_evaluates_correctly() {
    let mut engine = Engine::new();

    let state: JsValue = to_value(&ClientFeatures {
        version: 2,
        features: vec![ClientFeature {
            name: "feature".into(),
            enabled: true,
            strategies: Some(vec![Strategy {
              name: "default".into(),
              constraints: Some(vec![Constraint {
                  context_name: "userId".into(),
                  operator: Operator::In,
                  values: Some(vec!["5".into()]),
                  case_insensitive: false,
                  inverted: false,
                  value: None,
              }]),
              segments: None,
              variants: None,
              parameters: None,
              sort_order: None,
            }]),
            ..ClientFeature::default()
        }],
        segments: None,
        query: None,
    }).unwrap();

    let _ = engine.take_state(state);

    let context: JsValue = to_value(&Context {
        user_id: Some("5".into()),
        session_id: None,
        environment: None,
        app_name: None,
        current_time: None,
        remote_address: None,
        properties: None,
    }).unwrap();

    let result = engine.is_enabled("feature", context).unwrap();
    assert_eq!(result, true);
}
