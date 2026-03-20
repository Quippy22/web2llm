pub(crate) mod scorer;

use scraper::{Html, Selector};
use url::Url;

use crate::config::Web2llmConfig;
use crate::error::{Result, Web2llmError};
use crate::fetch::{FetchMode, get_html};
use crate::output::PageResult;

#[cfg(feature = "rendered")]
use tokio::sync::OnceCell;

/// The main extraction type. Holds the parsed HTML document,
/// ready for scoring and Markdown conversion.
pub struct PageElements {
    document: Html,
    url: Url,
    title: String,
}

impl PageElements {
    /// Fetches the page at `url` using the provided client,
    /// parses the HTML body, and returns a `PageElements` ready for
    /// scoring and Markdown conversion.
    pub(crate) async fn parse(
        url: Url,
        client: &reqwest::Client,
        mode: FetchMode,
        #[cfg(feature = "rendered")] browser: &OnceCell<chromiumoxide::Browser>,
    ) -> Result<Self> {
        #[cfg(feature = "rendered")]
        let (html_content, _is_dynamic) = get_html(&url, client, mode, browser).await?;

        #[cfg(not(feature = "rendered"))]
        let (html_content, _is_dynamic) = get_html(&url, client, mode).await?;

        let document = Html::parse_document(&html_content);
        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_default();

        Ok(Self::from_document(document, url, title))
    }

    /// Converts all surviving scored elements to Markdown chunks.
    pub(crate) fn into_result(self, config: &Web2llmConfig) -> Result<PageResult> {
        let body = self
            .document
            .select(&Selector::parse("body").unwrap())
            .next()
            .ok_or(Web2llmError::EmptyContent)?;

        // One clean call to the engine
        let chunks = scorer::process(body, config)?;

        if chunks.is_empty() {
            return Err(Web2llmError::EmptyContent);
        }

        Ok(PageResult::new(self.url.as_str(), &self.title, chunks))
    }

    /// Extracts all unique absolute URLs from the entire document.
    pub fn get_urls(&self) -> Vec<String> {
        let selector = Selector::parse("a[href]").unwrap();
        let mut urls: Vec<String> = self
            .document
            .select(&selector)
            .filter_map(|el| el.value().attr("href"))
            .filter_map(|href| self.url.join(href).ok())
            .map(|url| url.to_string())
            .collect();

        urls.sort();
        urls.dedup();
        urls
    }

    fn from_document(document: Html, url: Url, title: String) -> Self {
        Self {
            document,
            url,
            title,
        }
    }
}
