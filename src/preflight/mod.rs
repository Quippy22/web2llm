pub(crate) mod robots;
mod validate;

use crate::Result;
use url::Url;

/// FASTEST PATH: Synchronous pre-flight check (validation only).
#[inline(always)]
pub(crate) fn run_sync(raw_url: &str, block_private_hosts: bool) -> Result<Url> {
    validate::validate(raw_url, block_private_hosts)
}

/// FAST PATH: Pre-flight check for a single URL.
pub(crate) async fn run(
    raw_url: &str,
    user_agent: &str,
    block_private_hosts: bool,
    check_robots: bool,
    client: &reqwest::Client,
) -> Result<Url> {
    let url = run_sync(raw_url, block_private_hosts)?;

    if check_robots {
        robots::check_single(&url, user_agent, client).await?;
    }

    Ok(url)
}

/// BATCH PATH: Concurrent pre-flight for multiple URLs.
pub(crate) async fn run_batch(
    urls: Vec<String>,
    user_agent: &str,
    block_private_hosts: bool,
    check_robots: bool,
    client: &reqwest::Client,
) -> Vec<(String, Result<Url>)> {
    let mut results = Vec::with_capacity(urls.len());
    let mut valid_to_check = Vec::new();

    for url_str in urls {
        match run_sync(&url_str, block_private_hosts) {
            Ok(url) => valid_to_check.push((url_str, url)),
            Err(e) => results.push((url_str, Err(e))),
        }
    }

    if !check_robots || valid_to_check.is_empty() {
        for (raw, url) in valid_to_check {
            results.push((raw, Ok(url)));
        }
        return results;
    }

    let robots_results = robots::check_batch(valid_to_check, user_agent, client).await;
    results.extend(robots_results);

    results
}
