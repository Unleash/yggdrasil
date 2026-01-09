use crate::impact_metrics::types::{
    get_label_key, parse_label_key, CollectedMetric, MetricLabels, MetricOptions, MetricType,
    NumericMetricSample,
};
use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};

pub struct Gauge {
    opts: MetricOptions,
    values: DashMap<String, AtomicI64>,
}

impl Gauge {
    pub(crate) fn new(opts: MetricOptions) -> Self {
        Self {
            opts,
            values: DashMap::new(),
        }
    }

    pub fn set(&self, value: i64) {
        self.set_internal(value, None);
    }

    pub fn set_with_labels(&self, value: i64, labels: &MetricLabels) {
        self.set_internal(value, Some(labels));
    }

    fn set_internal(&self, value: i64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values
            .entry(key)
            .and_modify(|v| v.store(value, Ordering::Relaxed))
            .or_insert_with(|| AtomicI64::new(value));
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

    pub fn dec(&self) {
        self.dec_internal(1, None);
    }

    pub fn dec_by(&self, value: i64) {
        self.dec_internal(value, None);
    }

    pub fn dec_with_labels(&self, value: i64, labels: &MetricLabels) {
        self.dec_internal(value, Some(labels));
    }

    fn dec_internal(&self, value: i64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values
            .entry(key)
            .or_insert_with(|| AtomicI64::new(0))
            .fetch_sub(value, Ordering::Relaxed);
    }

    pub(crate) fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        for entry in self.values.iter() {
            let key = entry.key();
            let value = entry.value().swap(0, Ordering::Relaxed);
            if value != 0 {
                samples.push(NumericMetricSample::new(parse_label_key(key), value));
            }
        }

        self.values.retain(|_, v| v.load(Ordering::Relaxed) != 0);

        CollectedMetric::new(&self.opts.name, &self.opts.help, MetricType::Gauge, samples)
    }
}
