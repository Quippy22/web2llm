/// The unified error type for all `web2llm` operations.
///
/// Every fallible function in this crate returns `Result<T>`
/// which is an alias for `Result<T, Web2llmError>`.
#[derive(Debug, thiserror::Error)]
pub enum Web2llmError {
    /// A network or HTTP error.
    /// Includes connection failures, timeouts, and non-2xx status codes.
    #[error("HTTP error: {0}")]
    Http(String),

    /// A raw reqwest error.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Failed to convert HTML to Markdown, or the conversion produced no output.
    /// The string contains the specific reason for the failure.
    #[error("Markdown error: {0}")]
    Markdown(String),

    /// A filesystem error — typically a failed read or write during output persistence.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The page was fetched and parsed but no scoreable content was found.
    /// Usually means the page is empty, JS-rendered, or pure navigation.
    #[error("No content found")]
    EmptyContent,

    /// The URL failed validation checks.
    /// Could be malformed, use a disallowed scheme, or point to a private address.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// The target URL is blocked by the site's `robots.txt`.
    #[error("Disallowed by robots.txt")]
    Disallowed,

    /// An error in the configuration.
    /// Example: such as a zero 'rate_limit' or 'max_concurrency'.
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Web2llmError>;
