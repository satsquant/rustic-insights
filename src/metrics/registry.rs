use crate::config::MetricsConfig;
use crate::errors::ServerError;
use crate::metrics::types::{Metric, MetricType};
use prometheus::{
    CounterVec, Encoder, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MetricsRegistry {
    registry: Arc<Registry>,
    counters: Arc<RwLock<HashMap<String, CounterVec>>>,
    gauges: Arc<RwLock<HashMap<String, GaugeVec>>>,
    histograms: Arc<RwLock<HashMap<String, HistogramVec>>>,
    label_keys: RwLock<HashMap<String, Vec<String>>>,
    config: MetricsConfig,
}

impl MetricsRegistry {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            registry: Arc::new(Registry::new()),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            label_keys: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn register_metric(&self, metric: &Metric) -> Result<(), ServerError> {
        let full_name = format!(
            "{}_{}_{}",
            self.config.metrics_prefix, self.config.metrics_namespace, metric.name
        );

        let mut label_keys: Vec<String> = metric.labels.keys().cloned().collect();
        label_keys.sort();

        let label_keys_str: Vec<&str> = label_keys.iter().map(|s| s.as_str()).collect();

        match metric.metric_type {
            MetricType::Counter => {
                self.register_counter(&full_name, &metric.help, label_keys_str)
                    .await?;
            }
            MetricType::Gauge => {
                self.register_gauge(&full_name, &metric.help, label_keys_str)
                    .await?;
            }
            MetricType::Histogram => {
                self.register_histogram(&full_name, &metric.help, label_keys_str)
                    .await?;
            }
            MetricType::Summary => {
                return Err(ServerError::MetricRegistrationError(
                    "Summary metrics are not supported yet".to_string(),
                ));
            }
        }

        let mut label_keys_map = self.label_keys.write().await;
        label_keys_map.insert(full_name, label_keys);

        Ok(())
    }

    pub async fn update_metric(&self, metric: &Metric) -> Result<(), ServerError> {
        let full_name = format!(
            "{}_{}_{}",
            self.config.metrics_prefix, self.config.metrics_namespace, metric.name
        );

        let label_keys_map = self.label_keys.read().await;
        let label_keys = label_keys_map.get(&full_name).ok_or_else(|| {
            ServerError::MetricsProcessingError(format!("Metric '{}' not registered", full_name))
        })?;

        let label_values: Vec<&str> = label_keys
            .iter()
            .map(|key| metric.labels.get(key).map(|v| v.as_str()).unwrap_or(""))
            .collect();

        match metric.metric_type {
            MetricType::Counter => {
                let counters = self.counters.read().await;
                if let Some(counter) = counters.get(&full_name) {
                    let c = counter.with_label_values(&label_values);
                    c.inc_by(metric.value.value);
                } else {
                    return Err(ServerError::MetricsProcessingError(format!(
                        "Counter '{}' not registered",
                        full_name
                    )));
                }
            }
            MetricType::Gauge => {
                let gauges = self.gauges.read().await;
                if let Some(gauge) = gauges.get(&full_name) {
                    let g = gauge.with_label_values(&label_values);
                    g.set(metric.value.value);
                } else {
                    return Err(ServerError::MetricsProcessingError(format!(
                        "Gauge '{}' not registered",
                        full_name
                    )));
                }
            }
            MetricType::Histogram => {
                let histograms = self.histograms.read().await;
                if let Some(histogram) = histograms.get(&full_name) {
                    let h = histogram.with_label_values(&label_values);
                    h.observe(metric.value.value);
                } else {
                    return Err(ServerError::MetricsProcessingError(format!(
                        "Histogram '{}' not registered",
                        full_name
                    )));
                }
            }
            MetricType::Summary => {
                return Err(ServerError::MetricsProcessingError(
                    "Summary metrics are not supported yet".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn gather(&self) -> Result<String, ServerError> {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        if metric_families.is_empty() {
            tracing::warn!("No metrics were gathered from the registry");
            return Ok("# No metrics found in registry\n".to_string());
        }

        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| ServerError::MetricsProcessingError(e.to_string()))?;

        String::from_utf8(buffer).map_err(|e| ServerError::MetricsProcessingError(e.to_string()))
    }

    pub async fn get_metrics_count(&self) -> Result<usize, ServerError> {
        let counters_count = self.counters.read().await.len();
        let gauges_count = self.gauges.read().await.len();
        let histograms_count = self.histograms.read().await.len();

        Ok(counters_count + gauges_count + histograms_count)
    }

    async fn register_counter(
        &self,
        name: &str,
        help: &str,
        label_names: Vec<&str>,
    ) -> Result<(), ServerError> {
        let mut counters = self.counters.write().await;
        if !counters.contains_key(name) {
            let opts = Opts::new(name, help);
            let counter = CounterVec::new(opts, &label_names)
                .map_err(|e| ServerError::MetricRegistrationError(e.to_string()))?;

            self.registry
                .register(Box::new(counter.clone()))
                .map_err(|e| ServerError::MetricRegistrationError(e.to_string()))?;

            counters.insert(name.to_string(), counter);
        }
        Ok(())
    }

    async fn register_gauge(
        &self,
        name: &str,
        help: &str,
        label_names: Vec<&str>,
    ) -> Result<(), ServerError> {
        let mut gauges = self.gauges.write().await;
        if !gauges.contains_key(name) {
            let opts = Opts::new(name, help);
            let gauge = GaugeVec::new(opts, &label_names)
                .map_err(|e| ServerError::MetricRegistrationError(e.to_string()))?;

            self.registry
                .register(Box::new(gauge.clone()))
                .map_err(|e| ServerError::MetricRegistrationError(e.to_string()))?;

            gauges.insert(name.to_string(), gauge);
        }
        Ok(())
    }

    async fn register_histogram(
        &self,
        name: &str,
        help: &str,
        label_names: Vec<&str>,
    ) -> Result<(), ServerError> {
        let mut histograms = self.histograms.write().await;
        if !histograms.contains_key(name) {
            let opts = HistogramOpts::new(name, help);
            let histogram = HistogramVec::new(opts, &label_names)
                .map_err(|e| ServerError::MetricRegistrationError(e.to_string()))?;

            self.registry
                .register(Box::new(histogram.clone()))
                .map_err(|e| ServerError::MetricRegistrationError(e.to_string()))?;

            histograms.insert(name.to_string(), histogram);
        }
        Ok(())
    }
}
