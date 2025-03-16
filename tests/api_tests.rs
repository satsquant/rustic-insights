use actix_web::{App, http::StatusCode, test, web};
use rustic_insights::{
    AppConfig, AppState, Metric, MetricType, MetricValue, MetricsBatch, MetricsCollector,
    MetricsRegistry, api::configure_routes,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

fn create_test_app_state() -> Arc<AppState> {
    let config = AppConfig::default();
    let metrics_registry = MetricsRegistry::new(config.metrics.clone());
    let metrics_collector = MetricsCollector::new(metrics_registry);

    Arc::new(AppState {
        metrics_collector,
        start_time: SystemTime::now(),
        version: "0.1.0".to_string(),
    })
}

fn create_test_metric(
    name: &str,
    metric_type: MetricType,
    value: f64,
    labels: Option<HashMap<String, String>>,
) -> Metric {
    let labels = labels.unwrap_or_else(|| {
        let mut map = HashMap::new();
        map.insert("service".to_string(), "test_service".to_string());
        map.insert("instance".to_string(), "test_instance".to_string());
        map
    });

    Metric {
        name: name.to_string(),
        metric_type: metric_type.clone(),
        help: format!("Test {:#?} metric", metric_type),
        labels,
        value: MetricValue {
            value,
            timestamp: None,
        },
    }
}

#[actix_rt::test]
async fn test_health_check() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["status"], "ok");
    assert_eq!(response["version"], "0.1.0");
    assert!(response["timestamp"].is_string());
}

#[actix_rt::test]
async fn test_status_endpoint() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["status"], "running");
    assert!(response["uptime_seconds"].is_number());
    assert!(response["metrics_count"].is_number());
    assert!(response["start_time"].is_string());
}

#[actix_rt::test]
async fn test_prometheus_metrics_endpoint() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/plain"));
}

#[actix_rt::test]
async fn test_ingest_single_counter_metric() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let metric = create_test_metric("request_count", MetricType::Counter, 42.0, None);

    let batch = MetricsBatch {
        metrics: vec![metric],
        source: "test_app".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["processed"], 1);
    assert_eq!(response["status"], "success");
    assert!(response["errors"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn test_ingest_single_gauge_metric() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let metric = create_test_metric("memory_usage", MetricType::Gauge, 128.5, None);

    let batch = MetricsBatch {
        metrics: vec![metric],
        source: "test_app".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["processed"], 1);
    assert_eq!(response["status"], "success");
}

#[actix_rt::test]
async fn test_ingest_single_histogram_metric() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let metric = create_test_metric("response_time", MetricType::Histogram, 0.235, None);

    let batch = MetricsBatch {
        metrics: vec![metric],
        source: "test_app".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["processed"], 1);
    assert_eq!(response["status"], "success");
}

#[actix_rt::test]
async fn test_ingest_multiple_metrics() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let counter = create_test_metric("request_count", MetricType::Counter, 42.0, None);
    let gauge = create_test_metric("memory_usage", MetricType::Gauge, 128.5, None);
    let histogram = create_test_metric("response_time", MetricType::Histogram, 0.235, None);

    let batch = MetricsBatch {
        metrics: vec![counter, gauge, histogram],
        source: "test_app".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["processed"], 3);
    assert_eq!(response["status"], "success");
}

#[actix_rt::test]
async fn test_invalid_metric_name() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let mut labels = HashMap::new();
    labels.insert("service".to_string(), "test_service".to_string());

    let batch = json!({
        "metrics": [{
            "name": "invalid metric name with spaces",
            "metric_type": "counter",
            "help": "Test counter metric",
            "labels": labels,
            "value": {
                "value": 42.0,
                "timestamp": null
            }
        }],
        "source": "test_app"
    });

    let req = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_empty_source() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let metric = create_test_metric("request_count", MetricType::Counter, 42.0, None);

    let batch = MetricsBatch {
        metrics: vec![metric],
        source: "".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_update_existing_metric() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes),
    )
    .await;

    let metric1 = create_test_metric("request_count", MetricType::Counter, 42.0, None);

    let batch1 = MetricsBatch {
        metrics: vec![metric1],
        source: "test_app".to_string(),
    };

    let req1 = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch1)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::OK);

    let metric2 = create_test_metric("request_count", MetricType::Counter, 10.0, None);

    let batch2 = MetricsBatch {
        metrics: vec![metric2],
        source: "test_app".to_string(),
    };

    let req2 = test::TestRequest::post()
        .uri("/api/metrics")
        .set_json(&batch2)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;

    assert_eq!(resp2.status(), StatusCode::OK);

    let body = test::read_body(resp2).await;
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["processed"], 1);
    assert_eq!(response["status"], "success");
}
