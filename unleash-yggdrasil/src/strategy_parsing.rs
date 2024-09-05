extern crate pest;

use std::collections::HashSet;
use std::env;
use std::io::Cursor;
use std::net::IpAddr;
use std::num::ParseFloatError;
use std::str::FromStr;

use crate::sendable_closures::{SendableContextResolver, SendableFragment};
use crate::state::SdkError;
use crate::EnrichedContext as Context;
use chrono::{DateTime, Utc};
use hostname;
use ipnetwork::{IpNetwork, IpNetworkError};
use murmur3::murmur3_32;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use rand::Rng;
use semver::Version;

#[derive(Parser)]
#[grammar = "strategy_grammar.pest"]
struct Strategy;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(and, Left))
            .op(Op::infix(or, Left))
    };
}

pub fn normalized_hash(
    group: &str,
    identifier: &str,
    modulus: u32,
    seed: u32,
) -> std::io::Result<u32> {
    let mut reader = Cursor::new(format!("{}:{}", &group, &identifier));
    murmur3_32(&mut reader, seed).map(|hash_result| hash_result % modulus + 1)
}

fn drain<const N: usize>(node: Pairs<Rule>) -> CompileResult<[Pair<Rule>; N]> {
    Ok(drain_partial(node)?.0)
}

fn drain_partial<const N: usize>(
    mut node: Pairs<Rule>,
) -> CompileResult<([Pair<Rule>; N], Pairs<Rule>)> {
    let mut results = vec![];
    for _ in 0..N {
        results.push(node.next().ok_or_else(|| {
            SdkError::StrategyParseError(format!(
                "Expected at least {N} elements when parsing the following pairs {node:?}"
            ))
        })?);
    }
    let array: [Pair<Rule>; N] = results.try_into().map_err(|_| {
        SdkError::StrategyParseError(format!(
            "Expected exactly {N} elements when parsing the following pairs {node:?}"
        ))
    })?;

    Ok((array, node))
}

type CompileResult<T> = Result<T, SdkError>;

pub type RuleFragment = Box<dyn SendableFragment + Send + Sync + 'static>;

pub type ContextResolver = Box<dyn SendableContextResolver + Send + Sync + 'static>;

trait Invertible {
    fn invert(&self, inverted: bool) -> bool;
}

impl Invertible for bool {
    fn invert(&self, inverted: bool) -> bool {
        if inverted {
            !self
        } else {
            *self
        }
    }
}

#[derive(Debug, Clone)]
enum OrdinalComparator {
    Lte,
    Lt,
    Gte,
    Gt,
    Eq,
}

#[derive(Debug, Clone)]
enum ContentComparator {
    In,
    NotIn,
}

#[derive(Debug, Clone)]
enum StringComparator {
    StartsWith,
    EndsWith,
    Contains,
}

struct StringComparatorType {
    ignore_case: bool,
    comparator_type: StringComparator,
}

//Context lifting properties - these resolve properties from the context
fn context_value(node: Pairs<Rule>) -> ContextResolver {
    let [child] = drain(node)
        .expect("Context node is empty, this should only happen if the grammar is missing");

    match child.as_rule() {
        Rule::user_id => Box::new(|context: &Context| context.user_id.clone()),
        Rule::app_name => Box::new(|context: &Context| context.app_name.clone()),
        Rule::environment => Box::new(|context: &Context| context.environment.clone()),
        Rule::session_id => Box::new(|context: &Context| context.session_id.clone()),
        Rule::remote_address => Box::new(|context: &Context| context.remote_address.clone()),
        Rule::current_time => Box::new(|context: &Context| context.current_time.clone()),
        Rule::random => {
            Box::new(|_: &Context| Some(rand::thread_rng().gen_range(0..99).to_string()))
        }
        Rule::property => context_property(child.into_inner()),
        _ => unreachable!(),
    }
}

pub(crate) fn coalesce_context_property(node: Pairs<Rule>) -> ContextResolver {
    let mut stickiness_resolvers = vec![];

    for child in node {
        stickiness_resolvers.push(context_value(child.into_inner()))
    }

    Box::new(move |context: &Context| {
        stickiness_resolvers
            .iter()
            .find_map(|resolver| resolver(context))
    })
}

fn context_property(node: Pairs<Rule>) -> ContextResolver {
    let [content_node] = drain(node)
        .expect("Context node is empty, this should only happen if the grammar is missing");
    let mut chars = content_node.as_str().chars();
    chars.next();
    chars.next_back();
    let context_name = chars.as_str().to_string();

    Box::new(move |context: &Context| -> Option<String> {
        match &context.properties {
            Some(props) => props.get(&context_name).cloned(),
            None => None,
        }
    })
}

