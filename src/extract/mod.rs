pub(crate) mod scorer;

use url::Url;

use crate::config::Web2llmConfig;
use crate::error::{Result, Web2llmError};
use crate::fetch::{FetchMode, get_html};
use crate::output::PageResult;

#[cfg(feature = "rendered")]
use tokio::sync::OnceCell;

/// The main extraction type.
pub struct PageElements {
    html_content: String,
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

        // Extract title quickly
        let title = {
            let dom = tl::parse(&html_content, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();
            dom.query_selector("title")
                .and_then(|mut iter| iter.next())
                .map(|node| node.get(parser).unwrap().inner_text(parser).into_owned())
                .unwrap_or_default()
        };

        Ok(Self {
            html_content,
            url,
            title,
        })
    }

    pub(crate) fn into_result(self, config: &Web2llmConfig) -> Result<PageResult> {
        let dom = tl::parse(&self.html_content, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let body_handle = dom
            .query_selector("body")
            .and_then(|mut iter| iter.next())
            .ok_or(Web2llmError::EmptyContent)?;

        let chunks = scorer::process(body_handle, parser, config)?;

        if chunks.is_empty() {
            return Err(Web2llmError::EmptyContent);
        }

        Ok(PageResult::new(self.url.as_str(), &self.title, chunks))
    }

    pub fn get_urls(&self) -> Vec<String> {
        let dom = tl::parse(&self.html_content, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut urls: Vec<String> = Vec::new();
        if let Some(iter) = dom.query_selector("a[href]") {
            for node_handle in iter {
                if let Some(href) = node_handle.get(parser).and_then(|n| n.as_tag()).and_then(|t| t.attributes().get("href").flatten()) {
                    let href_str = std::str::from_utf8(href.as_bytes()).unwrap_or("");
                    if let Ok(joined) = self.url.join(href_str) {
                        urls.push(joined.to_string());
                    }
                }
            }
        }

        urls.sort();
        urls.dedup();
        urls
    }
}
