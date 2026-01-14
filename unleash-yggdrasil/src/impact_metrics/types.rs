use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MetricLabels = HashMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricOptions {
    pub name: String,
    pub help: String,
    pub label_names: Vec<String>,
}

impl MetricOptions {
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            label_names: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BucketMetricOptions {
    pub name: String,
    pub help: String,
    pub label_names: Vec<String>,
    pub buckets: Vec<f64>,
}

impl BucketMetricOptions {
    pub fn new(name: impl Into<String>, help: impl Into<String>, buckets: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            label_names: Vec::new(),
            buckets,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NumericMetricSample {
    pub labels: MetricLabels,
    pub value: i64,
}

impl NumericMetricSample {
    pub(crate) fn new(labels: MetricLabels, value: i64) -> Self {
        Self { labels, value }
    }

    pub(crate) fn zero() -> Self {
        Self {
            labels: HashMap::new(),
            value: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HistogramBucket {
    pub le: f64,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BucketMetricSample {
    pub labels: MetricLabels,
    pub count: i64,
    pub sum: f64,
    pub buckets: Vec<HistogramBucket>,
}

impl BucketMetricSample {
    pub(crate) fn zero(bucket_boundaries: &[f64]) -> Self {
        let buckets = bucket_boundaries
            .iter()
            .map(|&le| HistogramBucket { le, count: 0 })
            .collect();
        Self {
            labels: HashMap::new(),
            count: 0,
            sum: 0.0,
            buckets,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MetricSample {
    Numeric(NumericMetricSample),
    Bucket(BucketMetricSample),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CollectedMetric {
    pub name: String,
    pub help: String,
    #[serde(rename = "type")]
    pub metric_type: MetricType,
    pub samples: Vec<MetricSample>,
}

impl CollectedMetric {
    pub(crate) fn new_numeric(
        name: impl Into<String>,
        help: impl Into<String>,
        metric_type: MetricType,
        samples: Vec<NumericMetricSample>,
    ) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type,
            samples: samples.into_iter().map(MetricSample::Numeric).collect(),
        }
    }

    pub(crate) fn new_bucket(
        name: impl Into<String>,
        help: impl Into<String>,
        samples: Vec<BucketMetricSample>,
    ) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type: MetricType::Histogram,
            samples: samples.into_iter().map(MetricSample::Bucket).collect(),
        }
    }

    pub fn numeric_samples(&self) -> Vec<&NumericMetricSample> {
        self.samples
            .iter()
            .filter_map(|s| match s {
                MetricSample::Numeric(n) => Some(n),
                MetricSample::Bucket(_) => None,
            })
            .collect()
    }

    pub fn bucket_samples(&self) -> Vec<&BucketMetricSample> {
        self.samples
            .iter()
            .filter_map(|s| match s {
                MetricSample::Numeric(_) => None,
                MetricSample::Bucket(b) => Some(b),
            })
            .collect()
    }
}

pub(crate) fn get_label_key(labels: Option<&MetricLabels>) -> String {
    match labels {
        None => String::new(),
        Some(labels) if labels.is_empty() => String::new(),
        Some(labels) => {
            let mut pairs: Vec<_> = labels.iter().collect();
            pairs.sort_by_key(|(k, _)| *k);
            pairs
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}

pub(crate) fn parse_label_key(key: &str) -> MetricLabels {
    if key.is_empty() {
        return HashMap::new();
    }

    key.split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                _ => None,
            }
        })
        .collect()
}
