use crate::Result;
use texting_robots::Robot;
use url::Url;

/// Checks whether `url` is allowed to be fetched according to the target
/// site's `robots.txt`.
///
/// Fetches `robots.txt` from the root of the target host and evaluates the
/// rules against `user_agent`. Respects both agent-specific rules and
/// wildcard `*` rules, with agent-specific rules taking precedence.
///
/// # Fail open
///
/// This function never blocks on uncertainty. If `robots.txt` is absent,
/// unreachable, or malformed, access is permitted. Only an explicit
/// `Disallow` rule for the given `user_agent` returns `Ok(false)`.
///
/// # Errors
///
/// This function does not propagate network or parse errors — all failure
/// cases fail open and return `Ok(true)`.
pub(crate) async fn is_allowed(
    url: &Url,
    user_agent: &str,
    client: &reqwest::Client,
) -> Result<bool> {
    let body = fetch_robots_txt(url, client).await;
    let body = match body {
        Ok(text) => text,
        Err(_) => return Ok(true),
    };
    if body.is_empty() {
        return Ok(true);
    }
    let robot = match Robot::new(user_agent, body.as_bytes()) {
        Ok(r) => r,
        Err(_) => return Ok(true),
    };
    Ok(robot.allowed(url.as_str()))
}

/// Fetches the `robots.txt` file for the host of `url`.
///
/// Returns the body as a `String` on success. Returns an empty string if
/// the file does not exist (404) or the server returns a non-success status.
/// This signals to the caller that no restrictions apply.
async fn fetch_robots_txt(url: &Url, client: &reqwest::Client) -> Result<String> {
    let robots_url = build_robots_url(url);
    let response = client.get(&robots_url).send().await?;
    if response.status().as_u16() == 404 {
        return Ok(String::new());
    }
    if !response.status().is_success() {
        return Ok(String::new());
    }
    Ok(response.text().await.unwrap_or_default())
}

/// Constructs the `robots.txt` URL for the host of `url`.
///
/// Preserves the scheme and port of the target URL. For example,
/// `https://example.com:8080/some/page` becomes
/// `https://example.com:8080/robots.txt`.
fn build_robots_url(url: &Url) -> String {
    match url.port() {
        Some(port) => format!(
            "{}://{}:{}/robots.txt",
            url.scheme(),
            url.host_str().unwrap_or_default(),
            port
        ),
        None => format!(
            "{}://{}/robots.txt",
            url.scheme(),
            url.host_str().unwrap_or_default()
        ),
    }
}
