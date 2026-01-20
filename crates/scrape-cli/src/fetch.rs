//! URL fetching module for CLI.

use std::time::Duration;

/// Configuration for URL fetching.
#[derive(Debug, Clone)]
pub struct FetchConfig {
    /// Request timeout.
    pub timeout: Duration,
    /// User-Agent header.
    pub user_agent: String,
    /// Maximum response size in bytes.
    pub max_size: usize,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: format!("scrape-cli/{}", env!("CARGO_PKG_VERSION")),
            max_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Error type for fetch operations.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    /// HTTP error.
    #[error("HTTP error: {0}")]
    Http(String),
    /// Timeout error.
    #[error("timeout after {0:?}")]
    Timeout(Duration),
    /// Response too large.
    #[error("response too large: {size} bytes (max: {max})")]
    TooLarge {
        /// Actual size.
        size: usize,
        /// Maximum allowed size.
        max: usize,
    },
    /// Invalid URL.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
}

/// Fetches HTML content from a URL.
///
/// # Errors
///
/// Returns `FetchError` if the request fails.
#[cfg(feature = "url")]
pub fn fetch_url(url: &str, config: &FetchConfig) -> Result<String, FetchError> {
    let response = ureq::get(url)
        .set("User-Agent", &config.user_agent)
        .timeout(config.timeout)
        .call()
        .map_err(|e| FetchError::Http(e.to_string()))?;

    // Check content length
    if let Some(len) = response.header("content-length") {
        if let Ok(size) = len.parse::<usize>() {
            if size > config.max_size {
                return Err(FetchError::TooLarge { size, max: config.max_size });
            }
        }
    }

    let mut reader = response.into_reader();
    let mut body = String::new();

    // Read with size limit
    use std::io::Read;
    reader
        .take(config.max_size as u64)
        .read_to_string(&mut body)
        .map_err(|e| FetchError::Http(e.to_string()))?;

    Ok(body)
}

#[cfg(not(feature = "url"))]
pub fn fetch_url(_url: &str, _config: &FetchConfig) -> Result<String, FetchError> {
    Err(FetchError::Http("URL support not compiled (use --features url)".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_config_default() {
        let config = FetchConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_size, 10 * 1024 * 1024);
        assert!(config.user_agent.starts_with("scrape-cli/"));
    }

    #[test]
    fn test_fetch_error_display() {
        let err = FetchError::Http("connection refused".into());
        assert_eq!(err.to_string(), "HTTP error: connection refused");

        let err = FetchError::Timeout(Duration::from_secs(30));
        assert_eq!(err.to_string(), "timeout after 30s");

        let err = FetchError::TooLarge { size: 20_000_000, max: 10_000_000 };
        assert_eq!(err.to_string(), "response too large: 20000000 bytes (max: 10000000)");
    }

    #[cfg(not(feature = "url"))]
    #[test]
    fn test_fetch_url_not_available() {
        let config = FetchConfig::default();
        let result = fetch_url("http://example.com", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_error_variants() {
        let err1 = FetchError::Http("test".into());
        assert!(matches!(err1, FetchError::Http(_)));

        let err2 = FetchError::Timeout(Duration::from_secs(5));
        assert!(matches!(err2, FetchError::Timeout(_)));

        let err3 = FetchError::TooLarge { size: 1000, max: 500 };
        assert!(matches!(err3, FetchError::TooLarge { .. }));

        let err4 = FetchError::InvalidUrl("bad url".into());
        assert!(matches!(err4, FetchError::InvalidUrl(_)));
    }

    #[test]
    fn test_fetch_config_customization() {
        let config = FetchConfig {
            timeout: Duration::from_secs(10),
            user_agent: "custom-agent/1.0".to_string(),
            max_size: 5 * 1024 * 1024,
        };
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.user_agent, "custom-agent/1.0");
        assert_eq!(config.max_size, 5 * 1024 * 1024);
    }

    #[test]
    fn test_fetch_config_zero_timeout() {
        let config = FetchConfig { timeout: Duration::from_secs(0), ..Default::default() };
        assert_eq!(config.timeout, Duration::from_secs(0));
    }

    #[test]
    fn test_fetch_error_too_large_boundary() {
        let err = FetchError::TooLarge { size: 10_485_761, max: 10_485_760 };
        let msg = err.to_string();
        assert!(msg.contains("10485761"));
        assert!(msg.contains("10485760"));
    }

    #[test]
    fn test_fetch_config_user_agent_contains_version() {
        let config = FetchConfig::default();
        assert!(config.user_agent.contains("scrape-cli/"));
    }
}
