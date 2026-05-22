//! Custom middleware example: using reqwest-middleware for retry logic.
//!
//! Run with: cargo run --example custom_middleware
//!
//! Requires Alertmanager running at http://localhost:9093
//!
//! This example requires additional dependencies:
//! ```toml
//! [dev-dependencies]
//! reqwest-retry = "0.9.1"
//! ```

use alert_manager_api::{Alert, AlertSeverity, AlertmanagerClient, AlertmanagerError};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::time::Duration;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure retry policy
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_millis(100), Duration::from_secs(5))
        .build_with_max_retries(3);

    // Build the base HTTP client
    let base_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // Wrap with retry middleware
    let middleware_client = ClientBuilder::new(base_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    // Create Alertmanager client with our custom middleware
    let client =
        AlertmanagerClient::with_client(middleware_client, Url::parse("http://localhost:9093")?);

    let alert = Alert::new("NetworkLatencyHigh")
        .with_severity(AlertSeverity::Warning)
        .with_label("service", "api-gateway")
        .with_label("region", "us-east-1")
        .with_summary("Network latency above threshold")
        .with_description("P99 latency is above 500ms for the past 5 minutes.");

    // The middleware will automatically retry on transient failures
    match client.push_alert(alert).await {
        Ok(()) => println!("Alert sent successfully!"),
        Err(e) => {
            // At this point, retries have been exhausted
            eprintln!("Failed to send alert after retries: {}", e);

            // You can still check if it was retryable for logging purposes
            if e.is_retryable() {
                eprintln!("This was a transient error (retries exhausted)");
            }

            match e {
                AlertmanagerError::Api { status, message } => {
                    eprintln!("Alertmanager returned HTTP {}: {}", status, message);
                }
                AlertmanagerError::Request(req_err) => {
                    eprintln!("Request error: {}", req_err);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
