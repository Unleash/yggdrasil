use crate::impact_metrics::types::{
    get_label_key, parse_label_key, BucketMetricOptions, BucketMetricSample, CollectedMetric,
    HistogramBucket, MetricLabels,
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Mutex;

/// Default Prometheus-style buckets
const DEFAULT_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Internal histogram data for a single label set
struct HistogramData {
    count: i64,
    sum: f64,
    buckets: HashMap<u64, i64>, // bucket boundary (as bits) -> count
}

impl HistogramData {
    fn new(bucket_boundaries: &[f64]) -> Self {
        let mut buckets = HashMap::new();
        for &le in bucket_boundaries {
            buckets.insert(le.to_bits(), 0);
        }
        Self {
            count: 0,
            sum: 0.0,
            buckets,
        }
    }
}

pub struct Histogram {
    opts: BucketMetricOptions,
    buckets: Vec<f64>, // sorted bucket boundaries including +Inf
    values: DashMap<String, Mutex<HistogramData>>,
}

impl Histogram {
    pub(crate) fn new(opts: BucketMetricOptions) -> Self {
        // Use default buckets if none provided
        let input_buckets = if opts.buckets.is_empty() {
            DEFAULT_BUCKETS.to_vec()
        } else {
            opts.buckets.clone()
        };

        // Sort, dedupe, filter out infinity, then add infinity at the end
        let mut sorted: Vec<f64> = input_buckets
            .into_iter()
            .filter(|&b| !b.is_infinite())
            .collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted.dedup();
        sorted.push(f64::INFINITY);

        Self {
            opts,
            buckets: sorted,
            values: DashMap::new(),
        }
    }

    pub fn observe(&self, value: f64) {
        self.observe_internal(value, None);
    }

    pub fn observe_with_labels(&self, value: f64, labels: &MetricLabels) {
        self.observe_internal(value, Some(labels));
    }

    fn observe_internal(&self, value: f64, labels: Option<&MetricLabels>) {
        let key = get_label_key(labels);

        let entry = self
            .values
            .entry(key)
            .or_insert_with(|| Mutex::new(HistogramData::new(&self.buckets)));

        let mut data = entry.lock().unwrap();
        data.count += 1;
        data.sum += value;

        // Increment all buckets where value <= bucket boundary (cumulative)
        for &le in &self.buckets {
            if value <= le {
                let bucket_key = le.to_bits();
                *data.buckets.entry(bucket_key).or_insert(0) += 1;
            }
        }
    }

    pub fn restore(&self, sample: &BucketMetricSample) {
        let key = get_label_key(Some(&sample.labels));

        let mut data = HistogramData::new(&self.buckets);
        data.count = sample.count;
        data.sum = sample.sum;

        for bucket in &sample.buckets {
            data.buckets.insert(bucket.le.to_bits(), bucket.count);
        }

        self.values.insert(key, Mutex::new(data));
    }

    pub(crate) fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        for entry in self.values.iter() {
            let key = entry.key();
            let data = entry.value().lock().unwrap();

            let bucket_samples: Vec<HistogramBucket> = self
                .buckets
                .iter()
                .map(|&le| {
                    let count = *data.buckets.get(&le.to_bits()).unwrap_or(&0);
                    HistogramBucket::new(le, count)
                })
                .collect();

            samples.push(BucketMetricSample::new(
                parse_label_key(key),
                data.count,
                data.sum,
                bucket_samples,
            ));
        }

        // Clear after collection
        self.values.clear();

        // If empty, return zero sample with all bucket boundaries
        if samples.is_empty() {
            samples.push(BucketMetricSample::zero(&self.buckets));
        }

        CollectedMetric::new_bucket(&self.opts.name, &self.opts.help, samples)
    }
}
