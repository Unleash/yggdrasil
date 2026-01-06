use crate::impact_metrics::types::{
    get_label_key, parse_label_key, CollectedMetric, MetricLabels, MetricOptions, MetricType,
    NumericMetricSample,
};
use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};

pub trait Counter: Send + Sync {
    fn inc(&self);
    fn inc_by(&self, value: i64);
    fn inc_with_labels(&self, value: i64, labels: Option<&MetricLabels>);
}

pub(crate) struct CounterImpl {
    opts: MetricOptions,
    values: DashMap<String, AtomicI64>,
}

impl CounterImpl {
    pub fn new(opts: MetricOptions) -> Self {
        Self {
            opts,
            values: DashMap::new(),
        }
    }

    pub fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        let keys: Vec<String> = self.values.iter().map(|e| e.key().clone()).collect();
        for key in keys {
            if let Some((_, atomic_value)) = self.values.remove(&key) {
                let value = atomic_value.into_inner();
                if value != 0 {
                    samples.push(NumericMetricSample::new(parse_label_key(&key), value));
                }
            }
        }

        if samples.is_empty() {
            samples.push(NumericMetricSample::zero());
        }

        CollectedMetric::new(
            &self.opts.name,
            &self.opts.help,
            MetricType::Counter,
            samples,
        )
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
