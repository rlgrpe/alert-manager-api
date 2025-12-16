use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

impl Display for AlertSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Critical => write!(f, "critical"),
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Info => write!(f, "info"),
        }
    }
}

/// Alertmanager alert payload
///
/// Alerts are identified by their labels. Two alerts with identical labels
/// are considered the same alert by Alertmanager and will be deduplicated.
///
/// See: <https://prometheus.io/docs/alerting/latest/clients/>
///
/// # Example
///
/// ```rust
/// use alert_manager_api::{Alert, AlertSeverity};
///
/// let alert = Alert::new("HighCPUUsage")
///     .with_severity(AlertSeverity::Warning)
///     .with_label("service", "api-server")
///     .with_label("instance", "prod-1")
///     .with_summary("CPU usage above 80%")
///     .with_description("The API server CPU usage has exceeded the warning threshold");
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// Labels identify the alert (used for deduplication and routing)
    pub labels: HashMap<String, String>,

    /// Annotations contain additional information (not used for dedup)
    pub annotations: HashMap<String, String>,

    /// Start time of the alert
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<DateTime<Utc>>,

    /// End time (if resolved)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<DateTime<Utc>>,

    /// Generator URL (link back to source)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator_url: Option<String>,
}

impl Alert {
    /// Create a new alert with the given name
    ///
    /// The `alertname` label is automatically set.
    pub fn new(alertname: &str) -> Self {
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), alertname.to_string());

        Self {
            labels,
            annotations: HashMap::new(),
            starts_at: Some(Utc::now()),
            ends_at: None,
            generator_url: None,
        }
    }

    /// Add a label to the alert
    ///
    /// Labels are used for routing and deduplication.
    /// Alerts with identical labels are considered the same alert.
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }

    /// Add severity label
    ///
    /// This is a convenience method that adds a "severity" label.
    pub fn with_severity(self, severity: AlertSeverity) -> Self {
        self.with_label("severity", &severity.to_string())
    }

    /// Add an annotation
    ///
    /// Annotations provide additional context but are not used for deduplication.
    pub fn with_annotation(mut self, key: &str, value: &str) -> Self {
        self.annotations.insert(key.to_string(), value.to_string());
        self
    }

    /// Add summary annotation
    ///
    /// The summary should be a short description of the alert.
    pub fn with_summary(self, summary: &str) -> Self {
        self.with_annotation("summary", summary)
    }

    /// Add description annotation
    ///
    /// The description can contain more detailed information about the alert.
    pub fn with_description(self, description: &str) -> Self {
        self.with_annotation("description", description)
    }

    /// Set generator URL
    ///
    /// This URL can link back to the source that generated the alert.
    pub fn with_generator_url(mut self, url: &str) -> Self {
        self.generator_url = Some(url.to_string());
        self
    }

    /// Set custom start time
    ///
    /// By default, the start time is set to the current time when the alert is created.
    pub fn with_starts_at(mut self, time: DateTime<Utc>) -> Self {
        self.starts_at = Some(time);
        self
    }

    /// Set end time to resolve the alert
    ///
    /// Setting an end time marks the alert as resolved.
    pub fn with_ends_at(mut self, time: DateTime<Utc>) -> Self {
        self.ends_at = Some(time);
        self
    }

    /// Mark the alert as resolved (sets ends_at to now)
    pub fn resolve(mut self) -> Self {
        self.ends_at = Some(Utc::now());
        self
    }

    /// Get the alertname label
    pub fn alertname(&self) -> Option<&str> {
        self.labels.get("alertname").map(|s| s.as_str())
    }
}

impl Default for Alert {
    fn default() -> Self {
        Self {
            labels: HashMap::new(),
            annotations: HashMap::new(),
            starts_at: Some(Utc::now()),
            ends_at: None,
            generator_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new("TestAlert")
            .with_severity(AlertSeverity::Warning)
            .with_label("service", "service")
            .with_annotation("description", "Test description");

        assert_eq!(
            alert.labels.get("alertname"),
            Some(&"TestAlert".to_string())
        );
        assert_eq!(alert.labels.get("severity"), Some(&"warning".to_string()));
        assert_eq!(alert.labels.get("service"), Some(&"service".to_string()));
        assert_eq!(
            alert.annotations.get("description"),
            Some(&"Test description".to_string())
        );
        assert!(alert.starts_at.is_some());
        assert!(alert.ends_at.is_none());
    }

    #[test]
    fn test_alert_serialization() {
        let alert = Alert::new("TestAlert").with_severity(AlertSeverity::Info);

        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("\"alertname\":\"TestAlert\""));
        assert!(json.contains("\"severity\":\"info\""));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(AlertSeverity::Critical.to_string(), "critical");
        assert_eq!(AlertSeverity::Warning.to_string(), "warning");
        assert_eq!(AlertSeverity::Info.to_string(), "info");
    }

    #[test]
    fn test_alert_resolve() {
        let alert = Alert::new("TestAlert").resolve();
        assert!(alert.ends_at.is_some());
    }

    #[test]
    fn test_alertname_getter() {
        let alert = Alert::new("MyAlert");
        assert_eq!(alert.alertname(), Some("MyAlert"));
    }

    #[test]
    fn test_alert_with_all_fields() {
        let now = Utc::now();
        let alert = Alert::new("FullAlert")
            .with_severity(AlertSeverity::Critical)
            .with_label("env", "production")
            .with_label("team", "backend")
            .with_summary("Critical issue")
            .with_description("Detailed description")
            .with_generator_url("http://example.com/alerts/1")
            .with_starts_at(now);

        assert_eq!(alert.labels.len(), 4); // alertname, severity, env, team
        assert_eq!(alert.annotations.len(), 2); // summary, description
        assert_eq!(
            alert.generator_url,
            Some("http://example.com/alerts/1".to_string())
        );
    }
}
