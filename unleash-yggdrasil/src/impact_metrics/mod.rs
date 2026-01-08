mod counter;
mod registry;
mod types;

pub use counter::Counter;
pub use registry::InMemoryMetricRegistry;
pub use types::{CollectedMetric, MetricOptions, MetricType, NumericMetricSample};

pub trait ImpactMetricRegistry: Send + Sync {
    fn counter(&self, opts: MetricOptions) -> std::sync::Arc<Counter>;
    fn get_counter(&self, name: &str) -> Option<std::sync::Arc<Counter>>;
}

pub trait ImpactMetricsDataSource: Send + Sync {
    fn collect(&self) -> Vec<CollectedMetric>;
    fn restore(&self, metrics: Vec<CollectedMetric>);
}
