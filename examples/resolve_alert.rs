//! Resolve example: sending and then resolving an alert.
//!
//! Run with: cargo run --example resolve_alert
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

    // First, send an alert
    let alert = Alert::new("DatabaseConnectionPoolExhausted")
        .with_severity(AlertSeverity::Critical)
        .with_label("service", "user-service")
        .with_label("instance", "prod-1")
        .with_label("database", "postgres-primary")
        .with_summary("Connection pool exhausted")
        .with_description("All database connections are in use. New requests will fail.");

    println!("Sending alert...");
    client.push_alert(alert).await?;
    println!("Alert sent!");

    // Simulate the issue being resolved
    println!("\nWaiting 5 seconds to simulate issue resolution...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Resolve the alert by sending it with ends_at set
    // IMPORTANT: Labels must match exactly for the resolution to work
    let resolved_alert = Alert::new("DatabaseConnectionPoolExhausted")
        .with_severity(AlertSeverity::Critical)
        .with_label("service", "user-service")
        .with_label("instance", "prod-1")
        .with_label("database", "postgres-primary")
        .with_summary("Connection pool exhausted")
        .resolve(); // This sets ends_at to the current time

    println!("Sending resolution...");
    client.push_alert(resolved_alert).await?;
    println!("Alert resolved!");

    Ok(())
}
