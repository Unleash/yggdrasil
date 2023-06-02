use proptest::prelude::*;
use unleash_yggdrasil::strategy_parsing::compile_rule;

proptest! {
    #[test]
    fn test_compile_rule(input in any::<String>().prop_filter("Exclude strings with \\\", \", \\ and empty strings", |s| !s.is_empty() && !s.contains("\\\"") && !s.contains("\"") && !s.contains("\\"))) {
        let rule = format!("user_id in [\"{}\"]", input);
        println!("rule: {:?}", rule);
        let result = compile_rule(&rule);
        prop_assert!(result.is_ok());
    }
}
