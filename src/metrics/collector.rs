use crate::errors::ServerError;
use crate::metrics::registry::MetricsRegistry;
use crate::metrics::types::{Metric, MetricsBatch, MetricsResponse};
use tracing::{debug, error, instrument};

pub struct MetricsCollector {
    registry: MetricsRegistry,
}

impl MetricsCollector {
    pub fn new(registry: MetricsRegistry) -> Self {
        Self { registry }
    }

    #[instrument(skip(self, batch), fields(source = %batch.source))]
    pub async fn process_batch(&self, batch: MetricsBatch) -> Result<MetricsResponse, ServerError> {
        let mut response = MetricsResponse::default();
        let total_metrics = batch.metrics.len();

        debug!(
            "Processing batch of {} metrics from {}",
            total_metrics, batch.source
        );

        for metric in batch.metrics {
            match self.process_metric(metric).await {
                Ok(_) => {
                    response.processed += 1;
                }
                Err(e) => {
                    error!("Failed to process metric: {}", e);
                    response.errors.push(e.to_string());
                }
            }
        }

        if !response.errors.is_empty() {
            response.status = "partial_success".to_string();

            if response.processed == 0 {
                return Err(ServerError::MetricsProcessingError(
                    "Failed to process any metrics in the batch".to_string(),
                ));
            }
        }

        Ok(response)
    }

    #[instrument(skip(self, metric), fields(name = %metric.name, type = ?metric.metric_type))]
    async fn process_metric(&self, metric: Metric) -> Result<(), ServerError> {
        match self.registry.update_metric(&metric).await {
            Ok(_) => {
                debug!("Updated existing metric: {}", metric.name);
                Ok(())
            }
            Err(_) => {
                debug!("Metric not found, attempting to register: {}", metric.name);
                self.registry.register_metric(&metric).await?;

                self.registry.update_metric(&metric).await?;
                debug!("Registered and updated new metric: {}", metric.name);
                Ok(())
            }
        }
    }

    pub fn get_metrics(&self) -> Result<String, ServerError> {
        self.registry.gather()
    }

    pub async fn get_metrics_count(&self) -> Result<usize, ServerError> {
        self.registry.get_metrics_count().await
    }
}
