use crate::errors::ServerError;
use regex::Regex;
use std::collections::HashMap;
use tracing::warn;

pub fn validate_metric_name(name: &str) -> Result<(), ServerError> {
    // Prometheus metric names must match [a-zA-Z_:][a-zA-Z0-9_:]*
    let re = Regex::new(r"^[a-zA-Z_:][a-zA-Z0-9_:]*$").unwrap();

    if !re.is_match(name) {
        warn!("Invalid metric name: {}", name);
        return Err(ServerError::ValidationError(format!(
            "Invalid metric name: {}. Must match [a-zA-Z_:][a-zA-Z0-9_:]*",
            name
        )));
    }

    Ok(())
}

pub fn validate_label_names(labels: &HashMap<String, String>) -> Result<(), ServerError> {
    // Prometheus label names must match [a-zA-Z_][a-zA-Z0-9_]*
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();

    for (key, _) in labels {
        if !re.is_match(key) {
            warn!("Invalid label name: {}", key);
            return Err(ServerError::ValidationError(format!(
                "Invalid label name: {}. Must match [a-zA-Z_][a-zA-Z0-9_]*",
                key
            )));
        }

        // Check reserved label names
        if key == "le" || key == "quantile" {
            warn!("Reserved label name used: {}", key);
            return Err(ServerError::ValidationError(format!(
                "'{}' is a reserved label name in Prometheus",
                key
            )));
        }
    }

    Ok(())
}

pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), ServerError> {
    if value.is_empty() {
        warn!("Empty value for field: {}", field_name);
        return Err(ServerError::ValidationError(format!(
            "{} cannot be empty",
            field_name
        )));
    }

    Ok(())
}
