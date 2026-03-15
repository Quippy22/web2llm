pub mod config;
pub mod error;
pub(crate) mod extract;
pub(crate) mod fetch;
pub mod output;
pub(crate) mod preflight;

pub use config::Web2llmConfig;
pub use error::Web2llmError;
pub use output::PageResult;

use std::num::NonZeroU32;
use std::sync::Arc;

use crate::error::Result;
use crate::extract::PageElements;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use tokio::sync::Semaphore;

/// The main entry point for the `web2llm` pipeline.
///
/// Holds configuration and exposes a `fetch` method that runs a URL
/// through the full pipeline — fetching, extracting, scoring, and
/// converting to clean Markdown optimized for LLM ingestion.
///
/// # Examples
///
/// ```no_run
/// use web2llm::Web2llm;
/// use web2llm::Web2llmConfig;
///
/// #[tokio::main]
/// async fn main() {
///     let config = Web2llmConfig::default();
///     let client = Web2llm::new(config).unwrap();
///     let result = client.fetch("https://example.com").await.unwrap();
///     println!("{}", result.markdown);
/// }
/// ```
pub struct Web2llm {
    /// The configuration for this instance.
    config: Web2llmConfig,
    /// Shared HTTP client used for all requests.
    client: reqwest::Client,
    /// Rate limiter used to throttle requests across all threads.
    limiter: Arc<DefaultDirectRateLimiter>,
    /// Semaphore used to limit the number of concurrent requests.
    semaphore: Arc<Semaphore>,
}

impl Web2llm {
    /// Creates a new `Web2llm` instance with the given configuration.
    ///
    /// # Errors
    /// Returns [`Web2llmError::Config`] if the configuration is invalid (e.g., zero rate limit).
    pub fn new(config: Web2llmConfig) -> Result<Self> {
        Self::validate_config(&config)?;

        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let limiter = Arc::new(RateLimiter::direct(Quota::per_second(
            NonZeroU32::new(config.rate_limit).unwrap(),
        )));
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));

        Ok(Self {
            config,
            client,
            limiter,
            semaphore,
        })
    }

    /// Internal validation layer to ensure configuration is valid before initialization.
    fn validate_config(config: &Web2llmConfig) -> Result<()> {
        if config.rate_limit == 0 {
            return Err(Web2llmError::Config(
                "rate_limit must be greater than zero".to_string(),
            ));
        }

        if config.max_concurrency == 0 {
            return Err(Web2llmError::Config(
                "max_concurrency must be greater than zero".to_string(),
            ));
        }

        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(Web2llmError::Config(
                "sensitivity must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Fetches the page at `url` and runs it through the full pipeline.
    ///
    /// Uses the user-agent and timeout from this instance's [`Web2llmConfig`].
    ///
    /// # Errors
    /// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
    /// Returns [`Web2llmError::EmptyContent`] if no scoreable content is found.
    pub async fn fetch(&self, url: &str) -> Result<PageResult> {
        let url = preflight::run(
            url,
            &self.config.user_agent,
            self.config.block_private_hosts,
            self.config.robots_check,
            &self.client,
        )
        .await?;
        let elements = PageElements::parse(url, &self.client).await?;
        elements.into_result(self.config.sensitivity)
    }
}

/// Convenience function — fetches `url` using [`Web2llmConfig::default`].
///
/// Equivalent to `Web2llm::new(Web2llmConfig::default()).unwrap().fetch(url).await`.
///
/// # Errors
/// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
/// Returns [`Web2llmError::EmptyContent`] if no scoreable content is found.
pub async fn fetch(url: &str) -> Result<PageResult> {
    Web2llm::new(Web2llmConfig::default())?.fetch(url).await
}
