use thiserror::Error;

/// The unified error type for all `web2llm` operations.
///
/// Every fallible function in this crate returns `Result<T>`
/// which is an alias for `Result<T, Web2LlmError>`.
#[derive(Debug, Error)]
pub enum Web2LlmError {
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
}

/// Convenience alias used throughout the crate.
/// Removes the need to specify `Web2LlmError` on every return type.
pub type Result<T> = std::result::Result<T, Web2LlmError>;
