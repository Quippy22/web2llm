mod scorer;
use std::time::Duration;

use htmd::convert;
use scraper::{Html, Selector};
use url::Url;

use crate::error::{Result, Web2llmError};
use crate::extract::scorer::ScoredElement;
use crate::fetch::get_html;
use crate::output::PageResult;

/// The main extraction type. Holds the raw body html of a fetched page,
/// ready for scoring and Markdown conversion.
pub struct PageElements {
    body_html: String,
    url: Url,
    title: String,
}

impl PageElements {
    /// Fetches the page at `url`, parses the HTML body, and returns
    /// a `PageElements` ready for scoring and Markdown conversion.
    ///
    /// This is the main entry point for content extraction.
    ///
    /// # Errors
    /// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
    pub(crate) async fn parse(url: Url, timeout: Duration, user_agent: &str) -> Result<Self> {
        let html = get_html(&url, timeout, user_agent).await?;
        let document = Html::parse_document(&html);
        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_default();

        Ok(Self::from_document(document, url, title))
    }

    /// Builds a `PageElements` from an already-parsed HTML document.
    /// Extracts the inner html of `<body>` for downstream scoring.
    ///
    /// Used internally by `parse` and directly in tests.
    fn from_document(html: Html, url: Url, title: String) -> Self {
        let body_html = html
            .select(&Selector::parse("body").unwrap())
            .next()
            .map(|b| b.inner_html())
            .unwrap_or_default();

        Self {
            body_html,
            url,
            title,
        }
    }

    /// Scores the body html and returns elements sorted by score descending.
    /// Uses the scorer's N-ary tree traversal — penalty subtrees are pruned
    /// and scores bubble up from leaves to root.
    fn score(&self, sensitivity: f32) -> Vec<ScoredElement> {
        let mut scored = scorer::score(&self.body_html, sensitivity);
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored
    }

    /// Converts all surviving scored elements to Markdown and joins them.
    /// Each element's cleaned html is passed to htmd, preserving
    /// headings, links, code blocks, and inline formatting.
    ///
    /// # Errors
    /// Returns [`Web2llmError::EmptyContent`] if no elements scored above the threshold.
    /// Returns [`Web2llmError::Markdown`] if html to Markdown conversion fails.
    pub(crate) fn into_result(self, sensitivity: f32) -> Result<PageResult> {
        let scored = self.score(sensitivity);
        if scored.is_empty() {
            return Err(Web2llmError::EmptyContent);
        }
        let markdown = scored
            .iter()
            .map(|s| -> Result<String> { Ok(convert(&s.html)?) })
            .collect::<Result<Vec<_>>>()?
            .join("\n\n");
        Ok(PageResult::new(self.url.as_str(), &self.title, markdown))
    }
}
