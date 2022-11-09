extern crate pest;

use std::collections::{HashMap};
use std::io::Cursor;
use std::num::ParseFloatError;

use murmur3::murmur3_32;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use semver::Version;

pub struct Context {
    user_id: String,
    app_name: String,
    environment: String,
    properties: HashMap<String, String>,
}

impl Context {
    fn new(user_id: String, properties: HashMap<String, String>) -> Self {
        Context {
            user_id,
            app_name: "".to_string(),
            environment: "".to_string(),
            properties,
        }
    }
}

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

#[derive(Debug)]
enum Comparator {
    Lte,
    Lt,
    Gte,
    Gt,
    Eq,
}

//Context lifting properties - these resolve properties from the context
fn context_value(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> String> {
    let child = node.next().unwrap();
    match child.as_rule() {
        Rule::user_id => Box::new(|context: &Context| -> String { context.user_id.clone() }),
        Rule::app_name => Box::new(|context: &Context| -> String { context.app_name.clone() }),
        Rule::environment => {
            Box::new(|context: &Context| -> String { context.environment.clone() })
        }
        Rule::property => context_property(child.into_inner()),
        _ => unreachable!(),
    }
}

// This is a special property, it's functionally identical to 'context_value'
// and our rule here does nothing but call 'context_value' to resolve the value
// this only gets a special place so that our syntax in the grammar is nice
fn stickiness_param(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> String> {
    context_value(node.next().unwrap().into_inner())
}

fn context_property(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> String> {
    let mut chars = node.next().unwrap().as_str().chars();
    chars.next();
    chars.next_back();
    let context_name = chars.as_str().to_string();

    Box::new(move |context: &Context| -> String {
        context.properties.get(&context_name).unwrap().clone()
    })
}

fn ordinal(node: Pair<Rule>) -> Comparator {
    match node.as_str() {
        "<" => Comparator::Lt,
        "<=" => Comparator::Lte,
        "==" => Comparator::Eq,
        ">" => Comparator::Gt,
        ">=" => Comparator::Gte,
        _ => unreachable!(),
    }
}

fn numeric(node: Pair<Rule>) -> f64 {
    node.as_str().parse::<f64>().unwrap()
}

fn semver(node: Pair<Rule>) -> Version {
    Version::parse(node.as_str()).unwrap()
}

fn percentage(node: Pair<Rule>) -> u8 {
    let mut chars = node.as_str().chars();
    chars.next_back();

    chars.as_str().parse::<u8>().unwrap()
}

fn group_id_param(node: Pairs<Rule>) -> String {
    let mut chars = node.as_str().chars();
    chars.next();
    chars.next_back();

    chars.as_str().into()
}

//Constraints
fn numeric_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let ordinal_operation = ordinal(node.next().unwrap());
    let number = numeric(node.next().unwrap());

    Box::new(move |context: &Context| {
        let context_value: f64 = context_getter(context).parse().unwrap();
        match ordinal_operation {
            Comparator::Lte => context_value <= number,
            Comparator::Lt => context_value < number,
            Comparator::Gte => context_value >= number,
            Comparator::Gt => context_value > number,
            Comparator::Eq => (context_value - number).abs() < f64::EPSILON,
        }
    })
}

fn semver_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let ordinal_operation = ordinal(node.next().unwrap());
    let semver = semver(node.next().unwrap());

    Box::new(move |context: &Context| {
        let context_value = context_getter(context).parse::<Version>().unwrap();
        match ordinal_operation {
            Comparator::Lte => context_value <= semver,
            Comparator::Lt => context_value < semver,
            Comparator::Gte => context_value >= semver,
            Comparator::Gt => context_value > semver,
            Comparator::Eq => context_value == semver,
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
            Some(stickiness_getter) => stickiness_getter(&context),
            None => "".to_string(), //This should be userId || sessionId || random
        };

        let group_id = match &group_id {
            Some(group_id) => group_id.clone(),
            None => {
                "".to_string() //Need to find a way to resolve the toggle name here
            }
        };

        let hash = normalized_hash(&group_id, &stickiness, 100);
        if let Ok(hash) = hash {
            hash <= percent_rollout.into()
        } else {
            false
        }
    })
}

fn list_constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let context_getter = context_value(node.next().unwrap().into_inner());
    let _ = node.next(); //For now we discard this node, it's always contains
    let list = node.next().unwrap();

    match list.as_rule() {
        Rule::numeric_list => {
            let values = harvest_list(list.into_inner());
            Box::new(move |context: &Context| {
                let context_value: f64 = context_getter(context).parse().unwrap();
                values.contains(&context_value)
            })
        }
        _ => unreachable!(),
    }
}

