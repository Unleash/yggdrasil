use crate::impact_metrics::types::{
    get_label_key, parse_label_key, BucketMetricOptions, BucketMetricSample, CollectedMetric,
    HistogramBucket, MetricLabels,
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Mutex;

const DEFAULT_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

struct HistogramData {
    count: i64,
    sum: f64,
    buckets: HashMap<u64, i64>,
}

impl HistogramData {
    fn new(count: i64, sum: f64, bucket_boundaries: &[f64]) -> Self {
        let mut buckets = HashMap::new();
        for &le in bucket_boundaries {
            buckets.insert(le.to_bits(), 0);
        }
        Self {
            count,
            sum,
            buckets,
        }
    }

    fn from_sample(sample: &BucketMetricSample, bucket_boundaries: &[f64]) -> Self {
        let mut buckets = HashMap::new();
        for &le in bucket_boundaries {
            buckets.insert(le.to_bits(), 0);
        }
        let mut data = Self {
            count: sample.count,
            sum: sample.sum,
            buckets,
        };
        for bucket in &sample.buckets {
            data.buckets.insert(bucket.le.to_bits(), bucket.count);
        }
        data
    }
}

pub struct Histogram {
    opts: BucketMetricOptions,
    buckets: Vec<f64>,
    values: DashMap<String, Mutex<HistogramData>>,
}

impl Histogram {
    pub(crate) fn new(opts: BucketMetricOptions) -> Self {
        let input_buckets = if opts.buckets.is_empty() {
            DEFAULT_BUCKETS.to_vec()
        } else {
            opts.buckets.clone()
        };

        let mut sorted: Vec<f64> = input_buckets
            .into_iter()
            .filter(|&b| !b.is_infinite() && !b.is_nan())
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
            .or_insert_with(|| Mutex::new(HistogramData::new(0, 0.0, &self.buckets)));

        let mut data = entry.lock().unwrap_or_else(|e| e.into_inner());
        data.count += 1;
        data.sum += value;

        for &le in &self.buckets {
            if value <= le {
                let bucket_key = le.to_bits();
                *data.buckets.entry(bucket_key).or_insert(0) += 1;
            }
        }
    }

    pub fn restore(&self, sample: &BucketMetricSample) {
        let key = get_label_key(Some(&sample.labels));

        let data = HistogramData::from_sample(sample, &self.buckets);

        self.values.insert(key, Mutex::new(data));
    }

    pub(crate) fn collect(&self) -> CollectedMetric {
        let mut samples = Vec::new();

        for entry in self.values.iter() {
            let key = entry.key();
            let mut data = entry.value().lock().unwrap_or_else(|e| e.into_inner());

            let bucket_samples: Vec<HistogramBucket> = self
                .buckets
                .iter()
                .map(|&le| {
                    let count = *data.buckets.get(&le.to_bits()).unwrap_or(&0);
                    HistogramBucket::new(le, count)
                })
                .collect();

            let count_snapshot = data.count;
            let sum_snapshot = data.sum;

            data.count = 0;
            data.sum = 0.0;
            for &le in &self.buckets {
                let bucket_key = le.to_bits();
                data.buckets.insert(bucket_key, 0);
            }

            samples.push(BucketMetricSample::new(
                parse_label_key(key),
                count_snapshot,
                sum_snapshot,
                bucket_samples,
            ));
        }

        self.values.retain(|_, v| {
            let data = v.get_mut().unwrap_or_else(|e| e.into_inner());
            data.count != 0 || data.sum != 0.0
        });

        if samples.is_empty() {
            samples.push(BucketMetricSample::zero(&self.buckets));
        }

        CollectedMetric::new_bucket(&self.opts.name, &self.opts.help, samples)
    }
}
