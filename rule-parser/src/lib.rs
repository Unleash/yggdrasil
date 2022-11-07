extern crate pest;
extern crate pest_derive;

use murmur3::murmur3_32;
use pest::prec_climber as pcl;
use pest::prec_climber::PrecClimber;
use pest_consume::{match_nodes, Error, Parser};
use semver::Version;
use std::io::Cursor;
use std::{cmp::PartialOrd, collections::HashMap};

type ParseResult<T> = Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

pub struct Context {
    user_id: String,
    environment: String,
    properties: HashMap<String, String>,
}

impl Context {
    fn new(user_id: String, properties: HashMap<String, String>) -> Self {
        Context {
            user_id,
            environment: "".to_string(),
            properties,
        }
    }
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

enum BooleanOperator {
    And,
    Or,
}

#[derive(pest_derive::Parser)]
#[grammar = "toggle_strategy.pest"]
pub struct ToggleStrategy;

lazy_static::lazy_static! {
    static ref PRECCLIMBER: PrecClimber<Rule> = PrecClimber::new(
        vec![
            pcl::Operator::new(Rule::and, pcl::Assoc::Left),
            pcl::Operator::new(Rule::or, pcl::Assoc::Left),
        ]
    );
}

#[pest_consume::parser]
impl ToggleStrategy {
    //Primary entry point into our parser
    fn strategy(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        // println!("Expression is {:#?}", node);

        Ok(match_nodes!(node.into_children();
            [expr(e), EOI(_)] => e,
        ))
    }

    #[prec_climb(term, PRECCLIMBER)]
    fn expr(
        left: Box<dyn Fn(&Context) -> bool>,
        op: Node,
        right: Box<dyn Fn(&Context) -> bool>,
    ) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        println!("DOING PREC CLIMB");
        Ok(Box::new(move |context: &Context| {
            left(context) && right(context)
        }))
    }

