use crate::api::handlers::{health_check, ingest_metrics, metrics, status};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/status", web::get().to(status))
            .route("/metrics", web::post().to(ingest_metrics)),
    )
    .route("/metrics", web::get().to(metrics));
}
