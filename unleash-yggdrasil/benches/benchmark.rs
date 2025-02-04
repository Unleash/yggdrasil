use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unleash_types::client_features::{
    ClientFeature, ClientFeatures, Constraint, Operator, Strategy,
};
use unleash_yggdrasil::{Context, EngineState};

fn is_enabled(engine: &EngineState, toggle_name: &str, context: &Context) {
    engine.is_enabled(toggle_name, context, &None);
}

fn benchmark_with_no_strategy(c: &mut Criterion) {
    let mut engine = EngineState::default();
    engine.apply_client_features(ClientFeatures {
        version: 2,
        features: vec![ClientFeature {
            name: "test".into(),
            enabled: true,
            ..ClientFeature::default()
        }],
        segments: None,
        query: None,
        meta: None,
    });
    let context = Context {
        user_id: None,
        session_id: None,
        environment: None,
        app_name: None,
        current_time: None,
        remote_address: None,
        properties: None,
    };
    c.bench_function("basic evaluation with no strategies", |b| {
        b.iter(|| is_enabled(&engine, black_box("test"), black_box(&context)))
    });
}

fn benchmark_with_single_constraint(c: &mut Criterion) {
    let mut engine = EngineState::default();
    engine.apply_client_features(ClientFeatures {
        version: 2,
        features: vec![ClientFeature {
            name: "test".into(),
            enabled: true,
            strategies: Some(vec![Strategy {
                name: "default".into(),
                segments: None,
                variants: None,
                constraints: Some(vec![Constraint {
                    context_name: "userId".into(),
                    operator: Operator::In,
                    case_insensitive: false,
                    inverted: false,
                    values: Some(vec!["7".into()]),
                    value: None,
                }]),
                parameters: None,
                sort_order: None,
            }]),
            ..ClientFeature::default()
        }],
        segments: None,
        query: None,
        meta: None,
    });
    let context = Context {
        user_id: Some("7".into()),
        session_id: None,
        environment: None,
        app_name: None,
        current_time: None,
        remote_address: None,
        properties: None,
    };
    c.bench_function("basic evaluation with one strategy", |b| {
        b.iter(|| is_enabled(&engine, black_box("test"), black_box(&context)))
    });
}

fn benchmark_with_two_constraints(c: &mut Criterion) {
    let mut engine = EngineState::default();
    engine.apply_client_features(ClientFeatures {
        version: 2,
        features: vec![ClientFeature {
            name: "test".into(),
            enabled: true,
            strategies: Some(vec![Strategy {
                name: "default".into(),
                segments: None,
                constraints: Some(vec![
                    Constraint {
                        context_name: "userId".into(),
                        operator: Operator::In,
                        case_insensitive: false,
                        inverted: false,
                        values: Some(vec!["7".into()]),
                        value: None,
                    },
                    Constraint {
                        context_name: "userId".into(),
                        operator: Operator::NotIn,
                        case_insensitive: false,
                        inverted: false,
                        values: Some(vec!["8".into()]),
                        value: None,
                    },
                ]),
                variants: None,
                parameters: None,
                sort_order: None,
            }]),
            ..ClientFeature::default()
        }],
        segments: None,
        query: None,
        meta: None,
    });
    let context = Context {
        user_id: Some("7".into()),
        session_id: None,
        environment: None,
        app_name: None,
        current_time: None,
        remote_address: None,
        properties: None,
    };
    c.bench_function("basic evaluation with two strategies", |b| {
        b.iter(|| is_enabled(&engine, black_box("test"), black_box(&context)))
    });
}

fn benchmark_engine_ingestion(c: &mut Criterion) {
    let mut engine = EngineState::default();
    let state = ClientFeatures {
        version: 2,
        features: vec![ClientFeature {
            name: "test".into(),
            enabled: true,
            strategies: Some(vec![Strategy {
                name: "default".into(),
                segments: None,
                constraints: Some(vec![
                    Constraint {
                        context_name: "userId".into(),
                        operator: Operator::In,
                        case_insensitive: false,
                        inverted: false,
                        values: Some(vec!["7".into()]),
                        value: None,
                    },
                    Constraint {
                        context_name: "userId".into(),
                        operator: Operator::NotIn,
                        case_insensitive: false,
                        inverted: false,
                        values: Some(vec!["8".into()]),
                        value: None,
                    },
                ]),
                parameters: None,
                sort_order: None,
                variants: None,
            }]),
            ..ClientFeature::default()
        }],
        segments: None,
        query: None,
        meta: None,
    };
    c.bench_function("engine ingestion", |b| {
        b.iter(|| engine.apply_client_features(black_box(state.clone())))
    });
}

criterion_group!(
    benches,
    benchmark_with_no_strategy,
    benchmark_with_single_constraint,
    benchmark_with_two_constraints,
    benchmark_engine_ingestion
);
criterion_main!(benches);
