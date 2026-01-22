use crate::impact_metrics::types::{
    get_label_key, parse_label_key, CollectedMetric, CounterMetricSample, MetricLabels,
    MetricOptions,
};
use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};

pub struct Counter {
    opts: MetricOptions,
    values: DashMap<String, AtomicI64>,
}

impl Counter {
    pub(crate) fn new(opts: MetricOptions) -> Self {
        Self {
            opts,
            values: DashMap::new(),
        }
    }

    pub fn inc(&self) {
        self.inc_internal(1, None);
    }

    pub fn inc_by(&self, value: i64) {
        self.inc_internal(value, None);
    }

    pub fn inc_with_labels(&self, value: i64, labels: &MetricLabels) {
        self.inc_internal(value, Some(labels));
    }

    fn inc_internal(&self, value: i64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values
            .entry(key)
            .or_insert_with(|| AtomicI64::new(0))
            .fetch_add(value, Ordering::Relaxed);
    }

    pub(crate) fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        for entry in self.values.iter() {
            let key = entry.key();
            let value = entry.value().swap(0, Ordering::Relaxed);
            if value != 0 {
                samples.push(CounterMetricSample::new(parse_label_key(key), value));
            }
        }

        self.values.retain(|_, v| v.load(Ordering::Relaxed) != 0);

        if samples.is_empty() {
            samples.push(CounterMetricSample::zero());
        }

        CollectedMetric::new_counter(&self.opts.name, &self.opts.help, samples)
    }
}
