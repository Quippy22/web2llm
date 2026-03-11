use std::time::Duration;

use url::Url;

use crate::error::Result;

/// Fetches the raw HTML content of a page at the given URL.
/// Uses a static HTTP request — no JavaScript execution.
/// Returns the full HTML body as a string for downstream parsing.
pub(crate) async fn get_html(url: &Url, timeout: Duration, user_agent: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(user_agent)
        .build()?;
    let response = client.get(url.as_str()).send().await?;
    let response = response.error_for_status()?;
    let html = response.text().await?;
    Ok(html)
}
