
# Metrics Server

A sophisticated metrics collection server built with Rust that integrates with Prometheus and Grafana.

## Features

- **High-performance** metrics collection using Actix-web
- **Prometheus integration** for metrics storage and querying
- **Grafana dashboards** for visualization
- **Idiomatic Rust** with proper error handling using thiserror
- **Docker setup** for easy deployment

## Architecture

This system consists of three main components:

1. **Metrics Collection Server**: A Rust-based HTTP server that receives metrics from various services
2. **Prometheus**: Time-series database for storing and querying metrics
3. **Grafana**: Visualization and dashboarding tool

## Getting Started

### Prerequisites

- Rust 1.76+
- Docker and Docker Compose

### Running with Docker Compose

```bash
# Build and start all services
docker-compose up -d

# Check logs
docker-compose logs -f

# Stop all services
docker-compose down
```

### Running the Metrics Server Locally

```bash
# Build the project
cargo build

# Run the server
cargo run

# Run tests
cargo test
```

## API Endpoints

### Metrics Collection

- **POST** `/api/metrics`: Submit metrics batch
  - Request Body: JSON containing metrics batch
  - Response: JSON with processing results

### Monitoring

- **GET** `/metrics`: Prometheus metrics endpoint
- **GET** `/api/health`: Health check endpoint
- **GET** `/api/status`: Server status endpoint

## Configuration

Configuration is managed through environment variables or config files:

- `APP__SERVER__HOST`: Server host (default: 127.0.0.1)
- `APP__SERVER__PORT`: Server port (default: 8080)
- `APP__METRICS__PROMETHEUS_ENDPOINT`: Prometheus endpoint (default: /metrics)
- `APP__METRICS__METRICS_PREFIX`: Metrics prefix (default: app)
- `APP__METRICS__METRICS_NAMESPACE`: Metrics namespace (default: metrics_server)


## Submitting Metrics

Metrics can be submitted as a JSON batch:

```json
{
  "metrics": [
    {
      "name": "request_count",
      "metric_type": "counter",
      "help": "Total number of requests",
      "labels": {
        "service": "api_gateway",
        "endpoint": "/users"
      },
      "value": {
        "value": 42.0,
        "timestamp": null
      }
    }
  ],
  "source": "my_application"
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
