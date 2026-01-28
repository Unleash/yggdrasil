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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterMetricSample {
    pub labels: MetricLabels,
    pub value: i64,
}

impl CounterMetricSample {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaugeMetricSample {
    pub labels: MetricLabels,
    pub value: f64,
}

impl GaugeMetricSample {
    pub(crate) fn new(labels: MetricLabels, value: f64) -> Self {
        Self { labels, value }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramBucket {
    #[serde(serialize_with = "serialize_le", deserialize_with = "deserialize_le")]
    pub le: f64,
    pub count: i64,
}

fn serialize_le<S>(le: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if le.is_infinite() {
        serializer.serialize_str("+Inf")
    } else {
        serializer.serialize_f64(*le)
    }
}

fn deserialize_le<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum F64OrInf {
        Num(f64),
        Str(String),
    }

    match F64OrInf::deserialize(d)? {
        F64OrInf::Num(x) => Ok(x),
        F64OrInf::Str(s) if s == "+Inf" => Ok(f64::INFINITY),
        F64OrInf::Str(s) => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(&s),
            &"a number or '+Inf' string",
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricSample {
    Counter(CounterMetricSample),
    Gauge(GaugeMetricSample),
    Bucket(BucketMetricSample),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectedMetric {
    pub name: String,
    pub help: String,
    #[serde(rename = "type")]
    pub metric_type: MetricType,
    pub samples: Vec<MetricSample>,
}

impl CollectedMetric {
    pub(crate) fn new_counter(
        name: impl Into<String>,
        help: impl Into<String>,
        samples: Vec<CounterMetricSample>,
    ) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type: MetricType::Counter,
            samples: samples.into_iter().map(MetricSample::Counter).collect(),
        }
    }

    pub(crate) fn new_gauge(
        name: impl Into<String>,
        help: impl Into<String>,
        samples: Vec<GaugeMetricSample>,
    ) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type: MetricType::Gauge,
            samples: samples.into_iter().map(MetricSample::Gauge).collect(),
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

    pub fn counter_samples(&self) -> Vec<&CounterMetricSample> {
        self.samples
            .iter()
            .filter_map(|s| match s {
                MetricSample::Counter(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    pub fn gauge_samples(&self) -> Vec<&GaugeMetricSample> {
        self.samples
            .iter()
            .filter_map(|s| match s {
                MetricSample::Gauge(g) => Some(g),
                _ => None,
            })
            .collect()
    }

    pub fn bucket_samples(&self) -> Vec<&BucketMetricSample> {
        self.samples
            .iter()
            .filter_map(|s| match s {
                MetricSample::Bucket(b) => Some(b),
                _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_serialize_infinity_to_plus_inf() {
        let bucket = HistogramBucket {
            le: f64::INFINITY,
            count: 5,
        };
        let json = serde_json::to_string(&bucket).unwrap();
        assert!(json.contains("\"le\":\"+Inf\""));
    }

    #[test]
    fn test_deserialize_plus_inf_to_infinity() {
        let json = r#"{"le":"+Inf","count":5}"#;
        let bucket: HistogramBucket = serde_json::from_str(json).unwrap();
        assert_eq!(bucket.le, f64::INFINITY);
    }

    #[test]
    fn test_round_trip_serialization_infinity() {
        let original = HistogramBucket {
            le: f64::INFINITY,
            count: 42,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HistogramBucket = serde_json::from_str(&json).unwrap();
        assert_eq!(original.le, deserialized.le);
    }

    #[test]
    fn test_serialize_deserialize_in_collected_metric() {
        let buckets = vec![
            HistogramBucket { le: 0.1, count: 1 },
            HistogramBucket { le: 0.5, count: 2 },
            HistogramBucket {
                le: f64::INFINITY,
                count: 3,
            },
        ];
        let sample = BucketMetricSample {
            labels: HashMap::new(),
            count: 3,
            sum: 0.7,
            buckets,
        };
        let metric = CollectedMetric::new_bucket("test_histogram", "test help", vec![sample]);

        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains("\"le\":\"+Inf\""));

        let deserialized: CollectedMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test_histogram");
        let bucket_samples = deserialized.bucket_samples();
        assert_eq!(bucket_samples.len(), 1);
        let buckets = &bucket_samples[0].buckets;
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[2].le, f64::INFINITY);
        assert_eq!(buckets[2].count, 3);
    }
}
