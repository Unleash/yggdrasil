use crate::impact_metrics::types::{CollectedMetric, MetricLabels, MetricOptions, MetricType};
use crate::impact_metrics::{Counter, Gauge, ImpactMetricRegistry, ImpactMetricsDataSource};
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct InMemoryMetricRegistry {
    counters: DashMap<String, Arc<Counter>>,
    gauges: DashMap<String, Arc<Gauge>>,
}

impl ImpactMetricRegistry for InMemoryMetricRegistry {
    fn define_counter(&self, opts: MetricOptions) {
        let name = opts.name.clone();
        self.counters
            .entry(name)
            .or_insert_with(|| Arc::new(Counter::new(opts)));
    }

    fn inc_counter(&self, name: &str) {
        if let Some(counter) = self.counters.get(name) {
            counter.inc();
        }
    }

    fn inc_counter_by(&self, name: &str, value: i64) {
        if let Some(counter) = self.counters.get(name) {
            counter.inc_by(value);
        }
    }

    fn inc_counter_with_labels(&self, name: &str, value: i64, labels: &MetricLabels) {
        if let Some(counter) = self.counters.get(name) {
            counter.inc_with_labels(value, labels);
        }
    }

    fn define_gauge(&self, opts: MetricOptions) {
        let name = opts.name.clone();
        self.gauges
            .entry(name)
            .or_insert_with(|| Arc::new(Gauge::new(opts)));
    }

    fn set_gauge(&self, name: &str, value: i64) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.set(value);
        }
    }

    fn set_gauge_with_labels(&self, name: &str, value: i64, labels: &MetricLabels) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.set_with_labels(value, labels);
        }
    }

    fn inc_gauge(&self, name: &str) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.inc();
        }
    }

    fn inc_gauge_by(&self, name: &str, value: i64) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.inc_by(value);
        }
    }

    fn inc_gauge_with_labels(&self, name: &str, value: i64, labels: &MetricLabels) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.inc_with_labels(value, labels);
        }
    }

    fn dec_gauge(&self, name: &str) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.dec();
        }
    }

    fn dec_gauge_by(&self, name: &str, value: i64) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.dec_by(value);
        }
    }

    fn dec_gauge_with_labels(&self, name: &str, value: i64, labels: &MetricLabels) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.dec_with_labels(value, labels);
        }
    }
}

impl ImpactMetricsDataSource for InMemoryMetricRegistry {
    fn collect(&self) -> Vec<CollectedMetric> {
        let counter_metrics = self.counters.iter().map(|entry| entry.value().collect());
        let gauge_metrics = self.gauges.iter().map(|entry| entry.value().collect());
        counter_metrics.chain(gauge_metrics).collect()
    }

    fn restore(&self, metrics: Vec<CollectedMetric>) {
        for metric in metrics {
            match metric.metric_type {
                MetricType::Counter => {
                    self.define_counter(MetricOptions::new(&metric.name, &metric.help));
                    for sample in metric.samples {
                        self.inc_counter_with_labels(&metric.name, sample.value, &sample.labels);
                    }
                }
                MetricType::Gauge => {
                    self.define_gauge(MetricOptions::new(&metric.name, &metric.help));
                    for sample in metric.samples {
                        self.set_gauge_with_labels(&metric.name, sample.value, &sample.labels);
                    }
                }
                MetricType::Histogram => {}
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
        NumericMetricSample::new(HashMap::new(), value)
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
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("test_counter", "testing"));

        registry.inc_counter("test_counter");

        let metrics = registry.collect();
        let expected = CollectedMetric::new(
            "test_counter",
            "testing",
            MetricType::Counter,
            vec![sample(1)],
        );

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_increment_with_custom_value_and_labels() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("labeled_counter", "with labels"));

        let lbls = labels(&[("foo", "bar")]);
        registry.inc_counter_with_labels("labeled_counter", 3, &lbls);
        registry.inc_counter_with_labels("labeled_counter", 2, &lbls);

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
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("multi_label", "label test"));

