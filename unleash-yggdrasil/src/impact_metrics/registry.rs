use crate::impact_metrics::types::{
    BucketMetricOptions, CollectedMetric, MetricLabels, MetricOptions, MetricType,
};
use crate::impact_metrics::{
    Counter, Gauge, Histogram, ImpactMetricRegistry, ImpactMetricsDataSource,
};
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct InMemoryMetricRegistry {
    counters: DashMap<String, Arc<Counter>>,
    gauges: DashMap<String, Arc<Gauge>>,
    histograms: DashMap<String, Arc<Histogram>>,
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

    fn define_histogram(&self, opts: BucketMetricOptions) {
        let name = opts.name.clone();
        self.histograms
            .entry(name)
            .or_insert_with(|| Arc::new(Histogram::new(opts)));
    }

    fn observe_histogram(&self, name: &str, value: f64) {
        if let Some(histogram) = self.histograms.get(name) {
            histogram.observe(value);
        }
    }

    fn observe_histogram_with_labels(&self, name: &str, value: f64, labels: &MetricLabels) {
        if let Some(histogram) = self.histograms.get(name) {
            histogram.observe_with_labels(value, labels);
        }
    }
}

impl ImpactMetricsDataSource for InMemoryMetricRegistry {
    fn collect(&self) -> Vec<CollectedMetric> {
        let counter_metrics = self.counters.iter().map(|entry| entry.value().collect());
        let gauge_metrics = self.gauges.iter().map(|entry| entry.value().collect());
        let histogram_metrics = self.histograms.iter().map(|entry| entry.value().collect());
        counter_metrics
            .chain(gauge_metrics)
            .chain(histogram_metrics)
            .collect()
    }

