# alert-manager-api

[![CI](https://github.com/rlgrpe/alert-manager-api/actions/workflows/ci.yml/badge.svg)](https://github.com/rlgrpe/alert-manager-api/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust async client library for [Prometheus Alertmanager](https://prometheus.io/docs/alerting/latest/alertmanager/).

## Features

- Async/await API built on `tokio` and `reqwest`
- Builder pattern for constructing alerts
- Batch alert sending
- Alert resolution support
- Custom middleware support via `reqwest-middleware`
- Comprehensive error handling with retry hints
- `tracing` integration for observability
- TLS support (native-tls or rustls)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
alert-manager-api = { git = "https://github.com/rlgrpe/alert-manager-api", tag = "v0.1.2" }
```

For rustls instead of native-tls:

```toml
[dependencies]
alert-manager-api = { git = "https://github.com/rlgrpe/alert-manager-api", tag = "v0.1.2", default-features = false, features = ["rustls-tls"] }
```

## Quick Start

```rust
use alert_manager_api::{AlertmanagerClient, Alert, AlertSeverity};
use std::time::Duration;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = AlertmanagerClient::new(
        Url::parse("http://localhost:9093")?,
        Duration::from_secs(10),
    )?;

    // Build and send an alert
    let alert = Alert::new("HighMemoryUsage")
        .with_severity(AlertSeverity::Warning)
        .with_label("service", "my-app")
        .with_label("instance", "localhost:8080")
        .with_summary("Memory usage is above 90%")
        .with_description("The service is using more than 90% of available memory");

    client.push_alert(alert).await?;
    Ok(())
}
```

## API Overview

### AlertmanagerClient

The main client for interacting with Alertmanager.

```rust
// Create with default HTTP client
let client = AlertmanagerClient::new(url, timeout)?;

// Create with custom middleware (for retry, logging, etc.)
let client = AlertmanagerClient::with_client(middleware_client, url);

// Push a single alert
client.push_alert(alert).await?;

// Push multiple alerts in one request
client.push_alerts(vec![alert1, alert2]).await?;
```

### Alert

Represents an alert to send to Alertmanager.

```rust
let alert = Alert::new("AlertName")
    // Labels identify the alert (used for deduplication and routing)
    .with_label("service", "api")
    .with_label("env", "production")
    .with_severity(AlertSeverity::Critical)

    // Annotations provide additional context
    .with_summary("Brief summary")
    .with_description("Detailed description")
    .with_annotation("runbook_url", "https://wiki.example.com/runbook")

    // Optional: link back to alert source
    .with_generator_url("http://prometheus:9090/graph?...")

    // Optional: custom timestamps
    .with_starts_at(start_time)
    .with_ends_at(end_time);
```

### AlertSeverity

Predefined severity levels:

```rust
AlertSeverity::Critical  // "critical"
AlertSeverity::Warning   // "warning"
AlertSeverity::Info      // "info"
```

### Resolving Alerts

To resolve (clear) an alert, send it with `ends_at` set:

```rust
// Option 1: Use resolve() to set ends_at to now
let resolved = Alert::new("HighMemoryUsage")
    .with_label("service", "my-app")
    .resolve();

// Option 2: Set a specific end time
let resolved = Alert::new("HighMemoryUsage")
    .with_label("service", "my-app")
    .with_ends_at(chrono::Utc::now());

client.push_alert(resolved).await?;
```

**Note:** Labels must match the original alert exactly for resolution to work.

## Error Handling

The library provides detailed error types:

```rust
use alert_manager_api::{AlertmanagerError, Result};

match client.push_alert(alert).await {
    Ok(()) => println!("Alert sent"),
    Err(e) => {
        // Check if the error is retryable
        if e.is_retryable() {
            // Network errors, timeouts, 5xx responses
            // Consider implementing retry logic
        }

        match e {
            AlertmanagerError::Api { status, message } => {
                eprintln!("Alertmanager returned {}: {}", status, message);
            }
            AlertmanagerError::Request(e) => {
                eprintln!("HTTP request failed: {}", e);
            }
            AlertmanagerError::Serialize(e) => {
                eprintln!("Failed to serialize alert: {}", e);
            }
            AlertmanagerError::BuildHttpClient(e) => {
                eprintln!("Failed to build HTTP client: {}", e);
            }
        }
    }
}
```

## Alert Deduplication

Alertmanager deduplicates alerts based on their **labels**. Two alerts with identical labels are considered the same alert:

```rust
// These are the SAME alert (identical labels)
Alert::new("HighCPU").with_label("instance", "host1")
Alert::new("HighCPU").with_label("instance", "host1").with_summary("Different text")

// These are DIFFERENT alerts (different labels)
Alert::new("HighCPU").with_label("instance", "host1")
Alert::new("HighCPU").with_label("instance", "host2")
```

## Custom Middleware

For retry logic, logging, or other middleware:

```rust
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

let client = ClientBuilder::new(reqwest::Client::new())
    .with(RetryTransientMiddleware::new_with_policy(retry_policy))
    .build();

let alertmanager = AlertmanagerClient::with_client(
    client,
    Url::parse("http://localhost:9093")?,
);
```

## Examples

See the [examples](./examples) directory for more usage patterns:

- `basic.rs` - Simple alert sending
- `batch_alerts.rs` - Sending multiple alerts
- `resolve_alert.rs` - Resolving alerts
- `custom_middleware.rs` - Using retry middleware

Run examples with:

```bash
cargo run --example basic
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT OR Apache-2.0

---

[GitHub Repository](https://github.com/rlgrpe/alert-manager-api)
