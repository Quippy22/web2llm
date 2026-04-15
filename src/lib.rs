//! # web2llm
//!
//! `web2llm` is a high-performance Rust crate designed to fetch web pages and convert their core content
//! into clean, token-efficient Markdown. It's optimized for feeding data into Large Language Models (LLMs)
//! and RAG pipelines.
//!
//! ## Key Features
//! - **High Performance**: Zero-copy tree traversal, LTO, and efficient scoring.
//! - **Clean Output**: Strips navigation, headers, footers, and non-essential attributes.
//! - **Shared Browser**: Single persistent headless Chromium instance for dynamic pages (requires `rendered` feature).
//! - **Adaptive Fetch**: Automatically detects SPAs and uses a browser fallback for full rendering.
//! - **SSRF Protection**: Validates URLs and blocks private host access by default.
//! - **Robots.txt Compliance**: Optionally respects robots.txt rules.
//! - **Rate Limiting**: Built-in support for throttling and concurrency control.
//! - **Recursive Crawling**: Discovers in-content links breadth-first and fetches them in one batch.
//!
//! ## Quick Start
//!
//! The easiest way to get started is using the convenience `fetch` function:
//!
//! ```no_run
//! use web2llm::fetch;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Fetch a page with default configuration
//!     match fetch("https://example.com".to_string()).await {
//!         Ok(result) => {
//!             println!("Title: {}", result.title);
//!             println!("Markdown content:\n{}", result.markdown());
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```
//!
//! For more control, use the [`Web2llm`] struct with a custom [`Web2llmConfig`].

pub mod config;
mod crawl;
pub mod error;
pub(crate) mod extract;
pub(crate) mod fetch;
pub mod output;
pub(crate) mod preflight;
pub(crate) mod tokens;

pub use config::Web2llmConfig;
pub use crawl::CrawlConfig;
pub use error::Web2llmError;
pub use fetch::FetchMode;
pub use output::PageResult;

use std::collections::{HashSet, VecDeque};
use std::num::NonZeroU32;
use std::sync::Arc;

use crate::error::Result;
use crate::extract::PageElements;
use futures::stream::StreamExt;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
#[cfg(feature = "rendered")]
use tokio::sync::OnceCell;
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
/// use web2llm::{Web2llm, Web2llmConfig, FetchMode};
///
/// #[tokio::main]
/// async fn main() {
///     let config = Web2llmConfig::default();
///     let client = Web2llm::new(config).unwrap();
///     let result = client.fetch("https://example.com").await.unwrap();
///     println!("{}", result.markdown());
/// }
/// ```
#[derive(Clone)]
pub struct Web2llm {
    /// The configuration for this instance.
    config: Web2llmConfig,
    /// Shared HTTP client used for all requests.
    client: reqwest::Client,
    /// Rate limiter used to throttle requests across all threads.
    limiter: Arc<DefaultDirectRateLimiter>,
    /// Semaphore used to limit the number of concurrent requests.
    semaphore: Arc<Semaphore>,
    /// Lazily-initialized headless browser for dynamic fetching.
    #[cfg(feature = "rendered")]
    browser: Arc<OnceCell<chromiumoxide::Browser>>,
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
        #[cfg(feature = "rendered")]
        let browser = Arc::new(OnceCell::new());

