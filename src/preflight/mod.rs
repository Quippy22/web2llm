pub(crate) mod robots;
mod validate;

use dashmap::DashMap;
use url::Url;

use crate::{Result, Web2llmError};

/// Runs the pre-flight validation and compliance checks for a URL.
///
/// 1. Validates that the URL is well-formed and optionally blocks private hosts.
/// 2. If `check_robots` is true, performs a strict check against the `robots_cache` DashMap.
///    If the map is empty, it lazy-initializes discovery (robots.txt + sitemaps) for the host.
///
/// # Errors
/// Returns [`Web2llmError::InvalidUrl`] if the URL is malformed or blocked.
/// Returns [`Web2llmError::Disallowed`] if `robots.txt` explicitly forbids access.
pub(crate) async fn run(
    raw_url: &str,
    user_agent: &str,
    block_private_hosts: bool,
    check_robots: bool,
    client: &reqwest::Client,
    robots_cache: &DashMap<String, bool>,
) -> Result<Url> {
    let url = validate::validate(raw_url, block_private_hosts)?;

    if check_robots {
        // 1. If discovery hasn't happened yet, do it now for the current URL
        if robots_cache.is_empty() {
            robots::build_map(
                robots::RobotsCheck::Single(url.to_string()),
                user_agent,
                client,
                robots_cache,
            )
            .await?;
        }

        // 2. Strict Check: The URL must be in the map to be allowed
        if !robots_cache.contains_key(url.as_str()) {
            return Err(Web2llmError::Disallowed);
        }
    }

    Ok(url)
}
