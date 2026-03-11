use chrono::{DateTime, Utc};

/// The result of a successful page fetch and extraction.
///
/// Contains the page's URL, title, and main content converted to Markdown,
/// along with a UTC timestamp of when the fetch occurred.
///
/// Returned by [`crate::Web2llm::fetch`] and the free [`crate::fetch`] function.
pub struct PageResult {
    /// The URL that was fetched.
    pub url: String,
    /// The page's `<title>` tag content, or an empty string if not found.
    pub title: String,
    /// The main page content converted to clean Markdown.
    /// Structural noise (nav, footer, sidebar) is excluded by the scoring stage.
    pub markdown: String,
    /// UTC timestamp of when the page was fetched.
    pub timestamp: DateTime<Utc>,
}

impl PageResult {
    /// Creates a new `PageResult`, stamping the current UTC time as the timestamp.
    ///
    /// Called internally by the extraction stage — consumers receive a fully
    /// populated `PageResult` and do not need to call this directly.
    pub fn new(url: &str, title: &str, markdown: String) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            markdown,
            timestamp: Utc::now(),
        }
    }
}
