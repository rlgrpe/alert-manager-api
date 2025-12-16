use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::time::Duration;
use tracing::{debug, instrument};
use url::Url;

use crate::errors::{AlertmanagerError, Result};
use crate::types::Alert;

/// Client for pushing alerts to Alertmanager
///
/// # Example
///
/// ```rust,no_run
/// use alert_manager_api::{AlertmanagerClient, Alert, AlertSeverity};
/// use url::Url;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = AlertmanagerClient::new(
///         Url::parse("http://localhost:9093")?,
///         Duration::from_secs(10),
///     )?;
///
///     let alert = Alert::new("TestAlert")
///         .with_severity(AlertSeverity::Info)
///         .with_label("service", "my-service");
///
///     client.push_alert(alert).await?;
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct AlertmanagerClient {
    client: ClientWithMiddleware,
    api_url: Url,
}

impl AlertmanagerClient {
    /// Create a new Alertmanager client
    ///
    /// # Arguments
    ///
    /// * `api_url` - Base URL of the Alertmanager instance (e.g., `http://localhost:9093`)
    /// * `timeout` - Request timeout duration
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(api_url: Url, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(AlertmanagerError::BuildHttpClient)?;

        let client = ClientBuilder::new(client).build();

        Ok(Self { client, api_url })
    }

    /// Create a new client with a custom reqwest middleware client
    ///
    /// This allows you to add custom middleware (retry, logging, etc.)
    pub fn with_client(client: ClientWithMiddleware, api_url: Url) -> Self {
        Self { client, api_url }
    }

    /// Push one or more alerts to Alertmanager
    ///
    /// Alertmanager deduplicates alerts by their labels.
    /// Alerts with identical labels are considered the same alert.
    ///
    /// # Arguments
    ///
    /// * `alerts` - Vector of alerts to push
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - Alertmanager returns a non-success status code
    #[instrument(
        name = "AlertmanagerClient::push_alerts",
        skip_all,
        fields(alert_count = alerts.len())
    )]
    pub async fn push_alerts(&self, alerts: Vec<Alert>) -> Result<()> {
        if alerts.is_empty() {
            debug!("No alerts to push");
            return Ok(());
        }

        let url = self.api_url.join("/api/v2/alerts").expect("Valid URL path");

        debug!(url = %url, "Pushing alerts to Alertmanager");

        let response = self
            .client
            .post(url)
            .json(&alerts)
            .send()
            .await
            .map_err(AlertmanagerError::Request)?;

        let status = response.status();

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(AlertmanagerError::Api {
                status: status.as_u16(),
                message,
            });
        }

        debug!("Alerts pushed successfully");
        Ok(())
    }

    /// Push a single alert
    ///
    /// Convenience method that wraps `push_alerts` for a single alert.
    pub async fn push_alert(&self, alert: Alert) -> Result<()> {
        self.push_alerts(vec![alert]).await
    }

    /// Get the base API URL
    pub fn api_url(&self) -> &Url {
        &self.api_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AlertSeverity;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_push_alert_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v2/alerts"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = AlertmanagerClient::new(
            Url::parse(&mock_server.uri()).unwrap(),
            Duration::from_secs(10),
        )
        .unwrap();

        let alert = Alert::new("TestAlert")
            .with_severity(AlertSeverity::Info)
            .with_label("service", "test");

        let result = client.push_alert(alert).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_alert_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v2/alerts"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad request"))
            .mount(&mock_server)
            .await;

        let client = AlertmanagerClient::new(
            Url::parse(&mock_server.uri()).unwrap(),
            Duration::from_secs(10),
        )
        .unwrap();

        let alert = Alert::new("TestAlert");

        let result = client.push_alert(alert).await;
        assert!(result.is_err());

        if let Err(AlertmanagerError::Api { status, message }) = result {
            assert_eq!(status, 400);
            assert_eq!(message, "Bad request");
        } else {
            panic!("Expected Api error");
        }
    }

    #[tokio::test]
    async fn test_push_empty_alerts() {
        let mock_server = MockServer::start().await;

        // No mock needed - should not make request
        let client = AlertmanagerClient::new(
            Url::parse(&mock_server.uri()).unwrap(),
            Duration::from_secs(10),
        )
        .unwrap();

        let result = client.push_alerts(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_multiple_alerts() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v2/alerts"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = AlertmanagerClient::new(
            Url::parse(&mock_server.uri()).unwrap(),
            Duration::from_secs(10),
        )
        .unwrap();

        let alerts = vec![
            Alert::new("Alert1").with_severity(AlertSeverity::Info),
            Alert::new("Alert2").with_severity(AlertSeverity::Warning),
        ];

        let result = client.push_alerts(alerts).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_alert_server_error_is_retryable() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v2/alerts"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service unavailable"))
            .mount(&mock_server)
            .await;

        let client = AlertmanagerClient::new(
            Url::parse(&mock_server.uri()).unwrap(),
            Duration::from_secs(10),
        )
        .unwrap();

        let alert = Alert::new("TestAlert");

        let result = client.push_alert(alert).await;
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.is_retryable());
        }
    }

    #[test]
    fn test_api_url_getter() {
        let url = Url::parse("http://localhost:9093").unwrap();
        let client = AlertmanagerClient::new(url.clone(), Duration::from_secs(10)).unwrap();
        assert_eq!(client.api_url(), &url);
    }
}
