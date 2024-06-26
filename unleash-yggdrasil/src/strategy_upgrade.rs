use std::collections::HashMap;

use unleash_types::client_features::{Constraint, Operator, Segment, Strategy, StrategyVariant};

use crate::state::SdkError;

const DEFAULT_STICKINESS: &str = "user_id | session_id | random";

enum StrategyType {
    Default,
    UserWithId,
    GradualRolloutUserId,
    GradualRolloutSessionId,
    GradualRolloutRandom,
    FlexibleRollout,
    RemoteAddress,
    ApplicationHostname,
    //This is a catch all handler on the enum type because we don't know what the
    // custom strategy will be called ahead of time
    #[allow(dead_code)]
    Custom(String),
}

trait IsCustom {
    fn is_custom(&self) -> bool;
}

impl IsCustom for Strategy {
    fn is_custom(&self) -> bool {
        matches!(
            StrategyType::from(self.name.as_str()),
            StrategyType::Custom(_)
        )
    }
}

pub fn upgrade(strategies: &[Strategy], segment_map: &HashMap<i32, Segment>) -> String {
    if strategies.is_empty() {
        return "true".into();
    }
    let mut custom_strat_count = 0;
    let rule_text = strategies
        .iter()
        .map(|strategy| {
            if strategy.is_custom() {
                custom_strat_count += 1;
            }
            upgrade_strategy(strategy, segment_map, custom_strat_count)
        })
        .collect::<Vec<String>>()
        .join(" or ");
    rule_text
}

pub fn build_variant_rules(
    strategies: &[Strategy],
    segment_map: &HashMap<i32, Segment>,
    toggle_name: &str,
) -> Vec<(String, Vec<StrategyVariant>, String, String)> {
    let mut custom_strat_count = 0;
    strategies
        .iter()
        .filter(|strategy| strategy.variants.is_some())
        .map(|strategy| {
            if strategy.is_custom() {
                custom_strat_count += 1;
            }
            (
                upgrade_strategy(strategy, segment_map, custom_strat_count),
                strategy.variants.clone().unwrap(),
                strategy
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("stickiness"))
                    .cloned()
                    .unwrap_or_else(|| "default".to_string()),
                strategy
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("groupId"))
                    .cloned()
                    .unwrap_or_else(|| toggle_name.to_owned()),
            )
        })
        .collect::<Vec<(String, Vec<StrategyVariant>, String, String)>>()
}

trait PropResolver {
    fn get_param(&self, key: &str) -> Option<&String>;
}

impl PropResolver for Strategy {
    fn get_param(&self, key: &str) -> Option<&String> {
        self.parameters
            .as_ref()
            .map(|params| params.get(key))
            .unwrap_or(None)
    }
}

impl From<&str> for StrategyType {
    fn from(strategy: &str) -> Self {
        match strategy {
            "default" => StrategyType::Default,
            "userWithId" => StrategyType::UserWithId,
            "gradualRolloutUserId" => StrategyType::GradualRolloutUserId,
            "gradualRolloutSessionId" => StrategyType::GradualRolloutSessionId,
            "gradualRolloutRandom" => StrategyType::GradualRolloutRandom,
            "flexibleRollout" => StrategyType::FlexibleRollout,
            "remoteAddress" => StrategyType::RemoteAddress,
            "applicationHostname" => StrategyType::ApplicationHostname,
            _ => StrategyType::Custom(strategy.to_string()),
        }
    }
}

