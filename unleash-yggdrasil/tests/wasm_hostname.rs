#![cfg(all(target_arch = "wasm32", feature = "hostname"))]

use unleash_yggdrasil::{state::EnrichedContext, strategy_parsing::compile_rule, Context};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen(module = "node:os")]
extern "C" {
    #[wasm_bindgen(js_name = hostname)]
    fn expected_hostname() -> String;
}

#[wasm_bindgen_test]
fn hostname_constraint_uses_node_hostname() {
    let hostname = expected_hostname();
    assert!(!hostname.is_empty());

    let context = Context::default();
    let context = EnrichedContext::from(&context, "test-toggle", None);
    for (candidate, expected) in [
        (hostname.clone(), true),
        (format!("{hostname}-different"), false),
    ] {
        let quoted = serde_json::to_string(&candidate).unwrap();
        let rule = compile_rule(&format!("hostname in [{quoted}]")).unwrap();
        assert_eq!(rule(&context), expected, "hostname candidate: {candidate}");
    }
}
