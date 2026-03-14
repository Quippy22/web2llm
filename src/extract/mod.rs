mod scorer;
use std::time::Duration;

use htmd::convert;
use scraper::{Html, Selector};
use url::Url;

use crate::error::{Result, Web2llmError};
use crate::extract::scorer::ScoredElement;
use crate::fetch::get_html;
use crate::output::PageResult;

/// A single element extracted from the page body.
/// Holds only direct text (not inherited from children)
/// and the full inner HTML for downstream conversion.
pub(crate) struct ExtractedElement {
    pub(crate) tag: String,  // tag name e.g. "article", "div", "p"
    pub(crate) html: String, // full inner HTML including all children
    pub(crate) text: String, // direct text nodes only, used for scoring
}

/// The main extraction type. Parses a page's body into scoreable
/// elements and converts the best candidates to Markdown.
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
    /// Returns `Web2llmError::Http` if the request fails or returns a non-2xx status.
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
    /// Walks every element inside `<body>`, collecting tag name,
    /// inner HTML, and direct text nodes into a flat vec.
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

    /// Scores all elements and returns them sorted by score descending.
    /// Elements scoring 0.0 (below word threshold or penalized tags) are excluded.
    fn score(&self, sensitivity: f32) -> Vec<ScoredElement> {
        let mut scored = scorer::score(&self.body_html, sensitivity);
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored
    }

    /// Converts all positively scored elements to Markdown and joins them.
    /// Each element's full inner HTML is passed to htmd, preserving
    /// nested structure, links, and formatting.
    pub(crate) fn into_result(self, sensitivity: f32) -> Result<PageResult> {
        let scored = self.score(sensitivity);
        if scored.is_empty() {
            return Err(Web2llmError::EmptyContent);
        }
        let markdown = scored
            .iter()
            .map(|s| -> Result<String> { Ok(convert(&s.element.html)?) })
            .collect::<Result<Vec<_>>>()?
            .join("\n\n");
        Ok(PageResult::new(self.url.as_str(), &self.title, markdown))
    }
}