fn to_ordinal_comparator(node: Pair<Rule>) -> OrdinalComparator {
    match node.as_str() {
        "<" => OrdinalComparator::Lt,
        "<=" => OrdinalComparator::Lte,
        "==" => OrdinalComparator::Eq,
        ">" => OrdinalComparator::Gt,
        ">=" => OrdinalComparator::Gte,
        _ => unreachable!(),
    }
}

fn to_content_comparator(node: Pair<Rule>) -> ContentComparator {
    match node.as_str() {
        "in" => ContentComparator::In,
        "not_in" => ContentComparator::NotIn,
        _ => unreachable!(),
    }
}

fn to_string_comparator(node: Pair<Rule>) -> StringComparatorType {
    match node.as_str() {
        "starts_with_any" => StringComparatorType {
            ignore_case: false,
            comparator_type: StringComparator::StartsWith,
        },
        "ends_with_any" => StringComparatorType {
            ignore_case: false,
            comparator_type: StringComparator::EndsWith,
        },
        "contains_any" => StringComparatorType {
            ignore_case: false,
            comparator_type: StringComparator::Contains,
        },
        "starts_with_any_ignore_case" => StringComparatorType {
            ignore_case: true,
            comparator_type: StringComparator::StartsWith,
        },
        "ends_with_any_ignore_case" => StringComparatorType {
            ignore_case: true,
            comparator_type: StringComparator::EndsWith,
        },
        "contains_any_ignore_case" => StringComparatorType {
            ignore_case: true,
            comparator_type: StringComparator::Contains,
        },
        _ => unreachable!(),
    }
}

fn numeric(node: Pair<Rule>) -> CompileResult<f64> {
    let value = node.as_str();
    value.parse::<f64>().map_err(|e| {
        SdkError::StrategyParseError(format!("Failed to compile {value} as a numeric value: {e}"))
    })
}

fn date(node: Pair<Rule>) -> CompileResult<DateTime<Utc>> {
    let value = node.as_str();
    value.parse::<DateTime<Utc>>().map_err(|e| {
        SdkError::StrategyParseError(format!("Failed to compile {value} as a date value: {e}"))
    })
}

fn semver(node: Pair<Rule>) -> CompileResult<Version> {
    let value = node.as_str();
    Version::parse(value).map_err(|e| {
        SdkError::StrategyParseError(format!("Failed to compile {value} as a semver value: {e}"))
    })
}

fn percentage(node: Pair<Rule>) -> CompileResult<u8> {
    let mut chars = node.as_str().chars();
    chars.next_back();
    let value = chars.as_str();
    value.parse::<u8>().map_err(|e| {
        SdkError::StrategyParseError(format!(
            "Failed to compile {value} as a percentage value: {e}"
        ))
    })
}

fn group_id_param(node: Pairs<Rule>) -> CompileResult<String> {
    let [content_node] = drain(node)?;
    Ok(string(content_node))
}

fn string(node: Pair<Rule>) -> String {
    let mut chars = node.as_str().chars();
    chars.next();
    chars.next_back();
    let string: String = chars.as_str().into();
    string.replace("\\\"", "\"")
}

fn ip(node: Pair<Rule>) -> Result<IpNetwork, IpNetworkError> {
    IpNetwork::from_str(&string(node))
}

//Constraints
fn numeric_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [context_getter, ordinal_operation, number] = drain(node)?;

    let context_getter = context_value(context_getter.into_inner());
    let ordinal_operation = to_ordinal_comparator(ordinal_operation);
    let number = numeric(number)?;

    Ok(Box::new(move |context: &Context| {
        let context_value = context_getter(context);
        match context_value {
            Some(context_value) => {
                let Ok(context_value) = context_value.parse::<f64>() else {
                    return false;
                };

                match ordinal_operation {
                    OrdinalComparator::Lte => context_value <= number,
                    OrdinalComparator::Lt => context_value < number,
                    OrdinalComparator::Gte => context_value >= number,
                    OrdinalComparator::Gt => context_value > number,
                    OrdinalComparator::Eq => (context_value - number).abs() < f64::EPSILON,
                }
            }
            None => false,
        }
    }))
}

fn date_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [context_getter_node, ordinal_operation_node, date_node] = drain(node)?;

    let context_getter = context_value(context_getter_node.into_inner());
    let ordinal_operation = to_ordinal_comparator(ordinal_operation_node);
    let date = date(date_node)?;

    Ok(Box::new(move |context: &Context| {
        let context_value = context_getter(context);
        match context_value {
            Some(context_value) => {
                let context_value = context_value.parse::<DateTime<Utc>>();

                let Ok(context_value) = context_value else {
                    return false;
                };

                match ordinal_operation {
                    OrdinalComparator::Lte => context_value <= date,
                    OrdinalComparator::Lt => context_value < date,
                    OrdinalComparator::Gte => context_value >= date,
                    OrdinalComparator::Gt => context_value > date,
                    OrdinalComparator::Eq => context_value == date,
                }
            }
            None => false,
        }
    }))
}

