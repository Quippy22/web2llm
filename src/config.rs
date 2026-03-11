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

    /// If `true`, requests to private, loopback, and link-local addresses are
    /// rejected during pre-flight validation. This prevents SSRF attacks when
    /// `web2llm` is used in a service that accepts user-supplied URLs.
    ///
    /// Set to `false` if you need to fetch from `localhost` or internal hosts
    /// in a trusted environment, such as local development or testing.
    ///
    /// Defaults to `true`.
    pub block_private_hosts: bool,
}

impl Web2llmConfig {
    pub fn new(user_agent: String, timeout: Duration, block_private_hosts: bool) -> Self {
        Self {
            user_agent,
            timeout,
            block_private_hosts,
        }
    }
}

impl Default for Web2llmConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("web2llm/{}", env!("CARGO_PKG_VERSION")),
            timeout: Duration::from_secs(30),
            block_private_hosts: true,
        }
    }
}