fn upgrade_strategy(
    strategy: &Strategy,
    segment_map: &HashMap<i32, Segment>,
    strategy_count: usize,
) -> String {
    let strategy_rule = match StrategyType::from(strategy.name.as_str()) {
        StrategyType::Default => "true".into(),
        StrategyType::UserWithId => upgrade_user_id_strategy(strategy),
        StrategyType::GradualRolloutUserId => upgrade_user_id_rollout_strategy(strategy),
        StrategyType::GradualRolloutSessionId => upgrade_session_id_rollout_strategy(strategy),
        StrategyType::GradualRolloutRandom => upgrade_random(strategy),
        StrategyType::FlexibleRollout => upgrade_flexible_rollout_strategy(strategy),
        StrategyType::RemoteAddress => upgrade_remote_address(strategy),
        StrategyType::ApplicationHostname => upgrade_hostname(strategy),
        StrategyType::Custom(_) => format!("external_value[\"customStrategy{strategy_count}\"]"),
    };

    let segments = strategy
        .segments
        .as_ref()
        .map(|segment| {
            segment.iter().map(|segment_id| {
                segment_map
                    .get(segment_id)
                    .map(|segment| segment.constraints.clone())
                    .ok_or(SdkError::StrategyEvaluationError)
            })
        })
        .map(|iter| iter.collect::<Result<Vec<Vec<Constraint>>, SdkError>>())
        .unwrap_or(Ok(vec![]));

    if segments.is_err() {
        return "false".into(); // We have a broken segment so default the entire strategy to false
    }

    let mut segment_constraints: Vec<Constraint> =
        segments.unwrap().into_iter().flatten().collect();

    let mut raw_constraints = vec![];
    raw_constraints.append(&mut strategy.constraints.clone().unwrap_or_default());
    raw_constraints.append(&mut segment_constraints);

    let constraints = upgrade_constraints(raw_constraints);
    match constraints {
        Some(constraints) => format!("({strategy_rule} and ({constraints}))"),
        None => strategy_rule,
    }
}

fn upgrade_flexible_rollout_strategy(strategy: &Strategy) -> String {
    let rollout = get_rollout_target(strategy, "rollout");

    match rollout {
        Some(rollout) => {
            let mut rule: String = format!("{rollout}%");

            rule = format!(
                "{rule} sticky on {}",
                upgrade_stickiness(strategy.get_param("stickiness"))
            );

            if let Some(group_id) = strategy.get_param("groupId") {
                rule = format!("{rule} with group_id of \"{group_id}\"");
            }

            rule
        }
        None => "false".into(),
    }
}

fn upgrade_user_id_strategy(strategy: &Strategy) -> String {
    match strategy.get_param("userIds") {
        Some(user_ids) => {
            let user_ids = user_ids
                .split(',')
                //The escaping of quotes is to tolerate a legacy validation in bug in the Unleash server,
                // which would save an incorrect parameter set for the strategy. This will still likely
                // cause Yggdrasil to evalute the strategy as off for most cases, but it will allow the rule to compile
                .map(|id| format!("\"{}\"", escape_quotes(id.trim())))
                .collect::<Vec<String>>()
                .join(",");
            format!("user_id in [{user_ids}]")
        }
        None => "false".into(),
    }
}

fn upgrade_remote_address(strategy: &Strategy) -> String {
    match strategy.get_param("IPs") {
        Some(addresses) => {
            let ips = addresses
                .split(',')
                .collect::<Vec<&str>>()
                .iter()
                .map(|x| x.trim())
                .map(|x| format!("\"{x}\""))
                .collect::<Vec<String>>()
                .join(", ");
            format!("remote_address contains_ip [{ips}]")
        }
        None => "false".into(),
    }
}

fn upgrade_session_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = get_rollout_target(strategy, "percentage");

    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!("{percentage}% sticky on session_id with group_id of \"{group_id}\"")
        }
        _ => "false".into(),
    }
}

fn upgrade_user_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = get_rollout_target(strategy, "percentage");

    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!("{percentage}% sticky on user_id with group_id of \"{group_id}\"")
        }
        _ => "false".into(),
    }
}

fn upgrade_hostname(strategy: &Strategy) -> String {
    let hostnames = strategy
        .get_param("hostNames")
        .cloned()
        .unwrap_or_else(|| "".to_owned()); //intentional, unleash returns "" when no hostnames are set

    let hosts = hostnames
        .split(',')
        .collect::<Vec<&str>>()
        .iter()
        .map(|x| x.trim())
        .map(|x| format!("\"{x}\""))
        .collect::<Vec<String>>()
        .join(", ");
    format!("hostname in [{hosts}]")
}

fn upgrade_random(strategy: &Strategy) -> String {
    let percentage = get_rollout_target(strategy, "percentage");

    match percentage {
        Some(percent) => format!("random < {percent}"),
        None => "false".into(),
    }
}

fn get_rollout_target(strategy: &Strategy, target_property: &str) -> Option<usize> {
    strategy
        .get_param(target_property)
        .map(|value| value.parse::<usize>())
        .and_then(Result::ok)
}