fn semver_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let children: [Pair<Rule>; 3] = drain(node)?;
    let [context_getter_node, ordinal_operation_node, semver_node] = children;

    let context_getter = context_value(context_getter_node.into_inner());
    let ordinal_operation = to_ordinal_comparator(ordinal_operation_node);
    let semver = semver(semver_node)?;

    Ok(Box::new(move |context: &Context| {
        let context_value = context_getter(context);

        match context_value {
            Some(context_value) => {
                let context_value = context_value.parse::<Version>();

                let Ok(context_value) = context_value else {
                    return false;
                };

                match ordinal_operation {
                    OrdinalComparator::Lte => context_value <= semver,
                    OrdinalComparator::Lt => context_value < semver,
                    OrdinalComparator::Gte => context_value >= semver,
                    OrdinalComparator::Gt => context_value > semver,
                    OrdinalComparator::Eq => context_value == semver,
                }
            }
            None => false,
        }
    }))
}

fn rollout_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let (children, mut node) = drain_partial(node)?;
    let [rollout_node, stickiness_node] = children;

    let percent_rollout = percentage(rollout_node)?;

    let stickiness_resolver = coalesce_context_property(stickiness_node.into_inner());
    let group_id = node
        .next()
        .map(|node| group_id_param(node.into_inner()))
        .transpose()?;

    Ok(Box::new(move |context: &Context| {
        let stickiness = stickiness_resolver(context);

        if stickiness.is_none() {
            return false;
        }

        let group_id = match &group_id {
            Some(group_id) => group_id.clone(),
            None => context.toggle_name.clone(),
        };

        let hash = if let Some(stickiness) = stickiness {
            normalized_hash(&group_id, &stickiness, 100, 0)
        } else {
            // The original code does something different here - if we're using the
            // default strategy it generates a string of a number between 1 and 101
            // then uses that as the hash. This instead doesn't do that and just
            // uses a random number in place of the hash. Pretty sure it's the same thing
            Ok(rand::thread_rng().gen_range(0..99) as u32)
        };

        if let Ok(hash) = hash {
            hash <= percent_rollout.into()
        } else {
            // This should probably never occur, it only happens if we
            // don't feed enough input to the hashing function
            false
        }
    }))
}

fn get_hostname() -> CompileResult<String> {
    //This is primarily for testing purposes
    if let Ok(hostname_env) = env::var("hostname") {
        return Ok(hostname_env);
    }

    hostname::get()
        .map_err(|_| SdkError::StrategyEvaluationError)
        .and_then(|os_str| {
            os_str
                .into_string()
                .map_err(|_| SdkError::StrategyEvaluationError)
        })
}

fn hostname_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [hostname_node] = drain(node)?;

    let target_hostnames: HashSet<String> = harvest_string_list(hostname_node.into_inner())
        .iter()
        .map(|x| x.to_lowercase())
        .collect();

    Ok(Box::new(move |_: &Context| match get_hostname() {
        Ok(hostname) => target_hostnames.contains(&hostname.to_lowercase()),
        Err(_) => false,
    }))
}

fn ip_matching_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [context_node, ip_node] = drain(node)?;

    let context_getter = context_value(context_node.into_inner());
    let ip_list = harvest_ip_list(ip_node.into_inner());

    Ok(Box::new(move |context| {
        if let Some(context_value) = context_getter(context) {
            if let Ok(context_ip) = context_value.parse::<IpAddr>() {
                return ip_list.iter().any(|range| range.contains(context_ip));
            }
        }
        false
    }))
}

fn list_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [context_node, comparator_node, list_node] = drain(node)?;

    let context_getter = context_value(context_node.into_inner());
    let comparator = to_content_comparator(comparator_node);
    let list = list_node;

    Ok(match list.as_rule() {
        Rule::empty_list => Box::new(move |_context: &Context| match comparator {
            ContentComparator::In => false,
            ContentComparator::NotIn => true,
        }),
        Rule::numeric_list => {
            let values = harvest_list(list.into_inner())?;
            Box::new(move |context: &Context| {
                let context_value = context_getter(context);
                match context_value {
                    Some(context_value) => {
                        let Ok(context_value) = context_value.parse::<f64>() else {
                            return false;
                        };
                        match comparator {
                            ContentComparator::In => values.contains(&context_value),
                            ContentComparator::NotIn => !values.contains(&context_value),
                        }
                    }
                    None => false,
                }
            })
        }
        Rule::string_list => {
            let values = harvest_set(list.into_inner());
            Box::new(move |context: &Context| {
                let context_value = context_getter(context);

                match comparator {
                    ContentComparator::In => match context_value {
                        Some(context_value) => values.contains(&context_value),
                        None => false,
                    },
                    ContentComparator::NotIn => match context_value {
                        Some(context_value) => !values.contains(&context_value),
                        None => true,
                    },
                }
            })
        }
        _ => unreachable!(),
    })
}