    fn restore(&self, metrics: Vec<CollectedMetric>) {
        for metric in metrics {
            match metric.metric_type {
                MetricType::Counter => {
                    self.define_counter(MetricOptions::new(&metric.name, &metric.help));
                    for sample in metric.numeric_samples() {
                        self.inc_counter_with_labels(&metric.name, sample.value, &sample.labels);
                    }
                }
                MetricType::Gauge => {
                    self.define_gauge(MetricOptions::new(&metric.name, &metric.help));
                    for sample in metric.numeric_samples() {
                        self.set_gauge_with_labels(&metric.name, sample.value, &sample.labels);
                    }
                }
                MetricType::Histogram => {
                    let buckets: Vec<f64> = metric
                        .bucket_samples()
                        .first()
                        .map(|s| s.buckets.iter().map(|b| b.le).collect())
                        .unwrap_or_default();

                    self.define_histogram(BucketMetricOptions::new(
                        &metric.name,
                        &metric.help,
                        buckets,
                    ));

                    if let Some(histogram) = self.histograms.get(&metric.name) {
                        for sample in metric.bucket_samples() {
                            histogram.restore(sample);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact_metrics::types::{BucketMetricSample, HistogramBucket, NumericMetricSample};
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

    fn bucket(le: f64, count: i64) -> HistogramBucket {
        HistogramBucket { le, count }
    }

    #[test]
    fn should_increment_by_default_value() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("test_counter", "testing"));

        registry.inc_counter("test_counter");

        let metrics = registry.collect();
        let expected = CollectedMetric::new_numeric(
            "test_counter",
            "testing",
            MetricType::Counter,
            vec![NumericMetricSample::new(HashMap::new(), 1)],
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
        let expected = CollectedMetric::new_numeric(
            "labeled_counter",
            "with labels",
            MetricType::Counter,
            vec![NumericMetricSample::new(lbls, 5)],
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

        let mut samples_sorted: Vec<_> = result.numeric_samples();
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
        let expected = CollectedMetric::new_numeric(
            "noop_counter",
            "noop",
            MetricType::Counter,
            vec![NumericMetricSample::zero()],
        );

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_return_zero_value_after_flushing() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_counter(MetricOptions::new("flush_test", "flush"));

        registry.inc_counter("flush_test");

        let first_batch = registry.collect();
        let expected1 = CollectedMetric::new_numeric(
            "flush_test",
            "flush",
            MetricType::Counter,
            vec![NumericMetricSample::new(HashMap::new(), 1)],
        );
        assert_eq!(first_batch, vec![expected1]);

        let second_batch = registry.collect();
        let expected2 = CollectedMetric::new_numeric(
            "flush_test",
            "flush",
            MetricType::Counter,
            vec![NumericMetricSample::zero()],
        );
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
        assert_eq!(after_flush[0].numeric_samples(), vec![&sample(0)]);

        registry.restore(flushed);

        let restored = registry.collect();
        let mut samples_sorted: Vec<_> = restored[0].numeric_samples();
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
        let expected = CollectedMetric::new_numeric(
            "test_gauge",
            "gauge test",
            MetricType::Gauge,
            vec![NumericMetricSample::new(labels(&[("env", "prod")]), 10)],
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

        let mut samples_sorted: Vec<_> = result.numeric_samples();
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

    #[test]
    fn should_observe_histogram_values() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_histogram(BucketMetricOptions::new(
            "test_histogram",
            "testing histogram",
            vec![0.1, 0.5, 1.0, 2.5, 5.0],
        ));

        let env_labels = labels(&[("env", "prod")]);
        registry.observe_histogram_with_labels("test_histogram", 0.05, &env_labels);
        registry.observe_histogram_with_labels("test_histogram", 0.75, &env_labels);
        registry.observe_histogram_with_labels("test_histogram", 3.0, &env_labels);

        let metrics = registry.collect();
        let expected = CollectedMetric::new_bucket(
            "test_histogram",
            "testing histogram",
            vec![BucketMetricSample {
                labels: labels(&[("env", "prod")]),
                count: 3,
                sum: 3.8,
                buckets: vec![
                    bucket(0.1, 1),
                    bucket(0.5, 1),
                    bucket(1.0, 2),
                    bucket(2.5, 2),
                    bucket(5.0, 3),
                    bucket(f64::INFINITY, 3),
                ],
            }],
        );

        assert_eq!(metrics, vec![expected]);
    }

    #[test]
    fn should_track_different_label_combinations_separately_in_histogram() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_histogram(BucketMetricOptions::new(
            "multi_label_histogram",
            "histogram with multiple labels",
            vec![1.0, 10.0],
        ));

        registry.observe_histogram_with_labels(
            "multi_label_histogram",
            0.5,
            &labels(&[("method", "GET")]),
        );
        registry.observe_histogram_with_labels(
            "multi_label_histogram",
            5.0,
            &labels(&[("method", "POST")]),
        );
        registry.observe_histogram("multi_label_histogram", 15.0);

        let metrics = registry.collect();
        let result = &metrics[0];
        assert_eq!(result.name, "multi_label_histogram");
        assert_eq!(result.help, "histogram with multiple labels");
        assert_eq!(result.metric_type, MetricType::Histogram);

        let mut actual_samples: Vec<_> = result.bucket_samples().iter().cloned().cloned().collect();
        actual_samples.sort_by(|a, b| a.sum.total_cmp(&b.sum));

        let mut expected_samples = vec![
            BucketMetricSample {
                labels: labels(&[("method", "GET")]),
                count: 1,
                sum: 0.5,
                buckets: vec![bucket(1.0, 1), bucket(10.0, 1), bucket(f64::INFINITY, 1)],
            },
            BucketMetricSample {
                labels: labels(&[("method", "POST")]),
                count: 1,
                sum: 5.0,
                buckets: vec![bucket(1.0, 0), bucket(10.0, 1), bucket(f64::INFINITY, 1)],
            },
            BucketMetricSample {
                labels: HashMap::new(),
                count: 1,
                sum: 15.0,
                buckets: vec![bucket(1.0, 0), bucket(10.0, 0), bucket(f64::INFINITY, 1)],
            },
        ];
        expected_samples.sort_by(|a, b| a.sum.total_cmp(&b.sum));

        assert_eq!(actual_samples, expected_samples);
    }

    #[test]
    fn should_preserve_exact_data_when_restoring_histogram() {
        let registry = InMemoryMetricRegistry::default();
        registry.define_histogram(BucketMetricOptions::new(
            "restore_histogram",
            "testing histogram restore",
            vec![0.1, 1.0, 10.0],
        ));

        registry.observe_histogram_with_labels(
            "restore_histogram",
            0.05,
            &labels(&[("method", "GET")]),
        );
        registry.observe_histogram_with_labels(
            "restore_histogram",
            0.5,
            &labels(&[("method", "GET")]),
        );
        registry.observe_histogram_with_labels(
            "restore_histogram",
            5.0,
            &labels(&[("method", "POST")]),
        );
        registry.observe_histogram_with_labels(
            "restore_histogram",
            15.0,
            &labels(&[("method", "POST")]),
        );

        let first_collect = registry.collect();
        assert_eq!(first_collect.len(), 1);

        let empty_collect = registry.collect();
        let expected_empty = CollectedMetric::new_bucket(
            "restore_histogram",
            "testing histogram restore",
            vec![BucketMetricSample {
                labels: HashMap::new(),
                count: 0,
                sum: 0.0,
                buckets: vec![
                    bucket(0.1, 0),
                    bucket(1.0, 0),
                    bucket(10.0, 0),
                    bucket(f64::INFINITY, 0),
                ],
            }],
        );
        assert_eq!(empty_collect, vec![expected_empty]);

        registry.restore(first_collect.clone());

        let restored_collect = registry.collect();
        assert_eq!(restored_collect.len(), 1);
        assert_eq!(restored_collect[0].name, "restore_histogram");

        let mut restored_samples: Vec<_> = restored_collect[0]
            .bucket_samples()
            .iter()
            .cloned()
            .collect();
        let mut original_samples: Vec<_> =
            first_collect[0].bucket_samples().iter().cloned().collect();
        restored_samples.sort_by(|a, b| a.sum.total_cmp(&b.sum));
        original_samples.sort_by(|a, b| a.sum.total_cmp(&b.sum));

        assert_eq!(restored_samples, original_samples);
    }
}
