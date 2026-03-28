pub(crate) mod scorer;

use scraper::{Html, Selector};
use std::sync::OnceLock;
use url::Url;

use crate::config::Web2llmConfig;
use crate::error::{Result, Web2llmError};
use crate::fetch::{FetchMode, get_html};
use crate::output::PageResult;

#[cfg(feature = "rendered")]
use tokio::sync::OnceCell;

// Pre-parsed selectors for maximum performance
static TITLE_SELECTOR: OnceLock<Selector> = OnceLock::new();
static BODY_SELECTOR: OnceLock<Selector> = OnceLock::new();

fn get_title_selector() -> &'static Selector {
    TITLE_SELECTOR.get_or_init(|| Selector::parse("title").unwrap())
}

fn get_body_selector() -> &'static Selector {
    BODY_SELECTOR.get_or_init(|| Selector::parse("body").unwrap())
}

/// The main extraction type.
pub struct PageElements {
    document: Html,
    url: Url,
    title: String,
}

impl PageElements {
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
            .select(get_title_selector())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_default();

        Ok(Self {
            document,
            url,
            title,
        })
    }

    pub(crate) fn into_result(self, config: &Web2llmConfig) -> Result<PageResult> {
        let body = self
            .document
            .select(get_body_selector())
            .next()
            .ok_or(Web2llmError::EmptyContent)?;

        let chunks = scorer::process(body, config)?;

        if chunks.is_empty() {
            return Err(Web2llmError::EmptyContent);
        }

        Ok(PageResult::new(self.url.as_str(), &self.title, chunks))
    }

    pub fn get_urls(&self) -> Vec<String> {
        static LINK_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = LINK_SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        let mut urls: Vec<String> = self
            .document
            .select(selector)
            .filter_map(|el| el.value().attr("href"))
            .filter_map(|href| self.url.join(href).ok())
            .map(|url| url.to_string())
            .collect();

        urls.sort();
        urls.dedup();
        urls
    }
}
