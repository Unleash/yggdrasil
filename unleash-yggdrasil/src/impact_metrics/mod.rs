mod counter;
mod gauge;
mod histogram;
mod registry;
mod types;

pub use counter::Counter;
pub use gauge::Gauge;
pub use histogram::Histogram;
pub use registry::InMemoryMetricRegistry;
pub use types::{
    BucketMetricOptions, BucketMetricSample, CollectedMetric, HistogramBucket, MetricLabels,
    MetricOptions, MetricSample, MetricType, NumericMetricSample,
};

pub trait ImpactMetricRegistry {
    fn define_counter(&self, opts: MetricOptions);
    fn inc_counter(&self, name: &str);
    fn inc_counter_by(&self, name: &str, value: i64);
    fn inc_counter_with_labels(&self, name: &str, value: i64, labels: &MetricLabels);

    fn define_gauge(&self, opts: MetricOptions);
    fn set_gauge(&self, name: &str, value: i64);
    fn set_gauge_with_labels(&self, name: &str, value: i64, labels: &MetricLabels);
    fn inc_gauge(&self, name: &str);
    fn inc_gauge_by(&self, name: &str, value: i64);
    fn inc_gauge_with_labels(&self, name: &str, value: i64, labels: &MetricLabels);
    fn dec_gauge(&self, name: &str);
    fn dec_gauge_by(&self, name: &str, value: i64);
    fn dec_gauge_with_labels(&self, name: &str, value: i64, labels: &MetricLabels);

    fn define_histogram(&self, opts: BucketMetricOptions);
    fn observe_histogram(&self, name: &str, value: f64);
    fn observe_histogram_with_labels(&self, name: &str, value: f64, labels: &MetricLabels);
}

pub trait ImpactMetricsDataSource {
    fn collect(&self) -> Vec<CollectedMetric>;
    fn restore(&self, metrics: Vec<CollectedMetric>);
}
