//! Logic for fetching and parsing robots.txt to ensure compliance.
//!
//! This module provides synchronous and asynchronous paths for checking
//! whether a given URL is allowed to be fetched according to the site's rules.

use crate::{Result, Web2llmError};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use texting_robots::Robot;
use url::Url;

/// Checks a batch of URLs for robots.txt compliance concurrently.
/// Groups by host to avoid redundant network calls.
pub(crate) async fn check_batch(
    urls: Vec<(String, Url)>,
    user_agent: &str,
    client: &reqwest::Client,
) -> Vec<(String, Result<Url>)> {
    if urls.is_empty() {
        return vec![];
    }

    let mut results: Vec<(String, Result<Url>)> = urls
        .iter()
        .map(|(raw, url)| (raw.clone(), Ok(url.clone())))
        .collect();

    // 1. Group indices by unique host
    let mut host_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, (_, url)) in urls.iter().enumerate() {
        let host_key = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
        host_map.entry(host_key).or_default().push(i);
    }

    // 2. Fetch robots.txt for all unique hosts in parallel
    let user_agent_str = user_agent.to_string();
    let host_checks = stream::iter(host_map)
        .map(|(host_base, indices)| {
            let client = client.clone();
            let ua = user_agent_str.clone();
            async move {
                let base_url = Url::parse(&host_base).unwrap();
                let body = fetch_robots_txt(&base_url, &client).await;
                (body, indices, ua)
            }
        })
        .buffer_unordered(20);

    let host_results: Vec<(String, Vec<usize>, String)> = host_checks.collect().await;

    // 3. Apply rules to results
    for (body, indices, ua) in host_results {
        if body.is_empty() {
            continue; // Fail-open
        }

        if let Ok(robot) = Robot::new(&ua, body.as_bytes()) {
            for idx in indices {
                let forbidden = results[idx]
                    .1
                    .as_ref()
                    .map(|url| !robot.allowed(url.as_str()))
                    .unwrap_or(false);

                if forbidden {
                    results[idx].1 = Err(Web2llmError::Disallowed);
                }
            }
        }
    }

    results
}

/// Checks if a single URL is allowed by robots.txt.
pub(crate) async fn check_single(
    url: &Url,
    user_agent: &str,
    client: &reqwest::Client,
) -> Result<()> {
    let body = fetch_robots_txt(url, client).await;
    if body.is_empty() {
        return Ok(());
    }

    let robot = Robot::new(user_agent, body.as_bytes())
        .map_err(|e| Web2llmError::Config(format!("Failed to parse robots.txt: {}", e)))?;

    if robot.allowed(url.as_str()) {
        Ok(())
    } else {
        Err(Web2llmError::Disallowed)
    }
}

/// Fetches the robots.txt content for the given URL's host.
async fn fetch_robots_txt(url: &Url, client: &reqwest::Client) -> String {
    let scheme = url.scheme();
    let host = url.host_str().unwrap_or_default();
    let robots_url = match url.port() {
        Some(port) => format!("{}://{}:{}/robots.txt", scheme, host, port),
        None => format!("{}://{}/robots.txt", scheme, host),
    };

    match client.get(&robots_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        _ => String::new(),
    }
}
