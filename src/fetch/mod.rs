/// Fetches the raw HTML content of a page at the given URL.
/// Uses a static HTTP request — no JavaScript execution.
/// Returns the full HTML body as a string for downstream parsing.
pub async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    Ok(html)
}
