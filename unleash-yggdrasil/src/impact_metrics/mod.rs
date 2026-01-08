mod counter;
mod registry;
mod types;

pub use counter::Counter;
pub use registry::InMemoryMetricRegistry;
pub use types::{CollectedMetric, MetricLabels, MetricOptions, MetricType, NumericMetricSample};

pub trait ImpactMetricRegistry {
    fn define_counter(&self, opts: MetricOptions);
    fn inc_counter(&self, name: &str);
    fn inc_counter_by(&self, name: &str, value: i64);
    fn inc_counter_with_labels(&self, name: &str, value: i64, labels: &MetricLabels);
}

pub trait ImpactMetricsDataSource {
    fn collect(&self) -> Vec<CollectedMetric>;
    fn restore(&self, metrics: Vec<CollectedMetric>);
}
