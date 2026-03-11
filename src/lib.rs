pub mod config;
pub mod error;
pub(crate) mod extract;
pub(crate) mod fetch;
pub mod output;
pub(crate) mod preflight;

pub use config::Web2llmConfig;
pub use error::Web2llmError;
pub use output::PageResult;

use crate::error::Result;
use crate::extract::PageElements;

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
///
/// #[tokio::main]
/// async fn main() {
///     let result = web2llm::fetch("https://example.com").await.unwrap();
///     println!("{}", result.markdown);
/// }
/// ```
pub struct Web2llm {
    config: Web2llmConfig,
}

impl Web2llm {
    /// Creates a new `Web2llm` instance with the given configuration.
    ///
    /// Use [`Web2llmConfig::default`] for sensible defaults, or
    /// [`Web2llmConfig::new`] to supply your own user-agent and timeout.
    pub fn new(config: Web2llmConfig) -> Self {
        Self { config }
    }

    /// Fetches the page at `url` and runs it through the full pipeline.
    ///
    /// Uses the user-agent and timeout from this instance's [`Web2llmConfig`].
    ///
    /// # Errors
    /// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
    /// Returns [`Web2llmError::EmptyContent`] if no scoreable content is found.
    pub async fn fetch(&self, url: &str) -> Result<PageResult> {
        let elements =
            PageElements::parse(url, self.config.timeout, &self.config.user_agent).await?;
        elements.into_result()
    }
}

/// Convenience function — fetches `url` using [`Web2llmConfig::default`].
///
/// Equivalent to `Web2llm::new(Web2llmConfig::default()).fetch(url).await`.
///
/// # Errors
/// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
/// Returns [`Web2llmError::EmptyContent`] if no scoreable content is found.
pub async fn fetch(url: &str) -> Result<PageResult> {
    Web2llm::new(Web2llmConfig::default()).fetch(url).await
}
