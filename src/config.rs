use std::time::Duration;

/// User-facing configuration for the `web2llm` pipeline.
/// Controls fetch behavior and request identity.
/// Use `Web2llmConfig::default()` for sensible defaults.
pub struct Web2llmConfig {
    /// The user-agent string sent with every HTTP request.
    /// Also used for `robots.txt` compliance checks.
    pub user_agent: String,

    /// Maximum time to wait for a response before giving up.
    pub timeout: Duration,
}

impl Web2llmConfig {
    pub fn new(user_agent: String, timeout: Duration) -> Self {
        Self {
            user_agent,
            timeout,
        }
    }
}

impl Default for Web2llmConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("web2llm/{}", env!("CARGO_PKG_VERSION")),
            timeout: Duration::from_secs(30),
        }
    }
}
