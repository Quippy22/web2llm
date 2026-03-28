use crate::Result;
use dashmap::DashMap;
use texting_robots::Robot;
use url::Url;

/// Defines the scope of the robots.txt check.
pub enum RobotsCheck {
    /// Check a single URL (optimized for `fetch`).
    Single(String),
    /// Check a batch of URLs (optimized for `batch_fetch`).
    Batch(Vec<String>),
    /// Perform full discovery (optimized for `crawl`).
    Crawl(String),
}

/// The main entry point to populate the robots cache.
/// Dispatches to the appropriate implementation based on the `RobotsCheck` type.
pub(crate) async fn build_map(
    task: RobotsCheck,
    user_agent: &str,
    client: &reqwest::Client,
    cache: &DashMap<String, bool>,
) -> Result<()> {
    match task {
        RobotsCheck::Single(url) => check_single(&url, user_agent, client, cache).await,
        RobotsCheck::Batch(urls) => check_batch(&urls, user_agent, client, cache).await,
        RobotsCheck::Crawl(seed) => check_crawl(&seed, user_agent, client, cache).await,
    }
}

async fn check_single(
    url_str: &str,
    user_agent: &str,
    client: &reqwest::Client,
    cache: &DashMap<String, bool>,
) -> Result<()> {
    let url = Url::parse(url_str).map_err(|e| crate::Web2llmError::InvalidUrl(e.to_string()))?;
    let body = fetch_robots_txt(&url, client).await;

    // Fail-open: If no robots.txt, it's allowed
    if body.is_empty() {
        cache.insert(url.to_string(), false);
        return Ok(());
    }

    let robot = Robot::new(user_agent, body.as_bytes())
        .map_err(|e| crate::Web2llmError::Config(format!("Failed to parse robots.txt: {}", e)))?;

    if robot.allowed(url.as_str()) {
        cache.insert(url.to_string(), false);
    }
    Ok(())
}

async fn check_batch(
    urls: &[String],
    user_agent: &str,
    client: &reqwest::Client,
    cache: &DashMap<String, bool>,
) -> Result<()> {
    let host_map: DashMap<String, Vec<Url>> = DashMap::new();

    for url_str in urls {
        if let Ok(url) = Url::parse(url_str) {
            let host_key = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
            host_map.entry(host_key).or_default().push(url);
        }
    }

    for entry in host_map {
        let (host_base, host_urls) = entry;
        let base_url = Url::parse(&host_base).unwrap();
        let body = fetch_robots_txt(&base_url, client).await;

        if body.is_empty() {
            for u in host_urls {
                cache.insert(u.to_string(), false);
            }
            continue;
        }

        if let Ok(robot) = Robot::new(user_agent, body.as_bytes()) {
            for u in host_urls {
                if robot.allowed(u.as_str()) {
                    cache.insert(u.to_string(), false);
                }
            }
        }
    }
    Ok(())
}

async fn check_crawl(
    seed_url: &str,
    user_agent: &str,
    client: &reqwest::Client,
    cache: &DashMap<String, bool>,
) -> Result<()> {
    let url = Url::parse(seed_url).map_err(|e| crate::Web2llmError::InvalidUrl(e.to_string()))?;
    let body = fetch_robots_txt(&url, client).await;

    if body.is_empty() {
        cache.insert(seed_url.to_string(), false);
        return Ok(());
    }

    let robot = Robot::new(user_agent, body.as_bytes())
        .map_err(|e| crate::Web2llmError::Config(format!("Failed to parse robots.txt: {}", e)))?;

    // Discovery: Sitemaps
    for sitemap_url in &robot.sitemaps {
        if let Ok(resp) = client.get(sitemap_url).send().await {
            if let Ok(text) = resp.text().await {
                for part in text.split("<loc>") {
                    if let Some(end) = part.find("</loc>") {
                        let discovered = part[..end].trim();
                        if robot.allowed(discovered) {
                            cache.insert(discovered.to_string(), false);
                        }
                    }
                }
            }
        }
    }

    // Always include seed if allowed
    if robot.allowed(seed_url) {
        cache.insert(seed_url.to_string(), false);
    }

    Ok(())
}

async fn fetch_robots_txt(url: &Url, client: &reqwest::Client) -> String {
    let robots_url = build_robots_url(url);
    match client.get(&robots_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        _ => String::new(), // Fail open
    }
}

fn build_robots_url(url: &Url) -> String {
    let scheme = url.scheme();
    let host = url.host_str().unwrap_or_default();
    match url.port() {
        Some(port) => format!("{}://{}:{}/robots.txt", scheme, host, port),
        None => format!("{}://{}/robots.txt", scheme, host),
    }
}
