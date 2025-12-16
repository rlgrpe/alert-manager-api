//! # Alert Manager API
//!
//! A Rust client library for pushing alerts to [Prometheus Alertmanager](https://prometheus.io/docs/alerting/latest/alertmanager/).
//!
//! ## Features
//!
//! - Push alerts to Alertmanager via HTTP API
//! - Builder pattern for constructing alerts
//! - Automatic deduplication by Alertmanager based on labels
//! - Support for alert annotations and labels
//!
//! ## Example
//!
//! ```rust,no_run
//! use alert_manager_api::{AlertmanagerClient, Alert, AlertSeverity};
//! use url::Url;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AlertmanagerClient::new(
//!         Url::parse("http://localhost:9093")?,
//!         Duration::from_secs(10),
//!     )?;
//!
//!     let alert = Alert::new("HighMemoryUsage")
//!         .with_severity(AlertSeverity::Warning)
//!         .with_label("service", "my-app")
//!         .with_label("instance", "localhost:8080")
//!         .with_summary("Memory usage is above 90%")
//!         .with_description("The service is using more than 90% of available memory");
//!
//!     client.push_alert(alert).await?;
//!     Ok(())
//! }
//! ```

mod client;
mod errors;
mod types;

pub use client::AlertmanagerClient;
pub use errors::{AlertmanagerError, Result};
pub use types::{Alert, AlertSeverity};
