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

    pub fn with_label_names(
        name: impl Into<String>,
        help: impl Into<String>,
        label_names: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            label_names,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumericMetricSample {
    pub labels: MetricLabels,
    pub value: i64,
}

impl NumericMetricSample {
    pub fn new(labels: MetricLabels, value: i64) -> Self {
        Self { labels, value }
    }

    pub fn zero() -> Self {
        Self {
            labels: HashMap::new(),
            value: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectedMetric {
    pub name: String,
    pub help: String,
    #[serde(rename = "type")]
    pub metric_type: MetricType,
    pub samples: Vec<NumericMetricSample>,
}

impl CollectedMetric {
    pub fn new(
        name: impl Into<String>,
        help: impl Into<String>,
        metric_type: MetricType,
        samples: Vec<NumericMetricSample>,
    ) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type,
            samples,
        }
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