        registry.inc_counter_with_labels("multi_label", 1, &labels(&[("a", "x")]));
        registry.inc_counter_with_labels("multi_label", 2, &labels(&[("b", "y")]));
        registry.inc_counter_by("multi_label", 3);

        let metrics = registry.collect();
        let result = &metrics[0];

        assert_eq!(result.name, "multi_label");
        assert_eq!(result.samples.len(), 3);

        let mut samples_sorted: Vec<_> = result.samples.iter().collect();
        samples_sorted.sort_by_key(|s| s.value);

        assert_eq!(
            samples_sorted[0],
            &sample_with_labels(labels(&[("a", "x")]), 1)
        );
        assert_eq!(
            samples_sorted[1],
            &sample_with_labels(labels(&[("b", "y")]), 2)
        );
        assert_eq!(samples_sorted[2], &sample(3));
    }

    #[test]
    fn should_return_zero_value_when_empty() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("noop_counter", "noop"));

        let metrics = registry.collect();
        let expected =
            CollectedMetric::new("noop_counter", "noop", MetricType::Counter, vec![sample(0)]);

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_return_zero_value_after_flushing() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("flush_test", "flush"));

        registry.inc_counter("flush_test");

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
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("restore_test", "testing restore"));

        registry.inc_counter_with_labels("restore_test", 5, &labels(&[("tag", "a")]));
        registry.inc_counter_with_labels("restore_test", 2, &labels(&[("tag", "b")]));

        let flushed = registry.collect();

        let after_flush = registry.collect();
        assert_eq!(after_flush[0].samples, vec![sample(0)]);

        registry.restore(flushed);

        let restored = registry.collect();
        let mut samples_sorted: Vec<_> = restored[0].samples.iter().collect();
        samples_sorted.sort_by_key(|s| s.value);

        assert_eq!(
            samples_sorted[0],
            &sample_with_labels(labels(&[("tag", "b")]), 2)
        );
        assert_eq!(
            samples_sorted[1],
            &sample_with_labels(labels(&[("tag", "a")]), 5)
        );
    }

    #[test]
    fn should_support_gauge_inc_dec_and_set() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_gauge(MetricOptions::new("test_gauge", "gauge test"));

        let env_labels = labels(&[("env", "prod")]);
        registry.inc_gauge_with_labels("test_gauge", 5, &env_labels);
        registry.dec_gauge_with_labels("test_gauge", 2, &env_labels);
        registry.set_gauge_with_labels("test_gauge", 10, &env_labels);

        let metrics = registry.collect();
        let expected = CollectedMetric::new(
            "test_gauge",
            "gauge test",
            MetricType::Gauge,
            vec![sample_with_labels(labels(&[("env", "prod")]), 10)],
        );

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_track_gauge_values_separately_per_label_set() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_gauge(MetricOptions::new(
            "multi_env_gauge",
            "tracks multiple envs",
        ));

        registry.inc_gauge_with_labels("multi_env_gauge", 5, &labels(&[("env", "prod")]));
        registry.dec_gauge_with_labels("multi_env_gauge", 2, &labels(&[("env", "dev")]));
        registry.set_gauge_with_labels("multi_env_gauge", 10, &labels(&[("env", "test")]));

        let metrics = registry.collect();
        let result = &metrics[0];

        assert_eq!(result.name, "multi_env_gauge");
        assert_eq!(result.samples.len(), 3);

        let mut samples_sorted: Vec<_> = result.samples.iter().collect();
        samples_sorted.sort_by_key(|s| s.value);

        assert_eq!(
            samples_sorted[0],
            &sample_with_labels(labels(&[("env", "dev")]), -2)
        );
        assert_eq!(
            samples_sorted[1],
            &sample_with_labels(labels(&[("env", "prod")]), 5)
        );
        assert_eq!(
            samples_sorted[2],
            &sample_with_labels(labels(&[("env", "test")]), 10)
        );
    }
}
