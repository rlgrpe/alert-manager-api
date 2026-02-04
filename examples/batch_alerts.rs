//! Batch example: sending multiple alerts in a single request.
//!
//! Run with: cargo run --example batch_alerts
//!
//! Requires Alertmanager running at http://localhost:9093

use alert_manager_api::{Alert, AlertSeverity, AlertmanagerClient};
use std::time::Duration;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AlertmanagerClient::new(
        Url::parse("http://localhost:9093")?,
        Duration::from_secs(10),
    )?;

    // Create multiple alerts
    let alerts = vec![
        Alert::new("HighCPUUsage")
            .with_severity(AlertSeverity::Warning)
            .with_label("service", "api-gateway")
            .with_label("instance", "prod-1")
            .with_summary("CPU usage above 80%"),
        Alert::new("HighCPUUsage")
            .with_severity(AlertSeverity::Critical)
            .with_label("service", "api-gateway")
            .with_label("instance", "prod-2")
            .with_summary("CPU usage above 95%"),
        Alert::new("DiskSpaceLow")
            .with_severity(AlertSeverity::Warning)
            .with_label("service", "database")
            .with_label("instance", "db-primary")
            .with_summary("Disk space below 20%")
            .with_description("Primary database server is running low on disk space."),
        Alert::new("ServiceDown")
            .with_severity(AlertSeverity::Critical)
            .with_label("service", "payment-processor")
            .with_label("instance", "prod-1")
            .with_summary("Service is not responding")
            .with_annotation("runbook_url", "https://wiki.example.com/runbooks/payment"),
    ];

    // Send all alerts in a single HTTP request
    client.push_alerts(alerts).await?;

    println!("All alerts sent successfully!");
    Ok(())
}
