//! Basic example: sending a single alert to Alertmanager.
//!
//! Run with: cargo run --example basic
//!
//! Requires Alertmanager running at http://localhost:9093

use alert_manager_api::{Alert, AlertSeverity, AlertmanagerClient};
use std::time::Duration;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client pointing to your Alertmanager instance
    let client = AlertmanagerClient::new(
        Url::parse("http://localhost:9093")?,
        Duration::from_secs(10),
    )?;

    // Build an alert using the builder pattern
    let alert = Alert::new("HighMemoryUsage")
        .with_severity(AlertSeverity::Warning)
        .with_label("service", "my-application")
        .with_label("instance", "localhost:8080")
        .with_label("env", "production")
        .with_summary("Memory usage is above 90%")
        .with_description(
            "The service 'my-application' on instance 'localhost:8080' \
             is using more than 90% of available memory. Consider scaling \
             or investigating memory leaks.",
        )
        .with_generator_url("http://prometheus:9090/graph?g0.expr=process_resident_memory_bytes");

    // Send the alert
    client.push_alert(alert).await?;

    println!("Alert sent successfully!");
    Ok(())
}
