use crate::impact_metrics::types::{
    get_label_key, parse_label_key, CollectedMetric, MetricLabels, MetricType, NumericMetricSample,
};
use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};

pub trait Counter: Send + Sync {
    fn inc(&self);
    fn inc_by(&self, value: i64);
    fn inc_with_labels(&self, value: i64, labels: Option<&MetricLabels>);
}

pub(crate) struct CounterImpl {
    name: String,
    help: String,
    values: DashMap<String, AtomicI64>,
}

impl CounterImpl {
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            values: DashMap::new(),
        }
    }

    pub fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        for entry in self.values.iter() {
            let key = entry.key();
            let value = entry.value().swap(0, Ordering::Relaxed);
            if value != 0 {
                samples.push(NumericMetricSample::new(parse_label_key(key), value));
            }
        }

        self.values.retain(|_, v| v.load(Ordering::Relaxed) != 0);

        if samples.is_empty() {
            samples.push(NumericMetricSample::zero());
        }

        CollectedMetric::new(&self.name, &self.help, MetricType::Counter, samples)
    }
}

impl Counter for CounterImpl {
    fn inc(&self) {
        self.inc_with_labels(1, None);
    }

    fn inc_by(&self, value: i64) {
        self.inc_with_labels(value, None);
    }

    fn inc_with_labels(&self, value: i64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values
            .entry(key)
            .or_insert_with(|| AtomicI64::new(0))
            .fetch_add(value, Ordering::Relaxed);
    }
}