        Ok(Self {
            config,
            client,
            limiter,
            semaphore,
            #[cfg(feature = "rendered")]
            browser,
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

    /// Fetches the page and returns every single absolute URL found in the document.
    /// This is a "raw" extraction that includes navigation and footer links.
    pub async fn get_urls(&self, url: &str) -> Result<Vec<String>> {
        // 1. FASTEST Path: Synchronous validation
        let url = preflight::run_sync(url, self.config.block_private_hosts)?;

        // 2. Robots check (if enabled)
        if self.config.robots_check {
            preflight::robots::check_single(&url, &self.config.user_agent, &self.client).await?;
        }

        // 3. Execution
        #[cfg(feature = "rendered")]
        let elements = PageElements::parse(
            url.clone(),
            &self.client,
            self.config.fetch_mode,
            &self.browser,
        )
        .await?;

        #[cfg(not(feature = "rendered"))]
        let elements =
            PageElements::parse(url.clone(), &self.client, self.config.fetch_mode).await?;

        Ok(elements.get_urls())
    }

    /// Fetches the page at `url` and runs it through the full pipeline.
    ///
    /// Respects the instance's [`Web2llmConfig::rate_limit`] and [`Web2llmConfig::max_concurrency`].
    ///
    /// # Errors
    /// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
    /// Returns [`Web2llmError::EmptyContent`] if no scoreable content is found.
    #[inline(always)]
    pub async fn fetch(&self, url: &str) -> Result<PageResult> {
        // 1. FASTEST Path: Synchronous validation
        let url = preflight::run_sync(url, self.config.block_private_hosts)?;

        // 2. Robots check (if enabled)
        if self.config.robots_check {
            preflight::robots::check_single(&url, &self.config.user_agent, &self.client).await?;
        }

        // 3. Execution
        #[cfg(feature = "rendered")]
        let elements =
            PageElements::parse(url, &self.client, self.config.fetch_mode, &self.browser).await?;

        #[cfg(not(feature = "rendered"))]
        let elements = PageElements::parse(url, &self.client, self.config.fetch_mode).await?;

        elements.into_result(&self.config)
    }

    /// Fetches multiple URLs concurrently, respecting rate limits and concurrency.
    ///
    /// Returns a vector of tuples containing the original URL and the [`Result<PageResult>`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use web2llm::{Web2llm, Web2llmConfig, FetchMode};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let config = Web2llmConfig {
    ///     fetch_mode: FetchMode::Auto,
    ///     max_concurrency: 20,
    ///     ..Default::default()
    /// };
    /// let client = Web2llm::new(config).unwrap();
    /// let urls = vec!["https://example.com".to_string(), "https://google.com".to_string()];
    /// let results = client.batch_fetch(urls).await;
    /// # }
    /// ```
    pub async fn batch_fetch(&self, urls: Vec<String>) -> Vec<(String, Result<PageResult>)> {
        // 1. Stage 1: Concurrent Preflight
        let preflight_results = preflight::run_batch(
            urls,
            &self.config.user_agent,
            self.config.block_private_hosts,
            self.config.robots_check,
            &self.client,
        )
        .await;

        // 2. Separate valid URLs for the second stage
        let mut final_results = Vec::with_capacity(preflight_results.len());
        let mut to_fetch = Vec::new();

        for (raw, res) in preflight_results {
            match res {
                Ok(url) => to_fetch.push((raw, url)),
                Err(e) => final_results.push((raw, Err(e))),
            }
        }

        // 3. Stage 2: Concurrent Fetching
        let stream = futures::stream::iter(to_fetch).map(|(raw, url)| {
            let engine = self.clone();
            tokio::spawn(async move {
                let res = async {
                    // Resource Control (semaphore + rate limiting) happens inside the task
                    let _permit = engine.semaphore.acquire().await.map_err(|e| {
                        Web2llmError::Config(format!("Failed to acquire concurrency permit: {}", e))
                    })?;
                    engine.limiter.until_ready().await;

                    #[cfg(feature = "rendered")]
                    let elements = PageElements::parse(
                        url.clone(),
                        &engine.client,
                        engine.config.fetch_mode,
                        &engine.browser,
                    )
                    .await?;

                    #[cfg(not(feature = "rendered"))]
                    let elements =
                        PageElements::parse(url.clone(), &engine.client, engine.config.fetch_mode)
                            .await?;

                    elements.into_result(&engine.config)
                }
                .await;
                (raw, res)
            })
        });

        let mut fetched_results: Vec<(String, Result<PageResult>)> = if self.config.ordered {
            stream
                .buffered(self.config.max_concurrency)
                .map(|res| res.expect("Task panicked during batch fetch"))
                .collect()
                .await
        } else {
            stream
                .buffer_unordered(self.config.max_concurrency)
                .map(|res| res.expect("Task panicked during batch fetch"))
                .collect()
                .await
        };

        final_results.append(&mut fetched_results);
        final_results
    }

    /// Discovers links breadth-first starting from `url`, then batch fetches the full set.
    ///
    /// This is a two-stage crawl:
    /// 1. Repeatedly call [`Web2llm::get_urls`] for each URL in the current frontier.
    /// 2. Once discovery completes, call [`Web2llm::batch_fetch`] on the deduplicated URL set.
    ///
    /// Discovery is breadth-first and stops once [`CrawlConfig::max_depth`] is reached.
    /// When [`CrawlConfig::preserve_domain`] is enabled, only URLs on the same origin
    /// as the seed URL are expanded.
    ///
    /// Returns the same shape as [`Web2llm::batch_fetch`]:
    /// `Vec<(String, Result<PageResult>)>`.
    pub async fn crawl(
        &self,
        url: &str,
        crawl_config: CrawlConfig,
    ) -> Vec<(String, Result<PageResult>)> {
        let seed = match crawl::normalize_url(url) {
            Some(url) => url,
            None => {
                return vec![(
                    url.to_string(),
                    Err(Web2llmError::InvalidUrl(url.to_string())),
                )];
            }
        };

        let Some(seed_host) = seed.host_str().map(str::to_string) else {
            return vec![(
                url.to_string(),
                Err(Web2llmError::InvalidUrl("URL has no host".to_string())),
            )];
        };

        let seed_port = seed.port_or_known_default();
        let seed_url = seed.to_string();
        let mut discovered = Vec::from([seed_url.clone()]);
        let mut visited = HashSet::from([seed_url.clone()]);
        let mut frontier = VecDeque::from([seed_url]);
        let mut depth = 0;

        while depth < crawl_config.max_depth && !frontier.is_empty() {
            let level_size = frontier.len();
            let mut current_level = Vec::with_capacity(level_size);

            for _ in 0..level_size {
                if let Some(next_url) = frontier.pop_front() {
                    current_level.push(next_url);
                }
            }

            for current_url in current_level {
                let links = match self.get_urls(&current_url).await {
                    Ok(links) => links,
                    Err(_) => continue,
                };

                for link in links {
                    let Some(normalized) = crawl::normalize_url(&link) else {
                        continue;
                    };

                    if !crawl::should_follow(&normalized, &seed_host, seed_port, &crawl_config) {
                        continue;
                    }

                    let normalized = normalized.to_string();
                    if visited.insert(normalized.clone()) {
                        discovered.push(normalized.clone());
                        frontier.push_back(normalized);
                    }
                }
            }

            depth += 1;
        }

        self.batch_fetch(discovered).await
    }
}

/// Convenience function — fetches `url` using [`Web2llmConfig::default`].
///
/// Equivalent to `Web2llm::new(Web2llmConfig::default()).unwrap().fetch(&url).await`.
///
/// # Errors
/// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
/// Returns [`Web2llmError::EmptyContent`] if no scoreable content is found.
pub async fn fetch(url: String) -> Result<PageResult> {
    Web2llm::new(Web2llmConfig::default())?.fetch(&url).await
}

/// Convenience function — fetches multiple `urls` using [`Web2llmConfig::default`].
///
/// Returns a vector of tuples containing the original URL and the [`Result<PageResult>`].
///
/// # Errors
/// Returns [`Web2llmError::Config`] if default initialization fails.
pub async fn batch_fetch(urls: Vec<String>) -> Result<Vec<(String, Result<PageResult>)>> {
    Ok(Web2llm::new(Web2llmConfig::default())?
        .batch_fetch(urls)
        .await)
}

/// Convenience function — crawls `url` using [`Web2llmConfig::default`].
///
/// Equivalent to `Web2llm::new(Web2llmConfig::default()).unwrap().crawl(&url, crawl_config).await`.
pub async fn crawl(
    url: String,
    crawl_config: CrawlConfig,
) -> Result<Vec<(String, Result<PageResult>)>> {
    Ok(Web2llm::new(Web2llmConfig::default())?
        .crawl(&url, crawl_config)
        .await)
}