fn upgrade_constraints(constraints: Vec<Constraint>) -> Option<String> {
    if constraints.is_empty() {
        return None;
    };
    let constraint_rules = constraints
        .iter()
        .map(upgrade_constraint)
        .collect::<Vec<String>>();
    let squashed_rules = constraint_rules.join(" and ");
    Some(squashed_rules)
}

fn is_stringy(op: &Operator) -> bool {
    matches!(
        op,
        Operator::NotIn
            | Operator::In
            | Operator::StrEndsWith
            | Operator::StrStartsWith
            | Operator::StrContains
    )
}

fn escape_quotes(stringy_operator: &str) -> String {
    stringy_operator.replace('\"', "\\\"")
}

fn upgrade_constraint(constraint: &Constraint) -> String {
    let context_name = upgrade_context_name(&constraint.context_name);
    let op = upgrade_operator(&constraint.operator, constraint.case_insensitive);
    if op.is_none() {
        return "false".into();
    }
    let op = op.unwrap();
    let inversion = if constraint.inverted { "!" } else { "" };

    let value = if is_stringy(&constraint.operator) {
        let values = constraint
            .values
            .clone()
            .map(|values| {
                values
                    .iter()
                    .map(|x| format!("\"{}\"", escape_quotes(x)))
                    .collect::<Vec<String>>()
                    .join(", ")
            })
            .unwrap_or_default();
        format!("[{values}]")
    } else {
        if constraint.operator == Operator::SemverEq
            || constraint.operator == Operator::SemverLt
            || constraint.operator == Operator::SemverGt
        {
            // A silly special case where we want to ingest
            // broken semver operators so we can reject them.
            // Handling this in the grammar feels awful so we're
            // just not going to
            if constraint.value.as_ref().unwrap().starts_with('v') {
                return "false".into();
            }
        }
        constraint.value.clone().unwrap()
    };

    format!("{inversion}{context_name} {op} {value}")
}

fn upgrade_operator(op: &Operator, case_insensitive: bool) -> Option<String> {
    match op {
        Operator::In => Some("in".into()),
        Operator::NotIn => Some("not_in".into()),
        Operator::StrEndsWith => {
            if case_insensitive {
                Some("ends_with_any_ignore_case".into())
            } else {
                Some("ends_with_any".into())
            }
        }
        Operator::StrStartsWith => {
            if case_insensitive {
                Some("starts_with_any_ignore_case".into())
            } else {
                Some("starts_with_any".into())
            }
        }
        Operator::StrContains => {
            if case_insensitive {
                Some("contains_any_ignore_case".into())
            } else {
                Some("contains_any".into())
            }
        }
        Operator::NumEq => Some("==".into()),
        Operator::NumGt => Some(">".into()),
        Operator::NumGte => Some(">=".into()),
        Operator::NumLt => Some("<".into()),
        Operator::NumLte => Some("<=".into()),
        Operator::DateAfter => Some(">".into()),
        Operator::DateBefore => Some("<".into()),
        Operator::SemverEq => Some("==".into()),
        Operator::SemverLt => Some("<".into()),
        Operator::SemverGt => Some(">".into()),
        Operator::Unknown(_) => None,
    }
}

fn upgrade_stickiness(stickiness_param: Option<&String>) -> String {
    if let Some(stickiness_param) = stickiness_param {
        match stickiness_param.as_ref() {
            "random" => "random".into(),
            "default" => DEFAULT_STICKINESS.into(),
            _ => upgrade_context_name(stickiness_param),
        }
    } else {
        DEFAULT_STICKINESS.into()
    }
}

fn upgrade_context_name(context_name: &str) -> String {
    match context_name {
        "userId" => "user_id".into(),
        "sessionId" => "session_id".into(),
        "currentTime" => "current_time".into(),
        "environment" => "environment".into(),
        "appName" => "app_name".into(),
        "remoteAddress" => "remote_address".into(),
        _ => format!("context[\"{context_name}\"]"),
    }
}

#[cfg(test)]
mod tests {
    use crate::strategy_parsing::compile_rule;

    use super::*;
    use std::collections::HashMap;
    use test_case::test_case;

