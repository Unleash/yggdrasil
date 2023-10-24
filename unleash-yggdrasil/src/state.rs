use std::collections::HashMap;
use unleash_types::client_features::Context;

pub struct EnrichedContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
    pub strategy_results: Option<HashMap<usize, bool>>,
    pub(crate) toggle_name: String,
}

impl EnrichedContext {
    pub fn from(
        context: Context,
        toggle_name: String,
        strategy_results: Option<HashMap<usize, bool>>,
    ) -> Self {
        EnrichedContext {
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            environment: context.environment.clone(),
            app_name: context.app_name.clone(),
            current_time: context
                .current_time
                .clone()
                .or_else(|| Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string())),
            remote_address: context.remote_address.clone(),
            properties: context.properties,
            strategy_results,
            toggle_name,
        }
    }
}

#[derive(Debug)]
pub enum SdkError {
    StrategyEvaluationError,
    StrategyParseError,
}

#[cfg(test)]
mod test {
    use crate::strategy_parsing::compile_rule;

    use super::*;
    use unleash_types::client_features::Context;

    #[test]
    fn converting_a_context_to_enriched_context_assumes_now_for_time_if_not_set() {
        let context = Context::default();
        let enriched_context = EnrichedContext::from(context, "test".into(), None);
        chrono::DateTime::parse_from_rfc3339(
            &enriched_context
                .current_time
                .clone()
                .expect("current_time should be set"),
        )
        .expect("cannot parse retrieved dates");
    }

    #[test]
    fn converting_a_context_to_enriched_context_leaves_current_time_alone_if_set() {
        let context = Context {
            current_time: Some("2020-01-01T00:00:00Z".into()),
            ..Context::default()
        };
        let enriched_context = EnrichedContext::from(context, "test".into(), None);
        assert_eq!(
            enriched_context
                .current_time
                .expect("current_time should be set"),
            "2020-01-01T00:00:00Z"
        );
    }

    #[test]
    fn assumed_current_time_works_correctly_in_a_constraint() {
        let rule_text = "current_time > 2023-10-13T10:19:22Z";
        let rule = compile_rule(rule_text).unwrap();
        let context = Context::default();
        let enriched_context = EnrichedContext::from(context, "test".into(), None);

        assert!(rule(&enriched_context));
    }
}
