use rustic_insights::{
    AppConfig, AppState, MetricsCollector, MetricsRegistry, api::configure_routes,
};

use actix_web::{App, HttpServer, middleware, web};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up the logger");

    info!("Starting metrics server");

    let config = AppConfig::load().expect("Failed to load configuration");
    let server_config = config.server.clone();

    let metrics_registry = MetricsRegistry::new(config.metrics.clone());
    let metrics_collector = MetricsCollector::new(metrics_registry);

    let app_state = Arc::new(AppState {
        metrics_collector,
        start_time: SystemTime::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    info!(
        "Starting HTTP server at {}:{}",
        server_config.host, server_config.port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .configure(configure_routes)
    })
    .bind(format!("{}:{}", server_config.host, server_config.port))?
    .workers(server_config.workers)
    .run()
    .await
}
