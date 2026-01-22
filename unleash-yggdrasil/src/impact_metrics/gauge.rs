use crate::impact_metrics::types::{
    get_label_key, parse_label_key, CollectedMetric, GaugeMetricSample, MetricLabels, MetricOptions,
};
use dashmap::DashMap;

pub struct Gauge {
    opts: MetricOptions,
    values: DashMap<String, f64>,
}

impl Gauge {
    pub(crate) fn new(opts: MetricOptions) -> Self {
        Self {
            opts,
            values: DashMap::new(),
        }
    }

    pub fn set(&self, value: f64) {
        self.set_internal(value, None);
    }

    pub fn set_with_labels(&self, value: f64, labels: &MetricLabels) {
        self.set_internal(value, Some(labels));
    }

    fn set_internal(&self, value: f64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values.insert(key, value);
    }

    pub fn inc(&self) {
        self.inc_internal(1.0, None);
    }

    pub fn inc_by(&self, value: f64) {
        self.inc_internal(value, None);
    }

    pub fn inc_with_labels(&self, value: f64, labels: &MetricLabels) {
        self.inc_internal(value, Some(labels));
    }

    fn inc_internal(&self, value: f64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values
            .entry(key)
            .and_modify(|v| *v += value)
            .or_insert(value);
    }

    pub fn dec(&self) {
        self.dec_internal(1.0, None);
    }

    pub fn dec_by(&self, value: f64) {
        self.dec_internal(value, None);
    }

    pub fn dec_with_labels(&self, value: f64, labels: &MetricLabels) {
        self.dec_internal(value, Some(labels));
    }

    fn dec_internal(&self, value: f64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);
        self.values
            .entry(key)
            .and_modify(|v| *v -= value)
            .or_insert(-value);
    }

    pub(crate) fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        for mut entry in self.values.iter_mut() {
            let value = std::mem::take(entry.value_mut());
            if value != 0.0 {
                samples.push(GaugeMetricSample::new(parse_label_key(entry.key()), value));
            }
        }

        self.values.retain(|_, v| *v != 0.0);

        if samples.is_empty() {
            samples.push(GaugeMetricSample::zero());
        }

        CollectedMetric::new_gauge(&self.opts.name, &self.opts.help, samples)
    }
}
