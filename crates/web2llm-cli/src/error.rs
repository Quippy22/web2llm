//! Error types for `web2llm-cli`.
//!
//! The CLI wraps library, config-file, and output failures in one small
//! user-facing error enum so command handlers can return `Result<T>` cleanly.

/// The unified result type used throughout the CLI crate.
pub type Result<T> = std::result::Result<T, CliError>;

/// The unified error type for command execution.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// A `web2llm` library error.
    #[error("{0}")]
    Web2llm(#[from] web2llm::Web2llmError),

    /// A filesystem error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A TOML parsing error.
    #[error("TOML config error: {0}")]
    Toml(#[from] toml::de::Error),

    /// A JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A CLI-level configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// A batch-style command completed with at least one failed URL.
    #[error("{0} URLs failed during execution")]
    PartialFailure(usize),
}
