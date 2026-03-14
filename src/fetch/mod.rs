use url::Url;

use crate::error::Result;

/// Fetches the raw HTML content of a page at the given URL using the provided client.
///
/// Uses a static HTTP request — no JavaScript execution. Returns the full HTML
/// body as a string for downstream parsing into `PageElements`.
pub(crate) async fn get_html(url: &Url, client: &reqwest::Client) -> Result<String> {
    let response = client.get(url.as_str()).send().await?;
    let response = response.error_for_status()?;
    let html = response.text().await?;
    Ok(html)
}
