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

    /// Controls how aggressively secondary content is filtered.
    /// A value of `0.1` keeps everything within 10x of the best scoring branch.
    /// A value of `0.5` keeps only branches close to the best.
    /// Defaults to `0.1`.
    pub sensitivity: f32,

    /// If `true`, the pipeline will fetch and respect `robots.txt` before
    /// downloading the target page.
    /// Defaults to `true`.
    pub robots_check: bool,
}

impl Web2llmConfig {
    pub fn new(
        user_agent: String,
        timeout: Duration,
        block_private_hosts: bool,
        sensitivity: f32,
    ) -> Self {
        Self {
            user_agent,
            timeout,
            block_private_hosts,
            sensitivity,
            robots_check: true,
        }
    }

    /// Builder-style method to set whether to check `robots.txt`.
    pub fn with_robots_check(mut self, check: bool) -> Self {
        self.robots_check = check;
        self
    }
}

impl Default for Web2llmConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("web2llm/{}", env!("CARGO_PKG_VERSION")),
            timeout: Duration::from_secs(30),
            block_private_hosts: true,
            sensitivity: 0.1,
            robots_check: true,
        }
    }
}
