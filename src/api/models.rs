use crate::errors::ServerError;
use crate::metrics::types::{Metric, MetricsBatch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub metrics_count: usize,
    pub uptime_seconds: u64,
    pub start_time: String,
}

pub trait Validate {
    fn validate(&self) -> Result<(), ServerError>;
}

impl Validate for Metric {
    fn validate(&self) -> Result<(), ServerError> {
        if self.name.is_empty() {
            return Err(ServerError::ValidationError(
                "Metric name cannot be empty".to_string(),
            ));
        }

        // name should contain only alphanumeric characters, underscores, and colons
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
        {
            return Err(ServerError::ValidationError(
                "Metric name must contain only alphanumeric characters, underscores, and colons"
                    .to_string(),
            ));
        }

        if self.help.is_empty() {
            return Err(ServerError::ValidationError(
                "Help text cannot be empty".to_string(),
            ));
        }

        for (key, _value) in &self.labels {
            if key.is_empty() {
                return Err(ServerError::ValidationError(
                    "Label name cannot be empty".to_string(),
                ));
            }

            if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(ServerError::ValidationError(
                    "Label names must contain only alphanumeric characters and underscores"
                        .to_string(),
                ));
            }

            // Prometheus has some reserved label names
            if key == "le" || key == "quantile" {
                return Err(ServerError::ValidationError(format!(
                    "'{}' is a reserved label name",
                    key
                )));
            }
        }

        Ok(())
    }
}

impl Validate for MetricsBatch {
    fn validate(&self) -> Result<(), ServerError> {
        if self.source.is_empty() {
            return Err(ServerError::ValidationError(
                "Source cannot be empty".to_string(),
            ));
        }

        if self.metrics.is_empty() {
            return Err(ServerError::ValidationError(
                "Batch must contain at least one metric".to_string(),
            ));
        }

        for metric in &self.metrics {
            metric.validate()?;
        }

        // There should be no duplicate metric names within the same set of labels
        let mut seen_metrics = HashMap::new();
        for metric in &self.metrics {
            // Create a unique string key for this metric by combining name and sorted labels
            let mut key = format!("{}:", metric.name);

            let mut label_pairs: Vec<(&String, &String)> = metric.labels.iter().collect();
            label_pairs.sort_by(|a, b| a.0.cmp(b.0));

            for (k, v) in label_pairs {
                key.push_str(&format!("{}={},", k, v));
            }

            if seen_metrics.contains_key(&key) {
                return Err(ServerError::ValidationError(format!(
                    "Duplicate metric found: {} with the same set of labels",
                    metric.name
                )));
            }
            seen_metrics.insert(key, true);
        }

        Ok(())
    }
}
