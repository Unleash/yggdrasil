use proptest::prelude::*;
use unleash_yggdrasil::strategy_parsing::compile_rule;

proptest! {
    #[test]
    fn test_compile_rule(input in "\\PC*") {
        if !input.contains(r#""#) {
            let rule = format!("user_id in [\"{input}\"]");
            println!("rule: {:?}", rule);
            let result = compile_rule(&rule);
            prop_assert!(result.is_ok());
        }
    }
}
