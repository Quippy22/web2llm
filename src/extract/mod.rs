mod scorer;

use htmd::convert;
use scraper::{Html, Selector};
use url::Url;

use crate::error::{Result, Web2llmError};
use crate::extract::scorer::ScoredElement;
use crate::fetch::get_html;
use crate::output::PageResult;

/// The main extraction type. Holds the parsed HTML document,
/// ready for scoring and Markdown conversion.
///
/// Unlike the raw HTML string, this struct holds the full `scraper::Html`
/// tree, allowing the scorer to traverse the document without re-parsing.
pub struct PageElements {
    document: Html,
    url: Url,
    title: String,
}

impl PageElements {
    /// Fetches the page at `url` using the provided client,
    /// parses the HTML body, and returns a `PageElements` ready for
    /// scoring and Markdown conversion.
    ///
    /// This is the main entry point for content extraction.
    ///
    /// # Errors
    /// Returns [`Web2llmError::Http`] if the request fails or returns a non-2xx status.
    pub(crate) async fn parse(url: Url, client: &reqwest::Client) -> Result<Self> {
        let html_content = get_html(&url, client).await?;
        let document = Html::parse_document(&html_content);
        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_default();

        Ok(Self::from_document(document, url, title))
    }

    /// Builds a `PageElements` from an already-parsed HTML document.
    ///
    /// Used internally by `parse` and directly in tests.
    fn from_document(document: Html, url: Url, title: String) -> Self {
        Self {
            document,
            url,
            title,
        }
    }

    /// Scores the body elements and returns elements sorted by score descending.
    /// Uses the scorer's N-ary tree traversal — penalty subtrees are pruned
    /// and scores bubble up from leaves to root.
    fn score(&self, sensitivity: f32) -> Vec<ScoredElement> {
        let body_selector = Selector::parse("body").unwrap();
        let body = self.document.select(&body_selector).next();

        let mut scored = if let Some(body_el) = body {
            scorer::score(body_el, sensitivity)
        } else {
            Vec::new()
        };

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
            .map(|s| convert(&s.html).map_err(|e| Web2llmError::Markdown(e.to_string())))
            .collect::<Result<Vec<_>>>()?
            .join("\n\n");
        Ok(PageResult::new(self.url.as_str(), &self.title, markdown))
    }
}