    #[test]
    fn strategy_with_no_constraints_has_no_effect() {
        let mut parameters = HashMap::new();
        parameters.insert("userIds".into(), "123, 222, 88".into());

        let strategy = Strategy {
            name: "userWithId".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(output, "user_id in [\"123\",\"222\",\"88\"]".to_string())
    }

    #[test]
    fn strategy_with_empty_constraints_has_no_effect() {
        let mut parameters = HashMap::new();
        parameters.insert("userIds".into(), "123, 222, 88".into());

        let strategy = Strategy {
            name: "userWithId".into(),
            parameters: Some(parameters),
            constraints: Some(vec![]),
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(output, "user_id in [\"123\",\"222\",\"88\"]".to_string())
    }

    #[test]
    fn adds_parenthesis_to_constrained_strategy() {
        let constraint = Constraint {
            context_name: "userId".into(),
            values: Some(vec!["7".into()]),
            value: None,
            operator: Operator::In,
            case_insensitive: false,
            inverted: false,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint]),
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(output, "(true and (user_id in [\"7\"]))".to_string())
    }

    #[test]
    fn multiple_constraints_are_chained_with_ands() {
        let constraint = Constraint {
            context_name: "userId".into(),
            values: Some(vec!["7".into()]),
            value: None,
            operator: Operator::In,
            case_insensitive: false,
            inverted: false,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint.clone(), constraint]),
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output,
            "(true and (user_id in [\"7\"] and user_id in [\"7\"]))".to_string()
        )
    }

    #[test]
    fn multiple_strategies_are_chained_with_ors() {
        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy.clone(), strategy], &HashMap::new());
        assert_eq!(output, "true or true".to_string())
    }

    #[test]
    fn multiple_strategies_with_multiple_constraints_have_correct_order_of_operations() {
        let constraint = Constraint {
            context_name: "userId".into(),
            values: Some(vec!["7".into()]),
            value: None,
            operator: Operator::In,
            case_insensitive: false,
            inverted: false,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint.clone(), constraint]),
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy.clone(), strategy], &HashMap::new());
        assert_eq!(output.as_str(), "(true and (user_id in [\"7\"] and user_id in [\"7\"])) or (true and (user_id in [\"7\"] and user_id in [\"7\"]))")
    }

    #[test]
    fn no_strategy_is_always_true() {
        let output = upgrade(&vec![], &HashMap::new());
        assert_eq!(output.as_str(), "true")
    }

    #[test]
    fn upgrades_arbitrary_context_correctly() {
        let constraint = Constraint {
            context_name: "country".into(),
            values: Some(vec!["norway".into()]),
            value: None,
            operator: Operator::In,
            case_insensitive: false,
            inverted: false,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint]),
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output.as_str(),
            "(true and (context[\"country\"] in [\"norway\"]))"
        )
    }

    #[test]
    fn upgrades_flexible_rollout_with_all_parameters() {
        let mut parameters = HashMap::new();

        parameters.insert("rollout".into(), "55".into());
        parameters.insert("stickiness".into(), "userId".into());
        parameters.insert("groupId".into(), "Feature.flexibleRollout.userId.55".into());

        let strategy = Strategy {
            name: "flexibleRollout".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output.as_str(),
            "55% sticky on user_id with group_id of \"Feature.flexibleRollout.userId.55\""
        );
    }

    #[test]
    fn upgrades_flexible_rollout_without_group_id() {
        let mut parameters = HashMap::new();

        parameters.insert("rollout".into(), "55".into());
        parameters.insert("stickiness".into(), "userId".into());

        let strategy = Strategy {
            name: "flexibleRollout".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(output.as_str(), "55% sticky on user_id");
    }

    #[test]
    fn upgrades_flexible_rollout_without_stickiness() {
        let mut parameters = HashMap::new();

        parameters.insert("rollout".into(), "55".into());
        parameters.insert("groupId".into(), "Feature.flexibleRollout.userId.55".into());

        let strategy = Strategy {
            name: "flexibleRollout".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output.as_str(),
            "55% sticky on user_id | session_id | random with group_id of \"Feature.flexibleRollout.userId.55\""
        );
    }

    #[test]
    fn upgrades_flexible_rollout_with_only_rollout() {
        let mut parameters = HashMap::new();

        parameters.insert("rollout".into(), "55".into());

        let strategy = Strategy {
            name: "flexibleRollout".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output.as_str(),
            "55% sticky on user_id | session_id | random"
        );
    }

    #[test]
    fn upgrades_flexible_rollout_with_default_stickiness() {
        let mut parameters = HashMap::new();

        parameters.insert("rollout".into(), "55".into());
        parameters.insert("stickiness".into(), "default".into());
        parameters.insert("groupId".into(), "Feature.flexibleRollout.userId.55".into());

        let strategy = Strategy {
            name: "flexibleRollout".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output.as_str(),
            format!(
                "55% sticky on user_id | session_id | random with group_id of \"Feature.flexibleRollout.userId.55\""
            )
        );
    }

    #[test]
    fn upgrades_flexible_rollout_with_random_stickiness() {
        let mut parameters = HashMap::new();

        parameters.insert("rollout".into(), "55".into());
        parameters.insert("stickiness".into(), "random".into());
        parameters.insert("groupId".into(), "Feature.flexibleRollout.userId.55".into());

        let strategy = Strategy {
            name: "flexibleRollout".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
            sort_order: Some(1),
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert_eq!(
            output.as_str(),
            format!("55% sticky on random with group_id of \"Feature.flexibleRollout.userId.55\"")
        );
    }

    #[test_case(
        Operator::StrEndsWith,
        false,
        "user_id ends_with_any [\"some\", \"thing\"]"
    )]
    #[test_case(
        Operator::StrStartsWith,
        false,
        "user_id starts_with_any [\"some\", \"thing\"]"
    )]
    #[test_case(
        Operator::StrContains,
        false,
        "user_id contains_any [\"some\", \"thing\"]"
    )]
    #[test_case(
        Operator::StrEndsWith,
        true,
        "user_id ends_with_any_ignore_case [\"some\", \"thing\"]"
    )]
    #[test_case(
        Operator::StrStartsWith,
        true,
        "user_id starts_with_any_ignore_case [\"some\", \"thing\"]"
    )]
    #[test_case(
        Operator::StrContains,
        true,
        "user_id contains_any_ignore_case [\"some\", \"thing\"]"
    )]
    fn upgrades_string_list_operator(op: Operator, case_insensitive: bool, expected: &str) {
        let constraint = Constraint {
            context_name: "userId".into(),
            operator: op,
            case_insensitive,
            inverted: false,
            values: Some(vec!["some".into(), "thing".into()]),
            value: None,
        };
        let rule = upgrade_constraint(&constraint);
        assert_eq!(rule.as_str(), expected);
    }

    // These look janky but we don't actually care about the content
    // of the rule in the upgrader, only that the format is correct
    #[test_case(Operator::NumLte, "user_id <= 7")]
    #[test_case(Operator::NumLt, "user_id < 7")]
    #[test_case(Operator::NumGte, "user_id >= 7")]
    #[test_case(Operator::NumGt, "user_id > 7")]
    #[test_case(Operator::SemverLt, "user_id < 7")]
    #[test_case(Operator::SemverGt, "user_id > 7")]
    #[test_case(Operator::DateAfter, "user_id > 7")]
    #[test_case(Operator::DateBefore, "user_id < 7")]
    fn comparator_constraint(op: Operator, expected: &str) {
        let constraint = Constraint {
            context_name: "userId".into(),
            operator: op,
            case_insensitive: false,
            inverted: false,
            values: None,
            value: Some("7".into()),
        };
        let rule = upgrade_constraint(&constraint);
        assert_eq!(rule.as_str(), expected);
    }

    #[test_case(true, "!user_id <= 7")]
    #[test_case(false, "user_id <= 7")]
    fn handles_negation(is_inverted: bool, expected: &str) {
        let constraint = Constraint {
            context_name: "userId".into(),
            operator: Operator::NumLte,
            case_insensitive: false,
            inverted: is_inverted,
            values: None,
            value: Some("7".into()),
        };
        let rule = upgrade_constraint(&constraint);
        assert_eq!(rule.as_str(), expected);
    }

    #[test]
    fn upgrades_custom_strategy_to_a_named_lookup() {
        let custom_strategy = Strategy {
            name: "custom".into(),
            parameters: None,
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };

        let output = upgrade(&vec![custom_strategy], &HashMap::new());
        assert_eq!(output.as_str(), "external_value[\"customStrategy1\"]")
    }

    #[test]
    fn custom_strategy_count_is_only_incremented_for_custom_strategies() {
        let default_strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };

        let custom_strategy = Strategy {
            name: "custom".into(),
            parameters: None,
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };

        let output = upgrade(
            &vec![
                default_strategy.clone(),
                default_strategy.clone(),
                custom_strategy.clone(),
                custom_strategy.clone(),
                default_strategy.clone(),
                custom_strategy,
            ],
            &HashMap::new(),
        );
        assert_eq!(
            output.as_str(),
            "true or true or external_value[\"customStrategy1\"] or external_value[\"customStrategy2\"] or true or external_value[\"customStrategy3\"]"
        )
    }

    #[test]
    fn correctly_escapes_free_quotes_in_string_operators() {
        let constraint = Constraint {
            context_name: "userId".into(),
            operator: Operator::StrContains,
            case_insensitive: false,
            inverted: false,
            values: Some(vec!["some\"thing".into()]),
            value: None,
        };
        let rule = upgrade_constraint(&constraint);
        assert_eq!(rule.as_str(), "user_id contains_any [\"some\\\"thing\"]");
    }

    #[test]
    fn generates_valid_hostname_strategy_from_hostname_strategy() {
        let mut strategy_parameters = HashMap::new();
        strategy_parameters.insert("hostNames".into(), "DOS, pop-os".into());
        let strategy = Strategy {
            name: "applicationHostname".into(),
            parameters: Some(strategy_parameters),
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };

        let output = upgrade(&vec![strategy], &HashMap::new());
        assert!(compile_rule(&output).is_ok());
        assert_eq!(output.as_str(), "hostname in [\"DOS\", \"pop-os\"]");
    }

    #[test_case("gradualRolloutUserId")]
    #[test_case("gradualRolloutSessionId")]
    #[test_case("gradualRolloutRandom")]
    fn gradual_rollout_strategies_with_invalid_parameters_are_false(strategy_type: &str) {
        let strategy = Strategy {
            name: strategy_type.into(),
            parameters: Some(
                vec![("percentage".into(), "nonsense-value".into())]
                    .into_iter()
                    .collect(),
            ),
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };

        let rule = upgrade_strategy(&strategy, &HashMap::new(), 0);
        assert_eq!(rule.as_str(), "false");
    }

    #[test]
    fn remote_address_with_invalid_parameters_is_false() {
        let strategy = Strategy {
            name: "remoteAddress".into(),
            parameters: Some(HashMap::new()),
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };
        let rule = upgrade_strategy(&strategy, &HashMap::new(), 0);
        assert_eq!(rule.as_str(), "false");
    }

    #[test]
    fn user_id_strategy_with_no_user_ids_is_false() {
        let strategy = Strategy {
            name: "userWithId".into(),
            parameters: Some(HashMap::new()),
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };
        let rule = upgrade_strategy(&strategy, &HashMap::new(), 0);
        assert_eq!(rule.as_str(), "false");
    }

    #[test]
    fn remote_address_strategy_upgrades_to_ip_contains_constraint() {
        let strategy = Strategy {
            name: "remoteAddress".into(),
            parameters: Some(
                vec![("IPs".into(), "192.168.0.1, 192.168.0.2, 192.168.0.3".into())]
                    .into_iter()
                    .collect(),
            ),
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };
        let rule = upgrade_strategy(&strategy, &HashMap::new(), 0);
        assert_eq!(
            rule.as_str(),
            "remote_address contains_ip [\"192.168.0.1\", \"192.168.0.2\", \"192.168.0.3\"]"
        );
    }

    #[test]
    fn produces_compilable_rule_from_incorrectly_formatted_user_id_strategy_parameters() {
        let strategy = Strategy {
            name: "userWithId".into(),
            parameters: Some(
                vec![("userIds".into(), "[\"123\",\"456\",\"789\"]".into())]
                    .into_iter()
                    .collect(),
            ),
            constraints: None,
            segments: None,
            sort_order: None,
            variants: None,
        };

        let rule = upgrade_strategy(&strategy, &HashMap::new(), 0);

        assert!(compile_rule(&rule).is_ok());

        assert_eq!(rule.as_str(), "user_id in [\"[\\\"123\\\"\",\"\\\"456\\\"\",\"\\\"789\\\"]\"]");
    }
}