fn external_value(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [index_node] = drain(node)?;
    let strategy_index = string(index_node);
    Ok(Box::new(move |context| {
        context
            .external_results
            .as_ref()
            .and_then(|strategy_results| strategy_results.get(&strategy_index))
            .copied()
            .unwrap_or(false)
    }))
}

fn harvest_set(node: Pairs<Rule>) -> HashSet<String> {
    node.into_iter().map(string).collect::<HashSet<String>>()
}

fn harvest_string_list(node: Pairs<Rule>) -> Vec<String> {
    node.into_iter().map(string).collect::<Vec<String>>()
}

fn harvest_list(node: Pairs<Rule>) -> CompileResult<Vec<f64>> {
    let nodes: Result<Vec<f64>, ParseFloatError> =
        node.into_iter().map(|x| x.as_str().parse()).collect();

    nodes.map_err(|e| {
        SdkError::StrategyParseError(format!("Failed to compile list as a numeric list: {e}"))
    })
}

fn harvest_ip_list(node: Pairs<Rule>) -> Vec<IpNetwork> {
    node.into_iter()
        .filter_map(|n| match ip(n) {
            Ok(net) => Some(net),
            Err(_) => None,
        })
        .collect()
}

fn default_strategy_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let value = node.as_str();
    let enabled: bool = value.parse().map_err(|e| {
        SdkError::StrategyParseError(format!("Failed to compile {value} as a boolean value: {e}"))
    })?;
    Ok(Box::new(move |_: &Context| enabled))
}

fn string_fragment_constraint(node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let [context_getter_node, comparator_node, list_node] = drain(node)?;

    let context_getter = context_value(context_getter_node.into_inner());
    let comparator_details = to_string_comparator(comparator_node);
    let comparator = comparator_details.comparator_type;
    let ignore_case = comparator_details.ignore_case;

    let mut list = harvest_string_list(list_node.into_inner());

    if ignore_case {
        list = list.into_iter().map(|item| item.to_lowercase()).collect();
    };

    Ok(Box::new(move |context: &Context| {
        let mut value = context_getter(context);
        if ignore_case {
            value = value.map(|value| value.to_lowercase())
        }
        if let Some(value) = value {
            match comparator {
                StringComparator::Contains => list.iter().any(|item| value.contains(item)),
                StringComparator::StartsWith => list.iter().any(|item| value.starts_with(item)),
                StringComparator::EndsWith => list.iter().any(|item| value.ends_with(item)),
            }
        } else {
            false
        }
    }))
}

fn constraint(mut node: Pairs<Rule>) -> CompileResult<RuleFragment> {
    let first = node.next();
    let second = node.next();

    let (inverted, child) = match (first, second) {
        (Some(_), Some(second)) => (true, second),
        (Some(first), None) => (false, first),
        _ => unreachable!(),
    };

    let constraint = match child.as_rule() {
        Rule::date_constraint => date_constraint(child.into_inner()),
        Rule::numeric_constraint => numeric_constraint(child.into_inner()),
        Rule::semver_constraint => semver_constraint(child.into_inner()),
        Rule::rollout_constraint => rollout_constraint(child.into_inner()), //TODO: Do we need to support inversion here?
        Rule::default_strategy_constraint => default_strategy_constraint(child.into_inner()),
        Rule::string_fragment_constraint => string_fragment_constraint(child.into_inner()),
        Rule::list_constraint => list_constraint(child.into_inner()),
        Rule::hostname_constraint => hostname_constraint(child.into_inner()),
        Rule::external_value => external_value(child.into_inner()),
        Rule::ip_constraint => ip_matching_constraint(child.into_inner()),
        _ => unreachable!(),
    }?;

    Ok(Box::new(move |context: &Context| {
        constraint(context).invert(inverted)
    }))
}

fn eval(expression: Pairs<Rule>) -> CompileResult<RuleFragment> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::constraint => constraint(primary.into_inner()),
            Rule::expr => eval(primary.into_inner()),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::and => Ok(Box::new({
                let lhs = lhs?;
                let rhs = rhs?;

                move |context: &Context| -> bool { lhs(context) && rhs(context) }
            })),
            Rule::or => Ok(Box::new({
                let lhs = lhs?;
                let rhs = rhs?;

                move |context: &Context| -> bool { lhs(context) || rhs(context) }
            })),
            _ => unreachable!(),
        })
        .parse(expression)
}

