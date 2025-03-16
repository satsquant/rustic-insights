use rustic_insights::{
    config::AppConfig,
    metrics::{Metric, MetricType, MetricValue, MetricsBatch, MetricsCollector, MetricsRegistry},
};
use std::collections::HashMap;

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
        help: format!("Test {:?} metric", metric_type),
        labels,
        value: MetricValue {
            value,
            timestamp: None,
        },
    }
}

fn create_test_registry() -> MetricsRegistry {
    let config = AppConfig::default();
    MetricsRegistry::new(config.metrics.clone())
}

#[tokio::test]
async fn test_register_histogram() {
    let registry = create_test_registry();
    let metric = create_test_metric("test_histogram", MetricType::Histogram, 0.235, None);

    let result = registry.register_metric(&metric).await;
    assert!(result.is_ok(), "Failed to register histogram: {:?}", result);

    let update_result = registry.update_metric(&metric).await;
    assert!(
        update_result.is_ok(),
        "Failed to update histogram: {:?}",
        update_result
    );

    let metric2 = create_test_metric("test_histogram", MetricType::Histogram, 1.5, None);
    let update_result2 = registry.update_metric(&metric2).await;
    assert!(
        update_result2.is_ok(),
        "Failed to update histogram with second value: {:?}",
        update_result2
    );

    let count = registry.get_metrics_count().await.unwrap();
    assert_eq!(count, 1, "Should have exactly one metric registered");

    let metrics_data = registry.gather().unwrap();
    println!("Metrics data length: {}", metrics_data.len());

    if !metrics_data.is_empty() {
        println!(
            "Metrics data first 200 chars: {}",
            &metrics_data[..std::cmp::min(200, metrics_data.len())]
        );
    } else {
        println!("Metrics data is empty!");
    }

    let full_name = "app_metrics_server_test_histogram";
    println!("Looking for: '{}'", full_name);

    assert!(!metrics_data.is_empty(), "Metrics data should not be empty");

    assert!(
        metrics_data.contains(full_name),
        "Metrics data should contain '{}' but got: {}",
        full_name,
        metrics_data
    );
}

#[tokio::test]
async fn test_update_counter() {
    let registry = create_test_registry();
    let metric1 = create_test_metric("test_counter", MetricType::Counter, 1.0, None);
    let metric2 = create_test_metric("test_counter", MetricType::Counter, 2.0, None);

    registry.register_metric(&metric1).await.unwrap();

    let result = registry.update_metric(&metric2).await;
    assert!(result.is_ok());

    let metrics_data = registry.gather().unwrap();
    assert!(metrics_data.contains("test_counter"));
}

#[tokio::test]
async fn test_update_gauge() {
    let registry = create_test_registry();
    let metric1 = create_test_metric("test_gauge", MetricType::Gauge, 42.5, None);
    let metric2 = create_test_metric("test_gauge", MetricType::Gauge, 50.0, None);

    registry.register_metric(&metric1).await.unwrap();

    let result = registry.update_metric(&metric2).await;
    assert!(result.is_ok());

    let metrics_data = registry.gather().unwrap();
    assert!(metrics_data.contains("test_gauge"));
}

#[tokio::test]
async fn test_metrics_count() {
    let registry = create_test_registry();

    let count = registry.get_metrics_count().await.unwrap();
    assert_eq!(count, 0);

    let counter = create_test_metric("test_counter", MetricType::Counter, 1.0, None);
    registry.register_metric(&counter).await.unwrap();

    let count = registry.get_metrics_count().await.unwrap();
    assert_eq!(count, 1);

    let gauge = create_test_metric("test_gauge", MetricType::Gauge, 42.5, None);
    registry.register_metric(&gauge).await.unwrap();

    let count = registry.get_metrics_count().await.unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_different_label_sets() {
    let registry = create_test_registry();

    let mut labels1 = HashMap::new();
    labels1.insert("service".to_string(), "service1".to_string());

    let mut labels2 = HashMap::new();
    labels2.insert("service".to_string(), "service2".to_string());

    let counter1 = create_test_metric("test_counter", MetricType::Counter, 1.0, Some(labels1));
    let counter2 = create_test_metric("test_counter", MetricType::Counter, 1.0, Some(labels2));

    registry.register_metric(&counter1).await.unwrap();
    registry.register_metric(&counter2).await.unwrap();

    let count = registry.get_metrics_count().await.unwrap();
    assert_eq!(count, 1);

    let _ = registry.update_metric(&counter1).await;

    let metrics_data = registry.gather().unwrap();

    let full_name = "app_metrics_server_test_counter";
    println!("Looking for: '{}'", full_name);

    assert!(!metrics_data.is_empty(), "Metrics data should not be empty");

    assert!(
        metrics_data.contains(full_name),
        "Metrics data should contain '{}' but got: {}",
        full_name,
        metrics_data
    );
}

#[tokio::test]
async fn test_metrics_collector_process_batch() {
    let registry = create_test_registry();
    let collector = MetricsCollector::new(registry);

    let counter = create_test_metric("request_count", MetricType::Counter, 42.0, None);
    let gauge = create_test_metric("memory_usage", MetricType::Gauge, 128.5, None);
    let histogram = create_test_metric("response_time", MetricType::Histogram, 0.235, None);

    let batch = MetricsBatch {
        metrics: vec![counter, gauge, histogram],
        source: "test_app".to_string(),
    };

    let result = collector.process_batch(batch).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.processed, 3);
    assert_eq!(response.status, "success");
    assert!(response.errors.is_empty());
}

#[tokio::test]
async fn test_invalid_update_without_register() {
    let registry = create_test_registry();
    let metric = create_test_metric("test_counter", MetricType::Counter, 1.0, None);

    let result = registry.update_metric(&metric).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mismatched_metric_types() {
    let registry = create_test_registry();

    let counter = create_test_metric("test_metric", MetricType::Counter, 1.0, None);
    registry.register_metric(&counter).await.unwrap();

    let gauge = create_test_metric("test_metric", MetricType::Gauge, 42.5, None);
    let result = registry.update_metric(&gauge).await;

    assert!(result.is_err());
}
