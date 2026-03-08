pub async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let html = response.text().await?;

    Ok(html)
}
