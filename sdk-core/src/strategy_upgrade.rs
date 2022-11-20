use crate::state::{Constraint, Operator, Strategy};

pub fn upgrade(strategies: &Vec<Strategy>) -> String {
    if strategies.is_empty() {
        return "true".into();
    }
    let rule_text = strategies
        .iter()
        .map(|x| upgrade_strategy(x))
        .collect::<Vec<String>>()
        .join(" or ");
    rule_text
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

fn upgrade_strategy(strategy: &Strategy) -> String {
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

    let constraints = upgrade_constraints(&strategy.constraints);
    match constraints {
        Some(constraints) => format!("({} and ({}))", strategy_rule, constraints),
        None => strategy_rule,
    }
}

fn upgrade_flexible_rollout_strategy(strategy: &Strategy) -> String {
    let rollout = strategy.get_param("rollout");
    match rollout {
        Some(rollout) => {
            //should probably validate at this point that the rollout looks like a percent

            let mut rule: String = format!("{}%", rollout);

            if let Some(stickiness) = strategy.get_param("stickiness") {
                if stickiness.as_str() != "default" {
                    rule = format!("{} sticky on {}", rule, upgrade_context_name(stickiness));
                }
            }

            if let Some(group_id) = strategy.get_param("groupId") {
                rule = format!("{} with group_id of \"{}\"", rule, group_id);
            }

            rule
        }
        None => "false".into(),
    }
}

fn upgrade_user_id_strategy(strategy: &Strategy) -> String {
    match strategy.get_param("userIds") {
        Some(user_ids) => format!("user_id in [{}]", user_ids),
        None => "".into(),
    }
}

fn upgrade_remote_address(strategy: &Strategy) -> String {
    match strategy.get_param("IPs") {
        Some(addresses) => {
            let ips = addresses
                .split(",")
                .collect::<Vec<&str>>()
                .iter()
                .map(|x| x.trim())
                .map(|x| format!("\"{}\"", x))
                .collect::<Vec<String>>()
                .join(", ");
            format!("remote_address in [{}]", ips)
        }
        None => "".into(),
    }
}

fn upgrade_session_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = strategy.get_param("percentage");
    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!(
                "{}% sticky on session_id with group_id of \"{}\"",
                percentage, group_id
            )
        }
        _ => "".into(),
    }
}

fn upgrade_user_id_rollout_strategy(strategy: &Strategy) -> String {
    let percentage = strategy.get_param("percentage");
    let group_id = strategy.get_param("groupId");
    match (percentage, group_id) {
        (Some(percentage), Some(group_id)) => {
            format!(
                "{}% sticky on user_id with group_id of \"{}\"",
                percentage, group_id
            )
        }
        _ => "".into(),
    }
}

fn upgrade_random(strategy: &Strategy) -> String {
    match strategy.get_param("percentage") {
        Some(percent) => format!("random() < {}", percent),
        None => "".into(),
    }
}

fn upgrade_constraints(constraints: &Option<Vec<Constraint>>) -> Option<String> {
    if constraints.is_none() {
        return None;
    };
    let constraints = constraints.as_ref().unwrap();
    if constraints.is_empty() {
        return None;
    };
    let constraint_rules = constraints
        .iter()
        .map(|x| upgrade_constraint(x))
        .collect::<Vec<String>>();
    let squashed_rules = constraint_rules.join(" and ");
    Some(format!("{}", squashed_rules))
}

fn upgrade_constraint(constraint: &Constraint) -> String {
    let context_name = upgrade_context_name(&constraint.context_name);
    let op = upgrade_operator(&constraint.operator);
    let values = constraint
        .values
        .clone()
        .map(|values| {
            values
                .iter()
                .map(|x| format!("\"{}\"", x))
                .collect::<Vec<String>>()
                .join(", ")
        })
        .unwrap_or("".to_string());

    format!("{context_name} {op} [{values}]")
}

fn upgrade_operator(op: &Operator) -> String {
    match op {
        Operator::In => "in",
        Operator::NotIn => "not_in",
    }
    .into()
}

fn upgrade_context_name(context_name: &str) -> String {
    match context_name {
        "userId" => "user_id".into(),
        "sessionId" => "session_id".into(),
        "environment" => "environment".into(),
        "appName" => "app_name".into(),
        "remoteAddress" => "remote_address".into(),
        _ => format!("context[\"{}\"]", context_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn strategy_with_no_constraints_has_no_effect() {
        let mut parameters = HashMap::new();
        parameters.insert("userIds".into(), "123, 222, 88".into());

        let strategy = Strategy {
            name: "userWithId".into(),
            parameters: Some(parameters),
            constraints: None,
            segments: None,
        };

        let output = upgrade(&vec![strategy]);
        assert_eq!(output, "user_id in [123, 222, 88]".to_string())
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
        };

        let output = upgrade(&vec![strategy]);
        assert_eq!(output, "user_id in [123, 222, 88]".to_string())
    }

    #[test]
    fn adds_parenthesis_to_constrained_strategy() {
        let constraint = Constraint {
            context_name: "userId".into(),
            values: Some(vec!["7".into()]),
            value: None,
            operator: Operator::In,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint]),
            segments: None,
        };

        let output = upgrade(&vec![strategy]);
        assert_eq!(output, "(true and (user_id in [\"7\"]))".to_string())
    }

    #[test]
    fn multiple_constraints_are_chained_with_ands() {
        let constraint = Constraint {
            context_name: "userId".into(),
            values: Some(vec!["7".into()]),
            value: None,
            operator: Operator::In,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint.clone(), constraint.clone()]),
            segments: None,
        };

        let output = upgrade(&vec![strategy]);
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
        };

        let output = upgrade(&vec![strategy.clone(), strategy.clone()]);
        assert_eq!(output, "true or true".to_string())
    }

    #[test]
    fn multiple_strategies_with_multiple_constraints_have_correct_order_of_operations() {
        let constraint = Constraint {
            context_name: "userId".into(),
            values: Some(vec!["7".into()]),
            value: None,
            operator: Operator::In,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint.clone(), constraint.clone()]),
            segments: None,
        };

        let output = upgrade(&vec![strategy.clone(), strategy.clone()]);
        assert_eq!(output.as_str(), "(true and (user_id in [\"7\"] and user_id in [\"7\"])) or (true and (user_id in [\"7\"] and user_id in [\"7\"]))")
    }

    #[test]
    fn no_strategy_is_always_true() {
        let output = upgrade(&vec![]);
        assert_eq!(output.as_str(), "true")
    }

    #[test]
    fn upgrades_arbitrary_context_correctly() {
        let constraint = Constraint {
            context_name: "country".into(),
            values: Some(vec!["norway".into()]),
            value: None,
            operator: Operator::In,
        };

        let strategy = Strategy {
            name: "default".into(),
            parameters: None,
            constraints: Some(vec![constraint]),
            segments: None,
        };

        let output = upgrade(&vec![strategy]);
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
        };

        let output = upgrade(&vec![strategy]);
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
        };

        let output = upgrade(&vec![strategy]);
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
        };

        let output = upgrade(&vec![strategy]);
        assert_eq!(
            output.as_str(),
            "55% with group_id of \"Feature.flexibleRollout.userId.55\""
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
        };

        let output = upgrade(&vec![strategy]);
        assert_eq!(output.as_str(), "55%");
    }
}
