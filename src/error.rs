use thiserror::Error;

/// The unified error type for all `web2llm` operations.
///
/// Every fallible function in this crate returns `Result<T>`
/// which is an alias for `Result<T, Web2llmError>`.
#[derive(Debug, Error)]
pub enum Web2llmError {
    /// A network or HTTP error from `reqwest`.
    /// Includes connection failures, timeouts, and non-2xx status codes.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to convert HTML to Markdown.
    /// Wraps `std::io::Error` from `htmd`.
    #[error("Markdown conversion error: {0}")]
    Markdown(#[from] std::io::Error),

    /// The page was fetched and parsed but no scoreable content was found.
    /// Usually means the page is empty, JS-rendered, or pure navigation.
    #[error("No content found")]
    EmptyContent,

    /// The URL failed validation checks.
    /// Could be malformed, use a disallowed scheme (e.g. `ftp://`, `file://`),
    /// or point to a private/loopback address.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// The target URL is blocked by the site's `robots.txt`.
    /// Returned when the crawl rules explicitly disallow the configured user-agent.
    #[error("Disallowed by robots.txt")]
    Disallowed,

    /// Failed to parse the site's `robots.txt` file.
    /// This is distinct from a fetch failure — the file was retrieved but could not be read.
    #[error("Failed to parse robots.txt")]
    RobotsTxt,
}

/// Convenience alias used throughout the crate.
/// Removes the need to specify `Web2llmError` on every return type.
pub type Result<T> = std::result::Result<T, Web2llmError>;