    fn term(input: Node) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        println!("MATCHING A TERM");
        Ok(match_nodes!(input.into_children();
            [expr(n)] => n,
            [constraint(n)] => n,
        ))
    }

    fn constraint(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        println!("MATCHING A CONSTRAINT");
        match_nodes!(node.into_children();
            [
                numeric_constraint(constraint)
            ] => Ok(constraint),
            [
                semver_constraint(constraint)
            ] => Ok(constraint),
            [
                rollout_constraint(constraint)
            ] => Ok(constraint),
        )
    }

    //Context lifting properties - these resolve properties from the context
    fn context_value(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> String>> {
        Ok(match_nodes!(
            node.into_children();
            [user_id(comparator)] => comparator,
        ))
    }

    fn user_id(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> String>> {
        Ok(Box::new(|context: &Context| -> String {
            context.user_id.clone()
        }))
    }

    fn environment(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> String>> {
        Ok(Box::new(|context: &Context| -> String {
            context.environment.clone()
        }))
    }

    // This is a special property, it's functionally identical to 'context_value'
    // and our rule here does nothing but call 'context_value' to resolve the value
    // this only gets a special place so that our syntax in the rule grammar is nice
    fn stickiness_param(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> String>> {
        Ok(match_nodes!(
            node.into_children();
            [context_value(comparator)] => comparator,
        ))
    }

    // fn property(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> String>> {
    //     Ok(Box::new(|context: &Context| -> String {
    //         context.properties.get().clone()
    //     }))
    // }

    //Constraints
    fn numeric_constraint(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        Ok(match_nodes!(node.into_children();
            [
                context_value(context_getter),
                ordinal_operator(comparator),
                number(value),
            ] => {
                Box::new(move |context: &Context|  -> bool {
                    let context_value = context_getter(context);
                    let context_value = context_value.parse::<f64>();
                    match context_value {
                        Ok(context_value) => {
                            println!("Matching a numeric {:?} {:?} {:?}", &context_value, comparator, value);
                            match comparator {
                                    Comparator::Lte => context_value <= value,
                                    Comparator::Lt => context_value < value,
                                    Comparator::Gte => context_value >= value,
                                    Comparator::Gt => context_value > value,
                                    Comparator::Eq => (context_value - value).abs() < f64::EPSILON,

                            }
                        }
                        Err(_err) => false
                    }
                })
            },
        ))
    }

    fn semver_constraint(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        Ok(match_nodes!(node.into_children();
            [
                context_value(context_getter),
                ordinal_operator(comparator),
                semver(value),
            ] => {
                Box::new(move |context: &Context|  -> bool {
                    let context_value = context_getter(context);
                    let context_value = context_value.parse::<Version>();
                    match context_value {
                        Ok(context_value) => {
                            match comparator {
                                    Comparator::Lte => context_value <= value,
                                    Comparator::Lt => context_value < value,
                                    Comparator::Gte => context_value >= value,
                                    Comparator::Gt => context_value > value,
                                    Comparator::Eq => context_value == value,

                            }
                        }
                        Err(_err) => false
                    }
                })
            },
        ))
    }

    fn rollout_constraint(node: Node) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
        let (percent_rollout, group_id, stickiness_getter) = match_nodes!(node.into_children();
            [
                percentage(percent),
                stickiness_param(stickiness_getter),
                group_id_param(group_id)
            ] => {
                (percent, Some(group_id), Some(stickiness_getter))
            },
            [
                percentage(percent),
                stickiness_param(stickiness_getter)
            ] => {
                (percent, None, Some(stickiness_getter))
            },
            [
                percentage(percent),
                group_id_param(group_id)
            ] => {
                (percent, Some(group_id), None)
            },
            [
                percentage(percent)
            ] => {
                (percent, None, None)
            },
        );
        Ok(Box::new(move |context: &Context| -> bool {
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
        }))
    }

    fn group_id_param(node: Node) -> ParseResult<String> {
        Ok(match_nodes!(
            node.into_children();
            [string(comparator)] => comparator,
        ))
    }

    //Operators

    fn boolean_operator(node: Node) -> ParseResult<BooleanOperator> {
        Ok(match_nodes!(
            node.into_children();
            [and(op)] => op,
            [or(op)] => op,
        ))
    }

    fn and(node: Node) -> ParseResult<BooleanOperator> {
        Ok(BooleanOperator::And)
    }

    fn or(node: Node) -> ParseResult<BooleanOperator> {
        Ok(BooleanOperator::Or)
    }

    fn ordinal_operator(node: Node) -> ParseResult<Comparator> {
        Ok(match_nodes!(
            node.into_children();
            [lte(comparator)] => comparator,
            [lt(comparator)] => comparator,
            [gt(comparator)] => comparator,
            [gte(comparator)] => comparator,
            [eq(comparator)] => comparator,
        ))
    }

    fn lte(node: Node) -> ParseResult<Comparator> {
        Ok(Comparator::Lte)
    }

    fn lt(node: Node) -> ParseResult<Comparator> {
        Ok(Comparator::Lt)
    }

    fn gte(node: Node) -> ParseResult<Comparator> {
        Ok(Comparator::Gte)
    }

    fn gt(node: Node) -> ParseResult<Comparator> {
        Ok(Comparator::Gt)
    }

    fn eq(node: Node) -> ParseResult<Comparator> {
        Ok(Comparator::Eq)
    }

    //Atom literals
    fn percentage(node: Node) -> ParseResult<u8> {
        let mut chars = node.as_str().chars();
        chars.next_back();

        chars.as_str().parse::<u8>().map_err(|e| node.error(e))
    }

    fn number(node: Node) -> ParseResult<f64> {
        node.as_str().parse::<f64>().map_err(|e| node.error(e))
    }

    fn semver(node: Node) -> ParseResult<Version> {
        Version::parse(node.as_str()).map_err(|e| node.error(e))
    }

    fn string(node: Node) -> ParseResult<String> {
        let mut chars = node.as_str().chars();
        chars.next();
        chars.next_back();

        Ok(chars.as_str().into())
    }

    fn EOI(_end_node: Node) -> ParseResult<()> {
        Ok(())
    }
}

pub fn compile_rule(rule: &str) -> ParseResult<Box<dyn Fn(&Context) -> bool>> {
    let toggle_strategy = ToggleStrategy::parse(Rule::strategy, rule)?;
    let input = toggle_strategy.single()?;

    ToggleStrategy::strategy(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_case::test_case;

    fn context_from_user_id(user_id: &str) -> Context {
        Context::new(user_id.into(), HashMap::new())
    }

    // #[test_case("6", "user_id < 5", false)]
    // #[test_case("6.0", "user_id < 5", false)]
    // #[test_case("5", "user_id < 5", false)]
    // #[test_case("4", "user_id < 5", true)]
    // #[test_case("4.0", "user_id < 5", true)]
    // #[test_case("-4.0", "user_id < -5", false)]
    // #[test_case("-4.0", "user_id < -3", true)]
    // #[test_case("-4.0", "user_id < -4", false)]
    // #[test_case("6", "user_id < 5.0", false)]
    // #[test_case("6.0", "user_id < 5.0", false)]
    // #[test_case("5", "user_id < 5.0", false)]
    // #[test_case("4", "user_id < 5.0", true)]
    // #[test_case("4.0", "user_id < 5.0", true)]
    // #[test_case("-4.0", "user_id < -5.0", false)]
    // #[test_case("-4.0", "user_id < -3.0", true)]
    // #[test_case("-4.0", "user_id < -4.0", false)]
    // fn test_numeric_lt(user_id: &str, rule: &str, expected: bool) {
    //     run_test(user_id, rule, expected);
    // }

    // #[test_case("3", "user_id <= 4", true)]
    // #[test_case("3", "user_id <= 3", true)]
    // #[test_case("3", "user_id <= 2", false)]
    // #[test_case("-4.0", "user_id <= -5.2", false)]
    // fn test_numeric_lte(user_id: &str, rule: &str, expected: bool) {
    //     run_test(user_id, rule, expected);
    // }

    // #[test_case("5", "user_id > 4", true)]
    // #[test_case("4", "user_id > 4", false)]
    // #[test_case("3", "user_id > 4", false)]
    // fn test_numeric_gt(user_id: &str, rule: &str, expected: bool) {
    //     run_test(user_id, rule, expected);
    // }

    // #[test_case("3", "user_id >= 4", false)]
    // #[test_case("3", "user_id >= 3", true)]
    // #[test_case("3", "user_id >= 2", true)]
    // #[test_case("-4.0", "user_id >= -5.2", true)]
    // fn test_numeric_gte(user_id: &str, rule: &str, expected: bool) {
    //     run_test(user_id, rule, expected);
    // }

    // #[test_case("3.0", "user_id == 3.0", true)]
    // #[test_case("3", "user_id == 3.0", true)]
    // #[test_case("3.0", "user_id == 3", true)]
    // #[test_case("-3", "user_id == -3", true)]
    // #[test_case("4", "user_id == 4", true)]
    // #[test_case("4", "user_id == 3", false)]
    // fn test_numeric_eq(user_id: &str, rule: &str, expected: bool) {
    //     run_test(user_id, rule, expected);
    // }

    // #[test_case("30.0.0", "user_id == 30.0.0", true)]
    // #[test_case("3.0.0", "user_id > 3.0.0", false)]
    // #[test_case("3.0.0-beta", "user_id == 3.0.0-beta", true)]
    // #[test_case("3.0.0-beta.2", "user_id > 3.0.0-beta.1", true)]
    // #[test_case("3.0.0-beta", "user_id > 3.0.0-alpha", true)]
    // #[test_case("3.0.0-beta", "user_id < 3.0.0-alpha", false)]
    // #[test_case("3.0.0", "user_id > 3.0.0-alpha", true)]
    // #[test_case("3.0.0", "user_id >= 3.0.0", true)]
    // #[test_case("3.0.0-beta.stuff", "user_id == 3.0.0-beta.stuff", true)]
    // #[test_case("3.0.0-beta.stuff+build1", "user_id == 3.0.0-beta.stuff+build1", true)]
    // fn test_semver_gt(user_id: &str, rule: &str, expected: bool) {
    //     run_test(user_id, rule, expected);
    // }

    // fn run_test(user_id: &str, rule: &str, expected: bool) {
    //     let rule = compile_rule(rule).expect("");
    //     let context = context_from_user_id(user_id);

    //     assert_eq!(rule(&context), expected);
    // }

    // #[test_case("100%", true)]
    // #[test_case("99%", true)]
    // fn run_rollout_test(rule: &str, expected: bool) {
    //     let rule = compile_rule(rule).expect("");
    //     let context = Context::new("6".into(), HashMap::new());

    //     assert_eq!(rule(&context), expected);
    // }

    // #[test_case(
    //     "55% sticky on user_id with group_id of \"Feature.flexibleRollout.userId.55\"",
    //     true
    // )]
    // fn run_rollout_test_with_group_id(rule: &str, expected: bool) {
    //     let rule = compile_rule(rule).expect("");
    //     let context = Context::new("25".into(), HashMap::new());

    //     assert_eq!(rule(&context), expected);
    // }

    // #[test_case(
    //     "55% sticky on user_id with group_id of \"Feature.flexibleRollout.userId.55\"",
    //     true
    // )]
    // fn run_rollout_test_with_group_id_as_session(rule: &str, expected: bool) {
    //     let rule = compile_rule(rule).expect("");
    //     let context = Context::new("25".into(), HashMap::new());

    //     assert_eq!(rule(&context), expected);
    // }

    // #[test_case("100% sticky on user_id", true)]
    // fn run_rollout_test_with_stickiness(rule: &str, expected: bool) {
    //     let rule = compile_rule(rule).expect("");
    //     let context = Context::new("6".into(), HashMap::new());

    //     assert_eq!(rule(&context), expected);
    // }

    #[test_case("user_id > 5 and user_id < 7", false)]
    fn run_boolean_chain_test(rule: &str, expected: bool) {
        let rule = compile_rule(rule).expect("");
        let context = Context::new("6".into(), HashMap::new());

        assert_eq!(rule(&context), expected);
    }
}