fn harvest_list(node: Pairs<Rule>) -> Vec<f64> {
    let nodes: Result<Vec<f64>, ParseFloatError> =
        node.into_iter().map(|x| x.as_str().parse()).collect();
    nodes.unwrap()
}

fn default_strategy_constraint(node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let enabled: bool = node.as_str().chars().as_str().parse().unwrap();
    Box::new(move |_: &Context| enabled)
}

fn constraint(mut node: Pairs<Rule>) -> Box<dyn Fn(&Context) -> bool> {
    let child = node.next().unwrap();
    match child.as_rule() {
        Rule::numeric_constraint => numeric_constraint(child.into_inner()),
        Rule::semver_constraint => semver_constraint(child.into_inner()),
        Rule::rollout_constraint => rollout_constraint(child.into_inner()),
        Rule::default_strategy_constraint => default_strategy_constraint(child.into_inner()),
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
    let parse_result = Strategy::parse(Rule::strategy, rule);
    parse_result.map(|mut x| eval(x.next().unwrap().into_inner()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn context_from_user_id(user_id: &str) -> Context {
        Context::new(user_id.into(), HashMap::new())
    }

    #[test_case("1", "user_id == 1 and (user_id > 1 and user_id < 1)", false)]
    #[test_case("9", "user_id == 9 or (user_id > 5 and user_id < 5)", true)]
    #[test_case("4", "user_id < 5 or user_id > 5", true)]
    #[test_case("2", "user_id < 1 and user_id > 1", false)]
    #[test_case("5", "user_id < 6 and user_id > 2", true)]
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

    #[test_case("true", true)]
    #[test_case("false", false)]
    #[test_case("true or false", true)]
    #[test_case("true and true", true)]
    #[test_case("false or false", false)]
    #[test_case("false or true", true)]
    #[test_case("false and true", false)]
    // #[test_case("not (true and false)", true)]
    fn run_boolean_constraint(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("6".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }

    #[test_case("100%", true)]
    #[test_case("99%", true)]
    fn run_rollout_test(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("6".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }

    #[test_case("context[\"penguins\"] == 7", true)]
    #[test_case("context[\"squirrels\"] == -2", true)]
    #[test_case("context[\"squirrels\"] == 7", false)]
    fn gets_context_property(rule: &str, expected: bool) {
        let mut context_property = HashMap::new();
        context_property.insert("penguins".into(), "7".into());
        context_property.insert("squirrels".into(), "-2".into());

        let rule = compile_rule(rule).expect("");
        let context = Context::new("6".into(), context_property);

        assert_eq!(rule(&context), expected);
    }

    #[test_case(
        "55% sticky on user_id with group_id of \"Feature.flexibleRollout.userId.55\"",
        true
    )]
    fn run_rollout_test_with_group_id(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("25".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }

    #[test_case("55% with group_id of \"Feature.flexibleRollout.userId.55\"", true)]
    fn run_rollout_test_with_group_id_and_no_sticky(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("25".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }

    #[test_case("100% sticky on user_id", true)]
    fn run_rollout_test_with_stickiness(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("6".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }

    #[test_case("user_id in [1, 3, 6]", true)]
    #[test_case("user_id in [1, 3, 5]", false)]
    fn run_numeric_list_test(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("6".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }
}
