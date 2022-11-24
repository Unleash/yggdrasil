extern crate pest;

use std::collections::HashSet;
use std::io::Cursor;
use std::num::ParseFloatError;

use crate::InnerContext as Context;
use chrono::{DateTime, Utc};
use murmur3::murmur3_32;
use pest::error::Error;
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

pub fn normalized_hash(group: &str, identifier: &str, modulus: u32) -> std::io::Result<u32> {
    let mut reader = Cursor::new(format!("{}:{}", &group, &identifier));
    murmur3_32(&mut reader, 0).map(|hash_result| hash_result % modulus)
}

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

#[derive(Debug)]
enum OrdinalComparator {
    Lte,
    Lt,
    Gte,
    Gt,
    Eq,
}

#[derive(Debug)]
enum ContentComparator {
    In,
    NotIn,
}

#[derive(Debug)]
enum StringComparator {
    StartsWithAny,
    EndsWithAny,
    ContainsAny,
}

struct StringComparatorType {
    ignore_case: bool,
    comparator_type: StringComparator,
}

//Context lifting properties - these resolve properties from the context
fn context_value(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> Option<String>> {
    let child = node.next().unwrap();
    match child.as_rule() {
        Rule::user_id => Box::new(|context: &Context| context.user_id.clone()),
        Rule::app_name => Box::new(|context: &Context| context.app_name.clone()),
        Rule::environment => Box::new(|context: &Context| context.environment.clone()),
        Rule::session_id => Box::new(|context: &Context| context.session_id.clone()),
        Rule::remote_address => Box::new(|context: &Context| context.remote_address.clone()),
        Rule::random => {
            Box::new(|_: &Context| Some(rand::thread_rng().gen_range(0..99).to_string()))
        }
        Rule::property => context_property(child.into_inner()),
        _ => unreachable!(),
    }
}

// This is a special property, it's functionally identical to 'context_value'
// and our rule here does nothing but call 'context_value' to resolve the value
// this only gets a special place so that our syntax in the grammar is nice
fn stickiness_param(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> Option<String>> {
    context_value(node.next().unwrap().into_inner())
}

fn context_property(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> Option<String>> {
    let mut chars = node.next().unwrap().as_str().chars();
    chars.next();
    chars.next_back();
    let context_name = chars.as_str().to_string();

    Box::new(move |context: &Context| -> Option<String> {
        match &context.properties {
            Some(props) => props.get(&context_name).map(|x| x.clone()),
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
            comparator_type: StringComparator::StartsWithAny,
        },
        "ends_with_any" => StringComparatorType {
            ignore_case: false,
            comparator_type: StringComparator::EndsWithAny,
        },
        "contains_any" => StringComparatorType {
            ignore_case: false,
            comparator_type: StringComparator::ContainsAny,
        },
        "starts_with_any_ignore_case" => StringComparatorType {
            ignore_case: true,
            comparator_type: StringComparator::StartsWithAny,
        },
        "ends_with_any_ignore_case" => StringComparatorType {
            ignore_case: true,
            comparator_type: StringComparator::EndsWithAny,
        },
        "contains_any_ignore_case" => StringComparatorType {
            ignore_case: true,
            comparator_type: StringComparator::ContainsAny,
        },
        _ => unreachable!(),
    }
}

fn numeric(node: Pair<Rule>) -> f64 {
    node.as_str().parse::<f64>().unwrap()
}

fn date(node: Pair<Rule>) -> DateTime<Utc> {
    node.as_str().parse::<DateTime<Utc>>().unwrap()
}

fn semver(node: Pair<Rule>) -> Version {
    Version::parse(node.as_str()).unwrap()
}

fn percentage(node: Pair<Rule>) -> u8 {
    let mut chars = node.as_str().chars();
    chars.next_back();
    chars.as_str().parse::<u8>().unwrap()
}

fn group_id_param(mut node: Pairs<Rule>) -> String {
    string(node.next().unwrap())
}

fn string(node: Pair<Rule>) -> String {
    let mut chars = node.as_str().chars();
    chars.next();
    chars.next_back();
    chars.as_str().into()
}

//Constraints
fn numeric_constraint(inverted: bool, mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let ordinal_operation = to_ordinal_comparator(node.next().unwrap());
    let number = numeric(node.next().unwrap());

    Box::new(move |context: &Context| {
        let context_value = context_getter(context);
        match context_value {
            Some(context_value) => {
                let context_value: f64 = context_value.parse().unwrap();
                match ordinal_operation {
                    OrdinalComparator::Lte => context_value <= number,
                    OrdinalComparator::Lt => context_value < number,
                    OrdinalComparator::Gte => context_value >= number,
                    OrdinalComparator::Gt => context_value > number,
                    OrdinalComparator::Eq => (context_value - number).abs() < f64::EPSILON,
                }
                .invert(inverted)
            }
            None => false,
        }
    })
}

fn date_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let ordinal_operation = to_ordinal_comparator(node.next().unwrap());
    let date = date(node.next().unwrap());

    Box::new(move |context: &Context| {
        let context_value = context_getter(context);
        match context_value {
            Some(context_value) => {
                let context_value: DateTime<Utc> = context_value.parse().unwrap();
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
    })
}

fn semver_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let ordinal_operation = to_ordinal_comparator(node.next().unwrap());
    let semver = semver(node.next().unwrap());

    Box::new(move |context: &Context| {
        let context_value = context_getter(context);

        match context_value {
            Some(context_value) => {
                let context_value = context_value.parse::<Version>().unwrap();
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
    })
}

fn rollout_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let percent_rollout = percentage(node.next().unwrap());

    let second = node.next();
    let third = node.next();

    let (stickiness_getter, group_id) = match (second, third) {
        (Some(second), Some(third)) => {
            let sticky = stickiness_param(second.into_inner());
            let group_id = group_id_param(third.into_inner());
            (Some(sticky), Some(group_id))
        }
        (Some(second), None) => match second.as_rule() {
            Rule::stickiness_param => (Some(stickiness_param(second.into_inner())), None),
            Rule::group_id_param => (None, Some(group_id_param(second.into_inner()))),
            _ => unreachable!(),
        },
        _ => (None, None),
    };

    Box::new(move |context: &Context| {
        let stickiness = match &stickiness_getter {
            Some(stickiness_getter) => {
                let custom_stickiness = stickiness_getter(&context);
                // If we're sticky on a property that isn't on the context then
                // short circuit this strategy's evaluation to false
                if custom_stickiness.is_none() {
                    return false;
                }
                custom_stickiness
            }
            None => context.user_id.clone().or(context.session_id.clone()),
        };

        let group_id = match &group_id {
            Some(group_id) => group_id.clone(),
            None => {
                "".to_string() //Need to find a way to resolve the toggle name here
            }
        };

        let hash = if let Some(stickiness) = stickiness {
            normalized_hash(&group_id, &stickiness, 100)
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
    })
}

fn list_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let comparator = to_content_comparator(node.next().unwrap());
    let list = node.next().unwrap();

    match list.as_rule() {
        Rule::empty_list => Box::new(move |_context: &Context| match comparator {
            ContentComparator::In => false,
            ContentComparator::NotIn => true,
        }),
        Rule::numeric_list => {
            let values = harvest_list(list.into_inner());
            Box::new(move |context: &Context| {
                let context_value = context_getter(context);
                match context_value {
                    Some(context_value) => {
                        let context_value: f64 = context_value.parse().unwrap();
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
    }
}

fn harvest_set(node: Pairs<Rule>) -> HashSet<String> {
    node.into_iter()
        .map(|x| string(x))
        .collect::<HashSet<String>>()
}

fn harvest_string_list(node: Pairs<Rule>) -> Vec<String> {
    node.into_iter().map(|x| string(x)).collect::<Vec<String>>()
}

fn harvest_list(node: Pairs<Rule>) -> Vec<f64> {
    let nodes: Result<Vec<f64>, ParseFloatError> =
        node.into_iter().map(|x| x.as_str().parse()).collect();
    nodes.unwrap()
}

fn default_strategy_constraint(inverted: bool, node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let enabled: bool = node.as_str().chars().as_str().parse().unwrap();
    Box::new(move |_: &Context| enabled.invert(inverted))
}

fn string_fragment_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let comparator_details = to_string_comparator(node.next().unwrap());
    let comparator = comparator_details.comparator_type;
    let ignore_case = comparator_details.ignore_case;

    let mut list = harvest_string_list(node.next().unwrap().into_inner());
    if ignore_case {
        list = list.into_iter().map(|item| item.to_lowercase()).collect();
    };

    Box::new(move |context: &Context| {
        let mut value = context_getter(context);
        if ignore_case {
            value = value.map(|value| value.to_lowercase())
        }
        if let Some(value) = value {
            match comparator {
                StringComparator::ContainsAny => list.iter().any(|item| value.contains(item)),
                StringComparator::StartsWithAny => list.iter().any(|item| value.starts_with(item)),
                StringComparator::EndsWithAny => list.iter().any(|item| value.ends_with(item)),
            }
        } else {
            false
        }
    })
}

fn constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let first = node.next();
    let second = node.next();

    let (inverted, child) = match (first, second) {
        (Some(_), Some(second)) => (true, second),
        (Some(first), None) => (false, first),
        _ => unreachable!(),
    };

    match child.as_rule() {
        Rule::date_constraint => date_constraint(child.into_inner()),
        Rule::numeric_constraint => numeric_constraint(inverted, child.into_inner()),
        Rule::semver_constraint => semver_constraint(child.into_inner()),
        Rule::rollout_constraint => rollout_constraint(child.into_inner()),
        Rule::default_strategy_constraint => {
            default_strategy_constraint(inverted, child.into_inner())
        }
        Rule::string_fragment_constraint => string_fragment_constraint(child.into_inner()),
        Rule::list_constraint => list_constraint(child.into_inner()),
        _ => unreachable!(),
    }
}

fn eval(expression: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::constraint => constraint(primary.into_inner()),
            Rule::expr => eval(primary.into_inner()),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::and => {
                Box::new(move |context: &Context| -> bool { lhs(context) && rhs(context) })
            }
            Rule::or => Box::new(move |context: &Context| -> bool { lhs(context) || rhs(context) }),
            _ => unreachable!(),
        })
        .parse(expression)
}

pub fn compile_rule(rule: &str) -> Result<Box<dyn Fn(&Context) -> bool>, Error<Rule>> {
    println!("Compiling rule {}", rule);
    let parse_result = Strategy::parse(Rule::strategy, rule);
    parse_result.map(|mut x| eval(x.next().unwrap().into_inner()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use test_case::test_case;

    fn context_from_user_id(user_id: &str) -> Context {
        Context {
            user_id: Some(user_id.into()),
            properties: Some(HashMap::new()),
            session_id: None,
            environment: None,
            app_name: None,
            remote_address: None,
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
        let rule = "random() < 100";
        let rule = compile_rule(rule).expect("");
        let context = Context::default();

        assert_eq!(rule(&context), true);
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
        let context = context_from_user_id("6".into());

        assert_eq!(rule(&context), expected);
    }

    #[test_case("100%", true)]
    #[test_case("99%", true)]
    fn run_rollout_test(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = context_from_user_id("6".into());

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
            user_id: Some("6".into()),
            properties: Some(context_property),
            session_id: None,
            environment: None,
            app_name: None,
            remote_address: None,
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

    #[test_case("55% with group_id of \"Feature.flexibleRollout.userId.55\"", true)]
    fn run_rollout_test_with_group_id_and_no_sticky(rule: &str, expected: bool) {
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

    #[test_case("user_id in [1, 3, 6]", true)]
    #[test_case("user_id in [1, 3, 5]", false)]
    #[test_case("user_id not_in [1, 3, 5]", true)]
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

        assert_eq!(rule(&context), true);
    }

    //This needs to be swapped out for an arbitrary string test
    #[test]
    fn dashes_are_valid_characters() {
        let rule_text = "app_name not_in [\"web\", \"sun-app\"]";
        let rule = compile_rule(rule_text).unwrap();

        assert_eq!(rule(&Context::default()), true);
    }

    #[test]
    fn empty_lists_are_tolerated() {
        let rule_text = "app_name not_in []";
        let rule = compile_rule(rule_text).unwrap();

        assert_eq!(rule(&Context::default()), true);
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

        assert_eq!(rule(&Context::default()), false);
    }
}
