//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use wasm_engine::{evaluate, Context};

#[wasm_bindgen_test]
fn dsl_evaluates_correct() {
    let dsl_fragment = "user_id > 5";

    let context: JsValue = to_value(&Context {
        user_id: Some("6".into()),
        session_id: None,
        environment: None,
        app_name: None,
        current_time: None,
        remote_address: None,
        group_id: None,
        properties: None,
    })
    .unwrap();

    let result = evaluate(dsl_fragment, context).unwrap();
    assert_eq!(result, true);
}

#[wasm_bindgen_test]
fn invalid_rule_raises_an_error() {
    let dsl_fragment = "this does not compile";

    let context: JsValue = to_value(&Context {
        user_id: Some("6".into()),
        session_id: None,
        environment: None,
        app_name: None,
        current_time: None,
        remote_address: None,
        group_id: None,
        properties: None,
    })
    .unwrap();

    let result = evaluate(dsl_fragment, context);

    assert!(result.is_err());
}
