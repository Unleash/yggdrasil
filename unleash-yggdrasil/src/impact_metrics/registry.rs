use crate::impact_metrics::counter::CounterImpl;
use crate::impact_metrics::types::{CollectedMetric, MetricOptions, MetricType};
use crate::impact_metrics::{Counter, ImpactMetricRegistry, ImpactMetricsDataSource};
use dashmap::DashMap;
use std::sync::Arc;

pub struct InMemoryMetricRegistry {
    counters: DashMap<String, Arc<CounterImpl>>,
}

impl InMemoryMetricRegistry {
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
        }
    }
}

impl Default for InMemoryMetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ImpactMetricRegistry for InMemoryMetricRegistry {
    fn counter(&self, opts: MetricOptions) -> Arc<dyn Counter> {
        let counter = self
            .counters
            .entry(opts.name.clone())
            .or_insert_with(|| Arc::new(CounterImpl::new(&opts.name, &opts.help)));
        counter.clone()
    }

    fn get_counter(&self, name: &str) -> Option<Arc<dyn Counter>> {
        self.counters.get(name).map(|c| c.clone() as Arc<dyn Counter>)
    }
}

impl ImpactMetricsDataSource for InMemoryMetricRegistry {
    fn collect(&self) -> Vec<CollectedMetric> {
        self.counters
            .iter()
            .map(|entry| entry.value().collect())
            .collect()
    }

    fn restore(&self, metrics: Vec<CollectedMetric>) {
        for metric in metrics {
            if metric.metric_type == MetricType::Counter {
                let counter = self.counter(MetricOptions::new(&metric.name, &metric.help));
                for sample in metric.samples {
                    counter.inc_with_labels(sample.value, Some(&sample.labels));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact_metrics::types::NumericMetricSample;
    use std::collections::HashMap;

    fn sample(value: i64) -> NumericMetricSample {
        NumericMetricSample::zero_with_value(value)
    }

    fn sample_with_labels(labels: HashMap<String, String>, value: i64) -> NumericMetricSample {
        NumericMetricSample::new(labels, value)
    }

    fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn should_increment_by_default_value() {
        let registry = InMemoryMetricRegistry::new();
        let counter = registry.counter(MetricOptions::new("test_counter", "testing"));

        counter.inc();

        let metrics = registry.collect();
        let expected =
            CollectedMetric::new("test_counter", "testing", MetricType::Counter, vec![sample(1)]);

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_increment_with_custom_value_and_labels() {
        let registry = InMemoryMetricRegistry::new();
        let counter = registry.counter(MetricOptions::new("labeled_counter", "with labels"));

        let lbls = labels(&[("foo", "bar")]);
        counter.inc_with_labels(3, Some(&lbls));
        counter.inc_with_labels(2, Some(&lbls));

        let metrics = registry.collect();
        let expected = CollectedMetric::new(
            "labeled_counter",
            "with labels",
            MetricType::Counter,
            vec![sample_with_labels(lbls, 5)],
        );

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_store_different_label_combinations_separately() {
        let registry = InMemoryMetricRegistry::new();
        let counter = registry.counter(MetricOptions::new("multi_label", "label test"));

        counter.inc_with_labels(1, Some(&labels(&[("a", "x")])));
        counter.inc_with_labels(2, Some(&labels(&[("b", "y")])));
        counter.inc_by(3);

        let metrics = registry.collect();
        let result = &metrics[0];

        assert_eq!(result.name, "multi_label");
        assert_eq!(result.samples.len(), 3);

        let mut samples_sorted: Vec<_> = result.samples.iter().collect();
        samples_sorted.sort_by_key(|s| s.value);

        assert_eq!(samples_sorted[0], &sample_with_labels(labels(&[("a", "x")]), 1));
        assert_eq!(samples_sorted[1], &sample_with_labels(labels(&[("b", "y")]), 2));
        assert_eq!(samples_sorted[2], &sample(3));
    }

    #[test]
    fn should_return_zero_value_when_empty() {
        let registry = InMemoryMetricRegistry::new();
        registry.counter(MetricOptions::new("noop_counter", "noop"));

        let metrics = registry.collect();
        let expected =
            CollectedMetric::new("noop_counter", "noop", MetricType::Counter, vec![sample(0)]);

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_return_zero_value_after_flushing() {
        let registry = InMemoryMetricRegistry::new();
        let counter = registry.counter(MetricOptions::new("flush_test", "flush"));

        counter.inc();

        let first_batch = registry.collect();
        let expected1 =
            CollectedMetric::new("flush_test", "flush", MetricType::Counter, vec![sample(1)]);
        assert_eq!(first_batch, vec![expected1]);

        let second_batch = registry.collect();
        let expected2 =
            CollectedMetric::new("flush_test", "flush", MetricType::Counter, vec![sample(0)]);
        assert_eq!(second_batch, vec![expected2]);
    }

    #[test]
    fn should_restore_collected_metrics() {
        let registry = InMemoryMetricRegistry::new();
        let counter = registry.counter(MetricOptions::new("restore_test", "testing restore"));

        counter.inc_with_labels(5, Some(&labels(&[("tag", "a")])));
        counter.inc_with_labels(2, Some(&labels(&[("tag", "b")])));

        let flushed = registry.collect();

        let after_flush = registry.collect();
        assert_eq!(after_flush[0].samples, vec![sample(0)]);

        registry.restore(flushed);

        let restored = registry.collect();
        let mut samples_sorted: Vec<_> = restored[0].samples.iter().collect();
        samples_sorted.sort_by_key(|s| s.value);

        assert_eq!(samples_sorted[0], &sample_with_labels(labels(&[("tag", "b")]), 2));
        assert_eq!(samples_sorted[1], &sample_with_labels(labels(&[("tag", "a")]), 5));
    }
}
