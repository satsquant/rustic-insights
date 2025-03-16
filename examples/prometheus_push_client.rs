use prometheus::{Gauge, Histogram, HistogramOpts, IntCounter, Opts, Registry};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

/// Example application that collects metrics with the Prometheus client
/// and pushes them to our metrics collection server
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Prometheus Push Client Example");

    let client = Client::new();

    let metrics_server_url = "http://localhost:8080/api/metrics";
    let application_name = "example_app";
    let push_interval_secs = 5;

    let registry = Registry::new();

    // Create some example metrics
    // 1. A simple counter
    let counter_opts = Opts::new("http_requests_total", "Total number of HTTP requests")
        .const_label("application", application_name);
    let request_counter = IntCounter::with_opts(counter_opts)?;
    registry.register(Box::new(request_counter.clone()))?;

    // 2. A gauge for memory usage
    let gauge_opts = Opts::new("memory_usage_bytes", "Current memory usage in bytes")
        .const_label("application", application_name);
    let memory_gauge = Gauge::with_opts(gauge_opts)?;
    registry.register(Box::new(memory_gauge.clone()))?;

    // 3. A histogram for request duration
    let histogram_opts = HistogramOpts::new(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
    )
    .const_label("application", application_name)
    .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]);
    let request_histogram = Histogram::with_opts(histogram_opts)?;
    registry.register(Box::new(request_histogram.clone()))?;

    println!(
        "Metrics initialized. Will push metrics every {} seconds",
        push_interval_secs
    );

    // Simulate application activity and push metrics
    let mut iteration = 0;
    loop {
        iteration += 1;
        println!("\nIteration {}:", iteration);

        // Simulate some application activity
        simulate_activity(
            &request_counter,
            &memory_gauge,
            &request_histogram,
            iteration,
        );

        // Push metrics to the server
        match push_metrics_to_server(&client, metrics_server_url, &registry, application_name).await
        {
            Ok(response) => println!("✅ Metrics pushed successfully: {}", response),
            Err(e) => println!("❌ Failed to push metrics: {}", e),
        }

        // Wait before next iteration
        sleep(Duration::from_secs(push_interval_secs));
    }
}

/// Simulate application activity by updating metrics
fn simulate_activity(
    request_counter: &IntCounter,
    memory_gauge: &Gauge,
    request_histogram: &Histogram,
    iteration: u64,
) {
    // Simulate HTTP requests
    let requests = iteration * 5;
    request_counter.inc_by(requests);
    println!("- Processed {} HTTP requests", requests);

    // Simulate memory usage (oscillating pattern)
    let memory_usage = 100_000_000.0 + (iteration as f64 * 10_000.0) % 50_000_000.0;
    memory_gauge.set(memory_usage);
    println!("- Memory usage: {:.2} MB", memory_usage / 1_000_000.0);

    // Simulate request durations
    for _ in 0..5 {
        // Generate some reasonable response times between 50ms and 2s
        let duration = 0.05 + (iteration as f64 % 10.0) / 5.0;
        request_histogram.observe(duration);
        println!("- Request duration: {:.3}s", duration);
    }
}

/// Push metrics to our metrics collection server
async fn push_metrics_to_server(
    client: &Client,
    url: &str,
    registry: &Registry,
    source: &str,
) -> Result<String, Box<dyn Error>> {
    // Gather metrics from Prometheus registry
    let metric_families = registry.gather();

    // Create a batch of metrics to send
    let mut metrics = Vec::new();

    // Process each metric family
    for mf in metric_families {
        let name = mf.get_name();
        let help = mf.get_help();

        // Process each metric in the family
        for m in mf.get_metric() {
            // Extract labels
            let mut labels = HashMap::new();
            for label in m.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }

            // Determine metric type and value
            let (metric_type, value) = match mf.get_field_type() {
                prometheus::proto::MetricType::COUNTER => ("counter", m.get_counter().get_value()),
                prometheus::proto::MetricType::GAUGE => ("gauge", m.get_gauge().get_value()),
                prometheus::proto::MetricType::HISTOGRAM => {
                    // For histograms, we'll use the sum as the value
                    // In a real system, you might want to handle this differently
                    ("histogram", m.get_histogram().get_sample_sum())
                }
                _ => {
                    // Skip other metric types for simplicity
                    continue;
                }
            };

            // Create metric
            let metric = json!({
                "name": name,
                "metric_type": metric_type,
                "help": help,
                "labels": labels,
                "value": {
                    "value": value,
                    "timestamp": chrono::Utc::now().timestamp()
                }
            });

            metrics.push(metric);
        }
    }

    // Create metrics batch
    let batch = json!({
        "metrics": metrics,
        "source": source
    });

    // Send request
    let response = client.post(url).json(&batch).send().await?;

    // Handle response
    if response.status().is_success() {
        let text = response.text().await?;
        Ok(text)
    } else {
        let status = response.status();
        let text = response.text().await?;
        Err(format!("HTTP error {}: {}", status, text).into())
    }
}
