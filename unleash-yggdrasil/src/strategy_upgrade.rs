use std::collections::HashMap;

use unleash_types::client_features::{Constraint, Operator, Segment, Strategy, StrategyVariant};

use crate::state::SdkError;

const DEFAULT_STICKINESS: &str = "user_id | session_id | random";

pub fn upgrade(strategies: &Vec<Strategy>, segment_map: &HashMap<i32, Segment>) -> String {
    if strategies.is_empty() {
        return "true".into();
    }
    let rule_text = strategies
        .iter()
        .map(|x| upgrade_strategy(x, segment_map))
        .collect::<Vec<String>>()
        .join(" or ");
    rule_text
}

pub fn build_variant_rules(
    strategies: &[Strategy],
    segment_map: &HashMap<i32, Segment>,
) -> Vec<(String, Vec<StrategyVariant>, String)> {
    strategies
        .iter()
        .filter(|strategy| strategy.variants.is_some())
        .map(|strategy| {
            (
                upgrade_strategy(strategy, segment_map),
                strategy.variants.clone().unwrap(),
                strategy
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("stickiness"))
                    .cloned()
                    .unwrap_or_else(|| "default".to_string()),
            )
        })
        .collect::<Vec<(String, Vec<StrategyVariant>, String)>>()
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

fn upgrade_strategy(strategy: &Strategy, segment_map: &HashMap<i32, Segment>) -> String {
    let strategy_rule = match strategy.name.as_str() {
        "default" => "true".into(),
        "userWithId" => upgrade_user_id_strategy(strategy),
        "gradualRolloutUserId" => upgrade_user_id_rollout_strategy(strategy),
        "gradualRolloutSessionId" => upgrade_session_id_rollout_strategy(strategy),
        "gradualRolloutRandom" => upgrade_random(strategy),
        "flexibleRollout" => upgrade_flexible_rollout_strategy(strategy),
        "remoteAddress" => upgrade_remote_address(strategy),
        _ => "true".into(),
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
    let rollout = strategy.get_param("rollout");
    match rollout {
        Some(rollout) => {
            //should probably validate at this point that the rollout looks like a percent

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
                .map(|id| format!("\"{}\"", id.trim()))
                .collect::<Vec<String>>()
                .join(",");
            format!("user_id in [{user_ids}]")
        }
        None => "".into(),
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
            format!("remote_address in [{ips}]")
        }
        None => "".into(),
    }
}

fn upgrade_session_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = strategy.get_param("percentage");
    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!("{percentage}% sticky on session_id with group_id of \"{group_id}\"")
        }
        _ => "".into(),
    }
}

fn upgrade_user_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = strategy.get_param("percentage");
    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!("{percentage}% sticky on user_id with group_id of \"{group_id}\"")
        }
        _ => "".into(),
    }
}

fn upgrade_random(strategy: &Strategy) -> String {
    match strategy.get_param("percentage") {
        Some(percent) => format!("random < {percent}"),
        None => "".into(),
    }
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
                    .map(|x| format!("\"{x}\""))
                    .collect::<Vec<String>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "".to_string());
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
}
