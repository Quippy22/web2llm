use crate::fetch::FetchMode;
use std::time::Duration;

/// User-facing configuration for the `web2llm` pipeline.
/// Controls fetch behavior and request identity.
/// Use `Web2llmConfig::default()` for sensible defaults.
///
/// # Examples
///
/// ```
/// use web2llm::{Web2llmConfig, FetchMode};
/// use std::time::Duration;
///
/// let config = Web2llmConfig {
///     timeout: Duration::from_secs(10),
///     fetch_mode: FetchMode::Auto,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Web2llmConfig {
    /// The user-agent string sent with every HTTP request.
    /// Also used for `robots.txt` compliance checks.
    pub user_agent: String,

    /// Maximum time to wait for a response before giving up.
    pub timeout: Duration,

    /// If `true`, requests to private, loopback, and link-local addresses are
    /// rejected during pre-flight validation.
    ///
    /// Set to `false` if you need to fetch from `localhost` or internal hosts
    /// in a trusted environment, such as local development or testing.
    ///
    /// Defaults to `true`.
    pub block_private_hosts: bool,

    /// Controls how aggressively secondary content is filtered.
    /// A value of `0.1` keeps everything within 10x of the best scoring branch.
    /// A value of `0.5` keeps only branches close to the best.
    /// Defaults to `0.1`.
    pub sensitivity: f32,

    /// If `true`, the pipeline will fetch and respect `robots.txt` before
    /// downloading the target page.
    /// Defaults to `true`.
    pub robots_check: bool,

    /// The maximum number of requests allowed per second.
    /// Defaults to `5`.
    pub rate_limit: u32,

    /// The maximum number of concurrent requests allowed across the whole pipeline.
    /// Defaults to `10`.
    pub max_concurrency: usize,

    /// The fetching strategy to use.
    /// Defaults to `FetchMode::Auto`.
    pub fetch_mode: FetchMode,

    /// If `true`, `batch_fetch` will return results in the same order as the
    /// input URLs. If `false`, results are returned as they finish (faster).
    /// Defaults to `false`.
    pub ordered: bool,
}

impl Default for Web2llmConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("web2llm/{}", env!("CARGO_PKG_VERSION")),
            timeout: Duration::from_secs(30),
            block_private_hosts: true,
            sensitivity: 0.1,
            robots_check: true,
            rate_limit: 5,
            max_concurrency: 10,
            fetch_mode: FetchMode::Auto,
            ordered: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Web2llmConfig::default();
        assert!(config.user_agent.contains(env!("CARGO_PKG_VERSION")));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.block_private_hosts);
        assert_eq!(config.sensitivity, 0.1);
        assert!(config.robots_check);
    }
}
