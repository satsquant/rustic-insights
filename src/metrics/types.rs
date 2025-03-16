use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub help: String,
    pub labels: HashMap<String, String>,
    pub value: MetricValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsBatch {
    pub metrics: Vec<Metric>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub processed: usize,
    pub status: String,
    pub errors: Vec<String>,
}

impl Default for MetricsResponse {
    fn default() -> Self {
        Self {
            processed: 0,
            status: "success".to_string(),
            errors: Vec::new(),
        }
    }
}
