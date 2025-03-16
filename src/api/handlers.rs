use crate::api::models::{HealthResponse, StatusResponse, Validate};
use crate::errors::ServerError;
use crate::metrics::{MetricsBatch, MetricsCollector};
use actix_web::{HttpResponse, web};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, error, field, instrument};

pub struct AppState {
    pub metrics_collector: MetricsCollector,
    pub start_time: SystemTime,
    pub version: String,
}

#[instrument(skip(state))]
pub async fn health_check(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, ServerError> {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: state.version.clone(),
        timestamp: Utc::now().to_rfc3339(),
    };

    debug!("Health check performed");
    Ok(HttpResponse::Ok().json(response))
}

#[instrument(skip(state))]
pub async fn status(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, ServerError> {
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .map_err(|e| ServerError::InternalError(Box::new(e)))?;

    let start_time: DateTime<Utc> = state.start_time.clone().into();

    let metrics_count = state.metrics_collector.get_metrics_count().await?;

    let response = StatusResponse {
        status: "running".to_string(),
        metrics_count,
        uptime_seconds: uptime.as_secs(),
        start_time: start_time.to_rfc3339(),
    };

    debug!("Status check performed");
    Ok(HttpResponse::Ok().json(response))
}

#[instrument(skip(state))]
pub async fn metrics(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, ServerError> {
    let metrics_data = state.metrics_collector.get_metrics()?;

    debug!("Metrics endpoint called");
    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics_data))
}

#[instrument(skip(state, batch), fields(source = field::Empty, count = field::Empty))]
pub async fn ingest_metrics(
    state: web::Data<Arc<AppState>>,
    web::Json(batch): web::Json<MetricsBatch>,
) -> Result<HttpResponse, ServerError> {
    tracing::Span::current()
        .record("source", &batch.source.as_str())
        .record("count", &batch.metrics.len());

    debug!(
        "Received metrics batch with {} metrics",
        batch.metrics.len()
    );

    batch.validate()?;

    let response = match state.metrics_collector.process_batch(batch).await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to process metrics batch: {}", e);
            return Err(e);
        }
    };

    debug!("Processed {} metrics successfully", response.processed);
    Ok(HttpResponse::Ok().json(response))
}