pub fn compile_rule(rule: &str) -> CompileResult<RuleFragment> {
    let nodes = Strategy::parse(Rule::strategy, rule)
        .map_err(|e| SdkError::StrategyParseError(format!("Failed to parse rule {rule}: {e}")))?;

    let [strategy_node] = drain(nodes)?;
    let rule = eval(strategy_node.into_inner())?;

    Ok(rule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use test_case::test_case;

    fn context_from_user_id(user_id: &str) -> Context {
        Context {
            user_id: Some(user_id.into()),
            current_time: None,
            properties: Some(HashMap::new()),
            session_id: None,
            environment: None,
            app_name: None,
            remote_address: None,
            toggle_name: "".into(),
            external_results: None,
        }
    }

    // This needs the toggle name for it to actually be useful for the parsing engine so it makes no sense
    // to have a default implementation exposed in the library but it does make testing a lot easier for this
    // test module
    #[allow(clippy::derivable_impls)]
    impl Default for Context {
        fn default() -> Self {
            Self {
                user_id: Default::default(),
                session_id: Default::default(),
                environment: Default::default(),
                app_name: Default::default(),
                current_time: Default::default(),
                remote_address: Default::default(),
                properties: Default::default(),
                toggle_name: Default::default(),
                external_results: None,
            }
        }
    }

    #[test_case("1", "user_id == 1 and (user_id > 1 and user_id < 1)", false)]
    #[test_case("9", "user_id == 9 or (user_id > 5 and user_id < 5)", true)]
    #[test_case("4", "user_id < 5 or user_id > 5", true)]
    #[test_case("2", "user_id < 1 and user_id > 1", false)]
    #[test_case("5", "user_id < 6 and user_id > 2", true)]
    #[test_case("0", "(true and (true and (true)))", true)]
    #[test_case("0", "(true and (true and (false)))", false)]
    fn can_chain_operators(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    #[test_case("6", "user_id < 5", false)]
    #[test_case("6.0", "user_id < 5", false)]
    #[test_case("5", "user_id < 5", false)]
    #[test_case("4", "user_id < 5", true)]
    #[test_case("4.0", "user_id < 5", true)]
    #[test_case("-4.0", "user_id < -5", false)]
    #[test_case("-4.0", "user_id < -3", true)]
    #[test_case("-4.0", "user_id < -4", false)]
    #[test_case("6", "user_id < 5.0", false)]
    #[test_case("6.0", "user_id < 5.0", false)]
    #[test_case("5", "user_id < 5.0", false)]
    #[test_case("4", "user_id < 5.0", true)]
    #[test_case("4.0", "user_id < 5.0", true)]
    #[test_case("-4.0", "user_id < -5.0", false)]
    #[test_case("-4.0", "user_id < -3.0", true)]
    #[test_case("-4.0", "user_id < -4.0", false)]
    fn test_numeric_lt(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    #[test_case("3", "user_id <= 4", true)]
    #[test_case("3", "user_id <= 3", true)]
    #[test_case("3", "user_id <= 2", false)]
    #[test_case("-4.0", "user_id <= -5.2", false)]
    fn test_numeric_lte(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    #[test_case("5", "user_id > 4", true)]
    #[test_case("4", "user_id > 4", false)]
    #[test_case("3", "user_id > 4", false)]
    fn test_numeric_gt(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    #[test_case("3", "user_id >= 4", false)]
    #[test_case("3", "user_id >= 3", true)]
    #[test_case("3", "user_id >= 2", true)]
    #[test_case("-4.0", "user_id >= -5.2", true)]
    fn test_numeric_gte(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    #[test_case("3.0", "user_id == 3.0", true)]
    #[test_case("3", "user_id == 3.0", true)]
    #[test_case("3.0", "user_id == 3", true)]
    #[test_case("-3", "user_id == -3", true)]
    #[test_case("4", "user_id == 4", true)]
    #[test_case("4", "user_id == 3", false)]
    fn test_numeric_eq(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    #[test_case("30.0.0", "user_id == 30.0.0", true)]
    #[test_case("3.0.0", "user_id > 3.0.0", false)]
    #[test_case("3.0.0-beta", "user_id == 3.0.0-beta", true)]
    #[test_case("3.0.0-beta.2", "user_id > 3.0.0-beta.1", true)]
    #[test_case("3.0.0-beta", "user_id > 3.0.0-alpha", true)]
    #[test_case("3.0.0-beta", "user_id < 3.0.0-alpha", false)]
    #[test_case("3.0.0", "user_id > 3.0.0-alpha", true)]
    #[test_case("3.0.0", "user_id >= 3.0.0", true)]
    #[test_case("3.0.0-beta.stuff", "user_id == 3.0.0-beta.stuff", true)]
    #[test_case("3.0.0-beta.stuff+build1", "user_id == 3.0.0-beta.stuff+build1", true)]
    fn test_semver_gt(user_id: &str, rule: &str, expected: bool) {
        run_test(user_id, rule, expected);
    }

    fn run_test(user_id: &str, rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id(user_id);

        assert_eq!(rule(&context), expected);
    }

    #[test]
    fn test_random_parses() {
        let rule = "random < 100";
        let rule = compile_rule(rule).expect("");
        let context = Context::default();

        assert!(rule(&context));
    }

    #[test_case("true", true)]
    #[test_case("false", false)]
    #[test_case("true or false", true)]
    #[test_case("true and true", true)]
    #[test_case("false or false", false)]
    #[test_case("false or true", true)]
    #[test_case("false and true", false)]
    fn run_boolean_constraint(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("6");

        assert_eq!(rule(&context), expected);
    }

    #[test_case("100% sticky on random", true)]
    #[test_case("99% sticky on random", true)]
    fn run_rollout_test(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("6");

        assert_eq!(rule(&context), expected);
    }

    #[test_case("context[\"penguins\"] == 7", true)]
    #[test_case("context[\"squirrels\"] == -2", true)]
    #[test_case("context[\"squirrels\"] == 7", false)]
    fn gets_context_property(rule: &str, expected: bool) {
        let mut context_property = HashMap::new();
        context_property.insert("penguins".into(), "7".into());
        context_property.insert("squirrels".into(), "-2".into());

        let context = Context {
            current_time: None,
            user_id: Some("6".into()),
            properties: Some(context_property),
            session_id: None,
            environment: None,
            app_name: None,
            remote_address: None,
            toggle_name: "".into(),
            external_results: None,
        };

        let rule = compile_rule(rule).expect("");

        assert_eq!(rule(&context), expected);
    }

    #[test_case(
        "55% sticky on user_id with group_id of \"Feature.flexibleRollout.userId.55\"",
        true
    )]
    fn run_rollout_test_with_group_id(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("25");

        assert_eq!(rule(&context), expected);
    }

    #[test_case("100% sticky on user_id", true)]
    fn run_rollout_test_with_stickiness(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("6");

        assert_eq!(rule(&context), expected);
    }

    #[test_case("sticky on user_id | session_id", Some("42"); "Picks first if present")]
    #[test_case("sticky on environment | app_name | session_id", Some("7"); "Falls back multiple times to non null values")]
    #[test_case("sticky on environment | context[\"lies\"] | context[\"present\"] ", Some("1"); "Respects custom context")]
    #[test_case("sticky on environment | context[\"lies\"]", None; "Falls back to None eventually")]
    fn run_null_coalesce_test(rule: &str, expected: Option<&str>) {
        let expected = expected.map(|s| s.to_string());

        let mut props = HashMap::new();
        props.insert("present".into(), "1".into());

        let context = Context {
            user_id: Some("42".into()),
            session_id: Some("7".into()),
            properties: Some(props),
            ..Context::default()
        };

        let mut parse_result = Strategy::parse(Rule::stickiness_param, rule).unwrap();
        let stickiness_lookup =
            coalesce_context_property(parse_result.next().unwrap().into_inner());
        let result: Option<String> = stickiness_lookup(&context);

        assert_eq!(result, expected);
    }

    #[test_case("user_id in [1, 3, 6]", true)]
    #[test_case("user_id in [1, 3, 5]", false)]
    #[test_case("user_id not_in [1, 3, 5]", true)]
    #[test_case("user_id in [\"dfsfsd\"]", false)]
    #[test_case("user_id not_in [\"dfsfsd\"]", true)]
    #[test_case("!user_id in [1, 3, 6]", false)]
    #[test_case("!user_id not_in [1, 3, 6]", true)]
    #[test_case("!context[\"i_do_not_exist\"] in [\"dfsfsd\"]", true)]
    #[test_case("!context[\"i_do_not_exist\"] not_in [\"dfsfsd\"]", false)]
    fn run_numeric_list_test(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("6");

        assert_eq!(rule(&context), expected);
    }

    #[test]
    fn not_in_is_always_true_when_no_context_value() {
        let rule = "app_name not_in [\"\"]";
        let rule = compile_rule(rule).expect("");

        let context = Context::default();

        assert!(rule(&context));
    }

    //This needs to be swapped out for an arbitrary string test
    #[test_case("app_name not_in [\"Ær dette en helt ålreit string før eller siden fo Østhus\"]" ; "Norwegian characters")]
    #[test_case("app_name not_in [\"Ελληνικά\"]" ; "Greek characters")]
    #[test_case("app_name not_in [\"おはようございます\"]" ; "Japanese characters")]
    #[test_case("app_name not_in [\"😃💁\"]"; "Teenager characters" )]
    #[test_case("app_name not_in [\".&-/\"]"; "Limited punctuation characters" )]
    #[test_case("app_name not_in [\"Exclamation marks are cool!\"]"; "Exclamation marks" )]
    fn arbitrary_unicode_is_handled(input: &str) {
        let rule_text = input;
        let rule = compile_rule(rule_text).unwrap();

        assert!(rule(&Context::default()));
    }

    #[test]
    fn empty_lists_are_tolerated() {
        let rule_text = "app_name not_in []";
        let rule = compile_rule(rule_text).unwrap();

        assert!(rule(&Context::default()));
    }

    #[test_case("user_id starts_with_any [\"some\"]", true)]
    #[test_case("user_id ends_with_any [\".com\"]", true)]
    #[test_case("user_id contains_any [\"email\"]", true)]
    #[test_case("user_id contains_any [\"EMAIL\"]", false)]
    #[test_case("user_id contains_any_ignore_case [\"EMAIL\"]", true)]
    #[test_case("user_id ends_with_any_ignore_case [\".COM\"]", true)]
    #[test_case("user_id starts_with_any_ignore_case [\"SOME\"]", true)]
    fn run_string_operators_tests(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("some-email.com");

        assert_eq!(rule(&context), expected);
    }

    #[test_case("context[\"cutoff\"] == 2022-01-25T13:00:00.000Z", true)]
    #[test_case("context[\"cutoff\"] == 2022-01-25T12:00:00.000Z", false)]
    #[test_case("context[\"cutoff\"] > 2022-01-25T12:00:00.000Z", true)]
    #[test_case("context[\"cutoff\"] < 2022-01-25T14:00:00.000Z", true)]
    #[test_case("context[\"cutoff\"] < 2022-01-25T11:00:00.000Z", false)]
    #[test_case("context[\"cutoff\"] >= 2022-01-25T11:00:00.000Z", true)]
    fn run_date_operators_tests(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");

        let mut context = Context::default();
        let mut props = HashMap::new();
        props.insert("cutoff".into(), "2022-01-25T13:00:00.000Z".into());
        context.properties = Some(props);

        assert_eq!(rule(&context), expected);
    }

    #[test]
    fn handles_dates_with_truncated_milliseconds() {
        let rule = "current_time > 2022-01-29T13:00:00Z";

        let rule = compile_rule(rule).expect("");
        let mut context = Context::default();
        let mut props = HashMap::new();
        props.insert("cutoff".into(), "2022-01-25T13:00:00.000Z".into());
        context.properties = Some(props);

        assert!(!rule(&context));
    }

    #[test_case("!user_id > 8", false)]
    fn run_invert_rest(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("9");

        assert_eq!(rule(&context), expected);
    }

    //Annoying cases that failed in the spec test and are worth teasing out
    #[test]
    fn gradual_rollout_user_id_disabled_with_no_user() {
        let rule_text = "100% sticky on context[\"customField\"] with group_id of \"Feature.flexible.rollout.custom.stickiness_100\"";
        let rule = compile_rule(rule_text).unwrap();

        assert!(!rule(&Context::default()));
    }

    #[test]
    fn date_constraint_respects_timezones() {
        let context = Context {
            app_name: Some("2022-01-22T11:30:00.000Z".into()),
            ..Context::default()
        };

        let rule_text =
            "app_name > 2022-01-22T13:00:00.000+02:00 and app_name < 2022-01-22T14:00:00.000+02:00";
        let rule = compile_rule(rule_text).unwrap();
        assert!(rule(&context));
    }

    #[test]
    fn inversion_works_on_string_any_rules() {
        let context = Context {
            app_name: Some("email".into()),
            ..Context::default()
        };

        let rule_text = "!app_name contains_any [\"@another.com\"]";
        let rule = compile_rule(rule_text).unwrap();
        assert!(rule(&context));
    }

    #[test]
    fn escaping_strings_works() {
        let rule = "user_id in [\"Nobody likes \\\"scare quotes\\\"\"]";
        compile_rule(rule).unwrap();
    }

    #[test]
    fn missing_external_value_produces_false_without_error() {
        let rule = "external_value[\"i_do_not_exist\"]";
        let rule = compile_rule(rule).unwrap();

        let context = Context::default();
        let result = rule(&context);

        assert!(!result);
    }

    #[test]
    fn inversion_works_on_external_values() {
        let rule = "!external_value[\"test_value\"]";
        let rule = compile_rule(rule).unwrap();

        let mut custom_strategy_results = HashMap::new();
        custom_strategy_results.insert("test_value".to_string(), true);

        let context = Context {
            external_results: Some(custom_strategy_results),
            ..Default::default()
        };

        assert!(!rule(&context));
    }

    #[test]
    fn external_value_is_respected() {
        let rule = "external_value[\"test_value\"]";
        let rule = compile_rule(rule).unwrap();

        let mut custom_strategy_results = HashMap::new();
        custom_strategy_results.insert("test_value".to_string(), true);

        let mut context = Context {
            external_results: Some(custom_strategy_results),
            ..Default::default()
        };

        let true_result = rule(&context);

        context
            .external_results
            .as_mut()
            .unwrap()
            .insert("test_value".to_string(), false);

        let false_result = rule(&context);

        assert!(true_result);
        assert!(!false_result);
    }

    #[test]
    fn evaluates_quotes_in_stringy_rules_correctly() {
        let rule = compile_rule("user_id contains_any [\"some\\\"thing\"]").unwrap();

        let context = context_from_user_id("some\"thing");

        assert!(rule(&context));
    }

    #[test]
    fn evaluates_host_name_constraint_correctly() {
        std::env::set_var("hostname", "DOS");

        let rule = compile_rule("hostname in [\"DOS\"]").unwrap();
        let context = Context::default();
        assert!(rule(&context));

        std::env::remove_var("hostname");
    }

    #[test]
    fn evaluates_host_name_to_false_when_missing_hostname_values() {
        std::env::set_var("hostname", "DOS");

        let rule = compile_rule("hostname in [\"\"]").unwrap();
        let context = Context::default();
        assert!(!rule(&context));

        std::env::remove_var("hostname");
    }

    #[test]
    fn hostname_constraint_ignores_casing() {
        std::env::set_var("hostname", "DaRWin");

        let rule = compile_rule("hostname in [\"dArWin\", \"pop-os\"]").unwrap();
        let context = Context::default();
        assert!(rule(&context));

        std::env::remove_var("hostname");
    }

    #[test_case("127.0.0.1", "127.0.0.1", true; "Exact match")]
    #[test_case("127.0.0.1", "127.0.0.1, 10.0.0.1, 196.0.0.1, 192.168.0.0/16", true; "Contains exact match on first")]
    #[test_case("10.0.0.1", "127.0.0.1, 10.0.0.1, 196.0.0.1, 192.168.0.0/16", true; "Contains exact match on second")]
    #[test_case("196.0.0.1", "127.0.0.1, 10.0.0.1, 196.0.0.1, 192.168.0.0/16", true; "Contains exact match on third")]
    #[test_case("192.168.42.23", "127.0.0.1, 10.0.0.1, 196.0.0.1, 192.168.0.0/16", true; "Contains subnet match on fourth")]
    #[test_case("127.0.0.1", "10.0.0.1", false; "Rejects non containing ip address")]
    #[test_case("127.0.0.1", "192.168.0.0/16", false; "Rejects ip address not in subnet")]
    #[test_case("192.168.42.23", "192.168.0.0/16", true; "Matches ip address in subnet")]
    #[test_case("::1", "::1", true; "Matches base ipv6 address")]
    #[test_case("::1", "::1, 2001:DB8:0:0:0:0:0:1, 2001:DB8::1, 2001:DB8::/48", true; "Matches first in ipv6 list")]
    #[test_case("2001:DB8:0:0:0:0:0:1", "::1, 2001:DB8:0:0:0:0:0:1, 2001:DB8::1, 2001:DB8::/48", true; "Matches second in ipv6 list")]
    #[test_case("2001:DB8::1", "::1, 2001:DB8:0:0:0:0:0:1, 2001:DB8::1, 2001:DB8::/48", true; "Matches third in ipv6 list")]
    #[test_case("::1", "2001:DB8:0:0:0:0:0:1", false; "Rejects non matching ipv6")]
    #[test_case("::1", "2001:DB8::/48", false; "Rejects ipv6 not in subnet")]
    #[test_case("::1", "::1, 127.0.0.1", true; "Matches ipv6 in mixed ipv4/ipv6 list")]
    #[test_case("127.0.0.1", "::1, 127.0.0.1", true; "Matches ipv4 in mixed ipv4/ipv6 list")]
    #[test_case("127.0.0.1", "::1, 127.0.0.1, complete-nonsense", true; "Tolerates broken ip in strategy list")]
    fn remote_address_constraint_respects_subnets_and_ipv6(
        context_ip: &str,
        constraint_ips: &str,
        expected_value: bool,
    ) {
        let constraint_ips = constraint_ips
            .split(',')
            .map(|x| format!("\"{}\"", x.trim()))
            .collect::<Vec<String>>();

        let rule = format!("remote_address contains_ip [{}]", constraint_ips.join(","));
        println!("Current rule {}", rule);
        let rule = compile_rule(&rule).unwrap();

        let context = Context {
            remote_address: Some(context_ip.into()),
            ..Context::default()
        };

        assert_eq!(rule(&context), expected_value);
    }

    #[test]
    fn remote_address_constraint_never_matches_missing_context() {
        let rule = compile_rule("remote_address contains_ip [\"127.0.0.1\"]").unwrap();
        let context = Context::default();

        assert!(!rule(&context));
    }
}
