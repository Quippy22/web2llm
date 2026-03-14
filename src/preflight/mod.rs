mod robots;
mod validate;

use url::Url;

use crate::{Result, Web2llmError};

/// Runs the pre-flight validation and compliance checks for a URL.
///
/// 1. Validates that the URL is well-formed and optionally blocks private hosts.
/// 2. If `check_robots` is true, fetches and respects `robots.txt` for the host.
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
) -> Result<Url> {
    let url = validate::validate(raw_url, block_private_hosts)?;

    if check_robots && !robots::is_allowed(&url, user_agent, client).await? {
        return Err(Web2llmError::Disallowed);
    }

    Ok(url)
}
