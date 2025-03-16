use crate::errors::ServerError;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub prometheus_endpoint: String,
    pub metrics_prefix: String,
    pub metrics_namespace: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub metrics: MetricsConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ServerError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config_builder = Config::builder()
            .add_source(File::with_name("config/default"))
            // Add environment-specific settings
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add local overrides
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP"
            .add_source(Environment::with_prefix("APP").separator("__"));

        let config = config_builder
            .build()
            .map_err(|e| ServerError::ConfigurationError(e.to_string()))?;

        let app_config: AppConfig = config
            .try_deserialize()
            .map_err(|e| ServerError::ConfigurationError(e.to_string()))?;

        Ok(app_config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: num_cpus::get(),
            },
            metrics: MetricsConfig {
                prometheus_endpoint: "/metrics".to_string(),
                metrics_prefix: "app".to_string(),
                metrics_namespace: "metrics_server".to_string(),
            },
        }
    }
}
