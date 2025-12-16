use std::error::Error as StdError;
use thiserror::Error;

/// Result type alias for Alertmanager operations
pub type Result<T> = std::result::Result<T, AlertmanagerError>;

/// Errors that can occur when interacting with Alertmanager
#[derive(Debug, Error)]
pub enum AlertmanagerError {
    /// Failed to build HTTP client
    #[error("Failed to build HTTP client: {0}")]
    BuildHttpClient(#[source] reqwest::Error),

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Request(#[source] reqwest_middleware::Error),

    /// Failed to serialize alerts
    #[error("Failed to serialize alerts: {0}")]
    Serialize(#[source] serde_json::Error),

    /// Alertmanager API returned an error response
    #[error("Alertmanager API error: HTTP {status} - {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message from Alertmanager
        message: String,
    },
}

impl AlertmanagerError {
    /// Check if the error is retryable
    ///
    /// Returns `true` for:
    /// - Network/connection errors
    /// - Timeout errors
    /// - Server errors (5xx status codes)
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Request(source) => {
                if let Some(reqwest_err) = StdError::source(source) {
                    if let Some(err) = reqwest_err.downcast_ref::<reqwest::Error>() {
                        return err.is_connect() || err.is_timeout();
                    }
                }
                false
            }
            Self::Api { status, .. } => *status >= 500,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_retryable_5xx() {
        let error = AlertmanagerError::Api {
            status: 500,
            message: "Internal server error".to_string(),
        };
        assert!(error.is_retryable());

        let error = AlertmanagerError::Api {
            status: 502,
            message: "Bad gateway".to_string(),
        };
        assert!(error.is_retryable());

        let error = AlertmanagerError::Api {
            status: 503,
            message: "Service unavailable".to_string(),
        };
        assert!(error.is_retryable());
    }

    #[test]
    fn test_api_error_not_retryable_4xx() {
        let error = AlertmanagerError::Api {
            status: 400,
            message: "Bad request".to_string(),
        };
        assert!(!error.is_retryable());

        let error = AlertmanagerError::Api {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert!(!error.is_retryable());

        let error = AlertmanagerError::Api {
            status: 404,
            message: "Not found".to_string(),
        };
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_display() {
        let error = AlertmanagerError::Api {
            status: 500,
            message: "Internal server error".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Alertmanager API error: HTTP 500 - Internal server error"
        );
    }

    #[test]
    fn test_serialize_error_not_retryable() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error = AlertmanagerError::Serialize(json_err);
        assert!(!error.is_retryable());
    }
}
